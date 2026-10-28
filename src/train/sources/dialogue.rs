use phiecs::phasor::facet::Facet;
use crate::memory::Memo;
use crate::train::trainer::Trainer;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A conversational exchange turn containing user input and assistant response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTurn {
    pub prompt: String,
    pub response: String,
    pub category: Option<String>,
}

/// DialogueSource - handles loading and ingesting multi-turn conversational datasets.
pub struct DialogueSource {
    pub path: Option<String>,
}

impl DialogueSource {
    pub fn new(path: &str) -> Self {
        Self {
            path: Some(path.to_string()),
        }
    }

    pub fn default_curriculum() -> Self {
        Self { path: None }
    }

    /// Returns built-in conversational training exchanges.
    pub fn built_in_dialogues() -> Vec<DialogueTurn> {
        vec![
            DialogueTurn {
                prompt: "Hello, who are you and how do you work?".to_string(),
                response: "Hello! I am Phiano, a continuous-learning cognitive assistant. I represent language as complex waves on a continuous phase manifold.".to_string(),
                category: Some("greeting".to_string()),
            },
            DialogueTurn {
                prompt: "Can you explain how dolphins breathe?".to_string(),
                response: "Dolphins are warm-blooded marine mammals that breathe atmospheric air through a blowhole connected directly to their lungs.".to_string(),
                category: Some("explanation".to_string()),
            },
            DialogueTurn {
                prompt: "What is the difference between a brute fact and an institutional fact?".to_string(),
                response: "A brute fact exists independently of human institutions, such as the mass of the moon. An institutional fact requires human agreement and constitutive rules to exist, such as money, marriage, or citizenship.".to_string(),
                category: Some("philosophy".to_string()),
            },
            DialogueTurn {
                prompt: "Why does continuous learning prevent catastrophic forgetting?".to_string(),
                response: "In a continuous complex phase manifold, new concepts settle into harmonic equilibria without destroying the phase coordinates of previously learned words.".to_string(),
                category: Some("technical".to_string()),
            },
            DialogueTurn {
                prompt: "I hereby declare this meeting open.".to_string(),
                response: "Declaration registered. As a declarative speech act with double direction of fit, it brings about a new institutional state of affairs when authorized.".to_string(),
                category: Some("speech_act".to_string()),
            },
        ]
    }

    /// Loads dialogues from a JSON or JSONL file.
    pub fn load_turns(&self) -> Vec<DialogueTurn> {
        let path_str = match &self.path {
            Some(p) => p,
            None => return Self::built_in_dialogues(),
        };

        let path = Path::new(path_str);
        if !path.exists() {
            return Self::built_in_dialogues();
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(turns) = serde_json::from_str::<Vec<DialogueTurn>>(&content) {
                if !turns.is_empty() {
                    return turns;
                }
            }
        }

        let mut turns = Vec::new();
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                // An unreadable line (e.g. invalid UTF-8) is skipped, not fatal.
                let Ok(line) = line else { continue };
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(turn) = serde_json::from_str::<DialogueTurn>(trimmed) {
                    turns.push(turn);
                }
            }
        }

        if turns.is_empty() {
            Self::built_in_dialogues()
        } else {
            turns
        }
    }

    /// Trains dialogue turns into the phase manifold and registers patterns into memory.
    pub fn learn_into_facet(
        &self,
        facet: &mut Facet,
        memo: &mut Memo,
        trainer: &Trainer,
    ) -> usize {
        let turns = self.load_turns();
        let mut count = 0;

        for turn in &turns {
            // 1. Train individual prompt and response sentences
            trainer.train_sentence(facet, &turn.prompt);
            trainer.train_sentence(facet, &turn.response);

            // 2. Train combined transitional pair (User -> Assistant coupling)
            let combined = format!("{} {}", turn.prompt, turn.response);
            trainer.train_sentence(facet, &combined);

            let prompt_tokens = phiton::p64::words::Tokenizer::tokenize(&turn.prompt);
            let resp_tokens = phiton::p64::words::Tokenizer::tokenize(&turn.response);

            let prompt_wave = phiecs::phasor::wave::Wave::sentence(facet, &prompt_tokens);
            let resp_wave = phiecs::phasor::wave::Wave::sentence(facet, &resp_tokens);

            // 3. Record into 16-layer memory hierarchy (Pattern Band: L4-L7)
            memo.record((prompt_wave.re, prompt_wave.im), &turn.prompt);
            memo.record((resp_wave.re, resp_wave.im), &turn.response);

            count += 1;
        }

        count
    }
}
