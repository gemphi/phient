//! (`phient::train::curriculum` — it was phiano's `curriculum`.)

#![allow(dead_code)]

//! Child-Like Learning Curriculum - progressive knowledge acquisition.
//! Loads stage definitions from data/curriculum.json (no hardcoded words).

use crate::train::chunker::ChunkStore;
use crate::train::grounding::DefinitionGrounder;
use phiecs::phasor::facet::Facet;
use crate::train::trainer::Trainer;
use serde::Deserialize;
use std::time::Instant;

#[derive(Debug, Clone, Deserialize)]
pub struct CurriculumStageDef {
    pub name: String,
    pub description: String,
    pub words: Vec<String>,
    pub sentences: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurriculumFile {
    pub stages: Vec<CurriculumStageDef>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CurriculumResult {
    pub stages_completed: usize,
    pub words_learned: usize,
    pub sentences_trained: usize,
    pub bigrams_recorded: usize,
    pub definitions_grounded: usize,
    pub vocabulary_before: usize,
    pub vocabulary_after: usize,
    pub elapsed_ms: u128,
    pub stage_details: Vec<StageDetail>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StageDetail {
    pub name: String,
    pub words_learned: usize,
    pub sentences_trained: usize,
}

pub struct ChildCurriculum {
    pub stages: Vec<CurriculumStageDef>,
}

impl Default for ChildCurriculum {
    fn default() -> Self {
        Self::new()
    }
}

impl ChildCurriculum {
    /// Loads curriculum from data/curriculum.json.
    pub fn new() -> Self {
        let path = std::path::Path::new("data/curriculum.json");
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let file: CurriculumFile = serde_json::from_str(&content)
                    .unwrap_or_else(|e| {
                        eprintln!("[curriculum] Failed to parse JSON: {}", e);
                        CurriculumFile { stages: vec![] }
                    });
                Self { stages: file.stages }
            }
            Err(e) => {
                eprintln!("[curriculum] Could not load data/curriculum.json: {}", e);
                Self { stages: vec![] }
            }
        }
    }

    pub fn run(
        &self,
        facet: &mut Facet,
        trainer: &Trainer,
        chunk_store: &ChunkStore,
    ) -> CurriculumResult {
        let start = Instant::now();
        let vocab_before = facet.vocabulary_size();
        let mut words_learned = 0usize;
        let mut sentences_trained = 0usize;
        let mut bigrams_recorded = 0usize;
        let mut stage_details = Vec::new();

        let entries = chunk_store.load_all();
        let def_map: std::collections::HashMap<String, String> = entries.into_iter().collect();

        for stage in &self.stages {
            let stage_start = Instant::now();
            let mut stage_words = 0usize;
            let mut stage_sentences = 0usize;

            for word in &stage.words {
                facet.get_or_init(word);
                stage_words += 1;
            }

            for sentence in &stage.sentences {
                trainer.train_sentence(facet, sentence);
                stage_sentences += 1;
            }

            for word in &stage.words {
                if let Some(def) = def_map.get(word) {
                    trainer.train_sentence(facet, def);
                    let linked = format!("{} means {}", word, def);
                    trainer.train_sentence(facet, &linked);
                }
            }

            bigrams_recorded = facet.ngram_entries();

            words_learned += stage_words;
            sentences_trained += stage_sentences;

            println!(
                "  [curriculum] {} - {} words, {} sentences ({:?})",
                stage.name, stage_words, stage_sentences, stage_start.elapsed()
            );

            stage_details.push(StageDetail {
                name: stage.name.clone(),
                words_learned: stage_words,
                sentences_trained: stage_sentences,
            });
        }

        let grounded = DefinitionGrounder::ground_phases(facet, chunk_store);

        CurriculumResult {
            stages_completed: self.stages.len(),
            words_learned,
            sentences_trained,
            bigrams_recorded,
            definitions_grounded: grounded,
            vocabulary_before: vocab_before,
            vocabulary_after: facet.vocabulary_size(),
            elapsed_ms: start.elapsed().as_millis(),
            stage_details,
        }
    }
}
