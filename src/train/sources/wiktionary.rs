use crate::train::sources::DictionarySource;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// A Wiktionary JSON dump source.
///
/// The expected format is a JSON object mapping words to arrays of definitions:
/// { "word": ["definition 1", "definition 2"], ... }
///
/// Alternatively, it can handle the Kaikki.org Wiktionary JSONL format
/// where each line is a JSON object with "word" and "senses" fields.
pub struct WiktionarySource {
    pub file_path: String,
}

impl WiktionarySource {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
}

/// Simple JSON object format: { "word": ["def1", "def2"], ... }
#[derive(Deserialize)]
struct SimpleWiktionaryDump(HashMap<String, Vec<String>>);

/// Kaikki.org JSONL format: each line has { "word": "...", "senses": [{"glosses": ["..."]}] }
#[derive(Deserialize)]
struct KaikkiEntry {
    word: String,
    senses: Vec<KaikkiSense>,
}

#[derive(Deserialize)]
struct KaikkiSense {
    glosses: Option<Vec<String>>,
}

impl DictionarySource for WiktionarySource {
    fn fetch_all(&self) -> Vec<(String, String)> {
        let content = match fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[error] Could not read '{}': {}", self.file_path, e);
                return vec![];
            }
        };

        // Try simple JSON format first
        if let Ok(dump) = serde_json::from_str::<SimpleWiktionaryDump>(&content) {
            let mut entries = Vec::new();
            for (word, definitions) in dump.0 {
                for def in definitions {
                    entries.push((word.to_lowercase(), def));
                }
            }
            return entries;
        }

        // Try Kaikki.org JSONL format (one JSON object per line)
        let mut entries = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<KaikkiEntry>(line) {
                for sense in entry.senses {
                    if let Some(glosses) = sense.glosses {
                        for gloss in glosses {
                            entries.push((entry.word.to_lowercase(), gloss));
                        }
                    }
                }
            }
        }

        if entries.is_empty() {
            eprintln!("[warning] Could not parse Wiktionary file '{}'. Expected JSON object or JSONL format.", self.file_path);
        }

        entries
    }

    fn fetch_definitions(&self, word: &str) -> Vec<String> {
        let entries = self.fetch_all();
        entries
            .into_iter()
            .filter(|(w, _)| w == word)
            .map(|(_, d)| d)
            .collect()
    }
}
