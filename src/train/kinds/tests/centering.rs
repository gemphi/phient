#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

/// Double-centering leaves nothing when there is nothing but main effects.
///
/// Offsets built purely from a head term and a filler term have no
/// interaction, so the two-way residual is zero and any clustering of it is
/// clustering noise. If the real data behaves like this, the manifold
/// encodes no relations and no readout can recover them.
#[test]
fn test_double_centering_annihilates_a_pure_main_effect_table() {
    let mut f = Facet::new();
    let heads: Vec<String> = (0..12).map(|i| format!("h{i}")).collect();
    let fillers: Vec<String> = (0..12).map(|i| format!("f{i}")).collect();
    for w in heads.iter().chain(fillers.iter()) {
        f.get_or_init(w);
    }
    // theta(filler) - theta(head) is additive in the two words by
    // construction: the offset table has main effects and no interaction.
    let pairs: Vec<(String, String)> = heads
        .iter()
        .flat_map(|h| fillers.iter().map(move |fl| (h.clone(), fl.clone())))
        .collect();

    let table = OffsetTable::build(&f, &pairs, OffsetEstimator::DoubleCentered);
    let mut worst = 0.0f64;
    for o in &table.offsets {
        for a in o {
            // Residuals should sit at 0 (mod 2pi).
            let d = (a + std::f64::consts::PI).rem_euclid(TWO_PI) - std::f64::consts::PI;
            worst = worst.max(d.abs());
        }
    }
    assert!(
        worst < 1e-6,
        "a purely additive offset table should double-center to zero; \
         largest residual {worst:.6}"
    );
}

/// The medium admits exactly as many modes as it has channels.
///
/// This is where the bound comes from. Mode 64 on a 64-channel torus is mode
/// 0 — it aliases — so there is no 65th standing wave to have, and the cap
/// is geometry rather than a choice.
#[test]
fn test_the_harmonic_basis_has_exactly_channel_count_distinct_modes() {
    let basis = KindTable::harmonic();
    assert_eq!(basis.len(), PHASE_CHANNELS);

    // Every mode is distinct from every other.
    for a in 0..PHASE_CHANNELS {
        for b in (a + 1)..PHASE_CHANNELS {
            let same = (0..PHASE_CHANNELS).all(|k| {
                let d = (basis.rotations[a][k] - basis.rotations[b][k]).abs();
                d < 1e-9 || (d - TWO_PI).abs() < 1e-9
            });
            assert!(!same, "modes {a} and {b} are the same rotation");
        }
    }

    // And the (n+1)-th aliases onto the first.
    let wrapped = harmonic_rotation(PHASE_CHANNELS, PHASE_CHANNELS);
    for (k, w) in wrapped.iter().enumerate() {
        let d = (w - basis.rotations[0][k]).abs();
        assert!(d < 1e-9 || (d - TWO_PI).abs() < 1e-9, "mode n should alias to mode 0");
    }
}

/// A pair built on a mode must be recognised as that mode, with nothing
/// fitted.
#[test]
fn test_a_pair_built_on_a_mode_is_read_back_as_that_mode() {
    let basis = KindTable::harmonic();
    let mut f = Facet::new();
    f.get_or_init("head");
    let base: Vec<f64> = (0..PHASE_CHANNELS).map(|k| f.lexicon["head"].theta(k)).collect();

    for m in [1usize, 5, 17, 40] {
        let rot = harmonic_rotation(m, PHASE_CHANNELS);
        let name = format!("filler{m}");
        let p = f.get_or_init(&name);
        for k in 0..PHASE_CHANNELS {
            p.set_theta(k, base[k] + rot[k]);
        }
        p.sync_phase();

        let pairs = vec![("head".to_string(), name.clone())];
        let table = OffsetTable::build(&f, &pairs, OffsetEstimator::Raw);
        let (best, agree) = table.best_match(0, &basis.rotations).expect("a nearest mode");
        assert_eq!(best, m, "mode {m} read back as {best}");
        assert!(agree > 0.99, "agreement with its own mode should be ~1, got {agree:.4}");
    }
}

/// The split must not correlate with anything about the pairs.
///
/// This is the bug that produced a confident, entirely false saturation
/// curve. FNV-1a's low bit is the parity of the input bytes, so `h_x` and
/// `h_y` — differing by one in the last character — landed in opposite
/// halves without exception. Every planted relation of one kind went into
/// the fit set and every one of the other kind into the held set, and the
/// measurement dutifully reported that adding types makes things worse.
#[test]
fn test_the_fit_held_split_does_not_stratify_by_a_one_character_difference() {
    let n = 400;
    let mut same = 0usize;
    for i in 0..n {
        let h = format!("h{i}");
        if split_side(&h, &format!("{h}_x")) == split_side(&h, &format!("{h}_y")) {
            same += 1;
        }
    }
    // Independent sides land together about half the time. A hash whose low
    // bit tracks the last byte gives zero, which is what this caught.
    let frac = same as f64 / n as f64;
    assert!(
        (frac - 0.5).abs() < 0.1,
        "`_x` and `_y` should fall on the same side about half the time, got {frac:.3}"
    );

    // And the halves themselves have to be roughly balanced.
    let fit = (0..n).filter(|i| split_side(&format!("h{i}"), "f")).count();
    assert!(
        (fit as f64 / n as f64 - 0.5).abs() < 0.1,
        "unbalanced split: {fit} of {n}"
    );
}
