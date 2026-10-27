//! The concept reader (docs/174–175): the next word read under the
//! construction family the words said so far show — "does not", "is a
//! kind of" — and when that family has nothing to say, backing off onto
//! the counts beneath it. The hierarchy must BACK OFF, not replace: that
//! is what docs/170 claimed and what docs/175 measured a replacement
//! failing to do.

use super::count::top;
use crate::study::Reader;
use std::collections::HashMap;

#[derive(Default)]
pub struct Concept {
    /// Under a family, after a word: what came next.
    under: HashMap<(String, String), HashMap<String, u32>>,
    /// The flat counts, which the concept backs off onto.
    flat: HashMap<String, HashMap<String, u32>>,
    /// How many lines said each word — the closed stratum, grown online.
    said_in: HashMap<String, u32>,
    /// How many lines showed each construction; ≥2 makes it a family.
    shown: HashMap<(String, String), u32>,
}

impl Concept {
    pub fn new() -> Self {
        Self::default()
    }

    /// A word is known once this many lines have said it, so the families
    /// are built of the words that recur — the grammar, not the content.
    const KNOWN: u32 = 3;

    /// The constructions of `words`: adjacent pairs of known words.
    fn constructions(&self, words: &[String]) -> Vec<(String, String)> {
        let known = |w: &String| self.said_in.get(w).copied().unwrap_or(0) >= Self::KNOWN;
        words
            .windows(2)
            .filter(|p| known(&p[0]) && known(&p[1]))
            .map(|p| (p[0].clone(), p[1].clone()))
            .collect()
    }

    /// The family to read under: the most recent construction the words
    /// said show that two or more lines have shown — causal, it never
    /// looks past what was said.
    pub fn family(&self, said: &[String]) -> Option<(String, String)> {
        self.constructions(said)
            .into_iter()
            .rev()
            .find(|c| self.shown.get(c).copied().unwrap_or(0) >= 2)
    }
}

impl Reader for Concept {
    fn name(&self) -> &str {
        "phient concepts (the family, backing off)"
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        let last = said.last()?;
        if let Some(fam) = self.family(said) {
            let key = (format!("{} {}", fam.0, fam.1), last.clone());
            if let Some(w) = self.under.get(&key).and_then(top) {
                return Some(w.clone());
            }
        }
        self.flat.get(last).and_then(top).cloned()
    }

    fn tell(&mut self, words: &[String]) {
        let fams = self.constructions(words);
        let mut once: Vec<&(String, String)> = fams.iter().collect();
        once.sort();
        once.dedup();
        for c in once {
            *self.shown.entry(c.clone()).or_insert(0) += 1;
        }
        let keys: Vec<String> = fams.iter().map(|(a, b)| format!("{a} {b}")).collect();
        for p in words.windows(2) {
            *self.flat.entry(p[0].clone()).or_default().entry(p[1].clone()).or_insert(0) += 1;
            for k in &keys {
                *self.under.entry((k.clone(), p[0].clone())).or_default().entry(p[1].clone()).or_insert(0) += 1;
            }
        }
        let mut seen: Vec<&String> = words.iter().collect();
        seen.sort();
        seen.dedup();
        for w in seen {
            *self.said_in.entry(w.clone()).or_insert(0) += 1;
        }
    }
}
