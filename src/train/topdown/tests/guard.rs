#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

/// The guard has to be able to reject, or it is a comment.
///
/// A closed vocabulary where every word co-occurs with every other is the
/// pure collapse case: each word's contexts average to the same global mean,
/// so a full-step attraction cycle takes the whole lexicon to one angle.
/// This is not a contrived corner — it is the fixed point the downward pass
/// has by construction, and the reason it needs a guard at all.
#[test]
fn test_the_guard_rejects_a_collapsing_descent() {
    let vocab: Vec<String> = (0..40).map(|i| format!("w{i}")).collect();
    let mut sents: Vec<Vec<String>> = Vec::new();
    let mut r: u64 = 0x243F_6A88_85A3_08D3;
    for _ in 0..400 {
        let mut s: Vec<String> = Vec::new();
        while s.len() < 6 {
            r ^= r << 13;
            r ^= r >> 7;
            r ^= r << 17;
            let w = vocab[(r % vocab.len() as u64) as usize].clone();
            if !s.contains(&w) {
                s.push(w);
            }
        }
        sents.push(s);
    }

    let mut f = facet_over(&sents);
    let before = f.clone();

    let r = Descent::cycle_with(&mut f, &sents, Up::Bag, 40, 1.0, false, 0.0);
    assert!(
        r.rejected,
        "a full-step attraction cycle on a fully-connected vocabulary should \
         collapse: dispersion {:.3} -> {:.3}, band {:.3} -> {:.3}",
        r.dispersion_before, r.dispersion_after, r.band_before, r.band_after
    );
    // Rejection must leave the facet untouched, not partly moved.
    for (w, p) in &before.lexicon {
        for k in 0..PHASE_CHANNELS {
            assert!((f.lexicon[w].theta(k) - p.theta(k)).abs() < 1e-12, "{w} moved");
        }
    }
}

#[test]
fn test_descent_is_reproducible() {
    let sents = sentences(&[
        "alpha beta gamma delta",
        "beta gamma epsilon zeta",
        "gamma delta eta theta",
        "alpha epsilon theta iota",
    ]);
    let run = || {
        let mut f = facet_over(&sents);
        Descent::cycle(&mut f, &sents, Up::Bound, 5, 0.3, true);
        f
    };
    let a = run();
    let b = run();
    for (w, p) in &a.lexicon {
        for k in 0..PHASE_CHANNELS {
            assert!((p.theta(k) - b.lexicon[w].theta(k)).abs() < 1e-12, "{w} channel {k}");
        }
    }
}
