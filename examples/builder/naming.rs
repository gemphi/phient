//! The Augustinian picture (§1): every word names an object; a sentence is
//! a combination of names. A word's meaning is the one object it goes with
//! most often, found cross-situationally from the same episodes the other
//! learners see. What is done with the named objects is always the same
//! (bring them); there are no games and no posts, only names.

use super::learners::{Action, Learner};
use super::world::Episode;
use std::collections::HashMap;

/// Aspects whose values are objects that can be named: what, colour,
/// count, place (not the act, and not `-` or `any`).
const OBJECT_ASPECTS: [usize; 4] = [1, 2, 3, 4];

pub struct Naming {
    /// Token ⇒ (aspect, value, P(value | token)).
    names: HashMap<String, (usize, String, f64)>,
}

impl Naming {
    pub fn new(train: &[Episode]) -> Self {
        let mut seen: HashMap<String, f64> = HashMap::new();
        let mut with: HashMap<(String, usize, String), f64> = HashMap::new();
        for e in train {
            for t in &e.tokens {
                *seen.entry(t.clone()).or_default() += 1.0;
                for a in OBJECT_ASPECTS {
                    let v = &e.action[a];
                    if v != "-" && v != "any" {
                        *with.entry((t.clone(), a, v.clone())).or_default() += 1.0;
                    }
                }
            }
        }
        let mut names: HashMap<String, (usize, String, f64)> = HashMap::new();
        let mut keys: Vec<_> = with.into_iter().collect();
        keys.sort_by(|x, y| (x.0 .1, &x.0 .2).cmp(&(y.0 .1, &y.0 .2)));
        for ((t, a, v), c) in keys {
            let p = c / seen[&t];
            let better = names.get(&t).is_none_or(|n| p > n.2);
            if better {
                names.insert(t, (a, v, p));
            }
        }
        Self { names }
    }
}

impl Learner for Naming {
    fn name(&self) -> String {
        "naming table (Augustine, §1)".into()
    }
    fn act(&mut self, e: &Episode) -> Action {
        let mut out: Action = ["bring", "-", "any", "1", "here"].map(String::from);
        let mut best = [0.0f64; 5];
        for t in &e.tokens {
            if let Some((a, v, p)) = self.names.get(t) {
                if *p > best[*a] {
                    best[*a] = *p;
                    out[*a] = v.clone();
                }
            }
        }
        out
    }
}
