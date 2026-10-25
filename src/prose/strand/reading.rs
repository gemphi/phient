//! Reading a word in a scope with weights the text counts itself. A
//! scope's reading is its own share of the word mixed with its parent's
//! reading: `p_s(w) = λ_s · c_s(w)/n_s + (1 − λ_s) · p_parent(w)`. The weight
//! `λ_s` is not set by hand. It is counted from the scope's tokens, each
//! left out in turn (deleted interpolation, Jelinek & Mercer 1980): the
//! weight under which the scope's tokens best predict one another, scored
//! against the parent's reading with the same token left out. The scopes are
//! fitted from the root down. The weight is the Laplace rule over the
//! tokens' soft counts, the rule the prior is counted with, so a scope with
//! no evidence of its own leans on its parent.
//!
//! Witten–Bell, the code the strand divides with, gives a scope the weight
//! `n/(n + T)` whatever its words are. That is right for saying a text once
//! in order, and too trusting of a small scope when a word is read that
//! the scope did not say (docs/145).

use super::Strand;
use crate::relate::left_out_weight;
use std::collections::HashMap;

fn count(ids: &[u32]) -> HashMap<u32, u32> {
    ids.iter().fold(HashMap::new(), |mut m, id| {
        *m.entry(*id).or_default() += 1;
        m
    })
}

/// A tree of scopes over a run of tokens, each with its counted weight.
#[derive(Debug, Clone)]
pub struct ScopeSet {
    pub spans: Vec<(usize, usize)>,
    pub parent: Vec<Option<usize>>,
    pub counts: Vec<HashMap<u32, u32>>,
    /// How much of a scope's reading comes from its own counts.
    pub weight: Vec<f64>,
}

impl ScopeSet {
    /// `spans[i] = (start, end, parent)` over `ids`, every parent before
    /// its children; the root is read against `prior`.
    pub fn fit(ids: &[u32], spans: &[(usize, usize, Option<usize>)], prior: &dyn Fn(u32) -> f64) -> Self {
        let mut out = ScopeSet { spans: Vec::new(), parent: Vec::new(), counts: Vec::new(), weight: Vec::new() };
        // Each scope's tokens, read in it with themselves left out.
        let mut left_out: Vec<Vec<f64>> = Vec::with_capacity(spans.len());
        for &(a, b, p) in spans {
            let c = count(&ids[a..b]);
            let above: Vec<f64> = match p {
                Some(q) => left_out[q][a - out.spans[q].0..b - out.spans[q].0].to_vec(),
                None => ids[a..b].iter().map(|id| prior(*id)).collect(),
            };
            let n = b - a;
            let own: Vec<f64> = ids[a..b].iter().map(|id| if n > 1 { (c[id] - 1) as f64 / (n - 1) as f64 } else { 0.0 }).collect();
            let rows: Vec<(f64, f64, f64)> = own.iter().zip(&above).map(|(o, u)| (1.0, *o, *u)).collect();
            let w = left_out_weight(&rows, n as f64);
            left_out.push(own.iter().zip(&above).map(|(o, u)| w * o + (1.0 - w) * u).collect());
            out.spans.push((a, b));
            out.parent.push(p);
            out.counts.push(c);
            out.weight.push(w);
        }
        out
    }

    /// As `fit`, with each scope's weight counted on `witness` tokens the
    /// spans were not chosen by (cross-fitting, docs/148). A witness is
    /// `(position, word, prior)`: where it falls among `ids`, its word if
    /// `ids` has one, and its share before the text. Each scope's tokens
    /// are its counts, and the witnesses inside it are what those counts
    /// must predict, so a span chosen because its own tokens repeat earns
    /// its weight only as far as tokens it never saw repeat them too.
    pub fn fit_witnessed(ids: &[u32], spans: &[(usize, usize, Option<usize>)], witness: &[(usize, Option<u32>, f64)]) -> Self {
        let mut w: Vec<&(usize, Option<u32>, f64)> = witness.iter().collect();
        w.sort_by_key(|x| x.0);
        let mut out = ScopeSet { spans: Vec::new(), parent: Vec::new(), counts: Vec::new(), weight: Vec::new() };
        // Each scope's witnesses (a range of `w`) and their reading in it.
        let mut read_in: Vec<(usize, Vec<f64>)> = Vec::with_capacity(spans.len());
        for &(a, b, p) in spans {
            let c = count(&ids[a..b]);
            let (lo, hi) = (w.partition_point(|x| x.0 < a), w.partition_point(|x| x.0 < b));
            let above: Vec<f64> = match p {
                Some(q) => read_in[q].1[lo - read_in[q].0..hi - read_in[q].0].to_vec(),
                None => w[lo..hi].iter().map(|x| x.2).collect(),
            };
            let n = (b - a).max(1) as f64;
            let own: Vec<f64> = w[lo..hi].iter().map(|x| x.1.and_then(|id| c.get(&id)).copied().unwrap_or(0) as f64 / n).collect();
            let rows: Vec<(f64, f64, f64)> = own.iter().zip(&above).map(|(o, u)| (1.0, *o, *u)).collect();
            let weight = left_out_weight(&rows, rows.len() as f64);
            read_in.push((lo, own.iter().zip(&above).map(|(o, u)| weight * o + (1.0 - weight) * u).collect()));
            out.spans.push((a, b));
            out.parent.push(p);
            out.counts.push(c);
            out.weight.push(weight);
        }
        out
    }

    /// The reading of word `id` (`None`: a word the text never said) in
    /// scope `s`, `prior` its share before the text.
    pub fn read(&self, s: usize, id: Option<u32>, prior: f64) -> f64 {
        let above = match self.parent[s] {
            Some(p) => self.read(p, id, prior),
            None => prior,
        };
        let (a, b) = self.spans[s];
        let c = id.and_then(|i| self.counts[s].get(&i)).copied().unwrap_or(0) as f64;
        self.weight[s] * c / (b - a).max(1) as f64 + (1.0 - self.weight[s]) * above
    }
}

impl Strand {
    /// The whole division tree as scopes with counted weights.
    pub fn scopes(&self) -> ScopeSet {
        let ids: Vec<u32> = self.tokens.iter().map(|t| self.index[t]).collect();
        let spans: Vec<(usize, usize, Option<usize>)> = self.cut.spans.iter().map(|s| (s.start, s.end, s.parent)).collect();
        ScopeSet::fit(&ids, &spans, &|id| self.p(id))
    }

    /// The division tree with each weight counted on `witness` tokens the
    /// strand was not divided by: `(position among the strand's tokens, word)`.
    pub fn scopes_witnessed(&self, witness: &[(usize, String)]) -> ScopeSet {
        let ids: Vec<u32> = self.tokens.iter().map(|t| self.index[t]).collect();
        let spans: Vec<(usize, usize, Option<usize>)> = self.cut.spans.iter().map(|s| (s.start, s.end, s.parent)).collect();
        let w: Vec<(usize, Option<u32>, f64)> = witness.iter().map(|(p, word)| (*p, self.index.get(word).copied(), self.prior.p(word))).collect();
        ScopeSet::fit_witnessed(&ids, &spans, &w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_witnessed_weight_is_earned_on_tokens_the_span_never_saw() {
        // The first half repeats `1 2`; witnesses there say `1 2` again, so it is trusted.
        // The second half repeats `3 4` too, but its witnesses say other words: not trusted.
        let ids: Vec<u32> = [1, 2, 1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4, 3, 4].to_vec();
        let spans = [(0, 16, None), (0, 8, Some(0)), (8, 16, Some(0))];
        let witness: Vec<(usize, Option<u32>, f64)> = (0..8).map(|i| (i, Some(1 + (i % 2) as u32), 1.0 / 64.0)).chain((8..16).map(|i| (i, Some(5 + i as u32), 1.0 / 64.0))).collect();
        let s = ScopeSet::fit_witnessed(&ids, &spans, &witness);
        assert!(s.weight[1] > 0.6, "{:?}", s.weight);
        assert!(s.weight[2] < 0.2, "{:?}", s.weight);
        let loo = ScopeSet::fit(&ids, &spans, &|_| 1.0 / 64.0);
        assert!(loo.weight[2] > 0.6, "left out, the second half trusts itself: {:?}", loo.weight);
    }

    #[test]
    fn a_scope_leans_on_its_own_words_as_far_as_they_repeat() {
        // Two halves: the first repeats its own words, the second never repeats one.
        let ids: Vec<u32> = [1, 2, 1, 2, 1, 2, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10].to_vec();
        let prior = |_: u32| 1.0 / 64.0;
        let s = ScopeSet::fit(&ids, &[(0, 16, None), (0, 8, Some(0)), (8, 16, Some(0))], &prior);
        assert!(s.weight[1] > 0.8, "{:?}", s.weight);
        assert!(s.weight[2] < 0.15, "{:?}", s.weight);
        // A reading is a distribution: what the vocabulary holds plus what runs down to the unseen is one.
        for scope in 0..3 {
            let said: f64 = (1..=10).map(|id| s.read(scope, Some(id), prior(id))).sum();
            let unseen = s.read(scope, None, 1.0);
            assert!((said + unseen * (1.0 - 10.0 / 64.0) - 1.0).abs() < 1e-9, "scope {scope}: {said} + {unseen}");
        }
    }
}
