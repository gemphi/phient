//! Definitions as compositions, across every channel, in order, both ways.
//!
//! [`crate::train::grounding::DefinitionGrounder`] already moves a word
//! toward its definition. Measured, it halves phase dispersion and improves no
//! relation metric — and three properties of how it does it explain why:
//!
//! 1. **One channel.** It writes `theta(0)` only. The other 63 channels of the
//!    torus never see the definition, so 63/64 of the representation is
//!    untouched by meaning.
//! 2. **No order.** The target is the centroid of the definition's content
//!    words. A centroid is symmetric, so *a member of the board of directors*
//!    and *a board of the directors member* produce the same target. Anything
//!    carried by word order — argument roles, modification, "X of Y" versus
//!    "Y of X" — is discarded before it reaches the manifold.
//! 3. **One direction.** The headword moves toward its definers; the definers
//!    never move toward the headword. So *grandmother* learns from *mother* and
//!    *parent*, but *mother* learns nothing from appearing in *grandmother*'s
//!    definition, and the family of words that define each other never
//!    converges into a region.
//!
//! This module fixes all three. Composition runs on every channel; each
//! definition word is rotated by its position before it is summed, so the
//! superposition is a binding rather than a blur; and the pull is mutual, with
//! the definers drawn a smaller step toward the headword. Words that define
//! each other reinforce each other, which is what turns a definition graph into
//! a set of concept regions rather than a set of independent placements.
//!
//! Read and write passes stay separated (Jacobi, as in the grounder), so the
//! result does not depend on the order the dictionary is enumerated in.

mod structural;
mod compose;
mod graph;
mod counts;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use structural::*;
#[allow(unused_imports)]
use compose::*;
#[allow(unused_imports)]
use graph::*;
#[allow(unused_imports)]
use counts::*;

use phistd::GOLDEN_ANGLE;
use phiton::units::PHASE_CHANNELS;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use std::collections::HashMap;

/// How far the headword moves toward its composed definition each round.
pub const HEAD_STEP: f64 = 0.5;

/// How far each definition word moves back toward the headword, as a fraction
/// of `HEAD_STEP`.
///
/// Smaller than the forward step on purpose. A definer appears in many
/// entries; if it moved as far as the headword it would be dragged apart by
/// every entry it occurs in, and common words — which occur in the most
/// definitions — would move the most while carrying the least meaning.
pub const REINFORCE: f64 = 0.15;

/// Reinforcement for a **strong** pair: each word appears in the other's
/// definition.
///
/// Dict2vec (Tissier, Gravier & Habrard, EMNLP 2017) splits definitional pairs
/// exactly here, and the split is not cosmetic. *vehicle* is in the definition
/// of *car* and *car* is in the definition of *vehicle* — that reciprocity is
/// evidence of a semantic relation. *road* is in the definition of *car* but
/// *car* is not in the definition of *road*; the relation is real but weaker,
/// and treating the two alike gives *road* the same pull as *vehicle*.
///
/// The ratio here (0.8 : 0.45) is the one their grid search selected.
pub const BETA_STRONG: f64 = 0.8;

/// Reinforcement for a **weak** pair: one-way definitional membership.
pub const BETA_WEAK: f64 = 0.45;

/// Weight of the pull back toward a word's pre-composition phase.
///
/// Retrofitting (Faruqui et al., NAACL 2015) minimises a sum of two terms: stay
/// near where training put you, and move toward your lexical neighbours. This
/// module had only the second, and the measurement showed the cost — phase
/// dispersion fell from 0.986 to 0.327 in the best-scoring condition, which is
/// a third of the way to the collapse the whole harness exists to detect.
///
/// The anchor is that missing first term. `0.0` reproduces the unanchored
/// behaviour exactly, so the sweep over it is a controlled comparison.
pub const ANCHOR: f64 = 1.0;

/// Rate multiplier for words held in the grounding kernel.
pub const KERNEL_STEP: f64 = 0.1;

#[derive(Debug, Clone, Default)]
pub struct CompositionReport {
    pub entries_used: usize,
    pub heads_moved: usize,
    pub definers_reinforced: usize,
    pub rounds: usize,
    /// Mean absolute phase shift of a headword in the final round, in radians.
    /// Falling toward zero across rounds means the composition has converged.
    pub final_mean_shift: f64,
}

pub struct Conception;

/// Which words define which, and how strongly.
///
/// A dictionary is a directed graph: an edge `a → b` means *b* occurs in the
/// definition of *a*. Two facts fall out of the direction, and both are used:
///
/// * A **strong pair** is a two-way edge. Each word occurs in the other's
///   definition, which is much stronger evidence of a semantic relation than
///   one-way membership — *car*/*vehicle*, not *car*/*road*.
/// * Any edge at all, in either direction, means the two words must **never be
///   drawn as a negative sample** for each other. Contrastive training pushes a
///   word away from its negatives; sampling uniformly from the vocabulary will
///   occasionally draw a word from the target's own definition and push apart
///   precisely the pair the definition says belongs together. Dict2vec calls
///   the fix controlled negative sampling and measures it discarding ~2% of
///   generated negatives — small in count, and it is undoing the training
///   signal every time it fires.
#[derive(Debug, Clone, Default)]
pub struct DefinitionGraph {
    pub(super) edges: HashMap<String, std::collections::HashSet<String>>,
}
