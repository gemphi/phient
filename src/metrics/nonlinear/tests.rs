#[allow(unused_imports)]
use super::*;

#[test]
fn test_bias_is_conditional_not_constant() {
    // A bias depending only on the context would add the same value to every
    // candidate and could never change a ranking.
    let mut r = SectorReadout::new();
    let k = r.key(&[0.1, 0.1, 0.1, 0.1]);
    for _ in 0..30 {
        r.learn(k, 5, 40, 0.5);
    }
    assert!(r.bias_for(k, 5) > 0.0, "the observed target sector rises");
    assert!(r.bias_for(k, 40) < 0.0, "the sampled wrong sector falls");
    assert_ne!(r.bias_for(k, 5), r.bias_for(k, 40));
}

#[test]
fn test_distinct_contexts_get_distinct_cells() {
    let r = SectorReadout::new();
    let a = r.key(&[0.01, 0.01, 0.01, 0.01]);
    let b = r.key(&[3.20, 3.20, 3.20, 3.20]);
    assert_ne!(a, b);
    let near = r.key(&[0.011, 0.011, 0.011, 0.011]);
    assert_eq!(a, near, "phases inside one bucket share a cell");
}

/// A finer key distinguishes more contexts and covers fewer of them. This
/// is the trade-off `KEY_SECTORS` exists to control, and the reason the key
/// grid is coarser than the target grid: at 64 buckets over 4 channels the
/// cell space is 16.7M and an unseen context essentially never hits.
#[test]
fn test_finer_keys_cover_less() {
    let contexts: Vec<Vec<f64>> = (0..200)
        .map(|i| {
            let t = i as f64 * 0.031;
            vec![t.sin() + 1.0, t.cos() + 1.0, (2.0 * t).sin() + 1.0, t * 0.7]
        })
        .collect();

    let coverage_at = |sectors: usize| {
        let mut r = SectorReadout::with_shape(4, sectors);
        for c in contexts.iter().take(100) {
            let k = r.key(c);
            r.learn(k, 1, 2, 0.5);
        }
        r.reset_coverage();
        for c in contexts.iter().skip(100) {
            let k = r.key(c);
            r.bias_for(k, 1);
        }
        r.coverage()
    };

    let coarse = coverage_at(4);
    let fine = coverage_at(64);
    assert!(
        coarse > fine,
        "coarse key must cover more held-out contexts: {} vs {}",
        coarse,
        fine
    );
}

#[test]
fn test_rate_decays_and_stays_bounded() {
    let mut r = SectorReadout::new();
    let k = r.key(&[1.0; 4]);
    r.learn(k, 3, 9, 1.0);
    let first = r.bias_for(k, 3);
    for _ in 0..500 {
        r.learn(k, 3, 9, 1.0);
    }
    let later = r.bias_for(k, 3);
    assert!(later > first);
    assert!(later <= 2.0, "bounded: {}", later);
}

#[test]
fn test_gate_suppresses_weak_channels() {
    let mut c = vec![
        c64::from_polar(0.05, 1.0),
        c64::from_polar(2.0, 1.0),
        c64::from_polar(0.5, 1.0),
    ];
    assert_eq!(SectorReadout::gate(&mut c, 0.4), 2);
    assert_eq!(c[0].norm(), 0.0);
}
