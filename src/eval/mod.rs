//! `eval` — the evaluator: a text's semantic quality as coherence, novelty and
//! resonance, weighted into one score. It was phiano's `eval`.

pub mod constants;
pub mod oscillator;
mod verdict;
#[allow(unused_imports)]
use verdict::*;

use constants::*;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use phiecs::phasor::wave::Wave;
use std::fmt;
use std::f64::consts::PI;

/// Quality - qualitative assessment of a text's semantic quality.
/// (phiclm's `Verdict` is a different thing: a program's compile-and-run result.)
///
/// Derived from the coherence, novelty, and resonance scores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quality {
    /// Input was empty - no tokens to evaluate.
    Empty,
    /// Mostly unknown vocabulary - sounds like noise.
    Noise,
    /// Words don't resonate but the input is novel - possibly creative.
    DissonantNovel,
    /// Words do not resonate together at all.
    Incoherent,
    /// Coherent and well-grounded in existing knowledge.
    CoherentGrounded,
    /// Coherent and introduces novel ideas - insightful.
    CoherentNovel,
    /// Moderately coherent with some novel elements.
    ModerateNovel,
    /// Coherent but familiar - no new information.
    CoherentFamiliar,
    /// Weakly coherent - marginal meaning.
    WeaklyCoherent,
}

/// Eval - the result of evaluating a text against the facet.
///
/// Contains quantitative scores (coherence, novelty, resonance, overall)
/// and a qualitative verdict.
#[derive(Debug, Clone)]
pub struct Eval {
    /// How well the words resonate together (0.0 - 1.0).
    pub coherence: f64,
    /// How different the input is from the facet centroid (0.0 - 1.0).
    pub novelty: f64,
    /// Fraction of input tokens known by the facet (0.0 - 1.0).
    pub resonance: f64,
    /// Weighted combination of all scores (0.0 - 1.0).
    pub overall: f64,
    /// Qualitative assessment of the text.
    pub verdict: Quality,
}

/// Evaluator - scores text against the facet's semantic space.
///
/// Produces an `Eval` result with coherence, novelty, and resonance scores.
pub struct Evaluator;
