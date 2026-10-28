#[allow(unused_imports)]
use super::*;

/// Relation coupling deposits evidence on the channel of the relation a
/// pair was observed under, and nowhere else.
#[test]
fn test_relation_coupling_deposits_on_the_observed_channel() {
    let mut facet = Facet::new();
    let t = Trainer::new(0.05);
    t.train(&mut facet, "dog eats donut");
    let dog = facet.lexicon["dog"];
    let donut = facet.lexicon["donut"];
    // dog–eats and eats–donut are offset+1 (channel 0); dog–donut is
    // offset+2 (channel 1) since no markers are learned from one sentence.
    assert_eq!(dog.evidence_count(0), 1, "dog eats: offset+1");
    assert_eq!(dog.evidence_count(1), 1, "dog donut: offset+2");
    assert_eq!(dog.evidence_count(2), 0);
    assert_eq!(donut.evidence_count(1), 1);
    assert_eq!(facet.lexicon["eats"].evidence_count(0), 2, "eats touches both neighbours");

    // The control arm deposits nothing and leaves phases where the
    // ranking objective put them.
    let mut control = Facet::new();
    Trainer::new(0.05).without_relations().train(&mut control, "dog eats donut");
    assert!(!control.lexicon["dog"].has_evidence());
}

/// Relation coupling must separate what the corpus separates.
///
/// Attraction-only Kuramoto on `offset+1` drove its dispersion to 0.3 on
/// the prose corpus and agreed about everything. Two vocabularies that
/// never co-occur must end up apart on the channel, and the channel must
/// stay spread — which is what the contrastive term is for.
#[test]
fn test_relation_coupling_keeps_channels_dispersed() {
    let a: Vec<String> = (0..30).map(|i| format!("a{i}")).collect();
    let b: Vec<String> = (0..30).map(|i| format!("b{i}")).collect();
    let mut sentences = Vec::new();
    for s in 0..200u64 {
        let mut r = s.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let side = if s % 2 == 0 { &a } else { &b };
        let mut toks = Vec::new();
        for _ in 0..8 {
            r ^= r << 13;
            r ^= r >> 7;
            r ^= r << 17;
            toks.push(side[(r % 30) as usize].clone());
        }
        sentences.push(toks.join(" "));
    }
    let mut facet = Facet::new();
    let t = Trainer { learning_rate: 0.0, neg_samples: NEG_SAMPLES, definitions: None, seed: 0, relations: true, rounding: Default::default() };
    let toks: Vec<Vec<String>> = sentences.iter().map(|s| Tokenizer::tokenize(s)).collect();
    for s in &toks {
        t.initialize_tokens(&mut facet, s);
    }
    for _ in 0..30 {
        for s in &toks {
            t.couple_relations(&mut facet, s);
        }
    }
    let mean = |xs: &[String], ys: &[String]| -> f64 {
        let (mut sum, mut n) = (0.0, 0usize);
        for x in xs {
            for y in ys {
                if x != y {
                    sum += facet.lexicon[x].on(&facet.lexicon[y], 0);
                    n += 1;
                }
            }
        }
        sum / n as f64
    };
    let within = (mean(&a, &a) + mean(&b, &b)) / 2.0;
    let between = mean(&a, &b);
    let dispersion = facet.channel_dispersion(0);
    assert!(within > 0.8, "a community agrees with itself on offset+1: {within:.3}");
    assert!(
        between < within - 0.5,
        "communities that never co-occur are apart: within {within:.3}, between {between:.3}"
    );
    assert!(dispersion > 0.3, "offset+1 dispersion {dispersion:.3}");
}

/// Coupling on a channel pulls the pair together on that channel only.
#[test]
fn test_relation_coupling_moves_one_channel() {
    let mut facet = Facet::new();
    let t = Trainer { learning_rate: 0.0, neg_samples: 0, definitions: None, seed: 0, relations: true, rounding: Default::default() };
    t.initialize_tokens(&mut facet, &["alpha".into(), "beta".into()]);
    let before_a = facet.lexicon["alpha"];
    let before_b = facet.lexicon["beta"];
    // Directed: alpha's source side meets beta's target side.
    let d0 = phistd::wrap(before_b.theta_target(0) - before_a.theta(0)).abs();
    for _ in 0..200 {
        t.couple_relations(&mut facet, &["alpha".into(), "beta".into()]);
    }
    let a = facet.lexicon["alpha"];
    let b = facet.lexicon["beta"];
    let d0_after = phistd::wrap(b.theta_target(0) - a.theta(0)).abs();
    assert!(d0_after < d0 * 0.5 || d0_after < 0.1, "channel 0 converged: {d0} -> {d0_after}");
    for k in 1..PHASE_CHANNELS {
        assert!((a.theta(k) - before_a.theta(k)).abs() < 1e-9, "channel {k} untouched");
        assert!((b.theta_target_raw(k) - before_b.theta_target_raw(k)).abs() < 1e-9, "channel {k} untouched");
    }
    // The other sides never moved: no chain through a single variable.
    assert!((a.theta_target_raw(0) - before_a.theta_target_raw(0)).abs() < 1e-9);
    assert!((b.theta(0) - before_b.theta(0)).abs() < 1e-9);
    assert_eq!(a.evidence_count(0), 200);
}

/// The defining property of the contrastive rule: repeated training must not
/// drive the lexicon to a single point.
#[test]
fn test_repulsion_prevents_collapse() {
    let sentences = [
        "the cat sat on the mat",
        "the dog ran in the park",
        "the sun set on the sea",
        "rust guarantees memory safety",
        "the borrow checker prevents data races",
    ];

    let mut with_neg = Facet::new();
    let t1 = Trainer::new(0.05);
    for _ in 0..200 {
        for s in &sentences { t1.train_sentence(&mut with_neg, s); }
    }

    let mut without_neg = Facet::new();
    let t2 = Trainer::attraction_only(0.05);
    for _ in 0..200 {
        for s in &sentences { t2.train_sentence(&mut without_neg, s); }
    }

    let d_with = with_neg.phase_dispersion();
    let d_without = without_neg.phase_dispersion();
    assert!(
        d_with > d_without,
        "contrastive training must preserve more dispersion: {} vs {}",
        d_with, d_without
    );
}

#[test]
fn test_correction_spares_shared_function_words() {
    let mut f = Facet::new();
    let t = Trainer::new(0.05);
    t.train_sentence(&mut f, "rust is slow");
    let before = f.lexicon.get("is").map(|p| p.phase).unwrap();
    t.correct_mistake(&mut f, "rust is slow", "rust is fast");
    let after = f.lexicon.get("is").map(|p| p.phase).unwrap();
    // "is" appears in both phrases, so it must not have taken a π pulse
    let moved = phistd::wrap(after - before).abs();
    assert!(moved < 1.0, "shared word moved {} rad", moved);
}

#[test]
fn test_predictive_training_runs_and_is_bounded() {
    let mut f = Facet::new();
    let t = Trainer::new(0.05);
    for _ in 0..10 {
        t.train_full(&mut f, "the borrow checker prevents data races");
    }
    assert!(f.vocabulary_size() >= 6);
    for p in f.lexicon.values() {
        assert!(p.phase.is_finite() && p.phase >= 0.0 && p.phase < TWO_PI);
        assert!(p.amplitude >= 1.0 && p.amplitude <= AMPLITUDE_MAX);
    }
}

#[cfg(test)]
mod core;
#[cfg(test)]
mod facet;
#[cfg(test)]
mod storage;
#[cfg(test)]
mod helix;
