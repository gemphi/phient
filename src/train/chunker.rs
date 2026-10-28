use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Chunked dictionary store - splits large dictionaries into letter-based
/// subfolders for parallel ingestion. Designed for CPU-utilization on Linux
/// where file I/O is fast and rayon can parallelize across chunks.
///
/// Layout:
///   data/chunks/a/a.json
///   data/chunks/a/b.json
///   data/chunks/b/a.json
///   ...
pub struct ChunkStore {
    pub root: String,
}

#[derive(Deserialize)]
struct FlatDict(HashMap<String, String>);

impl ChunkStore {
    pub fn new(root: &str) -> Self { Self { root: root.to_string() } }

    /// Split a large JSON dictionary into chunk files organized by first letter.
    /// Uses rayon to write chunks in parallel.
    pub fn split(&self, source_path: &str) -> usize {
        let content = match fs::read_to_string(source_path) {
            Ok(c) => c,
            Err(e) => { eprintln!("[error] Cannot read '{}': {}", source_path, e); return 0; }
        };

        let dict = match serde_json::from_str::<FlatDict>(&content) {
            Ok(d) => d.0,
            Err(e) => { eprintln!("[error] JSON parse failed: {}", e); return 0; }
        };

        // Group words by first letter
        let mut buckets: HashMap<char, HashMap<String, String>> = HashMap::new();
        for (word, def) in dict {
            let key = word.chars().next().unwrap_or('_').to_ascii_lowercase();
            buckets.entry(key).or_default().insert(word, def);
        }

        let total = buckets.values().map(|b| b.len()).sum();
        println!("  [chunk] {} words into {} letter groups", total, buckets.len());

        // Write each letter group in parallel
        let entries: Vec<(char, HashMap<String, String>)> = buckets.into_iter().collect();
        entries.par_iter().for_each(|(letter, words)| {
            let dir = format!("{}/{}", self.root, letter);
            let _ = fs::create_dir_all(&dir);
            let path = format!("{}/{}.json", dir, letter);
            if let Ok(json) = serde_json::to_string(words) {
                let _ = fs::write(&path, json);
            }
        });

        total
    }

    /// Load all chunks in parallel using rayon and return (word, definition) pairs.
    pub fn load_all(&self) -> Vec<(String, String)> {
        let root = &self.root;
        if !Path::new(root).exists() {
            return vec![];
        }

        let letter_dirs: Vec<String> = fs::read_dir(root)
            .map(|entries| entries.filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path().to_string_lossy().to_string())
                .collect())
            .unwrap_or_default();

        let chunk_files: Vec<String> = letter_dirs.par_iter()
            .flat_map(|dir| {
                fs::read_dir(dir).map(|entries| entries.filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                    .map(|e| e.path().to_string_lossy().to_string())
                    .collect::<Vec<_>>())
                    .unwrap_or_default()
            })
            .collect();

        println!("  [chunk] Found {} chunk files", chunk_files.len());

        chunk_files.par_iter()
            .flat_map(|path| {
                fs::read_to_string(path).ok()
                    .and_then(|content| serde_json::from_str::<FlatDict>(&content).ok())
                    .map(|d| d.0.into_iter().collect::<Vec<_>>())
                    .unwrap_or_default()
            })
            .collect()
    }

    /// Loads the definition for a single word from its chunk file.
    /// Returns None if the word or chunk file doesn't exist.
    pub fn load_definition(&self, word: &str) -> Option<String> {
        let letter = word.chars().next()?.to_ascii_lowercase();
        let path = format!("{}/{}/{}.json", self.root, letter, letter);
        let content = fs::read_to_string(&path).ok()?;
        let dict: FlatDict = serde_json::from_str(&content).ok()?;
        dict.0.get(word).cloned()
    }

    /// Ingest all chunks into the manifold with training metrics.
    pub fn ingest_parallel(
        &self,
        manifold: &mut phiecs::phasor::facet::Facet,
        trainer: &crate::train::trainer::Trainer,
        epochs: usize,
    ) -> crate::train::trainer::TrainingMetric {
        let start = Instant::now();
        let entries = self.load_all();
        if entries.is_empty() {
            eprintln!("  [error] No chunks found in {}", self.root);
            return crate::train::trainer::TrainingMetric::empty();
        }

        println!("  [ingest] {} entries, {} epochs", entries.len(), epochs);
        let mut metrics = crate::train::trainer::TrainingMetric::empty();

        for epoch in 0..epochs {
            let epoch_start = Instant::now();
            for (word, def) in &entries {
                trainer.train_definition(manifold, word, def);
            }
            metrics.epochs_completed += 1;
            metrics.words_learned = manifold.vocabulary_size();
            let elapsed = epoch_start.elapsed();
            println!("  [epoch {}/{}] {} words, {:?}", epochs, epoch + 1, manifold.vocabulary_size(), elapsed);
        }

        metrics.total_time = start.elapsed();
        metrics
    }
}
