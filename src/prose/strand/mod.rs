//! A story as a strand of sixty-four levels, coded like DNA.
//!
//! Between a word and the whole story lie the uses of every level in
//! between — a pair, a phrase, a clause, a sentence, a turn, a section —
//! and the strand names sixty-four of them with the sixty-four codons:
//! `AAA` is the word, `TTT` the whole story ([`phistd::codon`]). Which
//! levels, and where each level's units begin and end, is not declared: the
//! story is divided by [`cut::Cut`], most informative division first, and
//! level `k` is the division at `m(k)` units, the counts spaced
//! geometrically from one unit to one per word. A unit is a scope; what it
//! *says* is its signature — the words it says beyond its parent, by D4's
//! `c · ln(p_unit / p_parent)` — so reading a level is reading the story
//! told in `m(k)` beats, each in the words that carry the most information
//! there. Nothing is generated: every word of every level is the story's.

mod code;
pub mod cut;
pub mod path;
pub mod prior;
pub mod reading;

pub use cut::{Cut, Span, Step};
pub use path::Path;
pub use prior::Prior;
pub use reading::ScopeSet;

use phiecs::quest::Row;
use phistd::codon::{codon, CODONS};
use std::collections::HashMap;

/// A story divided and laid out as sixty-four levels.
#[derive(Debug, Clone)]
pub struct Strand {
    pub tokens: Vec<String>,
    pub vocab: Vec<String>,
    pub index: HashMap<String, u32>,
    pub prior: Prior,
    pub cut: Cut,
    /// `units[k]`: how many units level `k` has (`units[0]` = tokens, `units[63]` = 1).
    pub units: [usize; CODONS],
}

/// A token that is a word, not a mark.
fn is_word(t: &str) -> bool {
    t.chars().any(char::is_alphanumeric)
}

impl Strand {
    pub fn new(text: &str, prior: Prior) -> Self {
        Self::from_tokens(super::tokens(text), prior)
    }

    /// A strand of a story already split into tokens.
    pub fn from_tokens(tokens: Vec<String>, prior: Prior) -> Self {
        let mut index: HashMap<String, u32> = HashMap::new();
        let mut vocab = Vec::new();
        let ids: Vec<u32> = tokens.iter().map(|t| *index.entry(t.clone()).or_insert_with(|| { vocab.push(t.clone()); vocab.len() as u32 - 1 })).collect();
        let p: Vec<f64> = vocab.iter().map(|w| prior.p(w)).collect();
        let marks: Vec<bool> = tokens.iter().map(|t| !is_word(t)).collect();
        let cut = Cut::divide(ids, marks, vocab.len(), &|id| p[id as usize]);
        let n = tokens.len().max(1);
        let mut units = [1usize; CODONS];
        for j in 1..CODONS {
            let k = CODONS - 1 - j;
            let geometric = (n as f64).powf(j as f64 / (CODONS - 1) as f64).round() as usize;
            units[k] = geometric.max(units[k + 1] + 1).min(n);
        }
        units[0] = n;
        Strand { tokens, vocab, index, prior, cut, units }
    }

    fn p(&self, id: u32) -> f64 {
        self.prior.p(&self.vocab[id as usize])
    }

    /// How likely `word` is, read in span `s` (the prior when `None`):
    /// Witten–Bell from the span's own counts to its parent's reading, the
    /// code the division was made with. A word the span never said keeps
    /// the share the span escapes to its parent; a word the story never
    /// said, the share that runs down from the prior.
    pub fn read(&self, s: Option<usize>, word: &str) -> f64 {
        let Some(i) = s else { return self.prior.p(word) };
        let sp = &self.cut.spans[i];
        let c = self.index.get(word).and_then(|id| sp.counts.get(id)).copied().unwrap_or(0) as f64;
        let t = sp.counts.len() as f64;
        (c + t * self.read(sp.parent, word)) / (sp.len() as f64 + t)
    }

    /// The unit of level `k` that holds token `i`.
    pub fn unit_of(&self, k: usize, i: usize) -> usize {
        let level = self.level(k);
        let at = level.partition_point(|s| self.cut.spans[*s].start <= i);
        level[at.saturating_sub(1)]
    }

    /// The spans of level `k`, in story order.
    pub fn level(&self, k: usize) -> Vec<usize> {
        self.cut.units_at(self.units[k])
    }

    /// The information level `k` adds beyond level `k + 1`, in nats: the
    /// gains of the divisions between them. Level 63 adds nothing.
    pub fn gain(&self, k: usize) -> f64 {
        if k + 1 >= CODONS {
            return 0.0;
        }
        let (lo, hi) = (self.units[k + 1], self.units[k]);
        self.cut.steps.iter().take(hi.saturating_sub(1)).skip(lo.saturating_sub(1)).map(|s| s.gain).sum()
    }

    /// What span `s` says beyond its parent: up to `n` words with positive
    /// `c · ln(p_s / p_parent)`, in the order the span first says them.
    pub fn signature(&self, s: usize, n: usize) -> Vec<String> {
        let sp = &self.cut.spans[s];
        let prior = |id: u32| self.p(id);
        let mut scored: Vec<(f64, usize, u32)> = Vec::new();
        for (&id, &c) in &sp.counts {
            if !is_word(&self.vocab[id as usize]) {
                continue;
            }
            let here = self.cut.share(Some(s), id, &prior);
            let above = self.cut.share(sp.parent, id, &prior);
            let score = c as f64 * (here / above).ln();
            if score > 0.0 {
                let first = (sp.start..sp.end).find(|i| self.cut.ids[*i] == id).unwrap_or(sp.start);
                scored.push((score, first, id));
            }
        }
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        scored.truncate(n);
        scored.sort_by_key(|x| x.1);
        scored.into_iter().map(|(_, _, id)| self.vocab[id as usize].clone()).collect()
    }

    /// The span's own text.
    pub fn text(&self, s: usize) -> String {
        let sp = &self.cut.spans[s];
        self.tokens[sp.start..sp.end].join(" ")
    }

    /// How many units the story's information supports: the leaves of the
    /// division tree pruned to what shortens the story's description.
    pub fn informative_units(&self) -> usize {
        self.cut.supported()
    }

    /// The finest level whose units the story's information still supports.
    pub fn informative_level(&self) -> usize {
        let u = self.informative_units();
        (0..CODONS).find(|k| self.units[*k] <= u).unwrap_or(CODONS - 1)
    }

    /// The level at which span `s` first appears (the coarsest level holding it).
    pub fn level_of(&self, s: usize) -> usize {
        (0..CODONS).rev().find(|k| self.level(*k).contains(&s)).unwrap_or(0)
    }

    /// The story told at level `k`: one line per unit, its signature.
    pub fn told_at(&self, k: usize, words: usize) -> Vec<String> {
        self.level(k).into_iter().map(|s| self.signature(s, words).join(" ")).collect()
    }

    /// The strand as rows under `root`: every span a scope named by the
    /// codon of the level it first appears at and its token range, `within`
    /// its parent's scope; each scope `says` (and `has-word`) its words,
    /// weighted by their count. A cast can be drawn into it as into any tree.
    pub fn rows(&self, root: &str) -> Vec<Row> {
        let mut names: HashMap<usize, String> = HashMap::new();
        let mut out = Vec::new();
        for (i, sp) in self.cut.spans.iter().enumerate() {
            let own = format!("{}:{}-{}", codon(self.level_of(i)), sp.start, sp.end);
            let name = match sp.parent {
                Some(p) => format!("{}/{own}", names[&p]),
                None => format!("scope:{root}"),
            };
            if let Some(p) = sp.parent {
                out.push(Row::new(&name, "within", &names[&p]));
            }
            for (&id, &c) in &sp.counts {
                let w = self.vocab[id as usize].clone();
                for kind in ["says", "has-word"] {
                    out.push(Row::weighted(&name, kind, &w, c as f64));
                }
            }
            names.insert(i, name);
        }
        out
    }
}

#[cfg(test)]
mod tests;
