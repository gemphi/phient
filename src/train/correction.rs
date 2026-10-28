//! (`phient::train::correction` — it was phiano's `correction`.)
//!
//! Persistent correction journal.
//!
//! A correction is the one thing this architecture does that a gradient-trained
//! model cannot: unlearn a specific association in microseconds without
//! disturbing anything else. That advantage is worth very little if the
//! correction evaporates the next time the model is rebuilt from source.
//!
//! `correct_mistake` previously returned `()` and left no record, so a
//! user-taught fix could not be replayed after a re-ingest, audited, or undone.
//! This journal makes teaching durable.

use phiecs::phasor::facet::Facet;
use crate::train::trainer::Trainer;
use serde::{Deserialize, Serialize};
use std::io::Result;
use std::time::{SystemTime, UNIX_EPOCH};

/// One taught correction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub wrong: String,
    pub correct: String,
    pub ts_ms: u64,
    /// Rotation applied, in radians. `None` means the full anti-phase pulse.
    pub strength: Option<f64>,
}

/// An append-only log of corrections, replayable onto a fresh facet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorrectionLog {
    pub entries: Vec<Correction>,
}

impl CorrectionLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Records a correction.
    pub fn record(&mut self, wrong: &str, correct: &str, strength: Option<f64>) {
        let ts_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.entries.push(Correction {
            wrong: wrong.to_string(),
            correct: correct.to_string(),
            ts_ms,
            strength,
        });
    }

    /// Re-applies every recorded correction, oldest first.
    ///
    /// Run this after re-ingesting a corpus so that what the user taught
    /// survives a rebuild. Returns the number of corrections replayed.
    pub fn replay(&self, facet: &mut Facet, trainer: &Trainer) -> usize {
        for c in &self.entries {
            match c.strength {
                Some(s) => trainer.correct_graded(facet, &c.wrong, &c.correct, s),
                None => trainer.correct_mistake(facet, &c.wrong, &c.correct),
            }
        }
        self.entries.len()
    }

    /// Saves the journal as JSON, atomically.
    pub fn save(&self, path: &str) -> Result<()> {
        let tmp = format!("{}.tmp", path);
        let json = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, path)
    }

    /// Loads the journal, returning an empty log if the file is absent.
    pub fn load(path: &str) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;

    #[test]
    fn test_replay_reproduces_a_correction() {
        let trainer = Trainer::new(0.05);
        let sentences = ["rust is slow", "rust is fast", "the compiler is strict"];

        // A facet that was corrected live.
        let mut live = Facet::new();
        for s in &sentences {
            trainer.train_sentence(&mut live, s);
        }
        let mut log = CorrectionLog::new();
        trainer.correct_mistake(&mut live, "rust is slow", "rust is fast");
        log.record("rust is slow", "rust is fast", None);

        // A facet rebuilt from source, then replayed onto.
        let mut rebuilt = Facet::new();
        for s in &sentences {
            trainer.train_sentence(&mut rebuilt, s);
        }
        assert_eq!(log.replay(&mut rebuilt, &trainer), 1);

        let e = Evaluator::new();
        let a = e.eval(&live, "rust is slow").coherence;
        let b = e.eval(&rebuilt, "rust is slow").coherence;
        assert!((a - b).abs() < 0.2, "replayed state should track the live one: {} vs {}", a, b);
    }

    #[test]
    fn test_roundtrip_through_disk() {
        let mut log = CorrectionLog::new();
        log.record("a b", "a c", Some(0.3));
        let path = std::env::temp_dir().join("phiano_corrections_test.json");
        let path = path.to_str().unwrap();
        log.save(path).unwrap();
        let back = CorrectionLog::load(path);
        assert_eq!(back.len(), 1);
        assert_eq!(back.entries[0].correct, "a c");
        assert_eq!(back.entries[0].strength, Some(0.3));
    }
}
