#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

fn facet_of(words: &[String]) -> Facet {
    let mut f = Facet::new();
    for w in words {
        f.lexicon
            .insert(w.clone(), SpectralPhasor::seeded(w, 1.0, 1));
    }
    f
}

/// Alignment must raise coherence on a group that has none.
#[test]
fn test_align_raises_coherence() {
    let words: Vec<String> = (0..40)
        .flat_map(|i| [format!("h{}", i), format!("f{}", i)])
        .collect();
    let mut f = facet_of(&words);
    let pairs: Vec<(String, String)> =
        (0..40).map(|i| (format!("h{}", i), format!("f{}", i))).collect();

    let before = Coherence::measure(&f, &pairs);
    let after = Coherence::align(&mut f, &pairs, 0.5);
    assert!(
        after > before + 0.2,
        "alignment must raise coherence: {} → {}",
        before,
        after
    );
}

/// **The degenerate optimum.** A collapsed manifold has perfect coherence,
/// because every offset is zero. The guard must catch that, and must restore
/// the manifold rather than leave it half-aligned.
#[test]
fn test_guard_rejects_collapse_and_restores() {
    // One head, many fillers. Alignment moves fillers toward
    // `head + mean_offset`, so a single shared head drags every filler onto
    // the same point — the degenerate solution this objective has, and the
    // reason the guard is not optional.
    //
    // The first version of this test used one shared *filler* instead, which
    // moves exactly one word and cannot collapse anything. It passed the
    // guard legitimately and proved nothing.
    let mut words: Vec<String> = (0..30).map(|i| format!("f{}", i)).collect();
    words.push("hub".to_string());
    let mut f = facet_of(&words);

    let pairs: Vec<(String, String)> =
        (0..30).map(|i| ("hub".to_string(), format!("f{}", i))).collect();
    let groups = vec![pairs];

    let before: Vec<f64> = (0..PHASE_CHANNELS).map(|k| f.lexicon["f0"].theta(k)).collect();

    let (coh, kept) = Coherence::align_groups(&mut f, &groups, 0.9, 20, 0.40);
    assert!(!kept, "a collapsing alignment must be rejected (coherence {})", coh);
    for (k, &b) in before.iter().enumerate() {
        assert_eq!(
            f.lexicon["f0"].theta(k),
            b,
            "rejection must restore channel {}",
            k
        );
    }
}

/// A healthy alignment across distinct groups must be kept.
#[test]
fn test_healthy_alignment_is_kept() {
    let real = [
        "cat", "dog", "oak", "pine", "rose", "iron", "gold", "wine", "bread", "stone",
        "animal", "tree", "flower", "metal", "drink", "food", "rock", "plant", "beast",
        "timber", "bloom", "ore", "liquor", "loaf", "boulder", "creature", "wood",
    ];
    let words: Vec<String> = real.iter().map(|s| s.to_string()).collect();
    let mut f = facet_of(&words);

    let groups = vec![
        vec![
            ("cat".to_string(), "animal".to_string()),
            ("dog".to_string(), "beast".to_string()),
        ],
        vec![
            ("oak".to_string(), "tree".to_string()),
            ("pine".to_string(), "timber".to_string()),
        ],
        vec![
            ("iron".to_string(), "metal".to_string()),
            ("gold".to_string(), "ore".to_string()),
        ],
    ];

    let (coh, kept) = Coherence::align_groups(&mut f, &groups, 0.3, 3, 0.40);
    assert!(kept, "a spread-preserving alignment must be kept");
    assert!(coh > 0.0);
    assert!(f.phase_dispersion() >= 0.40);
}
