//! Catastrophic-forgetting benchmark.
//!
//! `docs/45_native_learning_vs_bloated_llms.md` places Phiano in a comparison
//! matrix against Phi-4, GLM and GPT-4 and asserts **"Catastrophic Forgetting:
//! Zero"**. Nothing in the repository tested that. It is also the one claim
//! where this architecture genuinely should win — updating one word's phase
//! does not overwrite a shared weight matrix — which makes leaving it
//! unmeasured the largest missed opportunity in the project.
//!
//! # Design
//!
//! Two disjoint domains, A and B, and three models:
//!
//! | model | trained on | meaning |
//! |:---|:---|:---|
//! | **ceiling** | A and B together | no forgetting is possible |
//! | **sequential** | A, then B | the thing being measured |
//! | **floor** | B only | total forgetting |
//!
//! All three are scored on held-out **A**. Retention normalises the sequential
//! model between the two bounds, in log space because perplexity is
//! multiplicative:
//!
//! ```text
//! retention = (ln floor − ln sequential) / (ln floor − ln ceiling)
//! ```
//!
//! 1.0 means the second domain cost nothing. 0.0 means A was forgotten as
//! completely as if it had never been trained. Both bounds are measured rather
//! than assumed, so the number cannot be flattered by a weak baseline.

mod benchmark;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use benchmark::*;

use phiecs::phasor::facet::Facet;
use crate::metrics::harness::PhianoLM;
use crate::train::trainer::Trainer;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ForgettingReport {
    pub domain_a_name: String,
    pub domain_b_name: String,
    pub n_train_a: usize,
    pub n_train_b: usize,
    pub n_eval_a: usize,
    /// Trained on A and B together — the no-forgetting bound.
    pub ceiling_ppl: f64,
    /// Trained on A, then B — the measurement.
    pub sequential_ppl: f64,
    /// Trained on B only — the total-forgetting bound.
    pub floor_ppl: f64,
    /// Trained on A only, before B was ever seen.
    pub a_only_ppl: f64,
    /// The sequential model scored from counts alone, for reference. The n-gram
    /// tables are tallies nothing overwrites, so this cannot degrade.
    pub counts_only_ppl: f64,
    /// 1.0 = nothing forgotten, 0.0 = forgotten completely.
    pub retention: f64,
    /// How much worse A got, as a percentage, after learning B.
    pub degradation_pct: f64,
    pub verdict: String,
}

pub struct ForgettingBenchmark;
