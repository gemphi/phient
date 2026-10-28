//! Meta-learning: extracts common structure across tasks and uses it to
//! warm-start a new one.
//!
//! The prior is a **circular mean** of each word's phase across the tasks it
//! appeared in, and `adapt` actually applies it. The previous version sorted
//! raw phase values numerically — which breaks the circle, since 0.01 and 6.27
//! are adjacent on the manifold and maximally distant in a sorted list — took
//! the interquartile slice of that, stored it as `common_phases`, and then
//! never read it: `adapt` used only the adaptation rates.

use crate::eval::Evaluator;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use crate::train::trainer::Trainer;
use serde::Serialize;
use std::collections::HashMap;

/// How far a word is pulled toward its prior before training begins.
const WARM_START_STRENGTH: f64 = 0.25;

#[derive(Debug, Clone, Serialize)]
pub struct MetaModel {
    /// word → circular-mean phase across every task that used it.
    pub prior: Vec<(String, f64)>,
    /// Coherence gain observed per task, used to pace the next adaptation.
    pub adaptation_rates: Vec<f64>,
    pub n_tasks: usize,
}

#[derive(Debug, Default)]
pub struct MetaLearner;

impl MetaLearner {
    /// Trains on several tasks and extracts a per-word phase prior.
    pub fn learn(facet: &mut Facet, trainer: &Trainer, tasks: &[String]) -> MetaModel {
        let mut observed: HashMap<String, Vec<f64>> = HashMap::new();
        let mut adaptation_rates = Vec::with_capacity(tasks.len());
        let evaluator = Evaluator::new();

        for task in tasks {
            let before = evaluator.eval(facet, task).coherence;
            for _ in 0..16 {
                trainer.train_sentence(facet, task);
            }
            let after = evaluator.eval(facet, task).coherence;
            adaptation_rates.push(after - before);

            for token in Tokenizer::content_words(task) {
                if let Some(p) = facet.lexicon.get(&token) {
                    observed.entry(token).or_default().push(p.theta(0));
                }
            }
        }

        let prior = observed
            .into_iter()
            .map(|(w, phases)| (w, Self::circular_mean(&phases)))
            .collect();

        MetaModel { prior, adaptation_rates, n_tasks: tasks.len() }
    }

    /// Mean direction of a set of angles. Averaging angles numerically is wrong
    /// on a circle; averaging their unit vectors is not.
    pub fn circular_mean(phases: &[f64]) -> f64 {
        let (x, y) = phases
            .iter()
            .fold((0.0f64, 0.0f64), |(a, b), p| (a + p.cos(), b + p.sin()));
        y.atan2(x).rem_euclid(phiton::units::TWO_PI)
    }
}

impl MetaModel {
    /// Warm-starts a new task from the prior, then trains at an adapted rate.
    pub fn adapt(&self, facet: &mut Facet, trainer: &Trainer, new_task: &str) -> usize {
        for token in Tokenizer::tokenize(new_task) {
            facet.get_or_init(&token);
        }

        // Warm start: pull known words toward what they meant across prior tasks.
        let mut warmed = 0usize;
        for (word, target) in &self.prior {
            if let Some(p) = facet.lexicon.get_mut(word) {
                let d = phistd::wrap(target - p.theta(0));
                p.nudge(0, WARM_START_STRENGTH * d);
                p.sync_phase();
                warmed += 1;
            }
        }

        let avg_rate = match self.adaptation_rates.is_empty() {
            true => 0.05,
            false => {
                self.adaptation_rates.iter().sum::<f64>() / self.adaptation_rates.len() as f64
            }
        };

        let effective_lr = (0.05 + avg_rate.abs() * 0.1).min(0.15);
        let effective_epochs = (16.0 + avg_rate * 100.0).max(8.0) as usize;

        let adapted = Trainer { learning_rate: effective_lr, neg_samples: trainer.neg_samples, definitions: trainer.definitions.clone(), seed: trainer.seed, relations: trainer.relations, rounding: trainer.rounding };
        for _ in 0..effective_epochs {
            adapted.train_sentence(facet, new_task);
        }

        warmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_mean_wraps_correctly() {
        // 0.1 and 2π−0.1 are adjacent on the circle; their mean is ~0, not ~π.
        let m = MetaLearner::circular_mean(&[0.1, phiton::units::TWO_PI - 0.1]);
        assert!(!(0.05..=phiton::units::TWO_PI - 0.05).contains(&m), "got {}", m);
    }

    #[test]
    fn test_adapt_consumes_the_prior() {
        let mut facet = Facet::new();
        let t = Trainer::new(0.05);
        let tasks = vec!["sort the list".to_string(), "sort the array".to_string()];
        let model = MetaLearner::learn(&mut facet, &t, &tasks);
        assert!(!model.prior.is_empty(), "a prior must actually be built");

        let warmed = model.adapt(&mut facet, &t, "sort the vector");
        assert!(warmed > 0, "adapt must apply the prior, not ignore it");
    }
}
