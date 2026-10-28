use super::*;

impl Harness {
    /// Splits a corpus 80/10/10 deterministically.
    ///
    /// The shuffle is a hash-order sort, so the split is identical on every
    /// machine and every run — a split that moves between runs cannot support a
    /// comparison.
    pub fn split(sentences: Vec<String>, seed: u64) -> Split {
        let mut keyed: Vec<(u64, String)> = sentences
            .into_iter()
            .map(|s| (fnv1a(&s) ^ seed, s))
            .collect();
        keyed.sort_by_key(|(k, _)| *k);
        let s: Vec<String> = keyed.into_iter().map(|(_, v)| v).collect();

        let n = s.len();
        Split {
            train: s[..n * 80 / 100].to_vec(),
            valid: s[n * 80 / 100..n * 90 / 100].to_vec(),
            test: s[n * 90 / 100..].to_vec(),
        }
    }

    /// Builds a scoring model over a facet.
    pub fn language_model(facet: &Facet, use_phase: bool) -> PhianoLM<'_> {
        PhianoLM::new(facet, use_phase)
    }

    /// Held-out perplexity of the Phiano model, using the phase manifold as the
    /// back-off distribution.
    pub fn perplexity(facet: &Facet, sentences: &[String]) -> f64 {
        PhianoLM::new(facet, true).perplexity(sentences)
    }

    /// Held-out perplexity with the manifold replaced by a plain unigram
    /// back-off — identical counts, identical interpolation, phase removed.
    /// The difference between the two is the phase layer's contribution.
    pub fn perplexity_no_phase(facet: &Facet, sentences: &[String]) -> f64 {
        PhianoLM::new(facet, false).perplexity(sentences)
    }

    /// Mean coherence over a set of sentences.
    pub(super) fn mean_coherence(facet: &Facet, sentences: &[String]) -> f64 {
        if sentences.is_empty() {
            return 0.0;
        }
        let e = Evaluator::new();
        sentences.iter().map(|s| e.eval(facet, s).coherence).sum::<f64>() / sentences.len() as f64
    }

    /// Trains a facet with a clustering pass followed by several predictive
    /// passes per epoch.
    ///
    /// Centroid attraction teaches the manifold *what co-occurs*. Next-word
    /// ranking teaches it *what follows*. Only the second is the thing a
    /// language model is scored on, so this isolates its effect.
    pub fn train_predictive_heavy(
        split: &Split,
        trainer: &Trainer,
        epochs: usize,
        predictive_passes: usize,
    ) -> Facet {
        let mut facet = Facet::new();
        for _ in 0..epochs {
            for s in &split.train {
                trainer.train_sentence(&mut facet, s);
                for _ in 0..predictive_passes {
                    trainer.train_predictive(&mut facet, s);
                }
            }
        }
        facet
    }

    /// Trains the phase layer on the ranking objective alone.
    ///
    /// Centroid attraction teaches the manifold what co-occurs; next-word
    /// ranking teaches it what follows. Only the second is what a language model
    /// is scored on, and running both pulls the phases in two directions at
    /// once. This keeps the n-gram statistics (recorded either way) but trains
    /// the manifold on the ranking objective only.
    pub fn train_ranking_only(split: &Split, trainer: &Trainer, passes: usize) -> Facet {
        let mut facet = Facet::new();
        // One structural pass to populate n-grams and initialise the lexicon.
        let seed = Trainer { learning_rate: 0.0, neg_samples: 0, definitions: None, seed: trainer.seed, relations: false, rounding: Default::default() };
        for s in &split.train {
            seed.train_sentence(&mut facet, s);
        }
        if trainer.relations {
            let docs: Vec<Vec<String>> =
                split.train.iter().map(|s| phiton::p64::words::Tokenizer::tokenize(s)).collect();
            facet.learn_relations(&docs);
        }
        for _ in 0..passes {
            for s in &split.train {
                trainer.train_predictive(&mut facet, s);
                if trainer.relations {
                    let toks = phiton::p64::words::Tokenizer::tokenize(s);
                    trainer.couple_relations(&mut facet, &toks);
                }
            }
        }
        facet
    }

    /// Trains for `epochs` and records the full metric set each epoch.
    pub fn train_and_measure(
        split: &Split,
        trainer: &Trainer,
        epochs: usize,
        predictive: bool,
    ) -> (Facet, Vec<EpochMetric>) {
        let mut facet = Facet::new();
        let mut log = Vec::with_capacity(epochs);

        for epoch in 0..epochs {
            for s in &split.train {
                match predictive {
                    true => { trainer.train_full(&mut facet, s); }
                    false => { trainer.train_sentence(&mut facet, s); }
                }
            }

            log.push(EpochMetric {
                epoch,
                train_ppl: Self::perplexity(&facet, &split.train),
                valid_ppl: Self::perplexity(&facet, &split.valid),
                valid_ppl_no_phase: Self::perplexity_no_phase(&facet, &split.valid),
                coherence: Self::mean_coherence(&facet, &split.valid),
                phase_dispersion: facet.phase_dispersion(),
                band_dispersion: facet
                    .dispersion_top(crate::train::grounding::GUARD_BAND_TOP),
                sector_gini: facet.sector_gini(),
                vocab_size: facet.vocabulary_size(),
                mean_amplitude: facet.average_amplitude(),
            });
        }

        (facet, log)
    }
}
