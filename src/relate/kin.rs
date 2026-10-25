//! A word's kin (docs/138; Wittgenstein §67, family resemblance): the
//! words that were told the same company — the same words before them
//! (the pairs they close, `pair:u w` under `says`) and the same words after
//! them (`next`). Kinship is the cosine of those two count tables, and it
//! is found from what was told, never declared: a word with no kin has
//! none. What a word was never heard go on to, its kin may have been: the
//! kin reading is the mixture of the kin's `next` distributions, each
//! weighted by its kinship — a word's reading is its own plus its kin's
//! (docs/97's inheritance, with counts in place of the operator).

use super::counts::Tally;
use super::table::Table;
use phiecs::word::Word;
use std::collections::HashMap;
use std::rc::Rc;

/// Posting lists: a context word ⇒ the words it stands beside, with counts;
/// and each word's norm. Built once per learning.
#[derive(Default, Clone)]
pub struct Kin {
    left: HashMap<Word, Vec<(Word, f64)>>,
    right: HashMap<Word, Vec<(Word, f64)>>,
    norm_left: HashMap<Word, f64>,
    norm_right: HashMap<Word, f64>,
    built: bool,
    found: HashMap<Word, Rc<Vec<(Word, f64)>>>,
    /// Each word's kin reading over a kind's whole value list, once computed.
    base: HashMap<(Word, Word), Rc<Vec<f64>>>,
}

impl Kin {
    pub fn clear(&mut self) {
        *self = Kin::default();
    }
}

impl Tally {
    /// A word, as the tree knows words: said (`says`) and not a pair. ScopeSet,
    /// pairs and `scope|word` conditions have `next` tables too and are not kin.
    fn is_word(&self, w: Word) -> bool {
        let s = self.spelling_of(w);
        !s.starts_with("pair:") && self.count(None, super::SAYS, &s) > 0.0
    }

    fn build_kin(&self) {
        let mut kin = self.kin.borrow_mut();
        if kin.built {
            return;
        }
        for (w, t) in self.tables_under("next").into_iter().filter(|(w, _)| self.is_word(*w)) {
            let mut n2 = 0.0;
            for (x, n) in entries(t) {
                kin.right.entry(x).or_default().push((w, n));
                n2 += n * n;
            }
            kin.norm_right.insert(w, n2.sqrt());
        }
        // The left company: the pairs a word closes, `pair:u w`, said under it.
        for (w, t) in self.tables_under(super::SAYS).into_iter().filter(|(w, _)| self.is_word(*w)) {
            let spelling = self.spelling_of(w);
            let mut n2 = 0.0;
            for (x, n) in entries(t) {
                let pair = self.spelling_of(x);
                let Some(u) = pair.strip_prefix("pair:").and_then(|p| p.strip_suffix(&format!(" {spelling}"))) else { continue };
                let Some(u) = self.word(u) else { continue };
                kin.left.entry(u).or_default().push((w, n));
                n2 += n * n;
            }
            if n2 > 0.0 {
                kin.norm_left.insert(w, n2.sqrt());
            }
        }
        kin.built = true;
    }

    /// The kin of `word`, closest first, with kinship in (0, 1]: the mean of
    /// its left and right cosines with each other word. Empty for a word
    /// never told.
    pub fn kin(&self, word: &str) -> Rc<Vec<(String, f64)>> {
        let Some(w) = self.word(word) else { return Rc::new(Vec::new()) };
        self.build_kin();
        let found = {
            let kin = self.kin.borrow();
            match kin.found.get(&w) {
                Some(f) => f.clone(),
                None => {
                    let mut dots: HashMap<Word, (f64, f64)> = HashMap::new();
                    for (side, posting, norms) in [(0usize, &kin.right, &kin.norm_right), (1, &kin.left, &kin.norm_left)] {
                        let table = match side {
                            0 => self.table(word, "next"),
                            _ => self.table(word, super::SAYS),
                        };
                        let Some(table) = table else { continue };
                        let Some(&nw) = norms.get(&w) else { continue };
                        for (x, n) in entries(table) {
                            let key = match side {
                                0 => Some(x),
                                _ => self.spelling_of(x).strip_prefix("pair:").and_then(|p| p.strip_suffix(&format!(" {word}"))).and_then(|u| self.word(u)),
                            };
                            let Some(lists) = key.and_then(|k| posting.get(&k)) else { continue };
                            for (v, m) in lists {
                                if *v == w {
                                    continue;
                                }
                                let nv = norms.get(v).copied().unwrap_or(1.0);
                                let d = dots.entry(*v).or_default();
                                match side {
                                    0 => d.0 += n * m / (nw * nv),
                                    _ => d.1 += n * m / (nw * nv),
                                }
                            }
                        }
                    }
                    let mut v: Vec<(Word, f64)> = dots.into_iter().map(|(v, (r, l))| (v, (r + l) / 2.0)).filter(|x| x.1 > 0.0).collect();
                    v.sort_by(|a, b| b.1.total_cmp(&a.1));
                    Rc::new(v)
                }
            }
        };
        self.kin.borrow_mut().found.insert(w, found.clone());
        Rc::new(found.iter().map(|(v, s)| (self.spelling_of(*v), *s)).collect())
    }

    /// The kin reading of `kind` for `word` over `values`: the kin's
    /// distributions (each smoothed toward the prior with weight one) mixed
    /// by kinship — `Σ_v s_v · (n_v(x) + p(x)) / (N_v + 1)`, the counts
    /// passed sparsely, the prior's share summed once. `None` for a word
    /// with no kin. Kept per word when `values` is the kind's whole list.
    pub fn kin_base(&self, word: &str, kind: &str, values: &[String]) -> Option<Vec<f64>> {
        let (w, k) = (self.word(word)?, self.word(kind)?);
        let index = self.index_of(kind, values);
        if index.is_some() {
            if let Some(b) = self.kin.borrow().base.get(&(w, k)) {
                return Some(b.as_ref().clone());
            }
        }
        let kin = self.kin(word);
        if kin.is_empty() {
            return None;
        }
        let prior = self.prior(kind, values);
        let at = |x: Word| -> Option<usize> {
            match &index {
                Some(i) => i.get(&x).copied(),
                None => values.iter().position(|v| self.word(v) == Some(x)),
            }
        };
        let (mut base, mut total, mut of_prior) = (vec![0.0; values.len()], 0.0, 0.0);
        for (v, s) in kin.iter() {
            let Some(t) = self.table(v, kind) else { continue };
            let share = s / (t.sum() + 1.0);
            of_prior += share;
            for (x, n) in entries(t) {
                if let Some(i) = at(x) {
                    base[i] += share * n;
                }
            }
            total += s;
        }
        if total == 0.0 {
            return None;
        }
        let base: Vec<f64> = base.iter().zip(&prior).map(|(b, p)| (b + of_prior * p) / total).collect();
        if index.is_some() {
            self.kin.borrow_mut().base.insert((w, k), Rc::new(base.clone()));
        }
        Some(base)
    }
}

fn entries(t: &Table) -> Vec<(Word, f64)> {
    t.keys().into_iter().filter_map(|w| t.get(&w).map(|n| (w, *n))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prose;

    #[test]
    fn a_word_is_read_through_its_kin_where_it_was_never_heard() {
        let mut c = Tally::new(&["has-word", "within", "pair", "in-word", "in-pair"]);
        let text = ["the cat sat on the mat.", "the cat ate the fish.", "the dog sat on the rug.", "the dog ate the bone.", "a dog barked."];
        c.learn(&prose::paragraph("t/p1", &text.map(String::from)));
        let kin = c.kin("cat");
        assert_eq!(kin.first().map(|x| x.0.as_str()), Some("dog"), "{kin:?}");
        let values = c.values("next");
        let at = |v: &str, d: &[f64]| d[values.iter().position(|x| x == v).unwrap()];
        // `cat` was never heard before `barked`; its kin `dog` was.
        assert_eq!(c.count(Some("cat"), "next", "barked"), 0.0);
        let base = c.kin_base("cat", "next", &values).unwrap();
        let prior = c.prior("next", &values);
        assert!(at("barked", &base) > at("barked", &prior), "{base:?}");
    }
}
