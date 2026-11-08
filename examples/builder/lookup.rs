//! The whole-utterance lookup: a learner that remembers sentences.

use super::learners::{Action, Learner};
use super::world::Episode;
use std::collections::HashMap;

/// The whole utterance, remembered: seen ⇒ what followed most often; unseen
/// ⇒ the game's most frequent action.
pub struct Lookup {
    seen: HashMap<String, Action>,
    fallback: HashMap<String, Action>,
}

/// GameCounts of each action seen, keyed by utterance or by game.
type GameCounts = HashMap<String, HashMap<Action, usize>>;

fn most<K: std::hash::Hash + Eq + Clone + Ord>(m: &HashMap<K, usize>) -> Option<K> {
    m.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0))).map(|(k, _)| k.clone())
}

impl Lookup {
    pub fn new(train: &[Episode]) -> Self {
        let (mut by_key, mut by_game): (GameCounts, GameCounts) = Default::default();
        for e in train {
            *by_key.entry(e.key()).or_default().entry(e.action.clone()).or_default() += 1;
            *by_game.entry(e.game.clone()).or_default().entry(e.action.clone()).or_default() += 1;
        }
        let pick = |m: GameCounts| m.into_iter().filter_map(|(k, v)| Some((k, most(&v)?))).collect();
        Self { seen: pick(by_key), fallback: pick(by_game) }
    }
}

impl Learner for Lookup {
    fn name(&self) -> String {
        "lookup (whole utterance)".into()
    }
    fn act(&mut self, e: &Episode) -> Action {
        self.seen.get(&e.key()).or_else(|| self.fallback.get(&e.game)).cloned().unwrap_or_default()
    }
}
