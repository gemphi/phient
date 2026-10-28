//! Joint training of the operator and the lexicon, on Phiano's side.
//!
//! The mechanism — the table of functions as a parameter on the operator's
//! tape, contexts gathered on the tape, the proximal tether — is
//! [`phical::Joint`]. This module builds its examples from a corpus and
//! writes the learned functions back into a facet.
//!
//! Two things had to be true before it worked, both found by measuring:
//! the table memorises unless tethered (loss 0.065 → 0.043, held-out
//! perplexity 149 → 157 on the first free run), and every word has to be a
//! target, not only the frequent pool (targets from the pool alone: ranking
//! inside the pool up, perplexity over the vocabulary 149 → 168, because the
//! words outside it moved only as context and drifted into the softmax's way).

use super::space::{FunctionSpace, WordFunction};
use phiton::units::PHASE_CHANNELS;
use phiecs::phasor::facet::Facet;
pub use phical::{IndexExample, Joint};

/// Builds index examples with the same window and weighting as
/// [`super::space::examples`], on the same deterministic split.
pub fn index_examples(
    space: &FunctionSpace,
    sentences: &[Vec<String>],
    window: usize,
    pool: &[usize],
    holdout_every: u64,
) -> (Vec<IndexExample>, Vec<IndexExample>) {
    let in_pool: std::collections::HashSet<usize> = pool.iter().copied().collect();
    let (mut train, mut held) = (Vec::new(), Vec::new());
    for s in sentences {
        let toks: Vec<usize> = s.iter().filter_map(|w| space.index.get(w).copied()).collect();
        if toks.len() < 2 {
            continue;
        }
        let held_out = phistd::fnv1a(&s.join(" ")).is_multiple_of(holdout_every);
        for end in 1..toks.len() {
            let target = toks[end];
            if !in_pool.contains(&target) {
                continue;
            }
            let start = end.saturating_sub(window);
            let ctx: Vec<(usize, f64)> = (start..end).map(|j| (toks[j], 1.0 / (end - j) as f64)).collect();
            let ex = IndexExample { ctx, target };
            match held_out {
                true => held.push(ex),
                false => train.push(ex),
            }
        }
    }
    (train, held)
}

/// Writes learned functions back into a facet.
///
/// Angle → `theta_k`. Magnitude → evidence, rescaled so the word's total
/// evidence is what it was (or one observation per channel for a word that
/// had none), so the learned *shape* replaces the old one without changing
/// how loudly the word votes overall. Returns the number of words written.
pub fn write_back(facet: &mut Facet, space: &FunctionSpace, funcs: &[WordFunction]) -> usize {
    let mut written = 0;
    for (i, w) in space.words.iter().enumerate() {
        let p = match facet.lexicon.get_mut(w) {
            Some(p) => p,
            None => continue,
        };
        let f = &funcs[i];
        let total_prev = match p.has_evidence() {
            true => p.evidence_total(),
            false => PHASE_CHANNELS as f64 * (2.0f64).ln(),
        };
        let mag_sum: f64 = f.iter().map(|z| z.norm()).sum();
        if mag_sum < 1e-12 {
            continue;
        }
        for (k, z) in f.iter().enumerate().take(PHASE_CHANNELS) {
            if z.norm() > 1e-12 {
                p.set_theta(k, z.im.atan2(z.re));
            }
            let e = total_prev * z.norm() / mag_sum;
            let count = (e.exp() - 1.0).round().max(0.0) as u32;
            p.set_evidence_count(k, count);
        }
        p.sync_phase();
        written += 1;
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use phiton::p64::words::Tokenizer;
    use phistd::Adam;
    use phical::{Fno, FnoConfig};

    #[test]
    fn test_joint_examples_train_and_write_back() {
        let mut facet = Facet::new();
        let sentences: Vec<Vec<String>> = (0..80)
            .map(|i| Tokenizer::tokenize(&format!("alpha{} beta{} gamma{} delta{}", i % 4, i % 4, i % 4, i % 4)))
            .collect();
        for s in &sentences {
            for w in s {
                facet.get_or_init(w);
            }
        }
        let space = FunctionSpace::from_facet(&facet);
        let pool: Vec<usize> = (0..space.len()).collect();
        let (train, _) = index_examples(&space, &sentences, 3, &pool, 1_000_000);
        let cfg = FnoConfig { space: PHASE_CHANNELS, width: 4, layers: 1, modes: 8, margin: 0.3, lr: 5e-3, tie: phical::Tie::Circle };
        let mut joint = Joint::new(Fno::new(cfg, 3), &space.funcs, 3);
        let mut a1 = Adam::new(cfg.lr);
        let mut a2 = Adam::new(cfg.lr * 0.5);
        let first = joint.train_epoch(&train, &pool, 16, &mut a1, &mut a2, 0);
        let mut last = first;
        for e in 1..15 {
            last = joint.train_epoch(&train, &pool, 16, &mut a1, &mut a2, e);
        }
        assert!(last < first * 0.7, "loss must fall: {first:.3} -> {last:.3}");
        let n = write_back(&mut facet, &space, &joint.functions());
        assert_eq!(n, space.len());
        assert!(facet.lexicon["alpha0"].has_evidence());
    }
}
