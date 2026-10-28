use crate::train::sources::DictionarySource;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// A flat JSON dictionary source.
///
/// Expected format: { "word": "definition", "word2": "definition2", ... }
/// This is the format used by the Webster's English Dictionary JSON project
/// (https://github.com/matthewreagan/WebstersEnglishDictionary).
pub struct JsonDictionarySource {
    pub file_path: String,
}

impl JsonDictionarySource {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
}

#[derive(Deserialize)]
struct FlatDictionary(HashMap<String, String>);

impl DictionarySource for JsonDictionarySource {
    fn fetch_all(&self) -> Vec<(String, String)> {
        let content = match fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[error] Could not read '{}': {}", self.file_path, e);
                return vec![];
            }
        };

        match serde_json::from_str::<FlatDictionary>(&content) {
            Ok(dict) => {
                let mut entries: Vec<(String, String)> = dict
                    .0
                    .into_iter()
                    .map(|(word, def)| (word.to_lowercase(), def))
                    .collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                entries
            }
            Err(e) => {
                eprintln!("[error] Failed to parse JSON dictionary '{}': {}", self.file_path, e);
                vec![]
            }
        }
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
