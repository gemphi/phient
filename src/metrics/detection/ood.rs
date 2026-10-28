//! Out-of-distribution detection: flags inputs far from training distribution.

use phiton::units::TWO_PI;
use crate::metrics::data::preprocess::Preprocessor;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use std::f64::consts::PI;

#[derive(Debug, Default)]
pub struct OodDetector;

impl OodDetector {
    /// Returns true if the prompt is out-of-distribution (distance > threshold).
    pub fn is_ood(facet: &Facet, prompt: &str, threshold: f64) -> bool {
        Self::score(facet, prompt) > threshold
    }

    /// Continuous OOD score [0, 1] where 1 = maximally out-of-distribution.
    /// Computes distance from the prompt's context wave to the facet centroid.
    pub fn score(facet: &Facet, prompt: &str) -> f64 {
        if facet.lexicon.is_empty() {
            return 1.0;
        }

        let (prompt_x, prompt_y) = Preprocessor::sentence_to_wave(facet, prompt);
        let prompt_phase = prompt_y.atan2(prompt_x).rem_euclid(TWO_PI);

        let mut centroid_x = 0.0;
        let mut centroid_y = 0.0;
        for phasor in facet.lexicon.values() {
            centroid_x += phasor.amplitude * phasor.phase.cos();
            centroid_y += phasor.amplitude * phasor.phase.sin();
        }
        let centroid_phase = centroid_y.atan2(centroid_x).rem_euclid(TWO_PI);

        let mut diff = (prompt_phase - centroid_phase).abs();
        if diff > PI {
            diff = TWO_PI - diff;
        }

        diff / PI
    }

    /// Returns the fraction of prompt tokens that are unknown to the facet.
    pub fn unknown_fraction(facet: &Facet, prompt: &str) -> f64 {
        let tokens = Tokenizer::tokenize(prompt);
        if tokens.is_empty() {
            return 1.0;
        }
        let unknown = tokens.iter().filter(|t| !facet.lexicon.contains_key(*t)).count();
        unknown as f64 / tokens.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ood_score() {
        let mut facet = Facet::new();
        facet.get_or_init("rust");
        facet.get_or_init("code");
        facet.get_or_init("memory");

        let in_dist = OodDetector::score(&facet, "rust code memory");
        let out_dist = OodDetector::score(&facet, "quantum entanglement relativity");

        assert!((0.0..=1.0).contains(&in_dist));
        assert!((0.0..=1.0).contains(&out_dist));
    }
}
