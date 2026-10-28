#[allow(unused_imports)]
use super::*;

impl Trainer {

    pub(super) fn learn_chain_recursive(
        &self,
        facet: &mut Facet,
        chunk_store: &crate::train::chunker::ChunkStore,
        word: &str,
        depth_left: usize,
        learned: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if depth_left == 0 || visited.contains(word) {
            return;
        }
        visited.insert(word.to_string());

        // Skip words already well known. The previous threshold was 5.0, which
        // AMPLITUDE_MAX (2.0) makes unreachable, so this branch never fired and
        // known words were re-trained on every call.
        if let Some(phasor) = facet.lexicon.get(word) {
            if phasor.amplitude >= AMPLITUDE_MAX * 0.9 {
                return;
            }
        }

        let definition = match chunk_store.load_definition(word) {
            Some(d) => d,
            None => return,
        };

        self.train_definition(facet, word, &definition);
        learned.push(word.to_string());

        let def_tokens = Tokenizer::tokenize(&definition);
        for token in &def_tokens {
            if !facet.lexicon.contains_key(token) && !visited.contains(token) {
                self.learn_chain_recursive(facet, chunk_store, token, depth_left - 1, learned, visited);
            }
        }
    }

    /// Multi-epoch training with warmup and convergence detection.
    pub fn train_multi_epoch(
        &self,
        facet: &mut Facet,
        text: &str,
        max_epochs: usize,
        warmup: usize,
    ) -> MultiEpochResult {
        let tokens = Tokenizer::tokenize(text);
        if tokens.is_empty() {
            return MultiEpochResult { epochs: 0, tokens_learned: 0, converged: false };
        }

        self.initialize_tokens(facet, &tokens);
        self.record_ngrams(facet, &tokens);

        let mut converged = false;
        let mut epochs_done = 0;

        for epoch in 0..max_epochs {
            let effective_lr = if epoch < warmup && warmup > 0 {
                self.learning_rate * (epoch as f64 + 1.0) / warmup as f64
            } else {
                self.learning_rate
            };

            let target_phase = self.compute_centroid_phase(facet, &tokens);
            let mut max_shift = 0.0f64;

            for token in &tokens {
                if let Some(phasor) = facet.lexicon.get_mut(token) {
                    let phase_error = (target_phase - phasor.theta(0)).sin();
                    let shift = effective_lr * phase_error;
                    phasor.nudge(0, shift);
                    phasor.sync_phase();
                    max_shift = max_shift.max(shift.abs());

                    if phase_error.abs() < CONVERGENCE_THRESHOLD {
                        phasor.band_n += 1;
                    }
                    phasor.observe();
                }
            }

            let epoch_trainer = Self { learning_rate: effective_lr, neg_samples: self.neg_samples, definitions: self.definitions.clone(), seed: self.seed, relations: self.relations, rounding: self.rounding };
            let ch: Vec<usize> = (0..CHANNELS_PER_UPDATE.min(PHASE_CHANNELS)).collect();
            epoch_trainer.apply_negatives(facet, &tokens, &ch, fnv1a(text) ^ epoch as u64);
            epoch_trainer.couple_relations(facet, &tokens);

            epochs_done = epoch + 1;
            if max_shift < CONVERGENCE_THRESHOLD {
                converged = true;
                break;
            }
        }

        MultiEpochResult { epochs: epochs_done, tokens_learned: tokens.len(), converged }
    }
}
