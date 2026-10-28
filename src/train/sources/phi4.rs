use phiecs::phasor::facet::Facet;
use crate::train::sources::DictionarySource;
use crate::train::trainer::Trainer;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Phi4LearnSummary - metrics from learning the Phi-4 model references.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Phi4LearnSummary {
    pub vocab_tokens_loaded: usize,
    pub merges_trained: usize,
    pub doc_sentences_trained: usize,
    pub final_vocabulary_size: usize,
}

/// Phi4Source - extracts vocabulary, BPE token merges, and technical reasoning contexts
/// from the Phi-4 multimodal / instruct model files in the `refs/` directory.
pub struct Phi4Source {
    pub model_dir: PathBuf,
    pub refs_dir: PathBuf,
}

#[derive(Deserialize)]
struct VocabMap(HashMap<String, usize>);

impl Phi4Source {
    pub fn new(model_dir: &str) -> Self {
        let model_path = PathBuf::from(model_dir);
        let refs_path = model_path.parent().unwrap_or(&model_path).to_path_buf();
        Self {
            model_dir: model_path,
            refs_dir: refs_path,
        }
    }

    /// Automatically discovers the Phi-4 reference directory.
    pub fn discover() -> Self {
        let candidates = [
            "refs/Phi-4-multimodal-instruct",
            "../refs/Phi-4-multimodal-instruct",
            "phiano/refs/Phi-4-multimodal-instruct",
            "refs",
        ];

        for c in &candidates {
            if Path::new(c).exists() {
                return Self::new(c);
            }
        }

        Self::new("refs/Phi-4-multimodal-instruct")
    }

    /// Learns the complete Phi-4 model knowledge into the Phiano Facet:
    /// 1. Ingests and cleans 100,352 tiktoken vocabulary tokens
    /// 2. Trains Kuramoto phase resonance across BPE token merge pairs
    /// 3. Ingests curriculum and synthetic dataset sentences from Phi-4 tech reports & docs
    pub fn learn_into_facet(&self, facet: &mut Facet, trainer: &Trainer) -> Phi4LearnSummary {
        let mut tokens_loaded = 0;
        let mut merges_trained = 0;
        let mut docs_trained = 0;

        // 1. Ingest vocab.json
        let vocab_path = self.model_dir.join("vocab.json");
        if vocab_path.exists() {
            if let Ok(content) = fs::read_to_string(&vocab_path) {
                if let Ok(map) = serde_json::from_str::<VocabMap>(&content) {
                    for (token, _id) in map.0 {
                        let clean = token.replace('Ġ', "").replace('Ċ', "\n").trim().to_string();
                        if !clean.is_empty() && clean.len() >= 2 && clean.chars().all(|c| c.is_alphabetic() || c == '_' || c == '-') {
                            facet.get_or_init(&clean.to_lowercase());
                            tokens_loaded += 1;
                        }
                    }
                }
            }
        }

        // 2. Train BPE merges.txt (links root morphemes & suffixes)
        let merges_path = self.model_dir.join("merges.txt");
        if merges_path.exists() {
            if let Ok(content) = fs::read_to_string(&merges_path) {
                for line in content.lines().take(5000) { // Top 5,000 core merges
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() == 2 {
                        let p1 = parts[0].replace('Ġ', "").to_lowercase();
                        let p2 = parts[1].replace('Ġ', "").to_lowercase();
                        let combined = format!("{}{}", p1, p2);

                        if !p1.is_empty() && !p2.is_empty() {
                            let sentence = format!("{} {} {}", p1, p2, combined);
                            trainer.train_sentence(facet, &sentence);
                            merges_trained += 1;
                        }
                    }
                }
            }
        }

        // 3. Train from documentation and reference files
        let doc_files = [
            self.model_dir.join("data_summary_card.md"),
            self.model_dir.join("README.md"),
            self.model_dir.join("sample_inference_phi4mm.py"),
            self.refs_dir.join("glm-5.2.md"),
            self.refs_dir.join("phi4_rust_inference.rs"),
        ];

        for doc_file in &doc_files {
            if doc_file.exists() {
                if let Ok(content) = fs::read_to_string(doc_file) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        // Filter for clean prose or code comments
                        if !trimmed.is_empty() && !trimmed.starts_with("```") && trimmed.len() > 15 {
                            let clean_line = trimmed.trim_start_matches('#').trim_start_matches("//").trim();
                            trainer.train_sentence(facet, clean_line);
                            docs_trained += 1;
                        }
                    }
                }
            }
        }

        Phi4LearnSummary {
            vocab_tokens_loaded: tokens_loaded,
            merges_trained,
            doc_sentences_trained: docs_trained,
            final_vocabulary_size: facet.vocabulary_size(),
        }
    }
}

impl DictionarySource for Phi4Source {
    fn fetch_all(&self) -> Vec<(String, String)> {
        let vocab_path = self.model_dir.join("vocab.json");
        let mut entries = Vec::new();

        if let Ok(content) = fs::read_to_string(vocab_path) {
            if let Ok(map) = serde_json::from_str::<VocabMap>(&content) {
                for (token, id) in map.0 {
                    let clean = token.replace('Ġ', "").trim().to_string();
                    if !clean.is_empty() {
                        entries.push((clean.to_lowercase(), format!("Phi-4 tiktoken vocabulary item ID {}", id)));
                    }
                }
            }
        }
        entries
    }

    fn fetch_definitions(&self, word: &str) -> Vec<String> {
        let all = self.fetch_all();
        all.into_iter()
            .filter(|(w, _)| w == word)
            .map(|(_, d)| d)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phi4_source_discovery() {
        let source = Phi4Source::discover();
        assert!(!source.model_dir.as_os_str().is_empty());
    }
}

