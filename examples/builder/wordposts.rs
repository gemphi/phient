//! Two speeds for the post (docs/113). D2 fixes a word's post by its
//! counts: it moves an aspect by τ = 0.5 nats or it is not heard. That is the
//! fast speed, and it learns a word from one pointing (sepia). Here a word
//! told often enough (`SEEN` times under the aspect) may also earn its own
//! weight on that aspect, slowly, from how well its move predicts answers
//! it was not counted on — pulled back toward the post it holds by D2
//! (weight 1 with a post, 0 without) by a penalty `LAMBDA`. A word seen
//! rarely keeps D2 exactly, and while such a word holds a post in the
//! utterance, the whole aspect is read by D2. The condition keeps weight 1 (D4).
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c); the slow speed lets how much it counts be one of them.

use super::compose::POST;
use super::learners::{Action, GameCounts, Learner};
use super::weighted::moves;
use super::world::Episode;
use std::collections::HashMap;

pub const SEEN: f64 = 8.0;
pub const LAMBDA: f64 = 0.05;

pub struct WordPosts {
    c: GameCounts,
    /// (word, aspect) ⇒ weight learned away from the post.
    delta: HashMap<(String, usize), f64>,
    pub seen: f64,
}

fn weight(delta: &HashMap<(String, usize), f64>, seen: f64, word: &str, a: usize, nats: f64, told: f64) -> f64 {
    let post = if nats >= POST { 1.0 } else { 0.0 };
    match told >= seen {
        true => (post + delta.get(&(word.to_string(), a)).copied().unwrap_or(0.0)).max(0.0),
        false => post,
    }
}

impl WordPosts {
    pub fn new(train: &[Episode], epochs: usize, seed: u64, seen: f64) -> Self {
        let fold = |e: &Episode| (phistd::mix64(phistd::fnv1a(&e.key()) ^ seed ^ 0x5EE) % 4) as usize;
        let mut ex = Vec::new();
        for f in 0..4 {
            let c = GameCounts::new(&train.iter().filter(|e| fold(e) != f).cloned().collect::<Vec<_>>());
            for a in 0..5 {
                let values = c.values(a);
                for e in train.iter().filter(|e| fold(e) == f) {
                    let Some(i) = values.iter().position(|v| *v == e.action[a]) else { continue };
                    let (prior, ms) = moves(&c, e, a, &values);
                    ex.push((a, i, prior, ms));
                }
            }
        }
        let mut delta: HashMap<(String, usize), f64> = HashMap::new();
        for _ in 0..epochs {
            for (a, i, prior, ms) in &ex {
                let mut s: Vec<f64> = prior.iter().map(|p| p.ln()).collect();
                for m in ms {
                    let w = weight(&delta, seen, &m.word, *a, m.nats, m.told);
                    s.iter_mut().zip(&m.r).for_each(|(x, y)| *x += w * y);
                }
                let top = s.iter().cloned().fold(f64::MIN, f64::max);
                let z: f64 = s.iter().map(|x| (x - top).exp()).sum();
                let p: Vec<f64> = s.iter().map(|x| (x - top).exp() / z).collect();
                for m in ms.iter().filter(|m| m.told >= seen) {
                    let g = m.r[*i] - p.iter().zip(&m.r).map(|(q, y)| q * y).sum::<f64>();
                    let d = delta.entry((m.word.clone(), *a)).or_default();
                    *d += 0.05 * (g - LAMBDA * *d);
                }
            }
        }
        WordPosts { c: GameCounts::new(train), delta, seen }
    }

    /// The words that moved most from their post, per aspect.
    pub fn moved(&self, k: usize) -> String {
        let mut v: Vec<(&(String, usize), &f64)> = self.delta.iter().filter(|(_, d)| d.abs() > 0.05).collect();
        v.sort_by(|x, y| y.1.abs().total_cmp(&x.1.abs()).then(x.0.cmp(y.0)));
        v.iter().take(k).map(|((w, a), d)| format!("{w}@{} {:+.2}", super::world::ASPECTS[*a], d)).collect::<Vec<_>>().join(", ")
    }
}

impl Learner for WordPosts {
    fn name(&self) -> String {
        format!("Phical counts, movement, two-speed posts (own weight after {} sightings)", self.seen)
    }
    fn act(&mut self, e: &Episode) -> Action {
        let mut out: Action = Default::default();
        for (a, slot) in out.iter_mut().enumerate() {
            let values = self.c.values(a);
            let (prior, ms) = moves(&self.c, e, a, &values);
            let mut s: Vec<f64> = prior.iter().map(|p| p.ln()).collect();
            // A word still at the fast speed that holds a post (a sample met
            // by pointing, §28–30) is heard as D2 has it, and so is every
            // other word on this aspect: the habits the slow speed learned
            // were learned where that word was never said.
            let fresh = ms.iter().any(|m| m.told < self.seen && m.nats >= POST);
            for m in &ms {
                let w = match fresh {
                    true => if m.nats >= POST { 1.0 } else { 0.0 },
                    false => weight(&self.delta, self.seen, &m.word, a, m.nats, m.told),
                };
                s.iter_mut().zip(&m.r).for_each(|(x, y)| *x += w * y);
            }
            let best = values.iter().zip(&s).max_by(|x, y| x.1.total_cmp(y.1).then(y.0.cmp(x.0)));
            *slot = best.map(|x| x.0.clone()).unwrap_or_default();
        }
        out
    }
}

