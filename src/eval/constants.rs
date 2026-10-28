//! The evaluator's weights. They were phiano's `config` constants; phiano's
//! `config` re-exports them from here.

/// Novelty distance scaling factor.
/// novelty = 1 - exp(-distance × NOVELTY_SCALE)
pub const NOVELTY_SCALE: f64 = 0.3;

/// Coherence weight in overall eval score.
pub const EVAL_WEIGHT_COHERENCE: f64 = 0.4;

/// Novelty weight in overall eval score.
pub const EVAL_WEIGHT_NOVELTY: f64 = 0.3;

/// Resonance weight in overall eval score.
pub const EVAL_WEIGHT_RESONANCE: f64 = 0.3;
