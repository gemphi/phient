//! The facet under phiano's trainer: the tests of `phiecs::phasor::facet` and
//! `phiecs::phasor::wave` that train with `Trainer` or read grounding's
//! floor, so they stay with the trainer.

use crate::train::constants::LEARNING_RATE;
use phiton::units::TWO_PI;
use phiecs::phasor::spectral::{Rounding, SpectralPhasor};
use phiton::p64::words::Tokenizer;
use crate::train::trainer::Trainer;
use phiecs::phasor::facet::Facet;
use phiecs::phasor::wave::Wave;

fn facet_with(words: &[&str]) -> Facet {
    let mut f = Facet::new();
    for w in words {
        f.get_or_init(w);
    }
    f
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn a_quest_row_is_read_back_under_its_kind() {
    // Two rows, two kinds; each kind is the word between its pair.
    let docs = vec![s(&["cat", "is-a", "animal"]), s(&["cat", "has", "whiskers"])];
    let mut facet = Facet::new();
    facet.learn_relations(&docs);
    for d in &docs {
        for w in d {
            facet.get_or_init(w);
        }
    }
    assert!(facet.marker_channel("is-a").is_some());
    assert!(facet.marker_channel("has").is_some());
    assert_eq!(facet.marker_channel("cat"), None);
    let t = Trainer::attraction_only(LEARNING_RATE).with_rounding(Rounding::Dither);
    for _ in 0..64 {
        for d in &docs {
            t.couple_relations(&mut facet, d);
        }
    }
    let pool = s(&["animal", "whiskers", "cat"]);
    assert_eq!(facet.under("cat", "is-a", &pool)[0].0, "animal");
    assert_eq!(facet.under("cat", "has", &pool)[0].0, "whiskers");
    assert!(facet.calls_for_under("cat", "is-a", "animal") > facet.calls_for_under("cat", "is-a", "whiskers"));
    assert_eq!(facet.calls_for_under("cat", "purrs", "animal"), 0.0);
}

/// The failure the collapse guard exists to detect, and the reason the global
/// figure could not see it.
///
/// A long-tailed lexicon: 100 frequent words collapsed onto a single angle, and
/// 3,000 rare words spread uniformly. Every task the model is scored on draws
/// its candidates from the frequent band, so this manifold is useless — and
/// `phase_dispersion` reads above 0.95, well clear of the 0.40 floor. The band
/// measure reads it as the collapse it is.
#[test]
fn test_global_dispersion_is_blind_to_a_collapsed_frequent_band() {
    let mut f = Facet::new();

    const FREQUENT: usize = 100;
    const RARE: usize = 3_000;

    for i in 0..FREQUENT {
        let mut p = SpectralPhasor::new(1.234, 1.0, 1);
        p.count = 1_000;
        f.lexicon.insert(format!("frequent_{i}"), p);
    }
    for i in 0..RARE {
        let theta = TWO_PI * i as f64 / RARE as f64;
        let mut p = SpectralPhasor::new(theta, 1.0, 1);
        p.count = 1;
        f.lexicon.insert(format!("rare_{i}"), p);
    }

    let global = f.phase_dispersion();
    let band = f.dispersion_top(FREQUENT);

    assert!(
        global > 0.95,
        "global dispersion should be dominated by the tail, got {global:.4}"
    );
    assert!(
        band < 0.01,
        "the frequent band is one angle; dispersion should be ~0, got {band:.4}"
    );
    assert!(
        global >= crate::train::grounding::DISPERSION_FLOOR
            && band < crate::train::grounding::DISPERSION_FLOOR,
        "this is the blind spot: the global figure passes the floor ({global:.4}) \
         while the band fails it ({band:.4})"
    );
}

#[test]
fn relational_fit_at_the_last_position_is_relational_score() {
    let mut facet = Facet::new();
    let t = Trainer::new(0.05);
    for s in ["the dog sat down", "the cat sat down", "a bird sat down", "the dog ran off"] {
        t.train(&mut facet, s);
    }
    let toks = Tokenizer::tokenize("the dog sat");
    let (a, na) = facet.relational_score(&toks[..2], "sat");
    let (b, nb) = facet.relational_fit(&toks, 2, "sat");
    assert_eq!(na, nb, "the same pairs are read");
    assert!((a - b).abs() < 1e-12, "the Liskov reading at the end is the next-word reading");
    // In the middle, neighbours on both sides are read; a word with no lexicon entry has no fit.
    let (_, n_mid) = facet.relational_fit(&toks, 1, "cat");
    assert!(n_mid >= 1);
    assert_eq!(facet.relational_fit(&toks, 1, "zebra"), (0.0, 0));
    assert_eq!(facet.calls_for("zebra", "sat"), 0.0);
}

#[test]
fn test_role_binding_roundtrip() {
    let mut f = facet_with(&["dog", "bites", "man", "cat", "sleeps"]);
    let t = Trainer::new(0.05);
    t.train_sentence(&mut f, "the dog bites the man");

    let z = Wave::proposition(&f, "dog", "bites", "man");
    let subj = Wave::query_role(&f, z, 0, 5);
    assert!(!subj.is_empty());

    let z2 = Wave::proposition(&f, "man", "bites", "dog");
    assert!((z - z2).norm() > 0.05, "propositions must differ by argument order");
}
