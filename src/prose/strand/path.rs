//! A unit read through its whole path (docs/148). Read down the tree, a
//! unit's reading is its own share mixed with its parent's reading, and the
//! parent's weight is counted for the parent as a whole. A parent is two
//! children, so the weight that serves it carries the sibling's words into
//! each child's reading. docs/148 measured that this costs the tree its
//! place even when the weights are counted on the very words being read.
//!
//! A path reading counts the weights for the unit instead. The unit, every
//! scope above it to the whole, and the prior are one mixture,
//! `p(w) = Σ_j π_j · c_j(w)/n_j + π_prior · prior(w)`. The weights `π` are
//! counted on the unit's own tokens, each left out of every count it is in
//! (or on witness tokens the strand never saw). That is deleted
//! interpolation (Jelinek & Mercer 1980) with one context per unit, and the
//! Laplace rule over the soft counts, as for a single weight. A scope that
//! says nothing the unit's words need gets almost no weight, so the tree can
//! read at least as well as the unit straight under the whole.

use super::{ScopeSet, Strand};
use crate::relate::mixture_weights;

/// A unit's path, from the unit up to the whole, and its weights (the last for the prior).
#[derive(Debug, Clone)]
pub struct Path {
    pub chain: Vec<usize>,
    pub weights: Vec<f64>,
}

impl ScopeSet {
    /// The path of `unit` with weights counted on `rows`: `(word, prior)`
    /// for each token read. `left_out`: the rows are the unit's own tokens,
    /// each taken out of every count on the path; otherwise they are
    /// witnesses the counts never held.
    pub fn path(&self, unit: usize, rows: &[(Option<u32>, f64)], left_out: bool) -> Path {
        let chain: Vec<usize> = std::iter::successors(Some(unit), |s| self.parent[*s]).collect();
        let less = if left_out { 1.0 } else { 0.0 };
        // Each row's share in each scope of the path, then its prior.
        let shares: Vec<Vec<f64>> = rows
            .iter()
            .map(|(id, p)| {
                let mut v: Vec<f64> = chain
                    .iter()
                    .map(|s| {
                        let (a, b) = self.spans[*s];
                        let c = id.and_then(|i| self.counts[*s].get(&i)).copied().unwrap_or(0) as f64;
                        let n = (b - a) as f64 - less;
                        if n > 0.0 { (c - less).max(0.0) / n } else { 0.0 }
                    })
                    .collect();
                v.push(*p);
                v
            })
            .collect();
        let rows: Vec<(f64, Vec<f64>)> = shares.into_iter().map(|v| (1.0, v)).collect();
        let w = mixture_weights(&rows, chain.len() + 1);
        Path { chain, weights: w }
    }

    /// The reading of word `id` along `path`; `prior` its share before the text.
    pub fn read_path(&self, path: &Path, id: Option<u32>, prior: f64) -> f64 {
        let own: f64 = path
            .chain
            .iter()
            .zip(&path.weights)
            .map(|(s, w)| {
                let (a, b) = self.spans[*s];
                w * id.and_then(|i| self.counts[*s].get(&i)).copied().unwrap_or(0) as f64 / (b - a).max(1) as f64
            })
            .sum();
        own + path.weights[path.chain.len()] * prior
    }
}

impl Strand {
    /// The units the strand supports, each with its path read with weights
    /// counted on its own tokens left out: `(scopes, units, paths)`.
    pub fn unit_paths(&self) -> (ScopeSet, Vec<usize>, Vec<Path>) {
        let scopes = self.scopes();
        let units = self.cut.units_at(self.informative_units());
        let paths = units.iter().map(|u| {
            let sp = &self.cut.spans[*u];
            let rows: Vec<(Option<u32>, f64)> = self.tokens[sp.start..sp.end].iter().map(|t| (self.index.get(t).copied(), self.prior.p(t))).collect();
            scopes.path(*u, &rows, true)
        });
        let paths = paths.collect();
        (scopes, units, paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_gives_no_weight_to_a_parent_that_carries_the_sibling() {
        // Two halves with words of their own; the whole is both. Read in the
        // first half, its own counts carry the weight and the whole (half of
        // it the sibling's words) carries little.
        let ids: Vec<u32> = [1, 2, 1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4, 3, 4].to_vec();
        let prior = |_: u32| 1.0 / 64.0;
        let s = ScopeSet::fit(&ids, &[(0, 16, None), (0, 8, Some(0)), (8, 16, Some(0))], &prior);
        let rows: Vec<(Option<u32>, f64)> = ids[0..8].iter().map(|i| (Some(*i), prior(*i))).collect();
        let p = s.path(1, &rows, true);
        assert_eq!(p.chain, [1, 0]);
        assert!(p.weights[0] > 0.7 && p.weights[1] < 0.2, "{:?}", p.weights);
        assert!((p.weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        // A distribution: the vocabulary plus the unseen share is one.
        let said: f64 = (1..=4).map(|i| s.read_path(&p, Some(i), prior(i))).sum();
        assert!((said + s.read_path(&p, None, 1.0) * (1.0 - 4.0 / 64.0) - 1.0).abs() < 1e-9);
    }
}
