#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

/// Saturation must be measured out of sample, or it confirms any bound.
///
/// On planted data with exactly two relations, held-out agreement should not
/// keep climbing once k passes two. The in-sample equivalent would, which is
/// why it is not what this reports.
#[test]
fn test_saturation_flattens_once_the_planted_types_are_covered() {
    let heads: Vec<String> = (0..40).map(|i| format!("h{i}")).collect();
    let mut words: Vec<String> = Vec::new();
    for h in &heads {
        words.push(h.clone());
        words.push(format!("{h}_x"));
        words.push(format!("{h}_y"));
    }
    let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
    let mut f = facet_with(&refs);
    for h in &heads {
        let base: Vec<f64> = (0..PHASE_CHANNELS).map(|k| f.lexicon[h].theta(k)).collect();
        for (suffix, delta) in [("_x", 0.7), ("_y", -1.9)] {
            let p = f.lexicon.get_mut(&format!("{h}{suffix}")).unwrap();
            for (k, b) in base.iter().enumerate() {
                p.set_theta(k, b + delta);
            }
            p.sync_phase();
        }
    }
    let pairs: Vec<(String, String)> = heads
        .iter()
        .flat_map(|h| {
            [
                (h.clone(), format!("{h}_x")),
                (h.clone(), format!("{h}_y")),
            ]
        })
        .collect();

    let curve = KindTable::saturation(&f, &pairs, &[1, 2, 4, 8], 12);
    let at = |k: usize| curve.iter().find(|p| p.k == k).unwrap();

    assert!(
        at(2).gain() > at(1).gain(),
        "the second planted type should be worth having: {:.3} -> {:.3}",
        at(1).gain(),
        at(2).gain()
    );
    assert!(
        at(8).gain() <= at(2).gain() + 0.02,
        "there are only two relations here; k=8 should buy nothing: \
         k2 {:.3}, k8 {:.3}",
        at(2).gain(),
        at(8).gain()
    );
}
