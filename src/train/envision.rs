//! (`phient::train::envision` — it was phiano's `envision`.)

mod display;
mod gaps;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use display::*;
#[allow(unused_imports)]
use gaps::*;

use crate::train::chunker::ChunkStore;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use phiecs::phasor::wave::Wave;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Similarity below which a candidate is not worth suggesting.
const SUGGEST_THRESHOLD: f64 = 0.45;
/// How many suggestions to offer per unknown word.
const SUGGEST_K: usize = 5;
/// Stop asking about a word after this many unanswered attempts.
const MAX_ASKS: u32 = 3;
/// Spelling / context blend. Context carries more signal when it exists.
const ORTHOGRAPHIC_WEIGHT: f64 = 0.4;
const SEMANTIC_WEIGHT: f64 = 0.6;

/// Vision - a projection of what the model doesn't yet know.
#[derive(Debug, Clone)]
pub struct Vision {
    pub text: String,
    /// Words resolved automatically from a source, without asking.
    pub auto_learned: Vec<String>,
}

/// A record of what the model has asked about and what it still lacks.
///
/// `detect_gaps` used to be stateless, so the same unknown word asked about in
/// ten consecutive turns produced the same question ten times, with no way to
/// prioritise, escalate, or stop. Curiosity without memory is a tic.
#[derive(Debug, Default)]
pub struct GapLedger {
    pub(super) asked: HashMap<String, u32>,
    pub(super) encountered: HashMap<String, u32>,
    pub(super) resolved: HashSet<String>,
    pub(super) first_seen: HashMap<String, u64>,
}

/// Envision - detects knowledge gaps and projects what to learn next.
///
/// This is the model's one genuinely agentic behaviour: notice a hole in
/// itself, name it, propose a hypothesis, and try to fill it.
#[derive(Debug, Default)]
pub struct Envision {
    pub ledger: GapLedger,
}
