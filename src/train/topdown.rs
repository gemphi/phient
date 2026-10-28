//! The downward path: sentences constraining the words they are made of.
//!
//! Phiano is bottom-up in its entirety. Words are trained by next-word ranking;
//! a sentence is a sum of its words; a paragraph is a sum of its sentences. At
//! no point does a level above constrain the level below. That is the whole
//! architecture in one line, and it is why "a word is a hair, the sentence is
//! the coat" is currently a claim the code does not implement — the coat here
//! is defined as the sum of its hairs and has no say in where they lie.
//!
//! This module adds the missing direction. One *cycle* is:
//!
//! ```text
//! UP    sentence state  s_j = Σ  e^{i(θ_k(w) + pos·ρ_k)}   over words w in sentence j
//! DOWN  each word w  ←  rotated toward the mean of the sentences containing it
//! ```
//!
//! and the cycle is iterated, because a word moved on round one changes the
//! sentence states that move it on round two.
//!
//! # Three things that would make this measure nothing, and what is done about them
//!
//! **Self-reinforcement.** A word is part of the state it is being pulled
//! toward. Left alone, every word is partly pulled toward itself, the objective
//! improves, and nothing has been learned. Every target here is computed
//! *leave-one-out*: the word's own contribution is subtracted from its
//! sentence's state before the state is used to move it. This is why the
//! encoding is a superposition rather than the recurrence — subtraction has to
//! be exact.
//!
//! **Collapse.** "Pull each word toward the mean of its contexts" is an
//! attraction rule with a known fixed point: every word at one angle. Function
//! words are the accelerant, since `the` occurs in nearly every sentence and so
//! its target *is* the global mean. Two defences. First, [`Descent::contrast`]
//! subtracts the corpus-mean sentence state, so a word is pulled toward what its
//! contexts have *distinctively* rather than toward what all contexts share;
//! under contrast a word that occurs everywhere has a target of near-zero
//! magnitude and does not move. This is inverse document frequency, written in
//! phase. Second, the result is checked against the band-aware dispersion guard
//! and discarded if the frequent band has concentrated.
//!
//! **Measuring at the level you intervened on.** Constraining words from
//! sentences and then reporting a sentence score is close to circular. The
//! question worth asking is whether the downward path improves the level
//! *below* it — whether word-level relational structure gets better when
//! sentences are allowed to have an opinion. Both are reported; the word-level
//! one is the claim.
//!
//! # What it measured
//!
//! Two seeds on the 120,000-sentence dictionary corpus, word-level relational
//! scores at a frequency floor of 25 (pool 3,654 words, chance analogy MRR
//! 0.00027):
//!
//! ```text
//! arm                    analogy MRR   vs chance   nbr@10   pair>random
//! bottom-up only       0.00230 0.00215      ~8x    0.4-1.0%   62.8 61.4%
//! descent (bag)        0.01727 0.01230   45-63x    5.5-7.8%   72.0 71.8%
//! descent (bound)      0.00519 0.00443   16-19x    3.1-5.6%   66.6 68.0%
//! descent SHUFFLED     0.00143 0.00269    5-10x    0.7%       53.0 54.8%
//! ```
//!
//! The downward pass is **6.6x the bottom-up control and 6.7x its own null**.
//! The null is the load-bearing column: the same update rule applied to
//! sentences with the same lengths and word frequencies but no co-occurrence
//! structure lands at or below the control. The gain belongs to which words
//! occur together, not to the fact that something moved.
//!
//! These numbers replaced an earlier set measured before the seeding bug in
//! [`phistd::mix64`] was found — FNV-1a's high bits still carried the
//! spelling prefix after three bytes, so `cow`, `cod` and `cot` began life at
//! the same point on 63 of 64 channels. Every arm here, including the nulls, was
//! drawing from a lexicon pre-clustered by spelling. Re-run on fixed seeds the
//! effect roughly **doubled** (0.00814 to 0.0148 mean): the contamination was
//! suppressing the result, not manufacturing it.
//!
//! Two things this still does not do. The sentence level does not move — every
//! arm sits at 1.01x chance on held-out next-sentence selection and loses to
//! lexical overlap, including the arms whose words improved sevenfold.
//! Constraint from above improves the level below without improving the level it
//! came from, which is not the direction the hypothesis predicted. And the
//! order-sensitive encoding is consistently worse than the order-free one,
//! matching every earlier result here where positional rotation was tried.
//!
//! The no-contrast arm was rejected by the collapse guard on both seeds, at band
//! dispersion 0.100-0.105 with *global* dispersion 0.525 — above the 0.40 floor.
//! The global-only guard this project shipped before the band-aware fix would
//! have accepted a collapsed manifold every time.

mod rotation;
mod cycle;
mod votes;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use rotation::*;
#[allow(unused_imports)]
use cycle::*;
#[allow(unused_imports)]
use votes::*;

use phiton::units::{PHASE_CHANNELS, TWO_PI};
use phiecs::phasor::facet::Facet;
use phiecs::phasor::spectral::SpectralPhasor;
use phistd::c64;
use std::collections::HashMap;

/// Fraction of the way to its target a word moves per round.
///
/// Full steps make the cycle a fixed-point iteration on an attraction map,
/// which is the collapse the guard exists to catch. A partial step is the
/// anchor: the word keeps most of what the bottom-up pass gave it.
pub const DEFAULT_STEP: f64 = 0.25;

/// Rounds of the up-down cycle.
pub const DEFAULT_ROUNDS: usize = 4;

/// Minimum mean resultant length of a word's contexts for it to move.
///
/// A word whose sentences point in every direction has a short resultant, and
/// its argument is the angle of a rounding error. Words below this keep the
/// position bottom-up training gave them, which is the honest default: no
/// evidence, no move.
pub const DEFAULT_CONCENTRATION: f64 = 0.05;

/// Multiplier on the chance resultant, `sqrt(pi) / (2 * sqrt(n))`.
///
/// `n` unit vectors drawn uniformly at random have that expected resultant
/// length — 0.44 at four contexts, 0.09 at a hundred. A word must beat its own
/// chance level by this factor to move, so "agreement" means agreement above
/// what its context count produces for free. Without it the threshold is
/// backwards: rare words clear a fixed bar on noise and frequent ones, whose
/// evidence is real, fall under it.
pub const RAYLEIGH_C: f64 = 1.3;

/// How far a word's contexts must differ from the average context to move it.
///
/// Expressed as a fraction of the average context's own magnitude, so it is
/// scale-free. Below it, the word's contexts *are* the corpus average and the
/// residual's argument is a rounding error — which is the case for exactly the
/// words that would drag the manifold to a point.
pub const CONTRAST_MIN: f64 = 0.10;

/// How a sentence is summed from its words during the upward pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Up {
    /// Plain superposition. Order-free; the control.
    Bag,
    /// Each word rotated by its position. Order-sensitive, and still exactly
    /// subtractive, which leave-one-out requires.
    Bound,
}

/// What one cycle did, in the terms needed to decide whether to keep it.
#[derive(Debug, Clone)]
pub struct DescentReport {
    pub rounds: usize,
    pub sentences: usize,
    /// Words that had enough contextual agreement to move, on the last round.
    pub words_moved: usize,
    /// Words in the lexicon that occur in at least one sentence.
    pub words_seen: usize,
    pub dispersion_before: f64,
    pub dispersion_after: f64,
    pub band_before: f64,
    pub band_after: f64,
    /// True when the guard rejected the result and the facet was left alone.
    pub rejected: bool,
}

pub struct Descent;
