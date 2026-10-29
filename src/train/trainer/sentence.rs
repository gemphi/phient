#[allow(unused_imports)]
use super::*;
use phical::circle::Resultant;

impl Trainer {

    /// Trains on a single sentence.
    ///
    /// Steps:
    /// 1. Tokenize; initialize unseen tokens at identity-seeded phases
    /// 2. Record n-gram co-occurrences and directional phase lags
    /// 3. Pull each token toward the per-channel sentence centroid, plus a
    ///    directional syntactic neighbour lag on channel 0
    /// 4. Push sampled negatives away on the same channels
    /// 5. Update log-frequency amplitude; bump `band_n` for converged tokens
    ///
    /// Returns the number of tokens that were updated.
    pub fn train_sentence(&self, facet: &mut Facet, text: &str) -> usize {
        let tokens = Tokenizer::tokenize(text);
        if tokens.is_empty() {
            return 0;
        }

        self.initialize_tokens(facet, &tokens);
        self.record_ngrams(facet, &tokens);

        let n_tokens = tokens.len();
        let step_seed = fnv1a(text) ^ phistd::mix64(self.seed);

        // Which channels this step touches. Updating a rotating subset is
        // dropout-like regularisation and bounds the cost of a token update.
        let channels = self.update_channels(facet, step_seed);

        // Per-channel centroid of the sentence, weighted by amplitude and
        // down-weighting closed-class words. Function words appear in nearly
        // every sentence; at full weight they transitively couple the whole
        // vocabulary into one cluster.
        let mut targets = vec![0.0f64; channels.len()];
        for (ci, &k) in channels.iter().enumerate() {
            let mut centroid = Resultant::new();
            for t in &tokens {
                if let Some(p) = facet.lexicon.get(t) {
                    centroid.add(p.amplitude * Self::token_weight(t), p.theta(k));
                }
            }
            targets[ci] = centroid.direction();
        }

        // Channel-0 snapshot for the asymmetric syntactic term.
        let token_phases: Vec<f64> = tokens
            .iter()
            .map(|t| facet.lexicon.get(t).map(|p| p.theta(0)).unwrap_or(0.0))
            .collect();
        let beta_prev: Vec<f64> = tokens
            .iter()
            .enumerate()
            .map(|(i, t)| if i > 0 { facet.phase_lag(&tokens[i - 1], t) } else { 0.0 })
            .collect();
        let beta_next: Vec<f64> = tokens
            .iter()
            .enumerate()
            .map(|(i, t)| if i + 1 < n_tokens { facet.phase_lag(t, &tokens[i + 1]) } else { 0.0 })
            .collect();

        let mut updated = 0;
        for (i, token) in tokens.iter().enumerate() {
            let phasor = match facet.lexicon.get_mut(token) {
                Some(p) => p,
                None => continue,
            };

            // --- attraction: every touched channel toward its centroid ---
            let mut ch0_semantic = 0.0;
            for (ci, &k) in channels.iter().enumerate() {
                let err = (targets[ci] - phasor.theta(k)).sin();
                if k == 0 {
                    ch0_semantic = err;
                }
                phasor.nudge(k, self.learning_rate * err);
            }

            // --- channel 0 also carries word order (Sakaguchi lag) ---
            let mut syntax_force = 0.0;
            let mut syntax_neighbors = 0;
            if i > 0 {
                syntax_force += (token_phases[i - 1] - phasor.theta(0) + beta_prev[i]).sin();
                syntax_neighbors += 1;
            }
            if i + 1 < n_tokens {
                syntax_force += (token_phases[i + 1] - phasor.theta(0) - beta_next[i]).sin();
                syntax_neighbors += 1;
            }
            if syntax_neighbors > 0 {
                phasor.nudge(0, self.learning_rate * 0.3 * (syntax_force / syntax_neighbors as f64));
            }
            phasor.sync_phase();

            if ch0_semantic.abs() < CONVERGENCE_THRESHOLD {
                phasor.band_n += 1;
            }
            phasor.observe();
            updated += 1;
        }

        self.apply_negatives(facet, &tokens, &channels, step_seed);
        self.couple_relations(facet, &tokens);
        updated
    }

    /// Pushes sampled non-co-occurring words away from the sentence's centroid.
    pub(super) fn apply_negatives(
        &self,
        facet: &mut Facet,
        tokens: &[String],
        channels: &[usize],
        step_seed: u64,
    ) {
        if self.neg_samples == 0 {
            return;
        }
        facet.rebuild_sample_pool();

        // Collect the (word, channel, delta) triples first: sampling borrows the
        // facet immutably, applying borrows it mutably.
        let mut pushes: Vec<(String, usize, f64)> = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            let anchor = match facet.lexicon.get(token) {
                Some(p) => *p,
                None => continue,
            };
            for j in 0..self.neg_samples {
                let r = phistd::mix64(step_seed ^ ((i as u64) << 32) ^ (j as u64).wrapping_mul(0x2545F491));
                // Controlled negative sampling: when a definition graph is
                // attached, never draw a word that is definitionally related to
                // the anchor. A uniform draw will occasionally pick a word from
                // the anchor's own definition, and this update then pushes apart
                // exactly the pair the dictionary says belongs together — the
                // training signal working against itself. Dict2vec (Tissier et
                // al., EMNLP 2017) measures the filter firing on ~2% of draws.
                let sampled = match &self.definitions {
                    Some(g) => facet.sample_negative_controlled(r, token, |a, b| g.is_related(a, b), 8),
                    None => facet.sample_negative(r),
                };
                let neg = match sampled {
                    Some(w) if !tokens.iter().any(|t| t == w) => w.clone(),
                    _ => continue,
                };
                let negp = match facet.lexicon.get(&neg) {
                    Some(p) => p,
                    None => continue,
                };
                for &k in channels {
                    // gradient of −cos(Δ) — push the negative away from the anchor
                    let away = -(anchor.theta(k) - negp.theta(k)).sin();
                    pushes.push((neg.clone(), k, self.learning_rate * NEG_RATE * away));
                }
            }
        }

        for (word, k, delta) in pushes {
            if let Some(p) = facet.lexicon.get_mut(&word) {
                p.nudge(k, delta);
                if k == 0 {
                    p.sync_phase();
                }
            }
        }
    }
}
