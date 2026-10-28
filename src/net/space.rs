//! The function space: words and contexts as functions on the relation circle.
//!
//! A [`phiecs::phasor::SpectralPhasor`] is 64 angles and 64 evidence counts. Read
//! together they are 64 complex numbers `f(k) = a_k · e^{iθ_k}`, one per
//! relation channel — a function on the circle of relations, sampled at 64
//! points. This module makes that reading explicit and builds the objects the
//! operator learns on: a word's function, a context's function, and a
//! deterministic train/held-out split of `(context, next word)` pairs.
//!
//! Two encodings are needed because two kinds of model exist:
//!
//! * **Evidenced** (v4 lexicons trained since the promotion): `a_k` is the
//!   evidence `ln(1 + count_k)`, so a word's function is loud on the relations
//!   it takes part in and silent elsewhere.
//! * **Flat** (v1–v3 lexicons, such as the shipped `data/manifold.chroma`, and
//!   any word with no evidence): `a_k = 1`. The function is a pure phase
//!   pattern, which is all those models hold.
//!
//! Either way the function is normalised to unit energy, so a comparison is a
//! cosine and the operator cannot win by scaling.

mod encode;
#[cfg(test)]
mod tests;
pub use encode::*;

use phiton::units::PHASE_CHANNELS;
use phiecs::phasor::facet::Facet;
use phiecs::phasor::SpectralPhasor;
use phistd::fnv1a;
use phistd::c64;
pub use phistd::{cosine, normalise};
pub use phical::Example;
use std::collections::HashMap;

/// A word as a function on the relation circle: `PHASE_CHANNELS` samples.
pub type WordFunction = phistd::Function;

/// Which side(s) of a two-sided phasor a function space encodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sides {
    /// The source side only — the reading every measurement before format
    /// v5 used, and the one a one-sided lexicon has.
    Source,
    /// The target side only.
    Target,
    /// Source then target: 128 samples. The circle is walked twice.
    Both,
    /// Directed: contexts are built from source sides, the table a word is
    /// ranked against holds target sides — what a relation *is* under the
    /// split (Paper 1 §4).
    Directed,
}

/// The lexicon as a function space.
pub struct FunctionSpace {
    pub words: Vec<String>,
    pub index: HashMap<String, usize>,
    pub funcs: Vec<WordFunction>,
    pub counts: Vec<u32>,
    /// True if at least one word carried evidence (a v4 lexicon).
    pub evidenced: bool,
    /// For [`Sides::Directed`]: the target-side table words are ranked
    /// against, while `funcs` (source sides) build contexts. `None` means
    /// the table *is* `funcs`.
    pub table: Option<Vec<WordFunction>>,
}
