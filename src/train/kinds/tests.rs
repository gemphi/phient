#[allow(unused_imports)]
use super::*;

pub(super) use phiton::units::TWO_PI;

pub(super) fn facet_with(words: &[&str]) -> Facet {
    let mut f = Facet::new();
    for w in words {
        f.get_or_init(w);
    }
    f
}

/// Two planted relations, built by construction, must come back as two
/// types — without either being named.
#[test]
fn test_two_planted_relations_are_recovered_as_two_types() {
    let heads = ["a", "b", "c", "d", "e", "f"];
    let mut words: Vec<String> = Vec::new();
    for h in heads {
        words.push(h.to_string());
        words.push(format!("{h}_x"));
        words.push(format!("{h}_y"));
    }
    let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
    let mut f = facet_with(&refs);

    // Relation one: rotate by 0.7 on every channel. Relation two: by -1.9.
    for h in heads {
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
                (h.to_string(), format!("{h}_x")),
                (h.to_string(), format!("{h}_y")),
            ]
        })
        .collect();

    let types = KindTable::discover(&f, &pairs, 2, 12);
    assert_eq!(types.len(), 2);

    // Every `_x` pair lands in one type and every `_y` pair in the other,
    // and neither type is called anything.
    let of = |suffix: &str| -> RelationType {
        let (h, fl) = ("a".to_string(), format!("a{suffix}"));
        types.classify(&f, &h, &fl).unwrap().0
    };
    assert_ne!(of("_x"), of("_y"));
    for h in heads {
        assert_eq!(types.classify(&f, h, &format!("{h}_x")).unwrap().0, of("_x"));
        assert_eq!(types.classify(&f, h, &format!("{h}_y")).unwrap().0, of("_y"));
    }
}

/// Bind and unbind are inverses, through the handle only.
#[test]
fn test_bind_unbind_round_trips_through_the_handle() {
    let f = facet_with(&["alpha", "beta", "gamma", "delta"]);
    let pairs = vec![
        ("alpha".to_string(), "beta".to_string()),
        ("gamma".to_string(), "delta".to_string()),
    ];
    let types = KindTable::discover(&f, &pairs, 2, 8);
    let t = types.all()[0];

    let bound = types.bind(&f, "alpha", t).unwrap();
    let back = types.unbind(&bound, t).unwrap();
    for k in 0..PHASE_CHANNELS {
        let d = (back.theta(k) - f.lexicon["alpha"].theta(k)).abs();
        assert!(d < 1e-9 || (d - phiton::units::TWO_PI).abs() < 1e-9, "channel {k}");
    }
}

/// The null has to be a real null: permuting fillers must actually change
/// the pairs, or every saturation curve sits on top of its own floor and
/// the comparison is vacuous.
#[test]
fn test_the_shuffled_null_actually_permutes() {
    let pairs: Vec<(String, String)> = (0..50)
        .map(|i| (format!("h{i}"), format!("f{i}")))
        .collect();
    let null = permute_fillers(&pairs, 12345);
    let unchanged = pairs.iter().zip(&null).filter(|(a, b)| a.1 == b.1).count();
    assert!(
        unchanged < pairs.len() / 10,
        "{unchanged} of {} fillers stayed put",
        pairs.len()
    );
}

/// Centering must remove the head main effect and keep the relation.
///
/// Built so the two are separable: every pair's offset is a per-head
/// constant (the main effect) plus one of two relation rotations (the
/// interaction). Raw offsets are dominated by the head constant and cluster
/// by head. Head-centered offsets have the constant gone, so they must
/// cluster by relation instead — the thing the real data does not do.
#[test]
fn test_head_centering_removes_the_head_effect_and_keeps_the_relation() {
    const HEADS: usize = 24;
    let mut f = Facet::new();
    let mut pairs: Vec<(String, String)> = Vec::new();

    for i in 0..HEADS {
        let h = format!("h{i}");
        f.get_or_init(&h);
        // A large per-head constant: this is the main effect that buries the
        // relation in the raw difference.
        let bias = TWO_PI * i as f64 / HEADS as f64;
        for (suffix, rel) in [("_x", 0.35), ("_y", -0.35)] {
            let name = format!("{h}{suffix}");
            let base: Vec<f64> = (0..PHASE_CHANNELS).map(|k| f.lexicon[&h].theta(k)).collect();
            let p = f.get_or_init(&name);
            for (k, b) in base.iter().enumerate() {
                p.set_theta(k, b + bias + rel);
            }
            p.sync_phase();
            pairs.push((h.clone(), name));
        }
    }

    let cluster_of = |est: OffsetEstimator, i: usize, suffix: &str| -> usize {
        let types = KindTable::discover_with(&f, &pairs, 2, 20, est);
        let table = OffsetTable::build(&f, &pairs, est);
        let idx = pairs
            .iter()
            .position(|p| p.1 == format!("h{i}{suffix}"))
            .expect("pair present");
        table
            .best_match(idx, &types.rotations)
            .expect("a nearest type")
            .0
    };

    // Raw: two heads with different biases are in different clusters even
    // though they stand in the same relation.
    let raw_same_relation_far_heads =
        cluster_of(OffsetEstimator::Raw, 0, "_x") == cluster_of(OffsetEstimator::Raw, 12, "_x");
    // Head-centered: same relation, same cluster, whatever the head.
    let centered_same_relation = cluster_of(OffsetEstimator::HeadCentered, 0, "_x")
        == cluster_of(OffsetEstimator::HeadCentered, 12, "_x");
    let centered_different_relation = cluster_of(OffsetEstimator::HeadCentered, 0, "_x")
        != cluster_of(OffsetEstimator::HeadCentered, 0, "_y");

    assert!(
        !raw_same_relation_far_heads,
        "the fixture must actually bury the relation under the head effect, \
         or the test proves nothing"
    );
    assert!(
        centered_same_relation,
        "head-centering should put the same relation in one cluster across heads"
    );
    assert!(
        centered_different_relation,
        "head-centering should still separate the two relations"
    );
}

#[cfg(test)]
mod centering;

#[cfg(test)]
mod saturation;
