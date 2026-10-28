//! The assumptions, each a reader the arena scores before it is told.

use super::table::{back, Table};
use phient::study::Reader;
use std::collections::HashMap;

/// A context a reader asks in turn, most specific first; the first one it
/// has seen answers.
pub type Key = fn(&Ctx, &[String], usize) -> Option<String>;

/// What a reader knows about the language so far, beyond its tables: how
/// often each word was told (for the operator assumption).
#[derive(Default)]
pub struct Ctx {
    pub freq: HashMap<String, u32>,
    pub ops: std::collections::HashSet<String>,
    told: usize,
}

impl Ctx {
    /// Operators: the `k` most-told words, re-ranked every 200 sentences.
    fn learn(&mut self, words: &[String], k: usize) {
        for w in words {
            *self.freq.entry(w.clone()).or_insert(0) += 1;
        }
        self.told += 1;
        if self.told % 200 == 1 {
            let mut v: Vec<_> = self.freq.iter().collect();
            v.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
            self.ops = v.into_iter().take(k).map(|(w, _)| w.clone()).collect();
        }
    }
}

/// A reader is a name and a chain of keys over one table per key.
pub struct Chain {
    name: String,
    keys: Vec<Key>,
    tables: Vec<Table>,
    ctx: Ctx,
    ops: usize,
}

impl Chain {
    pub fn new(name: &str, keys: Vec<Key>) -> Self {
        let tables = keys.iter().map(|_| Table::default()).collect();
        Self { name: name.to_string(), keys, tables, ctx: Ctx::default(), ops: 50 }
    }

    /// Whether `w` is one of the operators, as this reader ranks them now.
    pub fn is_op(&self, w: &str) -> bool {
        self.ctx.ops.contains(w)
    }

    /// The first key's row that has been seen, as a distribution.
    pub fn dist(&self, said: &[String]) -> HashMap<String, f64> {
        let j = said.len();
        let row = self.keys.iter().zip(&self.tables).find_map(|(key, t)| key(&self.ctx, said, j).and_then(|k| t.row(&k)));
        let Some(row) = row else { return HashMap::new() };
        let n: u32 = row.values().sum();
        row.iter().map(|(w, c)| (w.clone(), *c as f64 / n as f64)).collect()
    }

    /// The guess at position `j` and which key answered it.
    pub fn answer(&self, words: &[String], j: usize) -> Option<(String, usize)> {
        self.keys.iter().zip(&self.tables).enumerate().find_map(|(i, (key, t))| {
            key(&self.ctx, words, j).and_then(|k| t.top(&k)).map(|w| (w.to_string(), i))
        })
    }
}

impl Reader for Chain {
    fn name(&self) -> &str {
        &self.name
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        self.answer(said, said.len()).map(|(w, _)| w)
    }

    fn tell(&mut self, words: &[String]) {
        for j in 1..words.len() {
            for (key, t) in self.keys.iter().zip(&mut self.tables) {
                if let Some(k) = key(&self.ctx, words, j) {
                    t.add(&k, &words[j]);
                }
            }
        }
        self.ctx.learn(words, self.ops);
    }
}

// ── the keys: one assumption each ────────────────────────────────────────

pub fn uni(_: &Ctx, _: &[String], _: usize) -> Option<String> {
    Some(String::new())
}
pub fn bi(_: &Ctx, w: &[String], j: usize) -> Option<String> {
    Some(back(w, j, 1))
}
pub fn tri(_: &Ctx, w: &[String], j: usize) -> Option<String> {
    Some(back(w, j, 2))
}
pub fn four(_: &Ctx, w: &[String], j: usize) -> Option<String> {
    Some(back(w, j, 3))
}

/// Bottom-up: a word too rare to have its own row is read by its ending.
pub fn suffix(c: &Ctx, w: &[String], j: usize) -> Option<String> {
    let p = &w[j - 1];
    match (c.freq.get(p).copied().unwrap_or(0) < 3, p.chars().count() > 4) {
        (true, true) => Some(format!("~{}", p.chars().rev().take(3).collect::<String>())),
        _ => None,
    }
}

/// Top-down: where in the sentence the word stands (1, 2, 3, later).
pub fn position(_: &Ctx, w: &[String], j: usize) -> Option<String> {
    Some(format!("@{} {}", j.min(4), w[j - 1]))
}

/// The BCPL assumption: the open operator (the last function word before
/// the word before) and the word before, as an expression parser's pending
/// operator and its operand.
pub fn slot(c: &Ctx, w: &[String], j: usize) -> Option<String> {
    let op = w[..j - 1].iter().rev().take(4).find(|x| c.ops.contains(*x))?;
    Some(format!("{op} … {}", w[j - 1]))
}
