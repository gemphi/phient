//! The candidate features the generator scores — one definition, used both
//! by the tuner (offline, from corpus positions) and by the generator (online,
//! from its decoding state), so what is learned is what is used.

use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;

pub const N_FEATURES: usize = 8;
pub const FEATURE_NAMES: [&str; N_FEATURES] =
    ["ln_count", "phase_align", "resonance", "function_word", "repeat", "operator_cos", "relational", "relational_pairs"];

/// Feature vector of one candidate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandidateFeature(pub [f64; N_FEATURES]);

impl CandidateFeature {
    /// `count`: the n-gram count. `target_phase`: the phase the decoder is
    /// steering toward. `collective_phase`: the flow's collective phase.
    /// `recent`: whether the word was emitted recently. `operator_cos`:
    /// the operator's cosine for the word, or 0 without one.
    /// `ctx`: the preceding tokens (the wheels); the relational features are
    /// `Facet::relational_score(ctx, word)` — the mean directed agreement
    /// over the evidenced pairs, and `ln(1 + pairs)`.
    #[allow(clippy::too_many_arguments)] // shared with generate/select.rs; each argument is one scored signal
    pub fn build(
        facet: &Facet,
        word: &str,
        count: u32,
        target_phase: f64,
        collective_phase: f64,
        recent: bool,
        operator_cos: f64,
        ctx: &[String],
    ) -> Self {
        let phase = facet.lexicon.get(word).map(|p| p.phase);
        let phase_align = phase.map_or(0.0, |p| (p - target_phase).cos().max(0.0));
        let resonance = phase.map_or(0.0, |p| (collective_phase - p).cos());
        let (rel, pairs) = facet.relational_score(ctx, word);
        Self([
            (count as f64).min(24.0).ln_1p(),
            phase_align,
            resonance,
            if Tokenizer::is_function_word(word) { 1.0 } else { 0.0 },
            if recent { 1.0 } else { 0.0 },
            operator_cos,
            rel,
            (pairs as f64).ln_1p(),
        ])
    }
}
