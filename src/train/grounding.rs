//! Definition-grounded word initialization.
//!
//! A word's position should be the centre of mass of what the word *means*, not
//! an artefact of how it is spelled. Dictionary definitions make that computable
//! without labels: a dictionary is a closed system in which every word is
//! defined by other words, so meaning is recoverable from structure alone.
//!
//! # Architecture
//!
//! ```text
//! Dictionary Definitions (ChunkStore)
//!   │
//!   ▼
//! DefinitionGrounder::ground_phases()   ── repeated GROUNDING_ROUNDS times
//!   ├─▶ READ pass:  centroid of each definition's content words (all words frozen)
//!   └─▶ WRITE pass: relax each word halfway toward its centroid
//! ```
//!
//! The read and write passes are separated (Jacobi rather than Gauss-Seidel) so
//! that the result does not depend on the order the dictionary happens to be
//! enumerated in, and the whole thing is iterated because definitions are
//! recursive — `cat` → `animal` → `organism` — and one pass propagates meaning
//! exactly one hop.

mod chunk;
mod grounder;
mod centroid;
#[cfg(test)]
mod tests;
pub use chunk::*;
#[allow(unused_imports)]
use grounder::*;
#[allow(unused_imports)]
use centroid::*;

use crate::train::chunker::ChunkStore;
use crate::train::conception::Conception;
use crate::train::constants::GROUNDING_ROUNDS;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;

/// Any source that can describe a symbol in terms of other symbols.
///
/// Grounding is not dictionary-specific: the mechanism takes a mapping from a
/// symbol to a bag of symbols and places the symbol at the bag's centre of
/// mass. A function body, a lead paragraph, a docstring or a window of
/// mentions all have that shape, so any of them can ground a manifold.
pub trait Groundable {
    /// `(symbol, description)` pairs.
    fn entries(&self) -> Vec<(String, String)>;
    /// A label for logging.
    fn source_name(&self) -> &str {
        "source"
    }
}

/// Below this, the manifold is concentrating rather than organising.
///
/// Phase dispersion is 1.0 when phases spread uniformly and 0.0 when every word
/// sits at the same angle. Composition legitimately lowers it — concept regions
/// are concentration — but the anchor sweep showed dispersion running from 0.54
/// at no anchor up to 0.81 at a strong one, and the unanchored, unguarded rule
/// reached 0.305 on an earlier source. A floor is what keeps "creates concept
/// regions" from becoming "collapses" without anyone noticing.
pub const DISPERSION_FLOOR: f64 = 0.40;

/// The band the guard actually watches.
///
/// The global dispersion figure is a tail average and cannot see the failure the
/// guard exists to detect: on a 30k vocabulary, the 500 most frequent words can
/// collapse onto a single angle while the number still reads 0.98, because the
/// rare words keep their initialisation and dominate the mean. Every scored task
/// draws its candidates from the frequent band, so the guard is applied there —
/// and to the whole lexicon, since either collapsing is a failure.
pub const GUARD_BAND_TOP: usize = 2_000;

/// Grounding engine for aligning lexical phases with source semantics.
pub struct DefinitionGrounder;
