//! ConditionSet at levels (docs/126). A word is read under conditions, and
//! the conditions come in a hierarchy: a token in a function in a file in a
//! crate; an order in a scene in a session; a word in a sentence in a
//! paragraph in a chapter. The hierarchy is a path — `crate/file/fn`, as
//! the quest's answers already lie on disk — and each component of the
//! path is a condition word the world knows. A Context Language Model keeps
//! this as one flat text it edits; here each level is a word with counts.
//!
//! The reading is D4 nested: the outermost level's conditions move the
//! prior; what they arrive at is the base the next level's counts are
//! smoothed toward and moved from; the innermost moves last. A condition
//! earns its post (D2, τ nats) against the prior — does it say anything at
//! all? — and moves the reading from where the levels above left it. A
//! level whose conditions were never told moves nothing, so a path never
//! seen reads as its nearest told ancestor. A level's counts are smoothed
//! toward the base by their breadth (`Tally::dist_breadth`), or by the
//! weight their own rows count when each is left out (`Tally::weights`,
//! docs/146): a chapter that has said a thousand things does not make what
//! it never said impossible, only rare.

use phiecs::Row;
use super::counts::Tally;

/// `scope:` prefixes a path component so a directory named `above` and the
/// word `above` are two conditions.
impl Tally {
    /// The reading of `kind` under conditions at levels, outermost first:
    /// scores (natural log) in the order of `values`, and the conditions
    /// that held a post, by level.
    pub fn read_levels(&self, levels: &[Vec<String>], kind: &str, values: &[String], tau: f64) -> (Vec<f64>, Vec<String>) {
        let prior = self.prior(kind, values);
        self.read_levels_from(levels, kind, values, tau, prior)
    }

    /// As `read_levels`, with the outermost level moving from `base` rather
    /// than from the prior (docs/137: the operator's reading as the base the
    /// counts move from). A post is still earned against the prior.
    pub fn read_levels_from(&self, levels: &[Vec<String>], kind: &str, values: &[String], tau: f64, base: Vec<f64>) -> (Vec<f64>, Vec<String>) {
        if self.weights == super::weights::Weighting::Path {
            return self.read_path(levels, kind, values, tau, base);
        }
        let prior = self.prior(kind, values);
        let mut base = base;
        let mut posts = Vec::new();
        for conds in levels {
            let mut s: Vec<f64> = base.iter().map(|p| p.ln()).collect();
            let mut moved = false;
            for c in conds {
                let Some(d) = self.dist_level(c, kind, values, &base) else { continue };
                // A post is earned against the prior (does the condition say
                // anything?); the movement is from what the levels above
                // arrived at.
                let Some(own) = self.dist(c, kind, values, &prior) else { continue };
                let kl: f64 = own.iter().zip(&prior).map(|(x, y)| x * (x / y).ln()).sum();
                if kl >= tau {
                    s.iter_mut().zip(&d).zip(&base).for_each(|((x, d), p)| *x += (d / p).ln());
                    posts.push(c.clone());
                    moved = true;
                }
            }
            if moved {
                let top = s.iter().cloned().fold(f64::MIN, f64::max);
                let z: f64 = s.iter().map(|x| (x - top).exp()).sum();
                base = s.iter().map(|x| (x - top).exp() / z).collect();
            }
        }
        (base.iter().map(|p| p.max(1e-300).ln()).collect(), posts)
    }
}

// `scopes`, `nest` and `SCOPE` moved down to `phiecs::scope` (they build
// rows, and the builders live beside the one intermediate); re-exported
// from this module so the address `relate::scopes` still answers.
pub use phiecs::scope::{nest, scopes, SCOPE};

#[cfg(test)]
mod tests {
    use super::*;

    fn row(a: &str, k: &str, o: &str) -> Row {
        Row::new(a, k, o)
    }

    #[test]
    fn an_inner_level_moves_what_the_outer_arrived_at() {
        let mut c = Tally::new(&["within", "has-word"]);
        // Under scope s, functions call `filter`; under word w, `count`.
        let mut rows = Vec::new();
        for i in 0..4 {
            let f = format!("f{i}");
            rows.push(row(&f, "within", "scope:s"));
            rows.push(row(&f, "calls", "filter"));
            rows.push(row(&f, "calls", if i < 2 { "iter" } else { "sum" }));
        }
        rows.push(row("g", "has-word", "w"));
        rows.push(row("g", "calls", "count"));
        rows.push(row("g", "calls", "iter"));
        c.learn(&rows);
        let values = c.values("calls");
        let at = |v: &str, s: &[f64]| s[values.iter().position(|x| x == v).unwrap()];
        let (outer, _) = c.read_levels(&[vec!["scope:s".into()]], "calls", &values, 0.0);
        assert!(at("filter", &outer) > at("count", &outer));
        let (both, posts) = c.read_levels(&[vec!["scope:s".into()], vec!["w".into()]], "calls", &values, 0.0);
        assert_eq!(posts.len(), 2);
        // The word moves the scope's reading toward what it was told: count
        // rises above where the scope alone left it, filter stays ahead of sum.
        assert!(at("count", &both) > at("count", &outer));
        assert!(at("filter", &both) > at("sum", &both));
    }
}
