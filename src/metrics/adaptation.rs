//! Adaptation efficiency: measures how many examples the model needs
//! to reach a competence threshold (Ch 14.3's efficiency metric).

use crate::eval::Evaluator;
use phiecs::phasor::facet::Facet;
use crate::train::trainer::Trainer;

#[derive(Debug, Default)]
pub struct Adaptation;

impl Adaptation {
    /// Measures adaptation efficiency: examples needed / max_examples.
    /// Lower = more efficient = more intelligent (per Ch 14.3).
    pub fn efficiency(
        facet: &mut Facet,
        trainer: &Trainer,
        task: &str,
        max_examples: usize,
    ) -> f64 {
        let evaluator = Evaluator::new();

        for n in 1..=max_examples {
            trainer.train_sentence(facet, task);

            let score = evaluator.eval(facet, task).coherence;
            if score > 0.8 {
                return n as f64 / max_examples as f64;
            }
        }

        1.0
    }

    /// Returns the number of examples needed to reach the threshold.
    pub fn examples_to_competence(
        facet: &mut Facet,
        trainer: &Trainer,
        task: &str,
        max_examples: usize,
        threshold: f64,
    ) -> usize {
        let evaluator = Evaluator::new();

        for n in 1..=max_examples {
            trainer.train_sentence(facet, task);
            let score = evaluator.eval(facet, task).coherence;
            if score >= threshold {
                return n;
            }
        }

        max_examples
    }
}
