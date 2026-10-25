//! The counts reading of quest rows (D2, D4): no operator, no epochs. A
//! relation is a count; a word is heard by its counts the moment it is
//! told, so one pointing teaches it (docs/113, docs/124).
//!
//! A row `(about, kind, other)` is counted twice: under the kind alone (the
//! prior, what the kind usually names) and under each of `about`'s given
//! conditions — the rows told about it under the `given` kinds (a function's
//! words and its signature's types; an utterance's words). Asked `kind` of
//! something new under conditions `c₁…cₙ`, the reading is
//! `ln prior(v) + Σᵢ ln(distᵢ(v) / prior(v))` over the kind's values, each
//! `distᵢ` the condition's counts smoothed toward the prior with weight one,
//! and only the conditions that move the prior by `tau` nats hold a post
//! (D2; `tau = 0` lets every condition weigh, the bag-of-words reading).
//!
//! Every name is a phiecs [`Word`] (docs/133): the strings live once in the
//! [`Lexicon`], and the counts are maps of ids, so a night of telling fits
//! in memory. The interface stays in words a caller can read.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it; here the relations are the counts themselves.

pub use super::table::Table;
use phiecs::quest::Row;
use phiecs::word::{Lexicon, Word};
use std::collections::{BTreeSet, HashMap};

/// A kind's sorted values and their prior.
pub type ValuesAndPrior = std::rc::Rc<(Vec<String>, Vec<f64>)>;

#[derive(Default, Clone)]
pub struct Tally {
    lexicon: Lexicon,
    /// Per kind, its sorted values and their prior, kept between readings
    /// and dropped when more is learned.
    cache: std::cell::RefCell<HashMap<Word, ValuesAndPrior>>,
    /// Per kind, each value's place in its sorted values.
    pub(super) index: std::cell::RefCell<HashMap<Word, std::rc::Rc<HashMap<Word, usize>>>>,
    /// `(condition, kind)` ⇒ value ⇒ count.
    n: HashMap<(Word, Word), Table>,
    /// `kind` ⇒ value ⇒ count: the prior.
    pub(super) prior: HashMap<Word, Table>,
    /// Each `about`'s given conditions.
    given: HashMap<Word, Vec<Word>>,
    given_kinds: Vec<Word>,
    /// A word's kin by what it was told (`kin.rs`), found once per learning.
    pub(super) kin: std::cell::RefCell<super::kin::Kin>,
    /// How much a level trusts its own counts (`weights.rs`).
    pub weights: super::weights::Weighting,
}

impl Tally {
    pub fn new(given_kinds: &[&str]) -> Self {
        let mut c = Tally::default();
        c.given_kinds = given_kinds.iter().map(|k| c.lexicon.intern(k)).collect();
        c
    }

    pub(super) fn word(&self, s: &str) -> Option<Word> {
        self.lexicon.get(s)
    }

    fn spelling(&self, w: Word) -> String {
        self.lexicon.spelling(w).unwrap_or("").to_string()
    }

    /// Tally `rows`; the given rows first, so a function's words condition
    /// its body rows told in the same call. Call it again to learn more:
    /// one more answer is one more sighting of each of its relations.
    pub fn learn(&mut self, rows: &[Row]) {
        self.cache.borrow_mut().clear();
        self.index.borrow_mut().clear();
        self.kin.borrow_mut().clear();
        let interned: Vec<(Word, Word, Word, f64)> = rows.iter().map(|r| (self.lexicon.intern(&r.about), self.lexicon.intern(&r.kind), self.lexicon.intern(&r.other), r.weight)).collect();
        for (about, _, other, _) in interned.iter().filter(|r| self.given_kinds.contains(&r.1)) {
            let v = self.given.entry(*about).or_default();
            if !v.contains(other) {
                v.push(*other);
            }
        }
        for (about, kind, other, weight) in interned.iter().filter(|r| !self.given_kinds.contains(&r.1)) {
            self.prior.entry(*kind).or_default().add(*other, *weight);
            if let Some(conds) = self.given.get(about) {
                for c in conds {
                    self.n.entry((*c, *kind)).or_default().add(*other, *weight);
                }
            }
        }
    }

    /// The conditions told about `about`.
    pub fn conditions(&self, about: &str) -> Vec<String> {
        self.word(about).and_then(|a| self.given.get(&a)).map(|v| v.iter().map(|w| self.spelling(*w)).collect()).unwrap_or_default()
    }

    /// Every value the kind named, sorted.
    pub fn values(&self, kind: &str) -> Vec<String> {
        self.word(kind).and_then(|k| self.prior.get(&k)).map(|m| m.keys().iter().map(|w| self.spelling(*w)).collect::<BTreeSet<_>>().into_iter().collect()).unwrap_or_default()
    }

    /// The kind's sorted values with their prior, computed once per learning.
    pub fn values_and_prior(&self, kind: &str) -> ValuesAndPrior {
        let Some(k) = self.word(kind) else { return std::rc::Rc::new((Vec::new(), Vec::new())) };
        if let Some(c) = self.cache.borrow().get(&k) {
            return c.clone();
        }
        let values = self.values(kind);
        let prior = self.prior(kind, &values);
        let c = std::rc::Rc::new((values, prior));
        self.cache.borrow_mut().insert(k, c.clone());
        c
    }

    pub(super) fn table(&self, c: &str, kind: &str) -> Option<&Table> {
        self.n.get(&(self.word(c)?, self.word(kind)?))
    }

    /// Every condition's table under `kind`.
    pub(super) fn tables_under(&self, kind: &str) -> Vec<(Word, &Table)> {
        let Some(k) = self.word(kind) else { return Vec::new() };
        self.n.iter().filter(|((_, kk), _)| *kk == k).map(|((c, _), t)| (*c, t)).collect()
    }

    pub(super) fn spelling_of(&self, w: Word) -> String {
        self.spelling(w)
    }

    /// How many sightings condition `c` has under `kind`.
    pub fn told(&self, c: &str, kind: &str) -> f64 {
        self.table(c, kind).map_or(0.0, Table::sum)
    }

    /// The raw count of `value` under `kind`, under condition `c` or, with
    /// `None`, over everything told.
    pub fn count(&self, c: Option<&str>, kind: &str, value: &str) -> f64 {
        let Some(v) = self.word(value) else { return 0.0 };
        let m = match c {
            Some(c) => self.table(c, kind),
            None => self.word(kind).and_then(|k| self.prior.get(&k)),
        };
        m.and_then(|m| m.get(&v)).copied().unwrap_or(0.0)
    }

    /// Everything `c` said under `kind` with its count — or everything told
    /// under `kind`, with `None`.
    pub fn said(&self, c: Option<&str>, kind: &str) -> Vec<(String, f64)> {
        let m = match c {
            Some(c) => self.table(c, kind),
            None => self.word(kind).and_then(|k| self.prior.get(&k)),
        };
        m.map(|m| m.keys().iter().filter_map(|w| m.get(w).map(|n| (self.spelling(*w), *n))).collect()).unwrap_or_default()
    }

    /// The raw total under `kind`, under condition `c` or over everything told.
    pub fn total(&self, c: Option<&str>, kind: &str) -> f64 {
        match c {
            Some(c) => self.told(c, kind),
            None => self.word(kind).and_then(|k| self.prior.get(&k)).map_or(0.0, Table::sum),
        }
    }

    /// The reading of `kind` under `conds`: each value's score
    /// `ln prior + Σ posts`, in the order of `values`, and which conditions
    /// held a post.
    pub fn read(&self, conds: &[String], kind: &str, values: &[String], tau: f64) -> (Vec<f64>, Vec<String>) {
        let prior = self.prior(kind, values);
        let mut s: Vec<f64> = prior.iter().map(|p| p.ln()).collect();
        let mut posts = Vec::new();
        for c in conds {
            let Some(d) = self.dist(c, kind, values, &prior) else { continue };
            let kl: f64 = d.iter().zip(&prior).map(|(x, y)| x * (x / y).ln()).sum();
            if kl >= tau {
                s.iter_mut().zip(&d).zip(&prior).for_each(|((x, d), p)| *x += (d / p).ln());
                posts.push(c.clone());
            }
        }
        (s, posts)
    }

    /// The kind's values ranked under `conds`, best first, with their scores.
    pub fn movement(&self, conds: &[String], kind: &str, tau: f64) -> Vec<(String, f64)> {
        let values = self.values(kind);
        let (s, _) = self.read(conds, kind, &values, tau);
        let mut out: Vec<(String, f64)> = values.into_iter().zip(s).collect();
        out.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        out
    }
}
