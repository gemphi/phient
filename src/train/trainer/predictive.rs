#[allow(unused_imports)]
use super::*;
use phical::circle::Resultant;

impl Trainer {

    /// Trains the model to *predict*, not merely to cluster.
    ///
    /// For each position the context wave is compared against the true next word
    /// and against a sampled wrong one. When the wrong word ranks at least as
    /// close, the true word is rotated toward the context and the wrong word
    /// away — a hinge loss on next-word retrieval, applied online with no
    /// backpropagation.
    ///
    /// Centroid attraction is a *descriptive* target ("be like your neighbours")
    /// whose optimum is collapse. Next-word prediction is the objective that
    /// forces a representation to encode syntax, semantics and facts, because
    /// predicting well requires them.
    ///
    /// Returns the number of hinge violations corrected.
    pub fn train_predictive(&self, facet: &mut Facet, text: &str) -> usize {
        let tokens = Tokenizer::tokenize(text);
        if tokens.len() < 2 {
            return 0;
        }
        self.initialize_tokens(facet, &tokens);
        facet.rebuild_sample_pool();

        let step_seed = fnv1a(text) ^ 0xA5A5_5A5A_A5A5_5A5A ^ phistd::mix64(self.seed);
        let channels = self.update_channels(facet, step_seed);

        let mut violations = 0;

        // Running per-channel context accumulators.
        //
        // Recomputing the prefix centroid at every position made this O(L²·D)
        // per sentence, which is why the ranking objective cost 15x what
        // co-occurrence training did. Carrying the sums forward makes it O(L·D)
        // and produces the same context, one token at a time.
        let mut acc = vec![Resultant::new(); channels.len()];

        let add_token = |facet: &Facet, acc: &mut [Resultant], t: &str| {
            if let Some(p) = facet.lexicon.get(t) {
                let w = p.amplitude * Self::token_weight(t);
                for (ci, &k) in channels.iter().enumerate() {
                    acc[ci].add(w, p.theta(k));
                }
            }
        };

        add_token(facet, &mut acc, &tokens[0]);

        for i in 1..tokens.len() {
            let ctx: Vec<f64> = acc.iter().map(|r| r.direction()).collect();

            let pos_word = tokens[i].clone();
            let r = phistd::mix64(step_seed ^ (i as u64).wrapping_mul(0x9E3779B9));
            let neg_word = match facet.sample_negative(r) {
                Some(w) if *w != pos_word && !tokens[..i].contains(w) => w.clone(),
                _ => {
                    add_token(facet, &mut acc, &pos_word);
                    continue;
                }
            };

            let (pos_score, neg_score) = {
                let pos = match facet.lexicon.get(&pos_word) { Some(p) => p, None => continue };
                let neg = match facet.lexicon.get(&neg_word) { Some(p) => p, None => continue };
                let mut ps = 0.0;
                let mut ns = 0.0;
                for (ci, &k) in channels.iter().enumerate() {
                    ps += (ctx[ci] - pos.theta(k)).cos();
                    ns += (ctx[ci] - neg.theta(k)).cos();
                }
                (ps / channels.len() as f64, ns / channels.len() as f64)
            };

            // Perceptron-style: only update when the ranking is wrong.
            if neg_score >= pos_score - HINGE_MARGIN {
                violations += 1;

                if let Some(p) = facet.lexicon.get_mut(&pos_word) {
                    for (ci, &k) in channels.iter().enumerate() {
                        p.nudge(k, self.learning_rate * (ctx[ci] - p.theta(k)).sin());
                    }
                    p.sync_phase();
                }
                if let Some(p) = facet.lexicon.get_mut(&neg_word) {
                    for (ci, &k) in channels.iter().enumerate() {
                        p.nudge(k, -self.learning_rate * 0.5 * (ctx[ci] - p.theta(k)).sin());
                    }
                    p.sync_phase();
                }
            }

            add_token(facet, &mut acc, &pos_word);
        }

        violations
    }

    /// One full pass: co-occurrence structure, then predictive ranking.
    ///
    /// Measurement does not favour this. Running both objectives is worse than
    /// running the ranking objective alone on every relational metric — the two
    /// pull the phases in different directions. Prefer [`Trainer::train`].
    pub fn train_full(&self, facet: &mut Facet, text: &str) -> usize {
        let n = self.train_sentence(facet, text);
        self.train_predictive(facet, text);
        n
    }

    /// The recommended training path: record structure, then rank.
    ///
    /// N-gram statistics, phase lags and amplitudes are recorded exactly as
    /// usual, but the manifold is shaped by the **ranking objective only** —
    /// no centroid attraction.
    ///
    /// Three independent measurements support this over
    /// [`Trainer::train_sentence`]:
    ///
    /// * predictive signal recovered rises from 0.9% to 24.3% of what unigram
    ///   frequency provides (`docs/how/RESULTS.md` §3),
    /// * analogy accuracy rises from exactly 0.00% to 0.62%, against a 0.0024%
    ///   chance rate, and mean reciprocal rank from 0.0005 to 0.0120 (§3e),
    /// * phase dispersion holds higher, 0.997 against 0.954, so it collapses
    ///   less.
    ///
    /// Since the context accumulators became incremental it is also the cheaper
    /// path: 5.4s against 10.1s over 12,000 dictionary definitions.
    pub fn train(&self, facet: &mut Facet, text: &str) -> usize {
        let tokens = Tokenizer::tokenize(text);
        if tokens.is_empty() {
            return 0;
        }
        self.initialize_tokens(facet, &tokens);
        self.record_ngrams(facet, &tokens);
        for t in &tokens {
            if let Some(p) = facet.lexicon.get_mut(t) {
                p.observe();
            }
        }
        self.train_predictive(facet, text);
        self.couple_relations(facet, &tokens);
        tokens.len()
    }

    /// Rotates one token toward a target phase, scaled by `weight`.
    ///
    /// The primitive credit assignment needs: reinforce a word in proportion to
    /// how much it contributed, rather than reinforcing every word in a winning
    /// output equally.
    pub fn nudge_token(&self, facet: &mut Facet, word: &str, target: f64, weight: f64) -> bool {
        match facet.lexicon.get_mut(word) {
            Some(p) => {
                let d = (target - p.theta(0)).sin();
                p.nudge(0, self.learning_rate * weight * d);
                p.sync_phase();
                true
            }
            None => false,
        }
    }

    /// Weight of a token when computing a sentence centroid.
    #[inline]
    pub(super) fn token_weight(token: &str) -> f64 {
        if Tokenizer::is_function_word(token) { FUNCTION_WORD_WEIGHT } else { 1.0 }
    }
}
