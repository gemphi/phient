//! Context classes discovered from gloss phase states (moved from
//! `phiano::frames`, 14.2).
//!
//! The one class function that reads a `Facet`, so it lives where `Facet` is
//! allowed. The pure class statistics (`shuffle_classes`, `collapse_small`,
//! `predictive_gain`) stay in `phiano::frames` until `PHASE_CHANNELS` moves to
//! `phistd` (14.4) and they can join `phical::classes`.

use phiecs::phasor::facet::Facet;
use phiecs::phasor::spectral::PHASE_CHANNELS;
use phiecs::phasor::wave::Wave;
use phiton::p64::words::Tokenizer;

/// K-means over gloss phase states, returning one class per gloss.
///
/// Farthest-point seeded and run to a fixed round count, so two runs on the same
/// data give the same classes.
pub fn discover_classes(facet: &Facet, texts: &[String], k: usize, rounds: usize) -> Vec<u16> {
    let states: Vec<Vec<f64>> = texts
        .iter()
        .map(|t| {
            let words = Tokenizer::tokenize(t);
            Wave::sentence_channels(facet, &words, false)
                .iter()
                .map(|z| z.arg())
                .collect()
        })
        .collect();
    if states.len() < k || k == 0 {
        return vec![0; texts.len()];
    }

    let agree = |a: &[f64], b: &[f64]| -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).cos()).sum::<f64>() / PHASE_CHANNELS as f64
    };

    let mut best_sim = vec![f64::NEG_INFINITY; states.len()];
    let mut seeds: Vec<usize> = Vec::with_capacity(k);
    let mut next = 0usize;
    loop {
        seeds.push(next);
        if seeds.len() >= k {
            break;
        }
        let chosen = states[next].clone();
        for (b, s) in best_sim.iter_mut().zip(states.iter()) {
            let a = agree(s, &chosen);
            if a > *b {
                *b = a;
            }
        }
        best_sim[next] = f64::INFINITY;
        match best_sim
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
        {
            Some(i) if best_sim[i].is_finite() => next = i,
            _ => break,
        }
    }

    let mut centroids: Vec<Vec<f64>> = seeds.iter().map(|s| states[*s].clone()).collect();
    let mut assign = vec![0usize; states.len()];
    for _ in 0..rounds {
        let mut moved = false;
        for (i, s) in states.iter().enumerate() {
            let best = centroids
                .iter()
                .enumerate()
                .max_by(|a, b| {
                    agree(s, a.1).partial_cmp(&agree(s, b.1)).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(c, _)| c)
                .unwrap_or(0);
            if assign[i] != best {
                assign[i] = best;
                moved = true;
            }
        }
        for (c, cen) in centroids.iter_mut().enumerate() {
            for kk in 0..PHASE_CHANNELS {
                let (mut x, mut y) = (0.0f64, 0.0f64);
                for (i, s) in states.iter().enumerate() {
                    if assign[i] == c {
                        x += s[kk].cos();
                        y += s[kk].sin();
                    }
                }
                if x.hypot(y) > 1e-12 {
                    cen[kk] = y.atan2(x);
                }
            }
        }
        if !moved {
            break;
        }
    }
    assign.into_iter().map(|a| a as u16).collect()
}
