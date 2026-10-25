//! How much a level trusts its own counts (docs/145–146). A condition's
//! reading is its own share of a value mixed with the base the levels
//! above arrived at: `d(v) = λ · c(v)/n + (1 − λ) · base(v)`. There are two
//! rules for the weight:
//!
//! - **Breadth** (docs/131; Witten & Bell): `λ = n / (n + T)`, `T` the
//!   values the condition has said. This is the code a text is said once
//!   with, in order.
//! - **Counted**: `λ` counted from the condition's own rows, each left out
//!   in turn and read against its own share without it and against the base
//!   (deleted interpolation, Jelinek & Mercer 1980). It is the
//!   Laplace-ruled fixed point of the share the rows' own counts explain.
//!   A condition whose values repeat leans on itself; one that never
//!   repeats a value leans on the base.
//!
//! docs/145 measured the two on a text's own held-out words. Breadth made
//! every scope finer than the whole read worse; counted weights made every
//! scope read better.
//!
//! A third rule changes the form, not the weight: **Path** (`path.rs`,
//! docs/149, the default) reads the base and every posted condition as one
//! mixture, with weights counted for the context being read.

use super::counts::Tally;

/// Which rule the levels read with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Weighting {
    Breadth,
    Counted,
    /// The levels as one mixture, weights counted for the context (`path.rs`, docs/149).
    #[default]
    Path,
}

/// The weight under which rows left out are best predicted. Each entry
/// is `(count, own, above)`: how many rows share it, a row's share in the
/// condition without that row, and its reading in the base; `n` is the
/// rows in all.
pub fn left_out_weight(entries: &[(f64, f64, f64)], n: f64) -> f64 {
    let mut w = 0.5;
    for _ in 0..200 {
        let r: f64 = entries
            .iter()
            .map(|(c, own, above)| {
                let x = w * own;
                if x > 0.0 { c * x / (x + (1.0 - w) * above) } else { 0.0 }
            })
            .sum();
        let next = (r + 1.0) / (n + 2.0);
        if (next - w).abs() < 1e-7 {
            return next;
        }
        w = next;
    }
    w
}

/// The weights of a mixture under which `rows` are best predicted: each
/// row is `(count, shares)`, its share in each of the `k` components (left
/// out of any count it is in). The Laplace-ruled fixed point of
/// the soft counts each component explains (docs/148): a component no row
/// needs keeps only the rule's floor.
pub fn mixture_weights(rows: &[(f64, Vec<f64>)], k: usize) -> Vec<f64> {
    let n: f64 = rows.iter().map(|r| r.0).sum();
    let mut w = vec![1.0 / k as f64; k];
    for _ in 0..500 {
        let mut r = vec![0.0; k];
        for (c, v) in rows {
            let z: f64 = v.iter().zip(&w).map(|(x, y)| x * y).sum();
            if z > 0.0 {
                r.iter_mut().zip(v.iter().zip(&w)).for_each(|(r, (x, y))| *r += c * x * y / z);
            }
        }
        let next: Vec<f64> = r.iter().map(|r| (r + 1.0) / (n + k as f64)).collect();
        let moved = next.iter().zip(&w).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        w = next;
        if moved < 1e-8 {
            break;
        }
    }
    w
}

impl Tally {
    /// As `dist`, smoothed toward `base` with the weight the condition's
    /// own rows count, each left out in turn.
    pub fn dist_counted(&self, c: &str, kind: &str, values: &[String], base: &[f64]) -> Option<Vec<f64>> {
        let m = self.table(c, kind)?;
        let n = m.sum();
        let index = self.index_of(kind, values);
        let entries: Vec<(f64, f64, f64)> = m
            .keys()
            .into_iter()
            .filter_map(|w| {
                let i = match &index {
                    Some(ix) => *ix.get(&w)?,
                    None => values.iter().position(|v| self.word(v) == Some(w))?,
                };
                let k = *m.get(&w)?;
                Some((k, if n > 1.0 { (k - 1.0).max(0.0) / (n - 1.0) } else { 0.0 }, base[i]))
            })
            .collect();
        let lambda = left_out_weight(&entries, n);
        // The same mixture in the form the breadth rule is written in: base weight `n (1 − λ) / λ`.
        self.dist_weighted(c, kind, values, base, |_| n * (1.0 - lambda) / lambda)
    }

    /// A level's reading of condition `c` under the rule the counts read with.
    pub fn dist_level(&self, c: &str, kind: &str, values: &[String], base: &[f64]) -> Option<Vec<f64>> {
        match self.weights {
            Weighting::Breadth => self.dist_breadth(c, kind, values, base),
            Weighting::Counted | Weighting::Path => self.dist_counted(c, kind, values, base),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phiecs::quest::Row;

    #[test]
    fn a_condition_that_repeats_itself_is_trusted_and_one_that_never_does_is_not() {
        let mut c = Tally::new(&["has-word"]);
        let mut rows = Vec::new();
        for i in 0..12 {
            // `w` is always followed by `x`; `u` by a different word each time.
            rows.push(Row::new(&format!("a{i}"), "has-word", "w"));
            rows.push(Row::new(&format!("a{i}"), "next", "x"));
            rows.push(Row::new(&format!("b{i}"), "has-word", "u"));
            rows.push(Row::new(&format!("b{i}"), "next", &format!("y{i}")));
        }
        c.learn(&rows);
        let values = c.values("next");
        let base = vec![1.0 / values.len() as f64; values.len()];
        let at = |d: &[f64], v: &str| d[values.iter().position(|x| x == v).unwrap()];
        let (wb, ct) = (c.dist_breadth("w", "next", &values, &base).unwrap(), c.dist_counted("w", "next", &values, &base).unwrap());
        assert!(at(&ct, "x") > at(&wb, "x"), "a value said every time is trusted more than breadth trusts it");
        let (wb, ct) = (c.dist_breadth("u", "next", &values, &base).unwrap(), c.dist_counted("u", "next", &values, &base).unwrap());
        assert!(at(&ct, "y0") < at(&wb, "y0"), "a condition that never repeats leans on the base");
        assert!((ct.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        assert!((left_out_weight(&[(4.0, 0.0, 0.5)], 4.0) - 1.0 / 6.0).abs() < 1e-9, "no evidence: the Laplace rule's floor");
    }
}
