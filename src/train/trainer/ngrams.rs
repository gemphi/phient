#[allow(unused_imports)]
use super::*;

impl Trainer {

    /// Records bigram, trigram and directional phase-lag statistics.
    pub(super) fn record_ngrams(&self, facet: &mut Facet, tokens: &[String]) {
        for window in tokens.windows(2) {
            facet.record_bigram(&window[0], &window[1]);
            // Anchor the lag to corpus order statistics rather than to the
            // phases the lag itself is moving, so the loop has a fixed point
            // outside its own geometry.
            let target = facet.target_phase_lag(&window[0], &window[1]);
            facet.record_phase_lag(&window[0], &window[1], target);
        }
        for window in tokens.windows(3) {
            facet.record_trigram(&window[0], &window[1], &window[2]);
        }
    }

    /// In-chat self-correction: suppresses a wrong association and reinforces
    /// the corrected one.
    ///
    /// Only tokens that appear in the wrong phrase and *not* in the correction
    /// are pushed: a word present in both (typically a function word) is not the
    /// thing that was wrong, and inverting it degrades the model globally to fix
    /// one specific fact.
    pub fn correct_mistake(&self, facet: &mut Facet, wrong_phrase: &str, correct_phrase: &str) {
        let wrong_tokens = Tokenizer::tokenize(wrong_phrase);
        let correct_tokens = Tokenizer::tokenize(correct_phrase);

        for token in &wrong_tokens {
            if correct_tokens.contains(token) {
                continue;
            }
            if let Some(phasor) = facet.lexicon.get_mut(token) {
                phasor.phase = (phasor.phase + PHASE_REPULSION).rem_euclid(TWO_PI);
                phasor.sync_channel0();
                phasor.amplitude = (phasor.amplitude * 0.8).max(CORRECTION_FLOOR);
            }
        }

        self.train_sentence(facet, correct_phrase);
    }

    /// Graded correction: rotates the offending tokens away from the corrected
    /// meaning by `strength` radians rather than by a full π.
    ///
    /// A π pulse is the maximum possible rotation, so it destroys every *other*
    /// association the word had in order to fix one. Most corrections want a
    /// nudge.
    pub fn correct_graded(
        &self,
        facet: &mut Facet,
        wrong_phrase: &str,
        correct_phrase: &str,
        strength: f64,
    ) {
        let wrong_tokens = Tokenizer::tokenize(wrong_phrase);
        let correct_tokens = Tokenizer::tokenize(correct_phrase);
        let offenders: Vec<String> = wrong_tokens
            .iter()
            .filter(|t| !correct_tokens.contains(t))
            .cloned()
            .collect();

        let target = self.compute_centroid_phase(facet, &correct_tokens);
        for token in &offenders {
            if let Some(p) = facet.lexicon.get_mut(token) {
                let away = -(target - p.theta(0)).sin();
                p.nudge(0, strength * away);
                p.sync_phase();
            }
        }
        self.train_sentence(facet, correct_phrase);
    }

    /// Initializes unseen tokens at identity-seeded phases.
    pub(super) fn initialize_tokens(&self, facet: &mut Facet, tokens: &[String]) {
        for token in tokens {
            let entry = facet
                .lexicon
                .entry(token.clone())
                .or_insert_with(|| SpectralPhasor::seeded(token, AMPLITUDE_INITIAL, BAND_N_INITIAL));
            entry.ensure_channels(token);
        }
    }

    /// Computes the amplitude-weighted centroid phase across all tokens
    /// (channel 0), down-weighting closed-class words.
    pub(super) fn compute_centroid_phase(&self, facet: &Facet, tokens: &[String]) -> f64 {
        let (mut sum_x, mut sum_y) = (0.0f64, 0.0f64);
        for token in tokens {
            if let Some(phasor) = facet.lexicon.get(token) {
                let w = phasor.amplitude * Self::token_weight(token);
                sum_x += phasor.theta(0).cos() * w;
                sum_y += phasor.theta(0).sin() * w;
            }
        }
        sum_y.atan2(sum_x)
    }

    /// Trains on a single sentence in online mode (single pass).
    pub fn train_online(&self, facet: &mut Facet, text: &str) -> usize {
        self.train_sentence(facet, text)
    }

    /// Batch-trains a corpus of sentences. Returns total token updates.
    #[allow(dead_code)]
    pub fn train_corpus(&self, facet: &mut Facet, sentences: &[String]) -> usize {
        self.ensure_relations(facet, sentences);
        sentences.iter().map(|s| self.train_sentence(facet, s)).sum()
    }

    /// Batch-trains a corpus on the recommended ranking path ([`Trainer::train`]),
    /// learning the relation inventory first. Returns total tokens seen.
    pub fn train_corpus_ranking(&self, facet: &mut Facet, sentences: &[String]) -> usize {
        self.ensure_relations(facet, sentences);
        sentences.iter().map(|s| self.train(facet, s)).sum()
    }

    /// Trains a corpus, then lets the sentences constrain the words.
    ///
    /// [`Trainer::train_corpus`] is the bottom-up half: words move to predict
    /// their neighbours, a sentence is whatever sum its words produce, and no
    /// level above ever pushes back. On the dictionary corpus that manifold
    /// measures **at its own null** on every relational probe — analogy, type
    /// discovery, harmonic modes, all of them.
    ///
    /// [`crate::train::topdown::Descent`] is the missing direction, and it is worth
    /// three-fold on analogy MRR (0.00247 to 0.00814, four times its shuffled
    /// null) and the difference between no relation types at any k and types at
    /// 15x their noise floor. Until now it existed only inside experiment
    /// binaries, so the facet this project actually ships was the arm that does
    /// not work.
    ///
    /// The guard is kept: if the downward pass collapses the frequent band, the
    /// composed manifold is discarded and the bottom-up one is returned
    /// unchanged. `rejected` on the report says which happened, and a caller
    /// that ignores it will silently ship the control.
    pub fn train_corpus_with_descent(
        &self,
        facet: &mut Facet,
        sentences: &[String],
    ) -> (usize, crate::train::topdown::DescentReport) {
        let tokens = self.train_corpus(facet, sentences);
        let toks: Vec<Vec<String>> = sentences
            .iter()
            .map(|s| phiton::p64::words::Tokenizer::tokenize(s))
            .collect();
        let report = crate::train::topdown::Descent::cycle(
            facet,
            &toks,
            crate::train::topdown::Up::Bag,
            crate::train::topdown::DEFAULT_ROUNDS,
            crate::train::topdown::DEFAULT_STEP,
            true,
        );
        (tokens, report)
    }

    /// Trains on a word-definition pair - the core learning unit.
    pub fn train_definition(&self, facet: &mut Facet, word: &str, definition: &str) {
        let combined = format!("{} {}", word, definition);
        self.train_sentence(facet, &combined);
    }

    /// Recursively learns a word and its definition chain.
    pub fn learn_definition_chain(
        &self,
        facet: &mut Facet,
        chunk_store: &crate::train::chunker::ChunkStore,
        word: &str,
        max_depth: usize,
    ) -> Vec<String> {
        let mut learned: Vec<String> = Vec::new();
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        self.learn_chain_recursive(facet, chunk_store, word, max_depth, &mut learned, &mut visited);
        learned
    }
}
