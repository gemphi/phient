#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

/// The anchor must bound how far composition can move a word, and must be
/// exactly inert at 0 so the sweep over it is a controlled comparison.
#[test]
fn test_anchor_bounds_drift_and_is_inert_at_zero() {
    let entries = vec![
        ("cat".to_string(), "small domestic feline animal".to_string()),
        ("feline".to_string(), "of the cat animal".to_string()),
    ];
    let words = ["cat", "animal", "small", "domestic", "feline"];
    let start = facet_with(&words);

    let drift = |anchor: f64| {
        let mut f = start.clone();
        Conception::compose_anchored(
            &mut f, &entries, 6, HEAD_STEP, REINFORCE, REINFORCE, false, None, anchor, None,
        );
        (0..PHASE_CHANNELS)
            .map(|k| {
                phical::circle::delta(start.lexicon["cat"].theta(k), f.lexicon["cat"].theta(k))
                    .abs()
            })
            .sum::<f64>()
    };

    let free = drift(0.0);
    let anchored = drift(ANCHOR);
    assert!(
        anchored < free,
        "the anchor must reduce drift: {} anchored vs {} free",
        anchored,
        free
    );

    // anchor = 0 must reproduce the unanchored path bit for bit.
    let mut a = start.clone();
    let mut b = start.clone();
    Conception::compose_anchored(
        &mut a, &entries, 3, HEAD_STEP, REINFORCE, REINFORCE, false, None, 0.0, None,
    );
    Conception::compose_all_bound(&mut b, &entries, 3, HEAD_STEP, REINFORCE, false);
    for k in 0..PHASE_CHANNELS {
        assert_eq!(a.lexicon["cat"].theta(k), b.lexicon["cat"].theta(k));
    }
}

/// Promotion must be able to create strong pairs, and must never invent a
/// relation between words the dictionary never connected.
///
/// The assertions are structural rather than "word X is promoted", because
/// which words are nearest depends on hash-seeded initial phases and a test
/// that depends on that is a test of the seed.
#[test]
fn test_knn_promotion_is_creative_symmetric_and_bounded() {
    let entries = vec![
        ("car".to_string(), "road vehicle engine".to_string()),
        ("vehicle".to_string(), "car transport".to_string()),
        ("road".to_string(), "a wide way".to_string()),
        ("engine".to_string(), "a machine".to_string()),
    ];
    let mut g = DefinitionGraph::build(&entries);
    let before = g.counts().0;

    let f = facet_with(&[
        "car", "vehicle", "road", "engine", "transport", "wide", "way", "machine", "orange",
    ]);
    let promoted = g.promote_neighbours(&f, 5);

    // Creative: the whole point is turning one-way edges into strong pairs.
    // The first implementation ranked each word only among the words it
    // defines, which made `a` reachable in `b`'s top-k only when the pair
    // was *already* strong. It promoted 0 of 271,300 pairs on the real
    // dictionary and passed its test.
    assert!(promoted > 0, "promotion must be able to create a strong pair");
    assert!(g.counts().0 > before, "strong count must rise");

    // Symmetric: strength is a property of the pair, not of the direction.
    for (a, b) in [("car", "vehicle"), ("car", "road"), ("road", "wide")] {
        assert_eq!(
            g.is_strong(a, b),
            g.is_strong(b, a),
            "strength must be symmetric for {}/{}",
            a,
            b
        );
    }

    // Bounded: "orange" appears in no definition and defines nothing, so no
    // amount of nearness may connect it to anything.
    for other in ["car", "road", "vehicle", "engine"] {
        assert!(
            !g.is_related("orange", other),
            "a word the dictionary never connected must stay unconnected: \
             orange/{}",
            other
        );
    }
}

/// The kernel must keep the words that define others and drop the leaves.
#[test]
fn test_grounding_kernel_keeps_the_defining_core() {
    // cat and feline define each other, so both survive. "aardvark" has an
    // entry but appears in nobody else's definition — it is reachable by
    // definition and defines nothing further, so it peels. "burrower" only
    // ever defines aardvark, so once aardvark is gone it peels too: that
    // second removal is what makes the rule recursive rather than a single
    // degree filter.
    let entries = vec![
        ("cat".to_string(), "feline animal".to_string()),
        ("feline".to_string(), "cat animal".to_string()),
        ("animal".to_string(), "a living cat or feline".to_string()),
        ("aardvark".to_string(), "burrower animal".to_string()),
        ("burrower".to_string(), "one that digs".to_string()),
    ];
    let g = DefinitionGraph::build(&entries);
    let k = g.kernel();
    assert!(
        k.contains("cat") && k.contains("feline") && k.contains("animal"),
        "the mutually-defining core survives: {:?}",
        k
    );
    assert!(!k.contains("aardvark"), "a word nothing defines must peel: {:?}", k);
    assert!(
        !k.contains("burrower"),
        "and so must one that only defined the peeled word: {:?}",
        k
    );
}

/// Iterating must converge, not oscillate: the mean step has to shrink.
#[test]
fn test_composition_converges() {
    let mut f = facet_with(&["cat", "animal", "small", "domestic", "feline"]);
    let entries = vec![
        ("cat".to_string(), "small domestic feline animal".to_string()),
        ("feline".to_string(), "of the cat animal".to_string()),
    ];
    let first = Conception::compose_all(&mut f, &entries, 1).final_mean_shift;
    let later = Conception::compose_all(&mut f, &entries, 6).final_mean_shift;
    assert!(
        later < first,
        "mean shift must fall across rounds: {} then {}",
        first,
        later
    );
}
