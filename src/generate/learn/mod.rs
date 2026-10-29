//! The learning machine for what used to be hand-tuned.
//!
//! Phiano and Phinet use an *X as Y* paradigm: a word never means anything
//! other than what it relates to, in what conditions. The generator's
//! candidate scorer, until now, was `count · (0.35 + 0.25·phase + 0.40·res)
//! · content · repeat` — five numbers chosen by hand. This module replaces
//! them with a **log-linear scorer whose weights are fit to held-out text**:
//! for every position in a corpus, the n-gram's candidates get features, the
//! true next word gets its softmax probability, and the weights descend the
//! negative log-likelihood. The objective is convex in the weights, so one
//! run finds the answer, and the answer is stored beside the lexicon
//! (`data/learned.json`) and read by the generator when present.
//!
//! The same [`Tuner`] fits any small weight vector against any held-out
//! objective; the generator is its first customer.

pub mod features;

use crate::generate::constants::LEARNED_FILE;
use serde::{Deserialize, Serialize};

pub use features::{CandidateFeature, FEATURE_NAMES, N_FEATURES};

/// Learned weights for the generator's log-linear candidate scorer.
///
/// `score = Σ_f w_f · x_f`; the candidate distribution is `softmax(score)`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenWeight {
    pub w: [f64; N_FEATURES],
    /// Held-out NLL per position when these weights were fit, for the record.
    pub heldout_nll: f64,
    /// The counts-only baseline's NLL on the same positions.
    pub baseline_nll: f64,
    /// Positions fit on.
    pub positions: usize,
}

impl GenWeight {
    /// The hand-tuned scorer, expressed in the log-linear form as closely as
    /// it can be: `ln count` term at 1, everything else at its old
    /// multiplicative log. This is what "no learning" means here.
    pub fn hand() -> Self {
        let mut w = [0.0; N_FEATURES];
        w[0] = 1.0; // ln(1 + min(count, 24))
        w[1] = 0.55; // phase alignment: ln((0.35 + 0.25) / 0.35) ≈ 0.54 at full alignment
        w[2] = 0.76; // resonance: ln((0.35 + 0.40) / 0.35)
        w[3] = (0.55f64 / 1.35).ln(); // function-word penalty
        w[4] = 0.15f64.ln(); // repeat penalty
        w[5] = 0.0; // operator cosine
        w[6] = 0.0; // relational agreement (the hand scorer never had it)
        w[7] = 0.0;
        Self { w, heldout_nll: f64::NAN, baseline_nll: f64::NAN, positions: 0 }
    }

    /// Tally only.
    pub fn counts_only() -> Self {
        let mut w = [0.0; N_FEATURES];
        w[0] = 1.0;
        Self { w, heldout_nll: f64::NAN, baseline_nll: f64::NAN, positions: 0 }
    }

    #[inline]
    pub fn score(&self, x: &CandidateFeature) -> f64 {
        self.w.iter().zip(&x.0).map(|(w, x)| w * x).sum()
    }

    /// The weights as the decoder uses them: the fitted `repeat` weight
    /// replaced by the decode-time constraint [`REPEAT_DECODE_PENALTY`].
    pub fn for_decoding(&self) -> Self {
        let mut d = self.clone();
        d.w[REPEAT_FEATURE] = REPEAT_DECODE_PENALTY;
        d
    }

    pub fn load(path: &str) -> Option<Self> {
        let s = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&s).ok()
    }

    pub fn load_default() -> Option<Self> {
        Self::load(LEARNED_FILE)
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(path, serde_json::to_string_pretty(self).map_err(std::io::Error::other)?)
    }
}


mod positions;
mod tuner;
#[cfg(test)]
mod tests;

pub use positions::{corpus_positions, Position, CANDIDATES, WINDOW};
pub use tuner::{Tuner, REPEAT_DECODE_PENALTY, REPEAT_FEATURE};
