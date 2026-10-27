//! The line every other model must beat: the next word from the word
//! before it, counted. Nothing is a baseline unless it is honest, so this
//! is the same reader the hierarchy is measured against (docs/175).

use crate::study::Reader;
use std::collections::HashMap;

#[derive(Default)]
pub struct Count {
    after: HashMap<String, HashMap<String, u32>>,
}

impl Count {
    pub fn new() -> Self {
        Self::default()
    }

    /// The word most often said after `word`.
    pub fn after(&self, word: &str) -> Option<&String> {
        self.after.get(word).and_then(top)
    }

    /// Everything said after `word`, with its count — what a WRITER samples
    /// from (docs/180): the same counts read as a distribution, not a top-1.
    pub fn choices(&self, word: &str) -> Vec<(&String, u32)> {
        self.after.get(word).map(|m| m.iter().map(|(w, n)| (w, *n)).collect()).unwrap_or_default()
    }
}

/// The most told of a word's continuations, ties by spelling.
pub fn top(m: &HashMap<String, u32>) -> Option<&String> {
    m.iter().max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0))).map(|(w, _)| w)
}

impl Reader for Count {
    fn name(&self) -> &str {
        "phient counts (the word before)"
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        self.after(said.last()?).cloned()
    }

    fn tell(&mut self, words: &[String]) {
        for p in words.windows(2) {
            *self.after.entry(p[0].clone()).or_default().entry(p[1].clone()).or_insert(0) += 1;
        }
    }
}
