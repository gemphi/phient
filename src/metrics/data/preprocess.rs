//! Data preprocessing: tokenization, vectorization, and normalization.
//! Reuses existing Tokenizer and Facet infrastructure.

use phiton::units::TWO_PI;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;

#[derive(Debug, Default)]
pub struct Preprocessor;

impl Preprocessor {
    /// Preprocesses raw text into cleaned tokens.
    pub fn text(text: &str) -> Vec<String> {
        let mut tokens = Tokenizer::tokenize(text);
        tokens.retain(|t| t.len() >= 2);
        tokens
    }

    /// Vectorizes tokens into phase-amplitude pairs using the facet.
    /// Returns Vec<(phase, amplitude)> for each known token.
    pub fn vectorize(facet: &Facet, tokens: &[String]) -> Vec<(f64, f64)> {
        tokens
            .iter()
            .filter_map(|t| {
                facet.lexicon.get(t).map(|p| (p.phase, p.amplitude))
            })
            .collect()
    }

    /// Converts a sentence into a complex wave vector (sum_x, sum_y).
    pub fn sentence_to_wave(facet: &Facet, text: &str) -> (f64, f64) {
        let tokens = Self::text(text);
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        for token in &tokens {
            if let Some(phasor) = facet.lexicon.get(token) {
                sum_x += phasor.amplitude * phasor.phase.cos();
                sum_y += phasor.amplitude * phasor.phase.sin();
            }
        }
        (sum_x, sum_y)
    }

    /// Normalizes a phase-amplitude vector to unit length.
    pub fn normalize_wave(sum_x: f64, sum_y: f64) -> (f64, f64) {
        let norm = (sum_x * sum_x + sum_y * sum_y).sqrt();
        if norm > 0.0 {
            (sum_x / norm, sum_y / norm)
        } else {
            (0.0, 0.0)
        }
    }

    /// Computes the phase angle from a wave vector.
    pub fn wave_phase(sum_x: f64, sum_y: f64) -> f64 {
        let angle = sum_y.atan2(sum_x);
        if angle < 0.0 {
            angle + TWO_PI
        } else {
            angle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_text() {
        let tokens = Preprocessor::text("Hello, World! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
    }

    #[test]
    fn test_vectorize() {
        let mut facet = Facet::new();
        facet.get_or_init("rust");
        facet.get_or_init("code");

        let tokens = vec!["rust".to_string(), "code".to_string()];
        let vec = Preprocessor::vectorize(&facet, &tokens);
        assert_eq!(vec.len(), 2);
    }
}
