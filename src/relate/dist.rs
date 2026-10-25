//! The prior and a condition's distribution over a kind's values
//! (docs/124, docs/131, docs/133): the smoothing, and the index that makes
//! a wide table's distribution a pass over its entries rather than over
//! every value.

use super::counts::Tally;
use super::table::Table;
use phiecs::word::Word;
use std::collections::HashMap;

impl Tally {
    /// Each value's place in the kind's sorted values, when `values` is
    /// that whole list (a caller may pass a subset, which gets no index).
    pub(super) fn index_of(&self, kind: &str, values: &[String]) -> Option<std::rc::Rc<HashMap<Word, usize>>> {
        let k = self.word(kind)?;
        let vp = self.values_and_prior(kind);
        if vp.0.len() != values.len() || vp.0.first() != values.first() || vp.0.last() != values.last() {
            return None;
        }
        if let Some(i) = self.index.borrow().get(&k) {
            return Some(i.clone());
        }
        let i = std::rc::Rc::new(values.iter().enumerate().filter_map(|(i, v)| self.word(v).map(|w| (w, i))).collect::<HashMap<_, _>>());
        self.index.borrow_mut().insert(k, i.clone());
        Some(i)
    }

    pub fn prior(&self, kind: &str, values: &[String]) -> Vec<f64> {
        let m = self.word(kind).and_then(|k| self.prior.get(&k));
        let total: f64 = m.map_or(0.0, Table::sum);
        let at = |v: &String| self.word(v).and_then(|w| m.and_then(|m| m.get(&w))).copied().unwrap_or(0.0);
        values.iter().map(|v| (at(v) + 0.01) / (total + 0.01 * values.len() as f64)).collect()
    }

    /// The condition's counts under `kind` over `values`, smoothed toward
    /// `base` with weight one: one sighting is never certainty.
    pub fn dist(&self, c: &str, kind: &str, values: &[String], base: &[f64]) -> Option<Vec<f64>> {
        self.dist_weighted(c, kind, values, base, |_| 1.0)
    }

    /// As `dist`, smoothed toward `base` with the weight of how many
    /// different values the condition has said (docs/131; Witten & Bell):
    /// a scope that has said a hundred things in five hundred rows keeps its
    /// base at a hundred, so what it never said is read as rare, not as
    /// impossible. For the levels, where a condition may be a whole
    /// chapter; a word's own reading keeps weight one.
    pub fn dist_breadth(&self, c: &str, kind: &str, values: &[String], base: &[f64]) -> Option<Vec<f64>> {
        self.dist_weighted(c, kind, values, base, |m| m.len().max(1) as f64)
    }

    pub(super) fn dist_weighted(&self, c: &str, kind: &str, values: &[String], base: &[f64], weight: impl Fn(&Table) -> f64) -> Option<Vec<f64>> {
        let m = self.table(c, kind)?;
        let total: f64 = m.sum();
        let w = weight(m);
        if let Some(index) = self.index_of(kind, values) {
            // The base everywhere, then the table's own entries by index.
            let mut d: Vec<f64> = base.iter().map(|b| w * b / (total + w)).collect();
            for word in m.keys() {
                if let (Some(&i), Some(n)) = (index.get(&word), m.get(&word)) {
                    d[i] += n / (total + w);
                }
            }
            return Some(d);
        }
        Some(values.iter().zip(base).map(|(v, b)| (self.word(v).and_then(|x| m.get(&x)).copied().unwrap_or(0.0) + w * b) / (total + w)).collect())
    }

}
