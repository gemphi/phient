//! (`phient::train::synthetic` — it was phiano's `synthetic`.)

use crate::eval::Evaluator;
use phiecs::phasor::facet::Facet;
use crate::train::trainer::Trainer;

/// ContrastPair - a pair of terms used for fine-tuning phase separation and attraction.
#[derive(Debug, Clone)]
pub struct ContrastPair {
    pub term_a: String,
    pub term_b: String,
    pub relationship: &'static str, // "synonym", "contrast", "co_occur"
}

/// SyntheticGenerator - generates self-curated training data (Phase 5).
pub struct SyntheticGenerator;

impl SyntheticGenerator {
    /// Generates synthetic sentence variations around a seed word using nearest phasors.
    pub fn generate_synthetic_sentence(facet: &Facet, seed_word: &str) -> Option<String> {
        let phasor = facet.lexicon.get(seed_word)?;
        let target_phase = phasor.phase;

        let mut resonant_words: Vec<(String, f64)> = facet
            .lexicon
            .iter()
            .map(|(word, p)| {
                let diff = phical::circle::distance(p.phase, target_phase);
                (word.clone(), diff)
            })
            .collect();

        resonant_words.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_words: Vec<String> = resonant_words
            .into_iter()
            .take(5)
            .map(|(w, _)| w)
            .collect();

        if top_words.len() >= 3 {
            Some(format!("{} is related to {}", seed_word, top_words[1..].join(" ")))
        } else {
            None
        }
    }

    /// Generates contrast pairs for fine-tuning phase distinction
    pub fn generate_contrast_pairs(facet: &Facet) -> Vec<ContrastPair> {
        let mut pairs = Vec::new();
        let words: Vec<String> = facet.lexicon.keys().cloned().collect();

        for i in 0..words.len().min(50) {
            for j in (i + 1)..words.len().min(50) {
                let w1 = &words[i];
                let w2 = &words[j];
                if let (Some(p1), Some(p2)) = (facet.lexicon.get(w1), facet.lexicon.get(w2)) {
                    let diff = phical::circle::distance(p1.phase, p2.phase);
                    if diff < 0.5 {
                        pairs.push(ContrastPair {
                            term_a: w1.clone(),
                            term_b: w2.clone(),
                            relationship: "synonym",
                        });
                    } else if diff > 2.5 {
                        pairs.push(ContrastPair {
                            term_a: w1.clone(),
                            term_b: w2.clone(),
                            relationship: "contrast",
                        });
                    }
                }
            }
        }
        pairs
    }
}

/// SyntheticCurriculumPipeline - manages multi-stage synthetic data generation, filtering, and retraining.
pub struct SyntheticCurriculumPipeline {
    pub min_coherence: f64, // Default: 0.45
    pub min_resonance: f64, // Default: 0.70
}

impl SyntheticCurriculumPipeline {
    pub fn new(min_coherence: f64, min_resonance: f64) -> Self {
        Self {
            min_coherence,
            min_resonance,
        }
    }

    /// Runs the complete synthetic data generation and quality filtering pipeline.
    /// Returns the number of high-quality synthetic sentences accepted and trained on.
    pub fn run_pipeline(&self, facet: &mut Facet, trainer: &Trainer) -> usize {
        let evaluator = Evaluator::new();
        let vocab: Vec<String> = facet.lexicon.keys().cloned().collect();
        let mut synthetic_corpus = Vec::new();

        // 1. Generate synthetic sentences
        for word in &vocab {
            if let Some(sent) = SyntheticGenerator::generate_synthetic_sentence(facet, word) {
                synthetic_corpus.push(sent);
            }
        }

        // 2. Filter quality (Coherence + Resonance threshold)
        let mut accepted_sentences = Vec::new();
        for sent in synthetic_corpus {
            let eval_res = evaluator.eval(facet, &sent);
            if eval_res.coherence >= self.min_coherence && eval_res.resonance >= self.min_resonance {
                accepted_sentences.push(sent);
            }
        }

        // 3. Re-train manifold on high-quality synthetic corpus
        let count = accepted_sentences.len();
        for sent in &accepted_sentences {
            trainer.train_sentence(facet, sent);
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_pipeline() {
        let mut facet = Facet::new();
        let trainer = Trainer::new(0.15);

        facet.get_or_init("rust");
        facet.get_or_init("ownership");
        facet.get_or_init("borrowing");

        let pipeline = SyntheticCurriculumPipeline::new(0.1, 0.1);
        let accepted = pipeline.run_pipeline(&mut facet, &trainer);

        assert!(accepted > 0);
    }
}
