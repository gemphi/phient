mod score;
mod evaluator;
mod converged;
#[allow(unused_imports)]
use score::*;
#[allow(unused_imports)]
use evaluator::*;
#[allow(unused_imports)]
use converged::*;

use crate::generate::constants as config;
use phiecs::phasor::facet::Facet;
use crate::generate::compose::flow::RiverFlow;
use crate::generate::compose::SectorPalette;
use phiton::p64::words::Tokenizer;
use phiecs::phasor::wave::Wave;
use std::collections::{HashMap, HashSet};

/// Evaluator — scores each of the 64 sector variations.
///
/// Based on the Flower-Hayes "Reviewing" process (evaluating + revising), and on
/// Huberman's 1968 chess-endgame design, in which progress is judged by a pair of
/// predicates over a **(stage, measure)** ordering rather than by a single number.
///
/// A flat weighted sum lets every quality be traded against every other: a
/// composition that is semantically wrong but long, diverse and sector-spanning
/// can outscore one that is short and right. A lexicographic order forbids that —
/// no amount of fine-grained score buys a stage.
pub struct Evaluator {
    /// The inner semantic evaluator.
    pub(super) inner: crate::eval::Evaluator,
}

/// SectorScore — the evaluation of a single sector variation.
#[derive(Debug, Clone)]
pub struct SectorScore {
    pub sector: u16,
    pub color: String,
    pub text: String,
    /// Coarse subgoal index. Compared **before** `score`; higher is further along.
    pub stage: u8,
    /// Fine-grained measure within the stage, in [0, 1].
    pub score: f64,
}

/// Cross-round memory of words that appeared in winning sectors.
///
/// Huberman's *killer heuristic* (thesis §2, p. 22): try first the moves that
/// produced a better position elsewhere in the search. Still standard in
/// alpha-beta engines nearly sixty years later.
#[derive(Debug, Default, Clone)]
pub struct KillerWord {
    pub(super) hits: HashMap<String, u32>,
}
