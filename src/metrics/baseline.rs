//! Baseline language models, scored by held-out perplexity.
//!
//! These previously built an eight-word salad from the lexicon and scored its
//! *coherence* — the Kuramoto order parameter, which the training rule
//! maximises and which reads 1.0 on a fully synchronised lexicon for any input
//! at all. Three such baselines therefore converged on the same number as the
//! model degraded, and none of them could ever get worse.
//!
//! A baseline has to be something the model can lose to. These are:
//!
//! * **uniform** — every word equally likely. The floor.
//! * **unigram** — word frequency alone. Trivially available, and strong.
//! * **Kneser-Ney trigram** — the standard smoothed n-gram model.
//! * **Phiano (counts)** — its own tables, absolute-discounted, unigram back-off.
//! * **Phiano (phase)** — the same, with the manifold as the back-off.
//!
//! All five are measured on text never trained on, so all five can go up.

pub mod kn;

use phiecs::phasor::facet::Facet;
use crate::metrics::harness::{Harness, PhianoLM, Split};
use crate::metrics::baseline::kn::KneserNey;
use phiton::p64::words::Tokenizer;
use serde::{Deserialize, Serialize};

/// Held-out perplexity of every baseline. Lower is better throughout.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BaselineScore {
    pub n_heldout: usize,
    pub uniform_ppl: f64,
    pub unigram_ppl: f64,
    pub kn_trigram_ppl: f64,
    /// Phiano's own n-gram tables with a unigram back-off (γ = 0).
    pub phiano_counts_ppl: f64,
    /// The same, with the phase manifold as the back-off (γ = 1).
    pub phiano_phase_ppl: f64,
}

impl BaselineScore {
    /// True when the model beats the standard smoothed n-gram baseline.
    pub fn beats_kn(&self) -> bool {
        self.phiano_counts_ppl.min(self.phiano_phase_ppl) < self.kn_trigram_ppl
    }

    /// True when the phase manifold improves on a unigram back-off.
    pub fn phase_helps(&self) -> bool {
        self.phiano_phase_ppl < self.phiano_counts_ppl
    }

    /// Fraction of the uniform→unigram information gap the manifold recovers.
    ///
    /// Uniform is the floor and unigram frequency is a strong, trivially
    /// available reference; the share of that gap the phase distribution closes
    /// is a scale-free measure of what the representation knows.
    pub fn phase_signal_recovered(&self) -> f64 {
        let denom = self.uniform_ppl.ln() - self.unigram_ppl.ln();
        match denom.abs() < 1e-9 {
            true => 0.0,
            false => (self.uniform_ppl.ln() - self.phiano_phase_ppl.ln()) / denom,
        }
    }
}

#[derive(Debug, Default)]
pub struct BaselineSet;

impl BaselineSet {
    /// Default held-out corpus path.
    pub const CORPUS: &'static str = "data/rust_book_corpus.txt";

    /// Loads and splits the evaluation corpus, if it is present on disk.
    pub fn load_split() -> Option<Split> {
        let raw = std::fs::read_to_string(Self::CORPUS).ok()?;
        let corpus: Vec<String> = Tokenizer::split_sentences(&raw)
            .into_iter()
            .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|s| Tokenizer::tokenize(s).len() >= 4)
            .collect();
        match corpus.len() < 20 {
            true => None,
            false => Some(Harness::split(corpus, 42)),
        }
    }

    /// Perplexity of a fixed distribution over the vocabulary.
    fn fixed_ppl(facet: &Facet, held_out: &[String], uniform: bool) -> f64 {
        let v = facet.vocabulary_size().max(1) as f64;
        let total: f64 = facet.lexicon.values().map(|p| p.count.max(1) as f64).sum();
        let floor = 1.0 / (v * 100.0);

        let mut log_sum = 0.0f64;
        let mut n = 0usize;
        for sentence in held_out {
            for token in Tokenizer::tokenize(sentence) {
                let p = match uniform {
                    true => 1.0 / v,
                    false => facet
                        .lexicon
                        .get(&token)
                        .map(|ph| ph.count.max(1) as f64 / total)
                        .unwrap_or(floor),
                };
                log_sum += p.max(floor).ln();
                n += 1;
            }
        }
        match n {
            0 => f64::INFINITY,
            _ => (-log_sum / n as f64).exp(),
        }
    }

    /// Scores every baseline against a held-out split.
    pub fn suite(facet: &Facet, split: &Split) -> BaselineScore {
        BaselineScore {
            n_heldout: split.valid.len(),
            uniform_ppl: Self::fixed_ppl(facet, &split.valid, true),
            unigram_ppl: Self::fixed_ppl(facet, &split.valid, false),
            kn_trigram_ppl: KneserNey::train(&split.train).perplexity(&split.valid),
            phiano_counts_ppl: PhianoLM::with_gamma(facet, 0.0).perplexity(&split.valid),
            phiano_phase_ppl: PhianoLM::with_gamma(facet, 1.0).perplexity(&split.valid),
        }
    }

    /// Scores every baseline, loading the corpus from disk.
    /// Returns `None` when no evaluation corpus is available.
    pub fn suite_from_disk(facet: &Facet) -> Option<BaselineScore> {
        Self::load_split().map(|split| Self::suite(facet, &split))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::train::trainer::Trainer;

    fn toy() -> (Facet, Split) {
        let mut corpus = Vec::new();
        for s in ["the cat", "the dog", "a bird"] {
            for v in ["sat on", "ran to", "looked at"] {
                for o in ["the mat", "the park", "a fence"] {
                    corpus.push(format!("{} {} {}", s, v, o));
                }
            }
        }
        let split = Harness::split(corpus, 42);
        let mut facet = Facet::new();
        let t = Trainer::new(0.05);
        for s in &split.train {
            t.train_sentence(&mut facet, s);
        }
        (facet, split)
    }

    #[test]
    fn test_baselines_are_finite_and_ordered() {
        let (facet, split) = toy();
        let b = BaselineSet::suite(&facet, &split);
        for x in [b.uniform_ppl, b.unigram_ppl, b.kn_trigram_ppl, b.phiano_counts_ppl] {
            assert!(x.is_finite() && x > 0.0, "non-finite baseline: {}", x);
        }
        // Knowing word frequency must beat knowing nothing.
        assert!(b.unigram_ppl < b.uniform_ppl, "unigram {} vs uniform {}", b.unigram_ppl, b.uniform_ppl);
    }

    #[test]
    fn test_baselines_can_get_worse() {
        // The property the old coherence baselines lacked: an untrained model
        // must score worse than a trained one.
        let (trained, split) = toy();
        let untrained = Facet::new();
        let a = BaselineSet::suite(&trained, &split);
        let b = BaselineSet::suite(&untrained, &split);
        assert!(
            a.phiano_counts_ppl < b.phiano_counts_ppl,
            "trained {} should beat untrained {}",
            a.phiano_counts_ppl, b.phiano_counts_ppl
        );
    }
}
