//! Data preprocessing: tokenization, vectorization, and normalization.
//! Reuses existing Tokenizer and Facet infrastructure.

use phical::circle::Resultant;
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
        let mut wave = Resultant::new();
        for token in &tokens {
            if let Some(phasor) = facet.lexicon.get(token) {
                wave.add(phasor.amplitude, phasor.phase);
            }
        }
        (wave.x, wave.y)
    }

    /// Normalizes a phase-amplitude vector to unit length.
    pub fn normalize_wave(sum_x: f64, sum_y: f64) -> (f64, f64) {
        Resultant { x: sum_x, y: sum_y }.unit()
    }

    /// Computes the phase angle from a wave vector, in [0, 2π).
    pub fn wave_phase(sum_x: f64, sum_y: f64) -> f64 {
        Resultant { x: sum_x, y: sum_y }.direction_lifted()
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
