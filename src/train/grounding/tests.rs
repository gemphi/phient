#[allow(unused_imports)]
use super::*;

use phiton::units::PHASE_CHANNELS;
use phiecs::phasor::spectral::SpectralPhasor;

struct Entries(Vec<(String, String)>);
impl Groundable for Entries {
    fn entries(&self) -> Vec<(String, String)> {
        self.0.clone()
    }
}

fn facet_of(words: &[&str]) -> Facet {
    let mut f = Facet::new();
    for w in words {
        f.lexicon
            .insert((*w).to_string(), SpectralPhasor::seeded(w, 1.0, 1));
    }
    f
}

/// The floor must actually fire, and firing must leave the caller's facet
/// untouched rather than partially composed.
///
/// A guard that has never been observed to reject is not a guard. This
/// drives dispersion down deliberately — every word composed toward one
/// shared definition — and asserts the rejection path both triggers and
/// preserves.
#[test]
fn test_dispersion_floor_rejects_and_preserves() {
    let words = ["alpha", "beta", "gamma", "delta", "epsilon", "core"];
    let mut f = facet_of(&words);

    // Every word defined by the same single term collapses them together.
    let entries: Vec<(String, String)> = words
        .iter()
        .filter(|w| **w != "core")
        .map(|w| ((*w).to_string(), "core".to_string()))
        .collect();

    let before: Vec<Vec<f64>> = words
        .iter()
        .map(|w| (0..PHASE_CHANNELS).map(|k| f.lexicon[*w].theta(k)).collect())
        .collect();

    // Composed with no anchor and many rounds, this drives dispersion to
    // near zero, which is exactly what the floor exists to catch.
    let mut candidate = f.clone();
    Conception::compose_anchored(
        &mut candidate,
        &entries,
        12,
        crate::train::conception::HEAD_STEP,
        crate::train::conception::BETA_STRONG,
        crate::train::conception::BETA_STRONG,
        false,
        None,
        0.0,
        None,
    );
    assert!(
        candidate.phase_dispersion() < DISPERSION_FLOOR,
        "the fixture must actually collapse, got dispersion {}",
        candidate.phase_dispersion()
    );

    // Now through the guarded path: rejected, and nothing written back.
    let moved = DefinitionGrounder::ground_best_from(&mut f, &Entries(entries), 0.0, 12);
    assert_eq!(moved, 0, "a collapsing composition must be rejected");
    for (w, row) in words.iter().zip(&before) {
        for (k, &b) in row.iter().enumerate() {
            assert_eq!(
                f.lexicon[*w].theta(k),
                b,
                "rejection must leave {} channel {} untouched",
                w,
                k
            );
        }
    }
}

/// A composition that keeps the manifold spread must be accepted.
///
/// The vocabulary has to be large enough for dispersion to mean anything —
/// on a handful of words the statistic is dominated by which few angles the
/// seeds happened to land on, and the guard rejects healthy fixtures for
/// reasons that have nothing to do with composition.
#[test]
fn test_healthy_composition_is_accepted() {
    // Real words, not generated names. `seeded` hashes the string, and
    // "head00".."head39" share a long prefix, which lands them in a narrow
    // band of channel 0 — a fixture that starts at dispersion 0.25 and
    // stays there tests the seeding, not the guard.
    let heads = [
        "cat", "dog", "horse", "eagle", "salmon", "oak", "rose", "copper", "granite",
        "wheat", "apple", "hammer", "chair", "violin", "wine", "bread", "cotton",
        "marble", "rifle", "sword",
    ];
    let fillers = [
        "animal", "plant", "metal", "stone", "food", "tool", "weapon", "bird", "fish",
        "tree", "flower", "grain", "fruit", "cloth", "drink", "wood", "iron", "seat",
        "music", "sharp", "heavy", "soft", "sweet", "bitter", "round", "long", "small",
        "large", "bright", "dark",
    ];
    let all: Vec<&str> = heads.iter().chain(fillers.iter()).copied().collect();
    let mut f = facet_of(&all);

    // Each head composed toward a different pair of fillers, so the
    // manifold organises without concentrating.
    let entries = Entries(
        heads
            .iter()
            .enumerate()
            .map(|(i, h)| {
                (
                    (*h).to_string(),
                    format!("{} {}", fillers[i], fillers[(i + 11) % fillers.len()]),
                )
            })
            .collect(),
    );

    let start_dispersion = f.phase_dispersion();
    let before = f.lexicon["cat"].theta(3);
    // A strong anchor, as the sweep measured it: alpha = 2.0 held dispersion at
    // 0.809 on the real dictionary. The accept path is what is under test
    // here, not the operating point.
    let moved = DefinitionGrounder::ground_best_from(&mut f, &entries, 2.0, 2);

    assert!(
        moved > 0,
        "a composition that keeps the manifold spread must be accepted \
         (dispersion {:.3} -> {:.3}, floor {:.2})",
        start_dispersion,
        f.phase_dispersion(),
        DISPERSION_FLOOR
    );
    assert!(f.phase_dispersion() >= DISPERSION_FLOOR);
    assert_ne!(
        f.lexicon["cat"].theta(3),
        before,
        "acceptance must write through to the caller's facet"
    );
}
