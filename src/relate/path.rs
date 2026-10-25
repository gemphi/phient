//! The levels read as a path (docs/149). `read_levels` reads its levels as
//! a chain: each level moves what the levels above arrived at, with a
//! weight counted for that level's own rows. docs/148 showed, on the
//! strand, that this form costs a nested reading its place. A level's
//! weight serves the level as a whole, and a scope carries the words of
//! everything else under it into every reading below it.
//!
//! The path form reads the base and every posted condition as one mixture:
//!
//! `p(v) = π_base · base(v) + Σ_j π_j · c_j(v)/n_j`
//!
//! The weights are counted for this context, on the rows of its innermost
//! posted condition (the most specific thing the context has said before).
//! Each row is left out of every count it could be in. A condition that
//! the innermost rows do not need — a chapter whose other paragraphs say
//! other things — keeps only the Laplace rule's floor. A post is still
//! earned against the prior (D2).

use super::counts::Tally;
use super::table::Table;
use super::weights::mixture_weights;

impl Tally {
    /// `read_levels_from` in the path form: scores (natural log) in the
    /// order of `values`, and the conditions that held a post.
    pub(super) fn read_path(&self, levels: &[Vec<String>], kind: &str, values: &[String], tau: f64, base: Vec<f64>) -> (Vec<f64>, Vec<String>) {
        let prior = self.prior(kind, values);
        let mut tables: Vec<&Table> = Vec::new();
        let mut posts = Vec::new();
        for c in levels.iter().flatten() {
            let (Some(m), Some(own)) = (self.table(c, kind), self.dist(c, kind, values, &prior)) else { continue };
            let kl: f64 = own.iter().zip(&prior).map(|(x, y)| x * (x / y).ln()).sum();
            if kl >= tau {
                tables.push(m);
                posts.push(c.clone());
            }
        }
        let Some(inner) = tables.last() else {
            return (base.iter().map(|p| p.max(1e-300).ln()).collect(), posts);
        };
        let index = self.index_of(kind, values);
        let at = |w| match &index {
            Some(ix) => ix.get(&w).copied(),
            None => values.iter().position(|v| self.word(v) == Some(w)),
        };
        // The innermost rows, grouped by value: each row's share in the base and in every table, left out.
        let sums: Vec<f64> = tables.iter().map(|m| m.sum()).collect();
        let rows: Vec<(f64, Vec<f64>)> = inner
            .keys()
            .into_iter()
            .filter_map(|w| {
                let i = at(w)?;
                let c = *inner.get(&w)?;
                let mut v = vec![base[i]];
                v.extend(tables.iter().zip(&sums).map(|(m, n)| {
                    let k = m.get(&w).copied().unwrap_or(0.0);
                    if *n > 1.0 { (k - 1.0).max(0.0) / (n - 1.0) } else { 0.0 }
                }));
                Some((c, v))
            })
            .collect();
        let pi = mixture_weights(&rows, tables.len() + 1);
        let mut d: Vec<f64> = base.iter().map(|b| pi[0] * b).collect();
        for ((m, n), p) in tables.iter().zip(&sums).zip(&pi[1..]) {
            for w in m.keys() {
                if let (Some(i), Some(k)) = (at(w), m.get(&w)) {
                    d[i] += p * k / n.max(1.0);
                }
            }
        }
        (d.iter().map(|p| p.max(1e-300).ln()).collect(), posts)
    }
}

#[cfg(test)]
mod tests {
    use super::super::weights::Weighting;
    use super::*;
    use phiecs::quest::Row;

    fn row(a: &str, k: &str, o: &str) -> Row {
        Row::new(a, k, o)
    }

    #[test]
    fn a_path_leans_on_the_condition_its_own_rows_repeat() {
        // Under scope `s`, functions call `filter` or `sum`; under word `w`, always `count`.
        let mut c = Tally::new(&["within", "has-word"]);
        let mut rows = Vec::new();
        for i in 0..6 {
            let f = format!("f{i}");
            rows.push(row(&f, "within", "scope:s"));
            rows.push(row(&f, "calls", if i % 2 == 0 { "filter" } else { "sum" }));
        }
        for i in 0..4 {
            let g = format!("g{i}");
            rows.push(row(&g, "within", "scope:s"));
            rows.push(row(&g, "has-word", "w"));
            rows.push(row(&g, "calls", "count"));
        }
        c.learn(&rows);
        c.weights = Weighting::Path;
        let values = c.values("calls");
        let at = |v: &str, s: &[f64]| s[values.iter().position(|x| x == v).unwrap()].exp();
        let (s, posts) = c.read_levels(&[vec!["scope:s".into()], vec!["w".into()]], "calls", &values, 0.0);
        assert_eq!(posts.len(), 2);
        assert!(at("count", &s) > 0.6, "the word's rows always say `count`: {}", at("count", &s));
        assert!((values.iter().map(|v| at(v, &s)).sum::<f64>() - 1.0).abs() < 1e-9, "a distribution");
    }
}
