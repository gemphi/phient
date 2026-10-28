use crate::train::sources::DictionarySource;
use std::fs;
use std::io::{BufRead, BufReader};

/// A local definitions file source.
///
/// File format: one entry per line, `word: definition text`
/// Lines starting with # are comments and are ignored.
pub struct LocalSource {
    pub file_path: String,
}

impl LocalSource {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    /// Parse the definitions file into (word, definition) pairs.
    pub fn parse_file(&self) -> Vec<(String, String)> {
        let file = match fs::File::open(&self.file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[error] Could not open '{}': {}", self.file_path, e);
                return vec![];
            }
        };

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Split on first colon
            if let Some(colon_pos) = trimmed.find(':') {
                let word = trimmed[..colon_pos].trim().to_lowercase();
                let definition = trimmed[colon_pos + 1..].trim().to_string();
                if !word.is_empty() && !definition.is_empty() {
                    entries.push((word, definition));
                }
            }
        }

        entries
    }
}

impl DictionarySource for LocalSource {
    fn fetch_all(&self) -> Vec<(String, String)> {
        self.parse_file()
    }

    fn fetch_definitions(&self, word: &str) -> Vec<String> {
        let entries = self.parse_file();
        entries
            .into_iter()
            .filter(|(w, _)| w == word)
            .map(|(_, d)| d)
            .collect()
    }
}
