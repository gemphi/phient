#[allow(unused_imports)]
use super::*;

impl DefinitionGrounder {
    /// Relaxes every known word toward the centroid of its definition.
    pub fn ground_phases(facet: &mut Facet, chunk_store: &ChunkStore) -> usize {
        Self::ground_from(facet, chunk_store)
    }

    /// Grounds a facet by the rule the measurements selected.
    ///
    /// The centroid grounder above writes `theta(0)` and nothing else, and it
    /// was measured: it halves phase dispersion and moves no relation metric.
    /// [`Conception`] composes across all 64 channels, and across five seeds on
    /// a 296-pair benchmark takes analogy MRR from 0.0002 ± 0.0001 to
    /// 0.0270 ± 0.0031 — an effect that clears its own noise by a wide margin.
    ///
    /// Both paths stay reachable for one release. `GROUND_BY_COMPOSITION`
    /// selects; the old path is retired once this one has run in anger.
    ///
    /// # The guard
    ///
    /// Composition concentrates the manifold — that is how it creates concept
    /// regions — and past a point concentration *is* collapse, which is the
    /// failure the whole harness exists to detect. So the result is checked
    /// before it is kept: if phase dispersion falls below
    /// [`DISPERSION_FLOOR`], the composed facet is discarded and the caller
    /// keeps what it had. A relational gain bought with a collapsed manifold is
    /// not a gain, and shipping one silently would undo the measurement
    /// discipline that produced the rule.
    pub fn ground_best(facet: &mut Facet, chunk_store: &ChunkStore) -> usize {
        if !crate::train::constants::GROUND_BY_COMPOSITION {
            return Self::ground_from(facet, chunk_store);
        }
        Self::ground_best_from(
            facet,
            chunk_store,
            crate::train::constants::COMPOSITION_ANCHOR,
            crate::train::constants::GROUNDING_ROUNDS,
        )
    }

    /// [`DefinitionGrounder::ground_best`] over any source, at explicit
    /// settings.
    ///
    /// Split out so the guard can be driven from a test: a rejection path that
    /// has never been observed to reject is not a guard, it is a comment.
    pub fn ground_best_from<G: Groundable>(
        facet: &mut Facet,
        source: &G,
        anchor: f64,
        rounds: usize,
    ) -> usize {
        let entries: Vec<(String, String)> = source
            .entries()
            .into_iter()
            .filter(|(w, _)| facet.lexicon.contains_key(w))
            .map(|(w, d)| (w, crate::train::sources::definition_core(&d)))
            .filter(|(_, d)| !d.split_whitespace().collect::<Vec<_>>().is_empty())
            .collect();

        if entries.is_empty() {
            return 0;
        }

        let (before, before_band) = dispersion_pair(facet);
        let mut candidate = facet.clone();
        let report = Conception::compose_anchored(
            &mut candidate,
            &entries,
            rounds,
            crate::train::conception::HEAD_STEP,
            crate::train::conception::BETA_STRONG,
            crate::train::conception::BETA_STRONG,
            false,
            None,
            anchor,
            None,
        );
        let (after, after_band) = dispersion_pair(&candidate);

        if after < DISPERSION_FLOOR || after_band < DISPERSION_FLOOR {
            let which = match after < DISPERSION_FLOOR {
                true => "global",
                false => "top-band",
            };
            println!(
                "  [compose] REJECTED on {}: dispersion {:.3} -> {:.3}, \
                 top-{} band {:.3} -> {:.3}, floor {:.2}. \
                 Keeping the pre-composition manifold.",
                which, before, after, GUARD_BAND_TOP, before_band, after_band, DISPERSION_FLOOR
            );
            return 0;
        }

        *facet = candidate;
        println!(
            "  [compose] {} of {} definitions composed, dispersion {:.3} -> {:.3}, \
             top-{} band {:.3} -> {:.3}",
            report.heads_moved,
            entries.len(),
            before,
            after,
            GUARD_BAND_TOP,
            before_band,
            after_band
        );
        report.heads_moved
    }

    /// Grounds a facet from any [`Groundable`] source.
    pub fn ground_from<G: Groundable>(facet: &mut Facet, source: &G) -> usize {
        let entries = source.entries();
        if entries.is_empty() {
            return 0;
        }

        println!(
            "  [ground] Re-seeding phases from {} {} entries...",
            entries.len(),
            source.source_name()
        );
        let mut grounded = 0usize;

        for round in 0..GROUNDING_ROUNDS {
            // ---- READ pass: compute every target against frozen phases ----
            let mut updates: Vec<(String, f64)> = Vec::with_capacity(entries.len());
            for (word, def) in &entries {
                if !facet.lexicon.contains_key(word) {
                    continue;
                }
                if let Some(target) = Self::definition_centroid(facet, def) {
                    updates.push((word.clone(), target));
                }
            }

            // ---- WRITE pass: relax halfway toward each target ----
            let mut moved = 0usize;
            let mut total_shift = 0.0f64;
            for (word, target) in &updates {
                if let Some(phasor) = facet.lexicon.get_mut(word) {
                    let current = phasor.theta(0);
                    let mut diff = target - current;
                    if diff > PI {
                        diff -= TWO_PI;
                    }
                    if diff < -PI {
                        diff += TWO_PI;
                    }
                    phasor.set_theta(0, current + 0.5 * diff);
                    phasor.sync_phase();
                    total_shift += (0.5 * diff).abs();
                    moved += 1;
                }
            }

            grounded = moved;
            let mean_shift = if moved > 0 { total_shift / moved as f64 } else { 0.0 };
            println!(
                "  [ground] round {}/{}: {} phases, mean shift {:.5} rad",
                round + 1,
                GROUNDING_ROUNDS,
                moved,
                mean_shift
            );

            // 0.5 damping halves the remaining error each round, so this
            // converges geometrically; stop once it stops mattering.
            if mean_shift < 0.001 {
                break;
            }
        }

        facet.grounded_version = crate::train::constants::GROUNDING_VERSION;
        println!("  [ground] Grounded {} word phases from definitions.", grounded);
        grounded
    }
}
