use phiecs::phasor::facet::Facet;
use crate::train::sources::DictionarySource;
use crate::train::trainer::Trainer;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The directory [`Phi4Source::discover`] falls back to when no candidate
/// exists on disk, and the one [`Phi4Source::discover`] reads when the search
/// finds nothing. Carried as a `&str` so `discover` can hand it to `at`.
pub const DEFAULT_DIR: &str = "refs/Phi-4-multimodal-instruct";

/// The vocabulary file every Phi-4 ingest is anchored on. Its absence is the
/// one condition that makes an ingest report nothing read (Requirement 3.15).
const VOCAB_FILE: &str = "vocab.json";

/// Phi4Error - the two ways a directory-pinned ingest can refuse before it
/// touches a facet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phi4Error {
    /// The directory given held no `vocab.json`; the path looked for is named
    /// so a caller can report exactly where the ingest went looking.
    NoVocabulary { looked_for: PathBuf },
    /// A file that was present could not be read; the path and the reason are
    /// carried so the Recorder can attribute the failure.
    Unreadable { path: PathBuf, reason: String },
}

impl std::fmt::Display for Phi4Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phi4Error::NoVocabulary { looked_for } => {
                write!(f, "no Phi-4 vocabulary at {}", looked_for.display())
            }
            Phi4Error::Unreadable { path, reason } => {
                write!(f, "cannot read {}: {}", path.display(), reason)
            }
        }
    }
}

impl std::error::Error for Phi4Error {}

/// Phi4Summary - what a finished ingest reports: the count of words read, the
/// count of merges read, the sentences trained, the final vocabulary size, and
/// the name of the facet the words landed in (Requirement 3.16).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Phi4Summary {
    pub vocab_tokens_loaded: usize,
    pub merges_trained: usize,
    pub doc_sentences_trained: usize,
    pub final_vocabulary_size: usize,
    pub facet: String,
}

/// Phi4Source - extracts vocabulary, BPE token merges, and technical reasoning
/// contexts from the Phi-4 model files in one directory.
///
/// Construct it with [`Phi4Source::at`] to read exactly the directory given, or
/// with [`Phi4Source::discover`] to search the usual candidates.
pub struct Phi4Source {
    pub model_dir: PathBuf,
}

#[derive(Deserialize)]
struct VocabMap(HashMap<String, usize>);

impl Phi4Source {
    /// Pin an ingest to `dir` alone. Reads the vocabulary and merge files from
    /// `dir` and no file outside it (Requirement 3.14).
    ///
    /// A missing vocabulary file is reported by the exact path looked for and
    /// no source is constructed, so nothing is ingested and every facet is left
    /// unchanged (Requirement 3.15).
    pub fn at(dir: &Path) -> Result<Self, Phi4Error> {
        let vocab = dir.join(VOCAB_FILE);
        if !vocab.exists() {
            return Err(Phi4Error::NoVocabulary { looked_for: vocab });
        }
        Ok(Self { model_dir: dir.to_path_buf() })
    }

    /// Searches the usual candidate directories and pins to the first that
    /// exists, falling back to [`DEFAULT_DIR`] — i.e. `at(DEFAULT_DIR)` — when
    /// none is found, so a caller that wants the search keeps it.
    pub fn discover() -> Result<Self, Phi4Error> {
        let candidates = [
            DEFAULT_DIR,
            "../refs/Phi-4-multimodal-instruct",
            "phiano/refs/Phi-4-multimodal-instruct",
            "refs",
        ];

        for c in &candidates {
            let dir = Path::new(c);
            if dir.exists() {
                if let Ok(source) = Self::at(dir) {
                    return Ok(source);
                }
            }
        }

        Self::at(Path::new(DEFAULT_DIR))
    }

    /// Reads the Phi-4 vocabulary and merges from this source's directory into
    /// `facet`, training sentence resonance across the merge pairs and the
    /// documentation beside them, and reports the counts under `facet_name`.
    ///
    /// Reads files from `self.model_dir` alone and no file outside it.
    pub fn learn_into_facet(
        &self,
        facet: &mut Facet,
        trainer: &Trainer,
        facet_name: &str,
    ) -> Phi4Summary {
        let tokens_loaded = self.ingest_vocab(facet);
        let merges_trained = self.train_merges(facet, trainer);
        let doc_sentences_trained = self.train_docs(facet, trainer);

        Phi4Summary {
            vocab_tokens_loaded: tokens_loaded,
            merges_trained,
            doc_sentences_trained,
            final_vocabulary_size: facet.vocabulary_size(),
            facet: facet_name.to_string(),
        }
    }

    /// Ingest `vocab.json`: one cleaned, lower-cased alphabetic token per entry.
    fn ingest_vocab(&self, facet: &mut Facet) -> usize {
        let vocab_path = self.model_dir.join(VOCAB_FILE);
        let mut tokens_loaded = 0;
        if let Ok(content) = fs::read_to_string(&vocab_path) {
            if let Ok(map) = serde_json::from_str::<VocabMap>(&content) {
                for (token, _id) in map.0 {
                    let clean = token.replace('Ġ', "").replace('Ċ', "\n").trim().to_string();
                    if !clean.is_empty()
                        && clean.len() >= 2
                        && clean.chars().all(|c| c.is_alphabetic() || c == '_' || c == '-')
                    {
                        facet.get_or_init(&clean.to_lowercase());
                        tokens_loaded += 1;
                    }
                }
            }
        }
        tokens_loaded
    }

    /// Train the top BPE merges (links root morphemes & suffixes).
    fn train_merges(&self, facet: &mut Facet, trainer: &Trainer) -> usize {
        let merges_path = self.model_dir.join("merges.txt");
        let mut merges_trained = 0;
        if let Ok(content) = fs::read_to_string(&merges_path) {
            for line in content.lines().take(5000) {
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
        merges_trained
    }

    /// Train from the documentation and reference files inside the directory.
    fn train_docs(&self, facet: &mut Facet, trainer: &Trainer) -> usize {
        let doc_files = [
            self.model_dir.join("data_summary_card.md"),
            self.model_dir.join("README.md"),
            self.model_dir.join("sample_inference_phi4mm.py"),
        ];

        let mut docs_trained = 0;
        for doc_file in &doc_files {
            if let Ok(content) = fs::read_to_string(doc_file) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("```") && trimmed.len() > 15 {
                        let clean_line =
                            trimmed.trim_start_matches('#').trim_start_matches("//").trim();
                        trainer.train_sentence(facet, clean_line);
                        docs_trained += 1;
                    }
                }
            }
        }
        docs_trained
    }
}

impl DictionarySource for Phi4Source {
    fn fetch_all(&self) -> Vec<(String, String)> {
        let vocab_path = self.model_dir.join(VOCAB_FILE);
        let mut entries = Vec::new();

        if let Ok(content) = fs::read_to_string(vocab_path) {
            if let Ok(map) = serde_json::from_str::<VocabMap>(&content) {
                for (token, id) in map.0 {
                    let clean = token.replace('Ġ', "").trim().to_string();
                    if !clean.is_empty() {
                        entries.push((
                            clean.to_lowercase(),
                            format!("Phi-4 tiktoken vocabulary item ID {}", id),
                        ));
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
mod tests;
