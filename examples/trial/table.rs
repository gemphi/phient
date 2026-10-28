//! One counting table, keyed by a context string, that keeps its best
//! continuation as it counts (so a guess is one lookup, not a scan).

use std::collections::HashMap;

/// What followed each context, and the most-told follower so far.
#[derive(Default)]
pub struct Table {
    next: HashMap<String, HashMap<String, u32>>,
    best: HashMap<String, (String, u32)>,
    total: HashMap<String, u32>,
}

impl Table {
    pub fn add(&mut self, key: &str, word: &str) {
        *self.total.entry(key.to_string()).or_insert(0) += 1;
        let n = self.next.entry(key.to_string()).or_default().entry(word.to_string()).or_insert(0);
        *n += 1;
        let n = *n;
        let best = self.best.entry(key.to_string()).or_insert_with(|| (word.to_string(), 0));
        if n > best.1 {
            *best = (word.to_string(), n);
        }
    }

    pub fn top(&self, key: &str) -> Option<&str> {
        self.best.get(key).map(|(w, _)| w.as_str())
    }

    /// What followed `key`, with counts.
    pub fn row(&self, key: &str) -> Option<&HashMap<String, u32>> {
        self.next.get(key)
    }

    /// How many times `word` followed `key`.
    pub fn count(&self, key: &str, word: &str) -> u32 {
        self.next.get(key).and_then(|m| m.get(word)).copied().unwrap_or(0)
    }

    /// Everything that followed `key`.
    pub fn followers(&self, key: &str) -> impl Iterator<Item = &str> {
        self.next.get(key).into_iter().flat_map(|m| m.keys().map(String::as_str))
    }

    /// How many times `key` was seen at all.
    pub fn seen(&self, key: &str) -> u32 {
        self.total.get(key).copied().unwrap_or(0)
    }
}

/// The context `k` words back from position `j` of `words`, padded with
/// `<s>` before the sentence: `k = 0` is the empty context.
pub fn back(words: &[String], j: usize, k: usize) -> String {
    (0..k)
        .rev()
        .map(|d| match j.checked_sub(d + 1) {
            Some(i) => words[i].as_str(),
            None => "<s>",
        })
        .collect::<Vec<_>>()
        .join(" ")
}
