#[allow(unused_imports)]
use super::*;

impl KindTable {
    pub fn len(&self) -> usize {
        self.rotations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rotations.is_empty()
    }

    /// Every type this corpus produced.
    pub fn all(&self) -> Vec<RelationType> {
        (0..self.rotations.len()).map(|i| RelationType(i as u8)).collect()
    }

    pub fn evidence(&self, t: RelationType) -> Option<&TypeEvidence> {
        self.evidence.get(t.0 as usize)
    }

    /// Discovers the types a corpus's pairs exhibit, at a given count.
    ///
    /// `k` is clamped to [`MAX_RELATION_TYPES`]. Use [`KindTable::saturation`]
    /// to choose it from evidence rather than by taste.
    pub fn discover(facet: &Facet, pairs: &[(String, String)], k: usize, rounds: usize) -> Self {
        Self::discover_with(facet, pairs, k, rounds, OffsetEstimator::Raw)
    }

    /// [`KindTable::discover`] under a chosen offset estimator.
    pub fn discover_with(
        facet: &Facet,
        pairs: &[(String, String)],
        k: usize,
        rounds: usize,
        est: OffsetEstimator,
    ) -> Self {
        let k = k.min(MAX_RELATION_TYPES);
        let table = OffsetTable::build(facet, pairs, est);
        let found = RoleDiscovery::cluster(&table.usable(), k, rounds);
        let rotations: Vec<Vec<f64>> = found.iter().map(|d| d.rotation.clone()).collect();
        let evidence = found
            .into_iter()
            .enumerate()
            .map(|(i, d)| TypeEvidence {
                relation: RelationType(i as u8),
                members: d.members,
                coherence: d.coherence,
            })
            .collect();
        Self { rotations, evidence }
    }

    /// The standing-wave modes of the medium, as relation types.
    ///
    /// Nothing is fitted. Mode `m` is [`harmonic_rotation`], and there are
    /// exactly [`PHASE_CHANNELS`] of them because a bounded medium sustains only
    /// integer harmonics. Compare against [`KindTable::discover`], which
    /// fits 64 free rotations of 64 channels each to the same pairs.
    pub fn harmonic() -> Self {
        let rotations: Vec<Vec<f64>> = (0..PHASE_CHANNELS)
            .map(|m| harmonic_rotation(m, PHASE_CHANNELS))
            .collect();
        let evidence = (0..PHASE_CHANNELS)
            .map(|i| TypeEvidence {
                relation: RelationType(i as u8),
                members: Vec::new(),
                coherence: 0.0,
            })
            .collect();
        Self { rotations, evidence }
    }

    /// Held-out agreement to the fixed harmonic basis, against the same null.
    ///
    /// There is no fit/held split here because there is nothing to fit — every
    /// pair is held out. The null is the same permuted-filler control, put
    /// through the identical basis, so the gain means what it means everywhere
    /// else in this module.
    pub fn harmonic_saturation(
        facet: &Facet,
        pairs: &[(String, String)],
        est: OffsetEstimator,
    ) -> SaturationPoint {
        let basis = Self::harmonic();
        let score = |set: &[(String, String)]| -> f64 {
            let table = OffsetTable::build(facet, set, est);
            let mut sum = 0.0;
            let mut n = 0usize;
            for i in 0..table.len() {
                if let Some((_, a)) = table.best_match(i, &basis.rotations) {
                    if a.is_finite() {
                        sum += a;
                        n += 1;
                    }
                }
            }
            match n {
                0 => 0.0,
                _ => sum / n as f64,
            }
        };
        SaturationPoint {
            k: PHASE_CHANNELS,
            held_out: score(pairs),
            shuffled: score(&permute_fillers(pairs, 0x9E37_79B9_7F4A_7C15)),
            shuffled_b: score(&permute_fillers(pairs, 0x1234_5678_9ABC_DEF1)),
        }
    }

    /// Which mode a pair sits closest to, and how closely.
    ///
    /// The mode index *is* the relation type — an integer, not a fitted vector.
    pub fn harmonic_mode(
        facet: &Facet,
        head: &str,
        filler: &str,
        est_table: &OffsetTable,
        index: usize,
    ) -> Option<(usize, f64)> {
        let _ = (facet, head, filler);
        let basis = Self::harmonic();
        est_table.best_match(index, &basis.rotations)
    }

    /// Binds a head to a relation type: the head's phases rotated by the type's.
    ///
    /// The caller supplies a handle it obtained from discovery. It never sees
    /// the rotation, which is the point — the representation is free to change
    /// without touching a single call site.
    pub fn bind(&self, facet: &Facet, head: &str, t: RelationType) -> Option<SpectralPhasor> {
        let rot = self.rotations.get(t.0 as usize)?;
        let p = facet.lexicon.get(head)?;
        let mut out = *p;
        for (k, r) in rot.iter().enumerate().take(PHASE_CHANNELS) {
            out.set_theta(k, p.theta(k) + r);
        }
        out.sync_phase();
        Some(out)
    }

    /// The inverse: recovers what a bound phasor was bound *from*.
    pub fn unbind(&self, bound: &SpectralPhasor, t: RelationType) -> Option<SpectralPhasor> {
        let rot = self.rotations.get(t.0 as usize)?;
        let mut out = *bound;
        for (k, r) in rot.iter().enumerate().take(PHASE_CHANNELS) {
            out.set_theta(k, bound.theta(k) - r);
        }
        out.sync_phase();
        Some(out)
    }

    /// Which type — if any — the step from `head` to `filler` belongs to.
    ///
    /// Returns the best type and its agreement. A pair standing in no discovered
    /// relation has a low agreement with all of them, so the caller can decide
    /// what to do about it; this does not invent a type to hold it.
    pub fn classify(
        &self,
        facet: &Facet,
        head: &str,
        filler: &str,
    ) -> Option<(RelationType, f64)> {
        let off = RoleDiscovery::offset(facet, head, filler)?;
        self.rotations
            .iter()
            .enumerate()
            .map(|(i, r)| (RelationType(i as u8), RoleDiscovery::agreement(&off, r)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}
