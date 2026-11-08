//! Movement with learned weights (docs/112): what docs/110 found for code,
//! tried on the builders' game. Every word that relates under the aspect
//! moves the answer by ln P(v | w, c) − ln P(v | c), as in `compose`; here
//! each move is weighted by α[aspect, how far it moved the answer when told],
//! and the condition by β[aspect]. No post threshold: a weight near zero is
//! a post not held. The weights are fitted by likelihood on held-out folds
//! inside the training set (counts from three quarters, answers from the
//! fourth), never on the test.

use super::learners::{kind, Action, GameCounts, Learner};
use super::world::Episode;

const B: usize = 5;

/// How far the word's told counts move the answer from the condition it
/// was told under (the post of D2, in nats): < 0.05, < 0.25, < 0.5, < 1, more.
/// The weight per bucket is a post learned instead of fixed at τ.
fn bucket(nats: f64) -> usize {
    [0.05, 0.25, 0.5, 1.0].iter().filter(|&&x| nats >= x).count()
}

/// A relating word's move under an aspect: the word, how far its told
/// counts moved the answer (nats, the post of D2), how often it was told
/// there, and ln P(v | w, c) − ln P(v | c) for every value.
pub struct Move {
    pub word: String,
    pub nats: f64,
    pub told: f64,
    pub r: Vec<f64>,
}

/// The condition and each relating word's move, for aspect `a`.
pub fn moves(c: &GameCounts, e: &Episode, a: usize, values: &[String]) -> (Vec<f64>, Vec<Move>) {
    let k0 = kind(a, Some(&e.game));
    let prior = c.condition(e, &k0, values);
    let mut out = Vec::new();
    for t in &e.tokens {
        let k = match c.has(t, &k0) {
            true => k0.clone(),
            false if c.games.get(t).is_some_and(|gs| gs.iter().any(|g| c.carries(a, g, &e.game))) => kind(a, None),
            false => continue,
        };
        let home = if k == k0 { prior.clone() } else { c.everywhere(&k, values) };
        let Some(told) = c.dist(t, &k, values, &home) else { continue };
        let Some(p) = c.dist(t, &k, values, &prior) else { continue };
        let n: f64 = c.told(t, &k);
        out.push(Move { word: t.clone(), nats: super::compose::kl(&told, &home), told: n, r: p.iter().zip(&prior).map(|(x, b)| (x.max(1e-12) / b).ln()).collect() });
    }
    (prior, out)
}

fn terms(c: &GameCounts, e: &Episode, a: usize, values: &[String]) -> (Vec<f64>, Vec<(usize, Vec<f64>)>) {
    let (prior, ms) = moves(c, e, a, values);
    (prior, ms.into_iter().map(|m| (a * B + bucket(m.nats), m.r)).collect())
}

pub struct Weighted {
    c: GameCounts,
    beta: [f64; 5],
    alpha: Vec<f64>,
}

impl Weighted {
    fn score(&self, a: usize, prior: &[f64], ts: &[(usize, Vec<f64>)]) -> Vec<f64> {
        let mut s: Vec<f64> = prior.iter().map(|p| self.beta[a] * p.ln()).collect();
        for (slot, r) in ts {
            s.iter_mut().zip(r).for_each(|(x, y)| *x += self.alpha[*slot] * y);
        }
        s
    }

    pub fn new(train: &[Episode], epochs: usize, seed: u64) -> Self {
        let fold = |e: &Episode| (phistd::mix64(phistd::fnv1a(&e.key()) ^ seed ^ 0x3E1) % 4) as usize;
        // Held-out examples: (aspect, truth index, condition, moves).
        let mut ex = Vec::new();
        for f in 0..4 {
            let c = GameCounts::new(&train.iter().filter(|e| fold(e) != f).cloned().collect::<Vec<_>>());
            for a in 0..5 {
                let values = c.values(a);
                for e in train.iter().filter(|e| fold(e) == f) {
                    let Some(i) = values.iter().position(|v| *v == e.action[a]) else { continue };
                    let (prior, ts) = terms(&c, e, a, &values);
                    ex.push((a, i, prior, ts));
                }
            }
        }
        let mut w = Weighted { c: GameCounts::new(train), beta: [1.0; 5], alpha: vec![1.0; 5 * B] };
        let (mut g2b, mut g2a) = ([1e-8; 5], vec![1e-8; 5 * B]);
        for _ in 0..epochs {
            for (a, i, prior, ts) in &ex {
                let s = w.score(*a, prior, ts);
                let top = s.iter().cloned().fold(f64::MIN, f64::max);
                let z: f64 = s.iter().map(|x| (x - top).exp()).sum();
                let p: Vec<f64> = s.iter().map(|x| (x - top).exp() / z).collect();
                let gb = prior[*i].ln() - p.iter().zip(prior).map(|(q, b)| q * b.ln()).sum::<f64>();
                g2b[*a] += gb * gb;
                w.beta[*a] = (w.beta[*a] + 0.1 * gb / g2b[*a].sqrt()).max(0.0);
                for (slot, r) in ts {
                    let g = r[*i] - p.iter().zip(r).map(|(q, y)| q * y).sum::<f64>();
                    g2a[*slot] += g * g;
                    w.alpha[*slot] = (w.alpha[*slot] + 0.1 * g / g2a[*slot].sqrt()).max(0.0);
                }
            }
        }
        w
    }

    /// The learned weights, one row per aspect.
    pub fn table(&self) -> String {
        let mut s = String::from("| aspect | β | α: <0.05 nats | <0.25 | <0.5 | <1 | ≥1 |\n|---|---|---|---|---|---|---|\n");
        for a in 0..5 {
            let row: Vec<String> = (0..B).map(|b| format!("{:.2}", self.alpha[a * B + b])).collect();
            s += &format!("| {} | {:.2} | {} |\n", super::world::ASPECTS[a], self.beta[a], row.join(" | "));
        }
        s
    }
}

impl Learner for Weighted {
    fn name(&self) -> String {
        "Phical counts, movement, weights learned per (aspect, post strength)".into()
    }
    fn act(&mut self, e: &Episode) -> Action {
        let mut out: Action = Default::default();
        for (a, slot) in out.iter_mut().enumerate() {
            let values = self.c.values(a);
            let (prior, ts) = terms(&self.c, e, a, &values);
            let s = self.score(a, &prior, &ts);
            let best = values.iter().zip(&s).max_by(|x, y| x.1.total_cmp(y.1).then(y.0.cmp(x.0)));
            *slot = best.map(|x| x.0.clone()).unwrap_or_default();
        }
        out
    }
}
