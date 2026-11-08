//! The usual machine-learning answer: per aspect, a softmax over the
//! aspect's values from a bag of the tokens and the game (multinomial
//! logistic regression, SGD). No word is told what it names; the weights
//! are shared out discriminatively.

use super::learners::{game_token, Action, GameCounts, Learner};
use super::world::{Dice, Episode, ASPECTS};
use std::collections::HashMap;

pub struct Bow {
    values: Vec<Vec<String>>,
    /// Per aspect: feature ⇒ weight per value.
    w: Vec<HashMap<String, Vec<f64>>>,
}

fn features(e: &Episode) -> Vec<String> {
    let mut f = e.tokens.clone();
    f.push(game_token(e));
    f.push("⟨bias⟩".into());
    f
}

/// The aspects an episode trains: all, or only colour for a word stationed
/// by a grammar word (§30) — the same knowledge the other learners get.
fn trained(e: &Episode) -> Vec<usize> {
    match e.tokens.first().map(String::as_str) {
        Some("colour") => vec![2],
        _ => (0..ASPECTS.len()).collect(),
    }
}

impl Bow {
    pub fn new(train: &[Episode], counts: &GameCounts, epochs: usize, seed: u64) -> Self {
        let values: Vec<Vec<String>> = (0..ASPECTS.len()).map(|a| counts.values(a)).collect();
        let mut w: Vec<HashMap<String, Vec<f64>>> = vec![HashMap::new(); ASPECTS.len()];
        let mut order: Vec<usize> = (0..train.len()).collect();
        let mut d = Dice(seed);
        for epoch in 0..epochs {
            for i in (1..order.len()).rev() {
                order.swap(i, d.below(i + 1));
            }
            let lr = 0.2 / (1.0 + epoch as f64 * 0.1);
            for &i in &order {
                let e = &train[i];
                let f = features(e);
                for a in trained(e) {
                    let n = values[a].len();
                    let Some(y) = values[a].iter().position(|v| *v == e.action[a]) else { continue };
                    let p = softmax(&f, &w[a], n);
                    for x in &f {
                        let row = w[a].entry(x.clone()).or_insert_with(|| vec![0.0; n]);
                        for (k, r) in row.iter_mut().enumerate() {
                            let g = p[k] - (k == y) as usize as f64;
                            *r -= lr * (g + 1e-4 * *r);
                        }
                    }
                }
            }
        }
        Self { values, w }
    }
}

fn softmax(f: &[String], w: &HashMap<String, Vec<f64>>, n: usize) -> Vec<f64> {
    let mut z = vec![0.0; n];
    for x in f {
        if let Some(row) = w.get(x) {
            for (k, r) in row.iter().enumerate() {
                z[k] += r;
            }
        }
    }
    let m = z.iter().cloned().fold(f64::MIN, f64::max);
    let e: Vec<f64> = z.iter().map(|v| (v - m).exp()).collect();
    let s: f64 = e.iter().sum();
    e.into_iter().map(|v| v / s).collect()
}

impl Learner for Bow {
    fn name(&self) -> String {
        "bag of words + softmax".into()
    }
    fn act(&mut self, e: &Episode) -> Action {
        let f = features(e);
        let mut out: Action = Default::default();
        for (a, slot) in out.iter_mut().enumerate() {
            let p = softmax(&f, &self.w[a], self.values[a].len());
            let k = p.iter().enumerate().max_by(|x, y| x.1.total_cmp(y.1)).map_or(0, |x| x.0);
            *slot = self.values[a].get(k).cloned().unwrap_or_default();
        }
        out
    }
}
