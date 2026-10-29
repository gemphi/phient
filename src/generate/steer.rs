//! Attention re-ranking, the torus ray-cast fallback, the phase kick and recency eviction.

use crate::generate::constants::TWO_PI;
use super::generator::Generator;
use phiecs::phasor::attention;
use crate::generate::constants::TORUS_DECODE_POOL;
use phiecs::phasor::facet::Facet;
use phiecs::phasor::flow::PhaseFlow;
use phiecs::phasor::spectral::{SpectralPhasor, TorusPhasor};
use phiecs::phasor::wave::Wave;
use phistd::c64;

impl Generator {
    #[allow(dead_code)]
    pub(super) fn attention_pick(
        &self,
        facet: &Facet,
        prompt_tokens: &[String],
        generated: &[String],
        candidates: &[(String, u32)],
        recent: &std::collections::HashSet<String>,
        target_phase: f64,
    ) -> Option<String> {
        let ctx: Vec<String> = generated.iter().take(8).cloned()
            .chain(prompt_tokens.iter().take(4).cloned()).collect();
        let scored = attention::Attention::next_words(facet, &ctx, candidates, target_phase, 5);
        scored.iter()
            .filter(|(w, _)| !recent.contains(w))
            .map(|(w, _)| w.clone())
            .next()
    }

    /// Torus ray-cast among speakable, high-amplitude words only.
    pub(super) fn torus_ray_cast(
        &self,
        facet: &Facet,
        flow: &PhaseFlow,
        target_phase: f64,
        recent: &std::collections::HashSet<String>,
    ) -> Option<String> {
        let target_phasor = SpectralPhasor::new(target_phase, 1.0, 0);
        let target_torus = TorusPhasor::from_spectral(&target_phasor);
        let target_wave = c64::new(target_phase.cos(), target_phase.sin());
        let pool = Wave::ray_cast(facet, target_wave, TORUS_DECODE_POOL * 4);

        let mut scored_pool: Vec<(String, f64)> = pool
            .into_iter()
            .filter(|(w, _)| Self::speakable(w) && !recent.contains(w.as_str()))
            .filter_map(|(w, _)| {
                facet.lexicon.get(&w).and_then(|p| {
                    if p.amplitude < 1.05 {
                        return None;
                    }
                    let word_torus = TorusPhasor::from_spectral(p);
                    let res = target_torus.resonance(&word_torus);
                    let flow_res = flow.resonance_with(facet, &w);
                    Some((w, 0.6 * res + 0.4 * flow_res + 0.05 * p.amplitude))
                })
            })
            .collect();

        scored_pool.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_pool.first().map(|(w, _)| w.clone())
    }

    /// Applies a phase-kick to the context wave after emitting a token.
    /// Uses learned β_ij phase lag between prev_word and word for syntax-aware steering.
    pub(super) fn apply_phase_kick(
        &self,
        facet: &Facet,
        prev_word: &Option<String>,
        word: &str,
        current_phase: &mut f64,
        phase_momentum: &mut f64,
    ) {
        if let Some(phasor) = facet.lexicon.get(word) {
            let beta = match prev_word {
                Some(prev) => facet.phase_lag(prev, word),
                None => 0.0,
            };
            let phase_diff = (phasor.phase - *current_phase + beta).sin();
            *current_phase = (*current_phase + 0.35 * phase_diff).rem_euclid(TWO_PI);
            *phase_momentum = (0.85 * *phase_momentum + 0.15 * phase_diff.abs().max(0.05)).min(0.5);
        }
    }

    /// Evicts the oldest token from the recent-words set.
    pub(super) fn evict_old(
        &self,
        recent: &mut std::collections::HashSet<String>,
        generated: &[String],
        window: usize,
    ) {
        if generated.len() > window {
            if let Some(old) = generated.get(generated.len() - window) {
                recent.remove(old);
            }
        }
    }

}
