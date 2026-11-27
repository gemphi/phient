#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

/// The phase distribution must be a distribution, not a constant.
///
/// This guards a bug that invalidated every γ sweep before it was found:
/// the per-candidate score was a *sum* over 16 channels, spanning ~32 units,
/// so exponentiating it underflowed the probability floor for every
/// candidate but the top one. `p_phase` then returned the same floor value
/// at nearly every position — a constant back-off, which loses to a unigram
/// by construction at every γ. The measured conclusion "γ* = 0, the manifold
/// contributes nothing" was partly a statement about the score scale.
#[test]
fn test_phase_distribution_is_not_saturated() {
    let split = Harness::split(toy_corpus(), 42);
    let (facet, _) = Harness::train_and_measure(&split, &Trainer::new(0.05), 3, true);
    let lm = PhianoLM::with_gamma(&facet, 1.0);

    let mut ctx_buf = Vec::new();
    let mut spread: f64 = 0.0;
    for sentence in &split.valid {
        let toks = Tokenizer::tokenize(sentence);
        for w in toks.windows(3) {
            if let Some(ctx) = lm.context_vec(&w[0], &w[1]) {
                lm.scores(&ctx, &mut ctx_buf);
                let hi = ctx_buf.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let lo = ctx_buf.iter().cloned().fold(f64::INFINITY, f64::min);
                spread = spread.max(hi - lo);
            }
        }
    }
    assert!(spread > 0.0, "scores are identical for every candidate");
    assert!(
        spread <= 2.0 + 1e-9,
        "scores must be a mean cosine in [-1, 1]; spread {} means the \
         softmax will saturate and the phase back-off will degenerate \
         into a constant",
        spread
    );
}

/// The readout must change scoring, and must change it only where the
/// phase term is actually used.
///
/// γ = 0 removes the phase back-off entirely, so a readout that moved the
/// γ = 0 perplexity would be leaking into the n-gram path and every
/// on-versus-off comparison built on it would be void.
#[test]
fn test_readout_changes_phase_scoring_only() {
    let split = Harness::split(toy_corpus(), 42);
    let (facet, _) = Harness::train_and_measure(&split, &Trainer::new(0.05), 3, true);

    let mut lm = PhianoLM::with_gamma(&facet, 1.0);
    let before = lm.sweep(&split.valid, &[1.0], false);
    lm.fit_readout(&split.train, 0.5, false);
    assert!(lm.readout_cells() > 0, "readout fitted no cells");
    let after = lm.sweep(&split.valid, &[1.0], false);

    let at = |rows: &[SweepRow], g: f64| {
        rows.iter().find(|r| (r.gamma - g).abs() < 1e-9).map(|r| r.ppl).unwrap()
    };

    assert!(
        (at(&before, 0.0) - at(&after, 0.0)).abs() < 1e-9,
        "readout leaked into the γ=0 path: {} vs {}",
        at(&before, 0.0),
        at(&after, 0.0)
    );
    assert!(
        (at(&before, 1.0) - at(&after, 1.0)).abs() > 1e-9,
        "readout was fitted but changed nothing at γ=1 — a bias that cannot \
         reorder candidates is the bug this table was rewritten to fix"
    );
}

/// Contrastive training must preserve more phase dispersion than
/// attraction-only training over the same data.
#[test]
fn test_contrastive_preserves_dispersion() {
    let split = Harness::split(toy_corpus(), 42);
    let (f1, _) = Harness::train_and_measure(&split, &Trainer::new(0.05), 6, false);
    let (f2, _) =
        Harness::train_and_measure(&split, &Trainer::attraction_only(0.05), 6, false);
    assert!(
        f1.phase_dispersion() > f2.phase_dispersion(),
        "contrastive {} vs attraction-only {}",
        f1.phase_dispersion(),
        f2.phase_dispersion()
    );
}

/// What each context construction does with the order of its two words
/// (`lm/context.rs`), measured on the toy corpus at seed 42.
///
/// The rolled construction binds each word to the place it stands in: a
/// swap gives a context pointing the other way, -0.481. The weighted one
/// was documented as order-blind - "the two words land in the same
/// superposition, and swapping them changes only their magnitudes" - and
/// it is not: it reads 0.453, well under the 1.0 that blindness would
/// give. The sum is weighted, but each channel is then normalised back to
/// the unit circle, and that normalisation keeps only a direction, which
/// the heavier word turns. Order is in there as a by-product.
///
/// The two were written out as two copies of one sum before 2026-11-27;
/// this is what would catch a weight put where a roll belongs.
#[test]
fn the_two_context_constructions_hear_order_differently() {
    let split = Harness::split(toy_corpus(), 42);
    let (facet, _) = Harness::train_and_measure(&split, &Trainer::new(0.05), 3, true);
    let lm = PhianoLM::with_gamma(&facet, 1.0);

    let mut pairs = Vec::new();
    for sentence in &split.valid {
        let toks = Tokenizer::tokenize(sentence);
        for w in toks.windows(2) {
            pairs.push((w[0].clone(), w[1].clone()));
        }
    }
    assert!(pairs.len() > 8, "the toy corpus gives {} pairs", pairs.len());

    let weighed = lm.order_sensitivity(&pairs, ContextKind::TwoWord);
    let bound = lm.order_sensitivity(&pairs, ContextKind::Bound);
    assert!((0.3..0.6).contains(&weighed), "the weighted construction read {weighed:.3}, not 0.453");
    assert!(bound < 0.0, "a rolled context should point the other way after a swap, read {bound:.3}");
    assert!(bound < weighed - 0.5, "bound {bound:.3} should be far under weighed {weighed:.3}");
}
