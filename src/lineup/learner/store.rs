//! The checkpoint: a learner written to disk whole and read back whole, so
//! a run starts where the last one stopped instead of at the seed.
//!
//! Everything that is learned goes to the file — the angles of every word
//! in every role, the operators and the pair, the biases, the Adagrad
//! accumulators beside each of them, the lines told so the epochs can go
//! on over them, the count of each word, and the random state, so a run
//! continued from the file takes the very steps the uninterrupted run
//! would have. What is NOT written is derived: the word index and the
//! sampler of wrong answers are rebuilt from the words and their counts on
//! the first line told or epoch run.

use super::plane::Plane;
use super::{Config, Learner};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::Path;

/// Bumped when the layout below changes; an older file is refused, not
/// misread.
const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Stored {
    version: u32,
    config: Config,
    word: Vec<String>,
    seen: Vec<u64>,
    reach: Plane,
    reached: Plane,
    cond: Plane,
    op: Plane,
    pair: Plane,
    bias: Vec<f32>,
    bias_acc: Vec<f32>,
    told: Vec<Vec<u32>>,
    rng: u64,
}

impl Learner {
    /// Writes the learner to `path`, whole.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let stored = Stored {
            version: VERSION,
            config: self.config.clone(),
            word: self.word.clone(),
            seen: self.seen.clone(),
            reach: self.reach.clone(),
            reached: self.reached.clone(),
            cond: self.cond.clone(),
            op: self.op.clone(),
            pair: self.pair.clone(),
            bias: self.bias.clone(),
            bias_acc: self.bias_acc.clone(),
            told: self.told.clone(),
            rng: self.rng,
        };
        let bytes = bincode::serialize(&stored).map_err(io::Error::other)?;
        std::fs::write(path, bytes)
    }

    /// Reads a learner back from `path`. It goes on from where it was
    /// saved: told more, it corrects itself on the new lines at once; run
    /// for more epochs, it passes over everything it was ever told.
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let s: Stored = bincode::deserialize(&bytes).map_err(io::Error::other)?;
        if s.version != VERSION {
            return Err(io::Error::other(format!("checkpoint version {} is not {VERSION}", s.version)));
        }
        let index: HashMap<String, u32> = s.word.iter().enumerate().map(|(i, w)| (w.clone(), i as u32)).collect();
        Ok(Self {
            config: s.config,
            word: s.word,
            index,
            seen: s.seen,
            reach: s.reach,
            reached: s.reached,
            cond: s.cond,
            op: s.op,
            pair: s.pair,
            bias: s.bias,
            bias_acc: s.bias_acc,
            told: s.told,
            sampler: Vec::new(),
            logq: Vec::new(),
            fresh_logq: 0.0,
            sampled: 0,
            rng: s.rng,
        })
    }

    /// How many lines the learner has been told, all of which its epochs
    /// pass over.
    pub fn lines_told(&self) -> usize {
        self.told.len()
    }
}
