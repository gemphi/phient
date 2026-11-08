//! Movement: how the words of an utterance are put together (trial 2).
//!
//! The condition alone — the game, under an aspect — already says what is
//! likely. A word counts only as far as it *moves* the answer away from
//! that: `ln P(v | w, c) − ln P(v | c)`, summed over the words that relate
//! under `c` at all. Two things decide whether a word relates:
//!
//! - its **post** (§29–30): the word's told counts must move the answer
//!   under this kind by at least `POST` nats (KL from the condition's) —
//!   a numeral has a post under count, not under what;
//! - the **game** (§21): a word never told in this game is read under the
//!   aspect in any game only where the game it was told in agrees with this
//!   one on that aspect — order and report agree on what a stone is, not on
//!   what is done with it. Agreement is judged between the two games, so a
//!   third game (corrected mistakes, §54) cannot veto it (docs/104).

use super::learners::{game_token, kind, GameCounts};
use super::world::{Episode, ASPECTS};
use std::collections::HashMap;

/// Nats a word must move an aspect's answer to hold a post there.
pub const POST: f64 = 0.5;

impl GameCounts {
    /// The condition's own distribution over `values`: the game token's
    /// counts under `k` (every episode of the game tells it).
    pub fn condition(&self, e: &Episode, k: &str, values: &[String]) -> Vec<f64> {
        let m = self.table(&game_token(e), k).unwrap_or_default();
        let total: f64 = m.values().sum();
        values.iter().map(|v| (m.get(v).copied().unwrap_or(0.0) + 0.01) / (total + 0.01 * values.len() as f64)).collect()
    }

    /// The plain aspect's condition: every game token's counts under `k`.
    pub fn everywhere(&self, k: &str, values: &[String]) -> Vec<f64> {
        let mut m: HashMap<String, f64> = HashMap::new();
        for t in self.words.iter().filter(|t| t.starts_with("⟨game:")) {
            for (v, x) in self.table(t, k).unwrap_or_default() {
                *m.entry(v).or_default() += x;
            }
        }
        let total: f64 = m.values().sum();
        values.iter().map(|v| (m.get(v).copied().unwrap_or(0.0) + 0.01) / (total + 0.01 * values.len() as f64)).collect()
    }

    /// Whether games `g1` and `g2` agree on aspect `a`: among tokens that
    /// hold the aspect firmly (a top answer with at least half their count)
    /// in both, nine in ten give the same top answer. `None` when no token
    /// is firm in both (no evidence).
    pub fn agree_between(&self, a: usize, g1: &str, g2: &str) -> Option<bool> {
        let top = |t: &str, g: &str| {
            let m = self.table(t, &kind(a, Some(g)))?;
            let total: f64 = m.values().sum();
            m.iter().max_by(|x, y| x.1.total_cmp(y.1).then(y.0.cmp(x.0))).filter(|x| *x.1 >= 0.5 * total).map(|x| x.0.clone())
        };
        let (mut both, mut same) = (0usize, 0usize);
        for (t, games) in &self.games {
            if games.iter().any(|g| g == g1) && games.iter().any(|g| g == g2) {
                if let (Some(x), Some(y)) = (top(t, g1), top(t, g2)) {
                    both += 1;
                    same += (x == y) as usize;
                }
            }
        }
        (both > 0).then_some(same as f64 >= 0.9 * both as f64)
    }

    /// May a word told in game `from` be read in game `to` under aspect `a`?
    /// By the two games' own agreement; with no evidence between them, by
    /// whether any two games of the language agree on the aspect.
    pub fn carries(&self, a: usize, from: &str, to: &str) -> bool {
        self.agree_between(a, from, to).unwrap_or(self.open[a])
    }
}

pub fn kl(p: &[f64], q: &[f64]) -> f64 {
    p.iter().zip(q).filter(|(x, _)| **x > 0.0).map(|(x, y)| x * (x / y).ln()).sum()
}

/// The value of aspect `a` the utterance moves to. `dist(t, k, base)` is a
/// token's reading under kind `k` as a distribution over `values`.
pub fn movement(c: &GameCounts, e: &Episode, a: usize, values: &[String], dist: impl Fn(&str, &str, &[f64]) -> Option<Vec<f64>>) -> String {
    let k0 = kind(a, Some(&e.game));
    // The condition is read from its count, for every learner: a game token
    // relates to every answer (it holds no post), and a function reads such
    // a spread relation as one peaked answer (docs/103: `undo` for every
    // order after one retraining).
    let counted = c.condition(e, &k0, values);
    let prior = counted.clone();
    let mut score: Vec<f64> = prior.iter().map(|p| p.ln()).collect();
    for t in &e.tokens {
        let k = match c.has(t, &k0) {
            true => k0.clone(),
            false if c.games.get(t).is_some_and(|gs| gs.iter().any(|g| c.carries(a, g, &e.game))) => kind(a, None),
            false => continue,
        };
        // Movement is always measured from this game's condition, also for a
        // word read under the aspect in any game.
        // The post is decided by the relation's count: a word told with
        // every answer holds no post there, however its function reads.
        // A word read from another game holds its post there or nowhere:
        // its counts are measured against the condition it was told under.
        let home = if k == k0 { counted.clone() } else { c.everywhere(&k, values) };
        let Some(told) = c.dist(t, &k, values, &home) else { continue };
        if kl(&told, &home) < c.post_by.get(a).copied().unwrap_or(c.post) {
            continue;
        }
        let Some(p) = dist(t, &k, &prior) else { continue };
        for (s, (x, b)) in score.iter_mut().zip(p.iter().zip(&prior)) {
            *s += (x.max(1e-12) / b).ln();
        }
    }
    let best = values.iter().zip(&score).max_by(|x, y| x.1.total_cmp(y.1).then(y.0.cmp(x.0)));
    best.map(|x| x.0.clone()).unwrap_or_default()
}

/// §11–17, the toolbox: for every word, the aspects of the order game in
/// which it holds a post, and what it moves the answer to there.
pub fn posts(c: &GameCounts) -> String {
    let words: Vec<&String> = c.words.iter().filter(|t| !t.starts_with("⟨game")).collect();
    let mut s = String::from("| word | holds a post under | moves the answer to |\n|---|---|---|\n");
    let probe = Episode { game: "order".into(), tokens: vec![], action: Default::default() };
    for w in words {
        let mut under = Vec::new();
        let mut to = Vec::new();
        for (a, name) in ASPECTS.iter().enumerate() {
            let k = kind(a, Some("order"));
            let values = c.values(a);
            let prior = c.condition(&probe, &k, &values);
            let Some(p) = c.dist(w, &k, &values, &prior) else { continue };
            if kl(&p, &prior) >= POST {
                under.push(name.to_string());
                let top = values.iter().zip(&p).max_by(|x, y| x.1.total_cmp(y.1)).map(|x| x.0.clone()).unwrap_or_default();
                to.push(format!("{name} = {top}"));
            }
        }
        if !under.is_empty() {
            s += &format!("| `{w}` | {} | {} |\n", under.join(", "), to.join(", "));
        }
    }
    s
}
