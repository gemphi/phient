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

    /// Every relation triple the dump carries: synonyms, hypernyms and
    /// antonyms, from both the entry level and each sense. Kaikki JSONL only;
    /// the simple `{word: [def]}` format carries no relations and yields none.
    pub fn relations(&self) -> Vec<Relation> {
        let content = match fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[error] Could not read '{}': {}", self.file_path, e);
                return vec![];
            }
        };
        let mut out = Vec::new();
        for line in content.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let Ok(entry) = serde_json::from_str::<KaikkiEntry>(line) else { continue };
            let about = entry.word.to_lowercase();
            push_relations(&mut out, &about, "synonym", &entry.synonyms);
            push_relations(&mut out, &about, "hypernym", &entry.hypernyms);
            push_relations(&mut out, &about, "antonym", &entry.antonyms);
            for sense in &entry.senses {
                push_relations(&mut out, &about, "synonym", &sense.synonyms);
                push_relations(&mut out, &about, "hypernym", &sense.hypernyms);
                push_relations(&mut out, &about, "antonym", &sense.antonyms);
            }
        }
        out
    }

    /// The relations as short trainable sentences `[about, kind, other]`, the
    /// marker in the middle so phient's relation vocabulary keys the pair on
    /// it. Feeding these beside the Guide lets a word resonate with its kin.
    pub fn relation_sentences(&self) -> Vec<Vec<String>> {
        self.relations()
            .into_iter()
            .filter(|r| !r.about.is_empty() && !r.other.is_empty() && r.about != r.other)
            .map(|r| vec![r.about, r.kind, r.other.to_lowercase()])
            .collect()
    }

    /// The part-of-speech of each entry, lowercased. Empty for the simple
    /// format or an entry that names no pos.
    pub fn parts_of_speech(&self) -> Vec<(String, String)> {
        let content = match fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut out = Vec::new();
        for line in content.lines().map(str::trim).filter(|l| !l.is_empty()) {
            if let Ok(entry) = serde_json::from_str::<KaikkiEntry>(line) {
                if let Some(pos) = entry.pos {
                    out.push((entry.word.to_lowercase(), pos.to_lowercase()));
                }
            }
        }
        out
    }
}

/// Simple JSON object format: { "word": ["def1", "def2"], ... }
#[derive(Deserialize)]
struct SimpleWiktionaryDump(HashMap<String, Vec<String>>);

/// Kaikki.org JSONL format: each line has { "word": "...", "senses": [{"glosses": ["..."]}] }
#[derive(Deserialize)]
struct KaikkiEntry {
    word: String,
    #[serde(default)]
    pos: Option<String>,
    #[serde(default)]
    senses: Vec<KaikkiSense>,
    #[serde(default)]
    synonyms: Vec<KaikkiRelation>,
    #[serde(default)]
    hypernyms: Vec<KaikkiRelation>,
    #[serde(default)]
    antonyms: Vec<KaikkiRelation>,
}

#[derive(Deserialize)]
struct KaikkiSense {
    glosses: Option<Vec<String>>,
    #[serde(default)]
    synonyms: Vec<KaikkiRelation>,
    #[serde(default)]
    hypernyms: Vec<KaikkiRelation>,
    #[serde(default)]
    antonyms: Vec<KaikkiRelation>,
}

/// One related word; Kaikki writes a relation as an object with a `word`.
#[derive(Deserialize)]
struct KaikkiRelation {
    word: String,
}

/// One relation triple lifted from an entry: `about` stands in `kind` to
/// `other` - `cat synonym feline`. The kind word is the marker phient's
/// relation channels key on, so a triple is one short sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relation {
    pub about: String,
    pub kind: String,
    pub other: String,
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

/// Append `(about, kind, other)` for each related word in `rels`.
fn push_relations(out: &mut Vec<Relation>, about: &str, kind: &str, rels: &[KaikkiRelation]) {
    for rel in rels {
        let other = rel.word.trim();
        if !other.is_empty() {
            out.push(Relation { about: about.to_string(), kind: kind.to_string(), other: other.to_string() });
        }
    }
}
