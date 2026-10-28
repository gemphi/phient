//! Generalization: local (in-distribution) versus extreme (unfamiliar) material.
//!
//! The previous implementation selected test words by *phase distance* from the
//! training words, then scored template sentences by *phase coherence* — the
//! selection criterion and the measurement were the same quantity, so the
//! result was close to circular, and both halves were driven by the collapse
//! dynamic rather than by anything about generalization.
//!
//! Here the split is by **vocabulary coverage** (how much of a sentence the
//! model has ever seen) and the measurement is **held-out perplexity** — two
//! different quantities, neither of which can be improved by synchronising the
//! lexicon.

use phiecs::phasor::facet::Facet;
use crate::metrics::harness::PhianoLM;
use phiton::p64::words::Tokenizer;
use serde::{Deserialize, Serialize};

/// Coverage above which a sentence counts as in-distribution.
const LOCAL_COVERAGE: f64 = 0.8;
/// Coverage below which a sentence counts as genuinely unfamiliar.
const EXTREME_COVERAGE: f64 = 0.5;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneralizationReport {
    /// Perplexity on held-out sentences the model has vocabulary for.
    pub local_score: f64,
    /// Perplexity on held-out sentences containing mostly unfamiliar words.
    pub extreme_score: f64,
    /// ln(extreme) − ln(local). Large means the model interpolates but does not
    /// abstract; near zero means it degrades gracefully into new territory.
    pub gap: f64,
    pub n_local: usize,
    pub n_extreme: usize,
}

#[derive(Debug, Default)]
pub struct Generalization;

impl Generalization {
    /// Fraction of a sentence's tokens the model knows.
    fn coverage(facet: &Facet, sentence: &str) -> f64 {
        let toks = Tokenizer::tokenize(sentence);
        match toks.is_empty() {
            true => 0.0,
            false => {
                toks.iter().filter(|t| facet.contains_word(t)).count() as f64 / toks.len() as f64
            }
        }
    }

    /// Splits held-out text by coverage and measures perplexity on each half.
    pub fn assess(facet: &Facet, held_out: &[String]) -> GeneralizationReport {
        let local: Vec<String> = held_out
            .iter()
            .filter(|s| Self::coverage(facet, s) >= LOCAL_COVERAGE)
            .cloned()
            .collect();
        let extreme: Vec<String> = held_out
            .iter()
            .filter(|s| Self::coverage(facet, s) <= EXTREME_COVERAGE)
            .cloned()
            .collect();

        let lm = PhianoLM::with_gamma(facet, 0.0);
        let local_score = match local.is_empty() {
            true => f64::NAN,
            false => lm.perplexity(&local),
        };
        let extreme_score = match extreme.is_empty() {
            true => f64::NAN,
            false => lm.perplexity(&extreme),
        };

        let gap = match local_score.is_finite() && extreme_score.is_finite() {
            true => extreme_score.ln() - local_score.ln(),
            false => f64::NAN,
        };

        GeneralizationReport {
            local_score,
            extreme_score,
            gap,
            n_local: local.len(),
            n_extreme: extreme.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::train::trainer::Trainer;

    #[test]
    fn test_unfamiliar_material_is_harder() {
        let mut facet = Facet::new();
        let t = Trainer::new(0.05);
        for _ in 0..3 {
            for s in ["the cat sat on the mat", "the dog ran in the park"] {
                t.train_sentence(&mut facet, s);
            }
        }
        let held = vec![
            "the cat ran on the mat".to_string(),
            "quantum chromodynamics describes gluon confinement".to_string(),
        ];
        let r = Generalization::assess(&facet, &held);
        assert_eq!(r.n_local + r.n_extreme, 2, "both halves should be populated");
        if r.local_score.is_finite() && r.extreme_score.is_finite() {
            assert!(r.extreme_score >= r.local_score, "unfamiliar material should not be easier");
        }
    }
}
