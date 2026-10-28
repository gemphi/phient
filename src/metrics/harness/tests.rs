#[allow(unused_imports)]
use super::*;

#[cfg(test)]
mod distribution;

pub(super) fn toy_corpus() -> Vec<String> {
    let mut c = Vec::new();
    let subjects = ["the cat", "the dog", "a bird", "my friend"];
    let verbs = ["sat on", "ran to", "looked at", "jumped over"];
    let objects = ["the mat", "the park", "a fence", "the table"];
    for s in &subjects {
        for v in &verbs {
            for o in &objects {
                c.push(format!("{} {} {}", s, v, o));
            }
        }
    }
    c
}

#[test]
fn test_split_is_deterministic_and_disjoint() {
    let a = Harness::split(toy_corpus(), 42);
    let b = Harness::split(toy_corpus(), 42);
    assert_eq!(a.train, b.train);
    assert_eq!(a.valid, b.valid);
    for v in &a.valid {
        assert!(!a.train.contains(v), "validation leaked into train");
    }
    for t in &a.test {
        assert!(!a.train.contains(t), "test leaked into train");
    }
}

#[test]
fn test_perplexity_is_finite() {
    let split = Harness::split(toy_corpus(), 42);
    let t = Trainer::new(0.05);
    let (facet, log) = Harness::train_and_measure(&split, &t, 2, true);
    assert!(facet.vocabulary_size() > 5);
    for m in &log {
        assert!(m.valid_ppl.is_finite(), "perplexity must be finite: {:?}", m);
        assert!(m.phase_dispersion >= 0.0 && m.phase_dispersion <= 1.0);
    }
}

#[test]
fn test_kn_baseline_beats_nothing_is_reported_honestly() {
    let split = Harness::split(toy_corpus(), 42);
    let kn = KneserNey::train(&split.train);
    let ppl = kn.perplexity(&split.valid);
    assert!(ppl.is_finite() && ppl > 1.0);
}

/// Training the same data twice must give the same model.
///
/// It did not. `rebuild_sample_pool` built the negative-sample pool by
/// iterating `lexicon`, a HashMap that Rust seeds randomly per process, so
/// every run drew a different negative sequence. Two runs of one
/// composition experiment returned analogy MRR 0.1066 and 0.0521 — same
/// ranking, magnitudes a factor of two apart. Every measured effect smaller
/// than that gap was unfalsifiable.
#[test]
fn test_training_is_reproducible() {
    let split = Harness::split(toy_corpus(), 42);
    let a = Harness::train_ranking_only(&split, &Trainer::new(0.05), 3);
    let b = Harness::train_ranking_only(&split, &Trainer::new(0.05), 3);

    assert_eq!(a.lexicon.len(), b.lexicon.len());
    for (w, pa) in &a.lexicon {
        let pb = b.lexicon.get(w).expect("same vocabulary");
        for k in 0..phiton::units::PHASE_CHANNELS {
            assert_eq!(
                pa.theta(k),
                pb.theta(k),
                "word {} channel {} differs between two identical runs",
                w,
                k
            );
        }
    }
}

/// The default context must be the one that was measured best, and the
/// no-phase path must be unaffected by the choice.
///
/// γ = 0 removes the phase term, so context construction cannot legally
/// touch it. If it did, every no-phase baseline in the results would have
/// silently moved when B3 landed.
#[test]
fn test_default_context_is_recurrent_and_gamma_zero_is_untouched() {
    assert_eq!(
        PhianoLM::DEFAULT_CONTEXT,
        ContextKind::Recurrent,
        "B3 selected the recurrent construction on measurement"
    );

    let split = Harness::split(toy_corpus(), 42);
    let (facet, _) = Harness::train_and_measure(&split, &Trainer::new(0.05), 3, true);

    let off = PhianoLM::with_gamma(&facet, 0.0);
    assert_eq!(
        off.perplexity(&split.valid),
        off.perplexity_two_word(&split.valid),
        "at γ=0 the context is never consulted, so both paths must agree exactly"
    );

    // At γ=1 the construction is consulted, so it must make a difference —
    // otherwise the default is a label with no effect behind it.
    let on = PhianoLM::with_gamma(&facet, 1.0);
    let rec = on.perplexity_kind(&split.valid, ContextKind::Recurrent);
    let two = on.perplexity_kind(&split.valid, ContextKind::TwoWord);
    assert!(rec.is_finite() && two.is_finite());
    assert!(
        (rec - two).abs() > 1e-9,
        "the context construction must change scoring at γ=1: {} vs {}",
        rec,
        two
    );
}

/// A seed must do both jobs: the same seed reproduces, a different seed
/// varies.
///
/// Half of this is easy to get wrong in each direction. A seed that is
/// accepted but never mixed in gives identical runs and a fake error bar of
/// zero; a seed that leaks into the corpus split rather than the training
/// stochasticity varies the *data*, and the spread then measures the split
/// rather than the model.
#[test]
fn test_seed_varies_training_without_varying_the_data() {
    let split = Harness::split(toy_corpus(), 42);
    let train = |seed: u64| {
        Harness::train_ranking_only(&split, &Trainer::new(0.05).with_seed(seed), 3)
    };

    let a = train(7);
    let b = train(7);
    let c = train(8);

    let differs = |x: &Facet, y: &Facet| {
        x.lexicon.iter().any(|(w, px)| match y.lexicon.get(w) {
            Some(py) => (0..phiton::units::PHASE_CHANNELS).any(|k| px.theta(k) != py.theta(k)),
            None => true,
        })
    };

    assert!(!differs(&a, &b), "the same seed must reproduce exactly");
    assert!(differs(&a, &c), "a different seed must change the model");

    // The vocabulary is a property of the corpus, not of the seed. If this
    // moves, the seed is varying the data and the spread it produces would
    // not be a measure of training variance.
    assert_eq!(
        a.vocabulary_size(),
        c.vocabulary_size(),
        "the seed must not change what data was seen"
    );
}
