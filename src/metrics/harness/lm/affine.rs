use super::super::*;

impl<'a> PhianoLM<'a> {

    /// The affine decomposition of the trigram probability in `P_base`.
    ///
    /// `P(c|a,b) = const + coef · P_base(c|ctx)` — being affine means a whole
    /// grid of mixing weights can be evaluated from one pass over the data.
    pub(in crate::metrics::harness) fn affine(&self, a: &str, b: &str, c: &str) -> (f64, f64) {
        // Absolute discounting from the facet's own interned tables. The
        // context key is a (u32, u32) tuple, so scoring a position no longer
        // allocates a joined String per lookup.
        let (bi_const, bi_coef) = match self.facet.bigram_stats(b, c) {
            Some(st) if st.total > 0 => {
                let lambda = DISCOUNT * st.types as f64 / st.total as f64;
                (
                    (st.count as f64 - DISCOUNT).max(0.0) / st.total as f64,
                    lambda,
                )
            }
            _ => (0.0, 1.0),
        };

        match self.facet.trigram_stats(a, b, c) {
            Some(st) if st.total > 0 => {
                let lambda = DISCOUNT * st.types as f64 / st.total as f64;
                (
                    (st.count as f64 - DISCOUNT).max(0.0) / st.total as f64
                        + lambda * bi_const,
                    lambda * bi_coef,
                )
            }
            _ => (bi_const, bi_coef),
        }
    }

    /// Probability of `c` following `a b`, at this model's γ.
    /// The context construction used by [`PhianoLM::probability`] and the
    /// γ sweep.
    ///
    /// B3: measured best of the three in both training regimes — phase-alone
    /// perplexity 173.08 against 182.69 for the two-word centroid under the
    /// ranking objective, and 188.63 against 192.57 under co-occurrence. The
    /// two-word centroid barely encodes order at all (swap-cosine 0.62: the
    /// swapped context is still more similar to the original than not), and it
    /// was the default anyway.
    pub const DEFAULT_CONTEXT: ContextKind = ContextKind::Recurrent;

    pub fn probability(&self, a: &str, b: &str, c: &str) -> f64 {
        let floor = 1.0 / (self.unigram.len().max(1) as f64 * 100.0);
        let idx = match self.index.get(c) {
            Some(i) => *i,
            None => return floor,
        };
        let (k, coef) = self.affine(a, b, c);
        let p_uni = self.unigram[idx].max(floor);
        let p_ph = match (self.gamma > 0.0, self.context_vec(a, b)) {
            (true, Some(ctx)) => self.p_phase(&ctx, idx),
            _ => p_uni,
        };
        let base = self.gamma * p_ph + (1.0 - self.gamma) * p_uni;
        (k + coef * base).max(1e-12)
    }

    /// Perplexity over held-out sentences, at [`PhianoLM::DEFAULT_CONTEXT`].
    ///
    /// B3: the recurrent state is carried across each sentence rather than
    /// rebuilt from the last two words at every position. `probability` sees
    /// only a trigram and structurally cannot do this, which is why the
    /// two-word centroid stayed the default long after it was measured worse.
    pub fn perplexity(&self, sentences: &[String]) -> f64 {
        self.perplexity_kind(sentences, Self::DEFAULT_CONTEXT)
    }

    /// Perplexity through the trigram `probability` path only.
    ///
    /// Kept as the comparison point B3 is measured against, and used wherever a
    /// caller genuinely has nothing but three words.
    pub fn perplexity_two_word(&self, sentences: &[String]) -> f64 {
        let mut log_sum = 0.0f64;
        let mut n = 0usize;
        for sentence in sentences {
            let toks = Tokenizer::tokenize(sentence);
            if toks.len() < 3 {
                continue;
            }
            for w in toks.windows(3) {
                log_sum += self.probability(&w[0], &w[1], &w[2]).ln();
                n += 1;
            }
        }
        phistd::prob::perplexity(log_sum, n)
    }

    /// Perplexity at an explicit context construction.
    pub fn perplexity_kind(&self, sentences: &[String], kind: ContextKind) -> f64 {
        if self.gamma <= 0.0 {
            // With the phase term switched off the context is never consulted,
            // so the cheap path is also the exact one.
            return self.perplexity_two_word(sentences);
        }

        let floor = 1.0 / (self.unigram.len().max(1) as f64 * 100.0);
        let (log_sum, n) = sentences
            .par_iter()
            .map(|sentence| {
                let toks = Tokenizer::tokenize(sentence);
                if toks.len() < 3 {
                    return (0.0f64, 0usize);
                }
                let mut h = vec![phistd::c64::new(0.0, 0.0); LM_CHANNELS];
                if kind.is_recurrent() {
                    self.advance(&mut h, &toks[0]);
                    self.advance(&mut h, &toks[1]);
                }

                let (mut sum, mut count) = (0.0f64, 0usize);
                for i in 2..toks.len() {
                    let (a, b, c) = (&toks[i - 2], &toks[i - 1], &toks[i]);
                    let idx = self.index.get(c).copied();

                    let ctx = match kind {
                        ContextKind::Recurrent => Some(Self::state_to_ctx(&h)),
                        ContextKind::Bound => self.context_vec_bound(a, b),
                        ContextKind::TwoWord => self.context_vec(a, b),
                    };

                    let p = match idx {
                        None => floor,
                        Some(idx) => {
                            let (k, coef) = self.affine(a, b, c);
                            let p_uni = self.unigram[idx].max(floor);
                            let p_ph = match ctx {
                                Some(ref v) => self.p_phase(v, idx),
                                None => p_uni,
                            };
                            let base = self.gamma * p_ph + (1.0 - self.gamma) * p_uni;
                            (k + coef * base).max(1e-12)
                        }
                    };
                    sum += p.ln();
                    count += 1;

                    if kind.is_recurrent() {
                        self.advance(&mut h, c);
                    }
                }
                (sum, count)
            })
            .reduce(|| (0.0f64, 0usize), |a, b| (a.0 + b.0, a.1 + b.1));

        phistd::prob::perplexity(log_sum, n)
    }

    /// Recurrent context state, per channel, over a sentence prefix.
    ///
    /// `h_k = λ_k · e^{i ω_k} · h_{k-1} + z_k`, the same diagonal complex
    /// recurrence `ContextWaveBuffer` uses. Unlike a two-word centroid this
    /// carries the whole prefix, with a per-channel timescale — information a
    /// trigram table structurally cannot have.
    #[inline]
    pub(in crate::metrics::harness) fn kernel(k: usize) -> phistd::c64 {
        crate::metrics::kernel::channel_kernel(k, LM_CHANNELS)
    }
}
