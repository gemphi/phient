//! Adversarial robustness: phase perturbation tests and brittleness scoring.
//! Implements Ch 14.2's adversarial examples concept for phase-oscillator models.

use phiton::units::TWO_PI;
use crate::eval::Evaluator;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use std::f64::consts::PI;

#[derive(Debug, Default)]
pub struct Adversarial;

impl Adversarial {
    /// Perturbs a word's phase by delta and checks if evaluation changes drastically.
    /// Returns true if the model is robust to this perturbation.
    pub fn perturbation_test(facet: &Facet, word: &str, delta: f64) -> bool {
        let evaluator = Evaluator::new();

        let original_score = match facet.lexicon.get(word) {
            Some(_) => {
                let sentence = format!("{} is a concept", word);
                evaluator.eval(facet, &sentence).coherence
            }
            None => return true,
        };

        let mut perturbed = facet.clone();
        if let Some(p) = perturbed.lexicon.get_mut(word) {
            p.phase = (p.phase + delta).rem_euclid(TWO_PI);
        }

        let perturbed_score = {
            let sentence = format!("{} is a concept", word);
            evaluator.eval(&perturbed, &sentence).coherence
        };

        (original_score - perturbed_score).abs() < 0.1
    }

    /// Average sensitivity across n random perturbations.
    pub fn sensitivity(facet: &Facet, prompt: &str, n_perturbations: usize) -> f64 {
        let evaluator = Evaluator::new();
        let baseline = evaluator.eval(facet, prompt).coherence;

        let tokens = Tokenizer::tokenize(prompt);
        if tokens.is_empty() {
            return 0.0;
        }

        let mut total_delta = 0.0;
        for i in 0..n_perturbations {
            let delta = (i as f64 / n_perturbations as f64) * PI * 0.5;
            let word = tokens[i % tokens.len()].clone();

            let mut perturbed = facet.clone();
            if let Some(p) = perturbed.lexicon.get_mut(&word) {
                p.phase = (p.phase + delta).rem_euclid(TWO_PI);
            }

            let perturbed_score = evaluator.eval(&perturbed, prompt).coherence;
            total_delta += (baseline - perturbed_score).abs();
        }

        total_delta / n_perturbations as f64
    }

    /// Brittleness score: high = small perturbation causes large output change.
    pub fn brittleness(facet: &Facet, prompt: &str) -> f64 {
        let sensitivity = Self::sensitivity(facet, prompt, 8);
        sensitivity.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perturbation_robustness() {
        let mut facet = Facet::new();
        facet.get_or_init("rust");
        facet.get_or_init("code");
        facet.get_or_init("memory");

        let robust = Adversarial::perturbation_test(&facet, "rust", 0.01);
        let _ = robust;
    }
}
