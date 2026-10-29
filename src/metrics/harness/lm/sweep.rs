use super::super::*;

impl<'a> PhianoLM<'a> {

    pub(in crate::metrics::harness) fn sweep_kinded(
        &self,
        sentences: &[String],
        betas: &[f64],
        kind: ContextKind,
        against_uniform: bool,
    ) -> Vec<SweepRow> {
        let recurrent = kind.is_recurrent();
        let floor = 1.0 / (self.unigram.len().max(1) as f64 * 100.0);
        let nb = betas.len();
        let ng = GAMMA_GRID.len();

        let (sums, n) = sentences
            .par_iter()
            .map(|sentence| {
                let mut local = vec![0.0f64; nb * ng];
                let mut count = 0usize;
                let toks = Tokenizer::tokenize(sentence);
                if toks.len() < 3 {
                    return (local, count);
                }

                let mut h = vec![phistd::c64::new(0.0, 0.0); LM_CHANNELS];
                if recurrent {
                    self.advance(&mut h, &toks[0]);
                    self.advance(&mut h, &toks[1]);
                }

                let mut score_buf: Vec<f64> = Vec::with_capacity(self.unigram.len());

                for i in 2..toks.len() {
                    let (a, b, c) = (&toks[i - 2], &toks[i - 1], &toks[i]);
                    let idx = match self.index.get(c) {
                        Some(v) => *v,
                        None => {
                            if recurrent {
                                self.advance(&mut h, c);
                            }
                            continue;
                        }
                    };

                    let ctx = match kind {
                        ContextKind::Recurrent => Self::state_to_ctx(&h),
                        ContextKind::Bound => match self.context_vec_bound(a, b) {
                            Some(v) => v,
                            None => [1.0, 0.0].repeat(LM_CHANNELS),
                        },
                        ContextKind::TwoWord => match self.context_vec(a, b) {
                            Some(v) => v,
                            None => [1.0, 0.0].repeat(LM_CHANNELS),
                        },
                    };

                    self.scores(&ctx, &mut score_buf);
                    let max = score_buf.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                    let (k_const, coef) = self.affine(a, b, c);
                    let p_uni = match against_uniform {
                        true => 1.0 / self.unigram.len().max(1) as f64,
                        false => self.unigram[idx].max(floor),
                    };

                    for (bi, beta) in betas.iter().enumerate() {
                        let z = phistd::prob::softmax_z(score_buf.iter().copied(), max, *beta);
                        let p_ph = match z > 0.0 {
                            true => phistd::prob::softmax_p(score_buf[idx], max, *beta, z).max(floor),
                            false => p_uni,
                        };
                        for (gi, g) in GAMMA_GRID.iter().enumerate() {
                            let base = g * p_ph + (1.0 - g) * p_uni;
                            local[bi * ng + gi] += (k_const + coef * base).max(1e-12).ln();
                        }
                    }

                    count += 1;
                    if recurrent {
                        self.advance(&mut h, c);
                    }
                }

                (local, count)
            })
            .reduce(
                || (vec![0.0f64; nb * ng], 0usize),
                |mut a, b| {
                    for (x, y) in a.0.iter_mut().zip(b.0.iter()) {
                        *x += y;
                    }
                    a.1 += b.1;
                    a
                },
            );

        let label = match against_uniform {
            false => kind.label().to_string(),
            true => format!("{}/unif", kind.label()),
        };
        let mut rows = Vec::with_capacity(nb * ng);
        for (bi, beta) in betas.iter().enumerate() {
            for (gi, g) in GAMMA_GRID.iter().enumerate() {
                rows.push(SweepRow {
                    context: label.clone(),
                    beta: *beta,
                    gamma: *g,
                    ppl: phistd::prob::perplexity(sums[bi * ng + gi], n),
                });
            }
        }
        rows
    }

    /// Evaluates the whole γ grid in a single pass over the data.
    ///
    /// The mixture constants at one position: the target's index, the trigram
    /// affine `(k, coef)`, and the unigram probability of the target — what a
    /// scored term is mixed with. `None` if the target is unknown.
    pub fn mixture_terms(&self, a: &str, b: &str, c: &str) -> Option<(usize, f64, f64, f64)> {
        let idx = *self.index.get(c)?;
        let (k, coef) = self.affine(a, b, c);
        let floor = 1.0 / (self.unigram.len().max(1) as f64 * 100.0);
        Some((idx, k, coef, self.unigram[idx].max(floor)))
    }

    /// The softmax temperature the phase term is scored at.
    pub const BETA: f64 = PHASE_BETA;

    /// Vocabulary in index order, for a scorer that wants to align to it.
    pub fn vocabulary(&self) -> Vec<String> {
        let mut v = vec![String::new(); self.index.len()];
        for (w, i) in &self.index {
            v[*i] = w.clone();
        }
        v
    }
}
