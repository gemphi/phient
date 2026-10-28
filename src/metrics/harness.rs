//! The evaluation harness.
//!
//! Every claim about this architecture is a hypothesis until it is measured
//! against held-out data. This module supplies the measurement:
//!
//! * a deterministic 80/10/10 split with no leakage,
//! * a Kneser-Ney trigram baseline to beat,
//! * held-out perplexity for the model itself,
//! * and, logged beside them, the two diagnostics that distinguish learning
//!   from collapse — phase dispersion and sector Gini.
//!
//! Coherence is deliberately *not* the headline number. It is the Kuramoto
//! order parameter, which the training rule maximises and which reads 1.0 on a
//! fully synchronised lexicon for any input, including noise. It can rise while
//! the model gets worse. Perplexity cannot.

mod perplexity;
mod run;
mod lm;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use perplexity::*;
#[allow(unused_imports)]
use run::*;
#[allow(unused_imports)]
use lm::*;

use crate::eval::Evaluator;
use phiecs::phasor::facet::Facet;
use crate::metrics::baseline::kn::KneserNey;
use phistd::fnv1a;
use phiton::p64::words::Tokenizer;
use crate::train::trainer::Trainer;
use rayon::prelude::*;
use serde::Serialize;

/// A deterministic corpus split. `test` is read once, at the end.
#[derive(Debug, Clone, Default)]
pub struct Split {
    pub train: Vec<String>,
    pub valid: Vec<String>,
    pub test: Vec<String>,
}

/// Per-epoch record. Written to `data/evaluation.json`.
#[derive(Debug, Clone, Serialize)]
pub struct EpochMetric {
    pub epoch: usize,
    pub train_ppl: f64,
    /// The number that matters: perplexity on text never trained on.
    pub valid_ppl: f64,
    /// Same model, same counts, unigram back-off instead of the phase manifold.
    /// The gap to `valid_ppl` is what the phase layer contributes, per epoch.
    pub valid_ppl_no_phase: f64,
    /// Kept for continuity, never used alone.
    pub coherence: f64,
    /// 1.0 = phases spread uniformly, 0.0 = fully collapsed.
    pub phase_dispersion: f64,
    /// The same over the frequent band. Collapse shows up here epochs before the
    /// global figure moves, because the global figure is diluted by the tail.
    pub band_dispersion: f64,
    /// Sector-occupancy inequality; rises as the manifold concentrates.
    pub sector_gini: f64,
    pub vocab_size: usize,
    pub mean_amplitude: f64,
}

/// Final comparison across the four configurations that isolate each component.
#[derive(Debug, Clone, Serialize)]
pub struct HarnessReport {
    pub kn_trigram_ppl: f64,
    pub phiano_best_valid_ppl: f64,
    pub phiano_final_valid_ppl: f64,
    pub best_epoch: usize,
    pub attraction_only_best_ppl: f64,
    /// Same counts, same interpolation, unigram back-off instead of phase.
    pub no_phase_backoff_ppl: f64,
    /// (gamma, perplexity) over the mixing grid, at the best epoch.
    pub gamma_sweep: Vec<(f64, f64)>,
    /// The mixing weight that minimised held-out perplexity.
    pub best_gamma: f64,
    /// Same sweep, on a facet trained with extra predictive passes. Answers
    /// whether the predictive objective is what makes the manifold contribute.
    pub gamma_sweep_predictive: Vec<(f64, f64)>,
    pub best_gamma_predictive: f64,
    pub verdict: String,
    pub epochs: Vec<EpochMetric>,
}

pub struct Harness;

/// One cell of the experiment grid.
#[derive(Debug, Clone, Serialize)]
pub struct SweepRow {
    /// How the context vector was built: "2-word" or "recurrent".
    pub context: String,
    /// Softmax inverse temperature of the phase distribution.
    pub beta: f64,
    /// Mixing weight between the phase back-off and a unigram back-off.
    pub gamma: f64,
    pub ppl: f64,
}

/// How the context vector is built from the prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextKind {
    /// Recency-weighted sum of the two preceding words. Order enters only as a
    /// magnitude weight, so a swap barely moves the vector.
    TwoWord,
    /// The same two words, each rotated by its offset from the prediction point.
    /// Position is carried in the phase, so a swap is a different context.
    Bound,
    /// Diagonal complex recurrence over the whole prefix, with a per-channel
    /// timescale. Order enters through the rotation kernel.
    Recurrent,
}

/// Channels used by the language model's phase back-off.
///
/// The full torus is 64 channels; scoring every position against all of them is
/// affordable but wasteful, and 16 already captures far more than the single
/// angle a 1-D representation could offer.
const LM_CHANNELS: usize = 16;

/// Inverse temperature of the phase back-off distribution.
///
/// Scores are the *mean* cosine across channels, so they live in [-1, 1] and
/// beta is a real temperature. With a per-channel sum the scale was ~16× wider,
/// the softmax saturated, and every non-top candidate underflowed the
/// probability floor — which made the phase back-off a constant and guaranteed
/// it lost to a unigram at every mixing weight.
const PHASE_BETA: f64 = 8.0;

/// Absolute discount for Phiano's own n-gram tables.
const DISCOUNT: f64 = 0.75;

/// Mixing weights swept when measuring the phase layer's contribution.
pub const GAMMA_GRID: [f64; 11] =
    [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

/// A proper language model over a trained facet.
///
/// Phiano's bigram and trigram tables are maximum-likelihood estimates: an
/// unseen n-gram has probability exactly zero, which makes held-out perplexity
/// infinite and the model brittle off-distribution. This wraps them in absolute
/// discounting with a hierarchical back-off:
///
/// ```text
/// P(c | a,b) = max(C(abc)−D,0)/C(ab) + λ(ab)·P(c | b)
/// P(c | b)   = max(C(bc)−D,0)/C(b)   + λ(b)·P_base(c | ctx)
/// P_base     = γ·P_phase(c | ctx) + (1−γ)·P_unigram(c)
/// ```
///
/// `P_phase` is a softmax over **mean phase coherence across `LM_CHANNELS`
/// channels** — the similarity the torus representation exists for, rather than
/// a single angle.
///
/// Because γ = 0 removes the manifold entirely while changing nothing else,
/// sweeping γ measures exactly what the phase layer contributes. If the best γ
/// is 0, it contributes nothing; if the best γ is positive, the gap is the
/// contribution, in perplexity, on held-out text.
pub struct PhianoLM<'a> {
    pub(super) facet: &'a Facet,
    pub(super) index: std::collections::HashMap<String, usize>,
    /// Flattened V × (2·LM_CHANNELS): cos and sin of every channel angle.
    pub(super) vecs: Vec<f64>,
    pub(super) unigram: Vec<f64>,
    pub(super) gamma: f64,
    /// Target sector of every vocabulary item, on channel 0. Precomputed so the
    /// readout lookup inside the scoring loop is one indexed read.
    pub(super) sectors: Vec<usize>,
    /// The conditional non-linear correction, once one has been fitted.
    pub(super) readout: Option<crate::metrics::nonlinear::SectorReadout>,
}

/// Weight of the readout's bias against the raw coherence score.
///
/// Coherence dots live in [-1, 1] and the bias is clamped to [-2, 2]. Weight 1
/// would let the table dominate the manifold outright, which would measure the
/// table rather than the combination.
const READOUT_WEIGHT: f64 = 0.25;
