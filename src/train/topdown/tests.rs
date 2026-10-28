#[allow(unused_imports)]
use super::*;

pub(super) fn sentences(raw: &[&str]) -> Vec<Vec<String>> {
    raw.iter()
        .map(|s| s.split_whitespace().map(|w| w.to_string()).collect())
        .collect()
}

pub(super) fn facet_over(sents: &[Vec<String>]) -> Facet {
    let mut f = Facet::new();
    for s in sents {
        for w in s {
            f.get_or_init(w);
        }
    }
    f
}

/// Two disjoint topics should separate: words that co-occur move together,
/// words that never co-occur do not.
#[test]
fn test_descent_pulls_co_occurring_words_together() {
    let sents = sentences(&[
        "cat dog animal fur",
        "dog animal fur paw",
        "cat animal paw fur",
        "engine piston fuel valve",
        "piston fuel valve shaft",
        "engine fuel shaft valve",
    ]);
    let mut f = facet_over(&sents);

    let before = f.resonance("cat", "dog");
    let before_cross = f.resonance("cat", "engine");

    let r = Descent::cycle(&mut f, &sents, Up::Bag, 6, 0.4, false);
    assert!(!r.rejected, "guard rejected: band {:.3}", r.band_after);
    assert!(r.words_moved > 0, "nothing moved");

    let after = f.resonance("cat", "dog");
    let after_cross = f.resonance("cat", "engine");

    assert!(
        after - before > after_cross - before_cross,
        "within-topic resonance should rise more than across-topic: \
         cat/dog {before:.3}->{after:.3}, cat/engine {before_cross:.3}->{after_cross:.3}"
    );
}

/// The word must not be part of what moves it — including its *other*
/// occurrences.
///
/// `the cat sat on the mat` contains `the` twice. Subtracting only the token
/// being scored leaves the second `the` inside the state that moves the
/// first, so the word is pulled toward itself and the cycle reports progress
/// on a corpus that says nothing about it. This fixture is the pure case:
/// every sentence is one word repeated, so a correct leave-one-out has
/// nothing left and nothing may move.
#[test]
fn test_leave_one_out_is_per_type_not_per_token() {
    let sents = sentences(&["alpha alpha alpha", "alpha alpha", "alpha alpha alpha alpha"]);
    let mut f = facet_over(&sents);
    let before: Vec<f64> = (0..PHASE_CHANNELS).map(|k| f.lexicon["alpha"].theta(k)).collect();
    let r = Descent::cycle(&mut f, &sents, Up::Bag, 4, 0.5, false);
    assert_eq!(r.words_moved, 0, "a corpus of one repeated word says nothing");
    for (k, &b) in before.iter().enumerate() {
        assert!((f.lexicon["alpha"].theta(k) - b).abs() < 1e-12);
    }
}

/// Contrast holds still the word that is everywhere, and lets the word that
/// is somewhere move.
///
/// Two disjoint topics; `the` occurs in every sentence of both, so its
/// contexts average to the corpus mean and subtracting that leaves nothing.
/// A topic word's contexts average to *its topic's* mean, which the corpus
/// mean does not cancel. Asserting only that `the` moves less would pass for
/// a rule that froze everything, so both halves are checked.
#[test]
fn test_contrast_holds_the_ubiquitous_word_and_frees_the_topical_one() {
    let mut sents: Vec<Vec<String>> = Vec::new();
    for i in 0..30 {
        let a = ["a1", "a2", "a3", "a4"];
        let b = ["b1", "b2", "b3", "b4"];
        sents.push(
            ["the", a[i % 4], a[(i + 1) % 4], a[(i + 2) % 4]]
                .iter()
                .map(|w| w.to_string())
                .collect(),
        );
        sents.push(
            ["the", b[i % 4], b[(i + 1) % 4], b[(i + 2) % 4]]
                .iter()
                .map(|w| w.to_string())
                .collect(),
        );
    }

    let base = facet_over(&sents);
    let moved = |f: &Facet, w: &str| -> f64 {
        (0..PHASE_CHANNELS)
            .map(|k| angular_distance(f.lexicon[w].theta(k), base.lexicon[w].theta(k)))
            .sum::<f64>()
            / PHASE_CHANNELS as f64
    };

    let mut plain = base.clone();
    let r_plain = Descent::cycle(&mut plain, &sents, Up::Bag, 4, 0.5, false);

    let mut contrasted = base.clone();
    let r_contrast = Descent::cycle(&mut contrasted, &sents, Up::Bag, 4, 0.5, true);

    // Without contrast this corpus collapses and the guard throws the
    // result away — which is the finding, not a test artefact. `the` is in
    // every sentence, so every word's target is dominated by the one thing
    // they all share.
    assert!(
        r_plain.rejected,
        "expected collapse without contrast; band {:.3} -> {:.3}",
        r_plain.band_before, r_plain.band_after
    );
    assert!(
        !r_contrast.rejected,
        "contrast should survive the guard; band {:.3} -> {:.3}",
        r_contrast.band_before, r_contrast.band_after
    );

    // And what survives has the right shape: the word that is everywhere
    // stays put, the word that belongs to one topic moves.
    assert!(
        moved(&contrasted, "a1") > 4.0 * moved(&contrasted, "the"),
        "a topic word should move and the ubiquitous one should not: \
         a1 {:.4}, the {:.4}",
        moved(&contrasted, "a1"),
        moved(&contrasted, "the")
    );
}

pub(super) fn angular_distance(a: f64, b: f64) -> f64 {
    ((a - b + std::f64::consts::PI).rem_euclid(TWO_PI) - std::f64::consts::PI).abs()
}

#[cfg(test)]
mod guard;
