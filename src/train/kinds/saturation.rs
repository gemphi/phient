#[allow(unused_imports)]
use super::*;

impl KindTable {

    /// Does held-out structure keep improving as types are added?
    ///
    /// For each `k`: fit centroids on half the pairs, then measure how well the
    /// *other* half's offsets agree with their nearest fitted centroid. In-sample
    /// tightness rises with `k` mechanically and would confirm any bound; this
    /// does not.
    ///
    /// The `shuffled` column repeats everything on pairs whose fillers have been
    /// permuted, which destroys the relations while preserving the vocabulary,
    /// the offset distribution's marginals and the cluster count. A held-out
    /// curve that the null tracks is measuring k-means, not language.
    ///
    /// A bounded type system shows up as [`SaturationPoint::gain`] flattening
    /// below the cap. A gain still climbing at the cap says the bound is wrong,
    /// and that is a result this is allowed to return.
    pub fn saturation(
        facet: &Facet,
        pairs: &[(String, String)],
        ks: &[usize],
        rounds: usize,
    ) -> Vec<SaturationPoint> {
        ks.iter()
            .map(|&k| Self::saturation_at(facet, pairs, k, rounds))
            .collect()
    }

    /// One point of [`KindTable::saturation`].
    ///
    /// Split out so a caller sweeping a long list of `k` can print each row as
    /// it lands. A sweep that speaks only once every value has finished gives no
    /// way to tell a slow run from a wedged one, and the large `k` values are
    /// both the slowest and the ones the claim turns on.
    pub fn saturation_at(
        facet: &Facet,
        pairs: &[(String, String)],
        k: usize,
        rounds: usize,
    ) -> SaturationPoint {
        Self::saturation_at_with(facet, pairs, k, rounds, OffsetEstimator::Raw)
    }

    /// [`KindTable::saturation_at`] under a chosen offset estimator.
    ///
    /// The estimator is applied to the null exactly as to the signal. That is
    /// what makes the comparison legitimate: centering mechanically reduces head
    /// domination, so a centered arm scored against an uncentered null would
    /// "improve" for free. Both arms are centered, or neither.
    pub fn saturation_at_with(
        facet: &Facet,
        pairs: &[(String, String)],
        k: usize,
        rounds: usize,
        est: OffsetEstimator,
    ) -> SaturationPoint {
        let (fit, held, null_fit, null_held) = Self::split(pairs);
        let (null_fit_b, null_held_b) = (
            permute_fillers(&fit, 0x1234_5678_9ABC_DEF1),
            permute_fillers(&held, 0x0FED_CBA9_8765_4321),
        );
        SaturationPoint {
            k,
            held_out: Self::held_out_agreement(facet, &fit, &held, k, rounds, est),
            shuffled: Self::held_out_agreement(facet, &null_fit, &null_held, k, rounds, est),
            shuffled_b: Self::held_out_agreement(facet, &null_fit_b, &null_held_b, k, rounds, est),
        }
    }

    /// The fit/held halves and their shuffled nulls.
    #[allow(clippy::type_complexity)]
    pub(super) fn split(
        pairs: &[(String, String)],
    ) -> (
        Vec<(String, String)>,
        Vec<(String, String)>,
        Vec<(String, String)>,
        Vec<(String, String)>,
    ) {
        // A deterministic split, keyed on the pair's *content* rather than its
        // position. Splitting on index parity looks equally deterministic and is
        // a trap: any input where pairs alternate by relation — which is exactly
        // how a per-head extractor emits them — puts every instance of one
        // relation in the fit half and every instance of another in the held
        // half, so the centroids are fitted on relations the evaluation never
        // contains. That produces strongly *negative* agreement and reads as
        // "more types are worse", which is a fact about the split.
        //
        // Hashing the pair is not by itself enough either. FNV-1a's final step
        // is a multiply by an odd prime, which preserves the low bit, so that
        // bit is a parity of the input bytes — and `_x` and `_y` differ by one
        // in their last byte, which put every pair of one relation on one side
        // and every pair of the other on the other, exactly as index parity had.
        // See [`split_side`].
        let mut fit = Vec::new();
        let mut held = Vec::new();
        for p in pairs {
            match split_side(&p.0, &p.1) {
                true => fit.push(p.clone()),
                false => held.push(p.clone()),
            }
        }
        let null_fit = permute_fillers(&fit, 0x9E37_79B9_7F4A_7C15);
        let null_held = permute_fillers(&held, 0xC2B2_AE3D_27D4_EB4F);
        (fit, held, null_fit, null_held)
    }

    /// Deliberately clusters *uncapped*.
    ///
    /// [`KindTable::discover`] clamps to [`MAX_RELATION_TYPES`], which is
    /// right for the type — its handles have to fit — and fatal here. A sweep
    /// that silently repeats k=64 for every k above it cannot find the cap too
    /// small; it would report a flat curve past the cap and call that
    /// saturation, confirming the bound by construction. The measurement
    /// therefore goes straight to the clustering.
    pub(super) fn held_out_agreement(
        facet: &Facet,
        fit: &[(String, String)],
        held: &[(String, String)],
        k: usize,
        rounds: usize,
        est: OffsetEstimator,
    ) -> f64 {
        let fit_table = OffsetTable::build(facet, fit, est);
        let found = RoleDiscovery::cluster(&fit_table.usable(), k, rounds);
        if found.is_empty() {
            return 0.0;
        }
        let rotations: Vec<Vec<f64>> = found.into_iter().map(|d| d.rotation).collect();

        // The held-out half is centered on *its own* statistics, not the fit
        // half's. Centering a held-out pair with means computed from the training
        // pairs would leak, and centering it with means that include itself would
        // not be held out either — this is the honest middle: the estimator is a
        // property of the set it is applied to, and the two sets are disjoint.
        let held_table = OffsetTable::build(facet, held, est);
        phical::circle::mean_nearest(&held_table.offsets, &rotations)
    }
}

/// Which half of the fit/held split a pair falls in.
///
/// The hash is finalised with a bit-mixing step before a bit is taken from it.
/// FNV-1a alone will not do: its last operation is a multiply by an odd
/// constant, so the low bit of the digest is the parity of the input bytes, and
/// two words differing by one in their final character land on opposite sides
/// every time. That is not a hypothetical — it stratified this very test, fitting
/// centroids on one planted relation and scoring them on the other, and the
/// resulting curve said confidently that more types are worse.
pub(super) fn split_side(head: &str, filler: &str) -> bool {
    let h = phiecs::phasor::spectral::fnv1a(&format!("{head}\u{1}{filler}"));
    // splitmix64's finaliser: avalanches every input bit into every output bit.
    let mut z = h.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    z >> 63 == 0
}
