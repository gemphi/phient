//! Does the manifold predict the next *sentence*?
//!
//! Every benchmark in this project so far has scored words. Analogy is
//! `word:word::word:word`. Pair-versus-random is word against word. Perplexity
//! is the next *word*. The relation set is 305 word pairs. And γ\* = 0 has held
//! across seven independent attempts — all seven of them next-word prediction.
//!
//! That is a gap in the measurement, not a settled result. If meaning is
//! carried by sentences rather than by words — if a word is a hair and the
//! sentence is the coat — then a representation that compresses *what comes
//! next at the sentence level* could be doing its job perfectly while losing
//! every word-level benchmark ever run against it.
//!
//! # The task
//!
//! Given `k` consecutive sentences of context, rank the true continuation
//! against `n` distractors drawn from elsewhere in the corpus. This is standard
//! next-sentence selection, and it is the smallest honest test of the claim.
//!
//! Three scorers, on identical items:
//!
//! * **phase** — the recurrent state after the context, scored against each
//!   candidate's phase state. This is the thing under test.
//! * **lexical** — word overlap between context and candidate. A strong, dull
//!   baseline: real continuations repeat words. Beating it means the manifold
//!   carries something beyond repetition.
//! * **chance** — `1/n`. If phase does not beat this, nothing else matters.
//!
//! # What would make this a real result
//!
//! Phase must beat lexical overlap, not merely chance. A representation that
//! only recovers word repetition has re-derived a bag of words in complex
//! arithmetic. The gap to *lexical* is the claim; the gap to chance is the floor.

mod benchmark;
mod state;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use benchmark::*;
#[allow(unused_imports)]
use state::*;

use phiecs::phasor::facet::Facet;
use phistd::fnv1a;
use phiton::p64::words::Tokenizer;
use phiecs::phasor::wave::Wave;
use phistd::c64;
use rayon::prelude::*;
use std::collections::HashSet;

/// Sentences of context shown before the prediction point.
pub const CONTEXT_LEN: usize = 3;

/// Candidates ranked per item: one true continuation and `CANDIDATES - 1`
/// distractors.
pub const CANDIDATES: usize = 50;

#[derive(Debug, Clone, Default)]
pub struct ScorerResult {
    pub name: String,
    pub top1: f64,
    pub top5: f64,
    pub mrr: f64,
    pub items: usize,
}

#[derive(Debug, Clone)]
pub struct SentenceReport {
    pub items: usize,
    pub candidates: usize,
    pub chance_top1: f64,
    pub chance_mrr: f64,
    pub scorers: Vec<ScorerResult>,
}

/// One prediction item: context sentences, the true continuation, distractors.
struct Item {
    pub(super) context: Vec<Vec<String>>,
    pub(super) candidates: Vec<Vec<String>>,
    /// Index of the true continuation within `candidates`.
    pub(super) truth: usize,
}

pub struct SentenceBenchmark;

/// How a sentence is compressed into a phase state.
///
/// The first version of this benchmark used `Bag` only, and reported that phase
/// loses to lexical overlap. That was a result about the encoder, not about the
/// hypothesis: a bag of words in complex arithmetic cannot beat a bag of words,
/// and testing it against one asks nothing. If meaning is carried by *how the
/// concepts connect*, the encoder has to carry connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentenceEncoding {
    /// Unbound superposition. Order-free; the control.
    Bag,
    /// Each word rotated by its position times the golden angle.
    Bound,
    /// Diagonal complex recurrence over the sentence, with a per-channel
    /// timescale — the construction the order experiment measured best of three
    /// at the word level.
    Recurrent,
}
