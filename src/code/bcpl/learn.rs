//! `bcpl::learn` (docs/170): the grammar learned from examples by DEPTH,
//! not by flat leaves.
//!
//! Meaning is not in the last two tokens — it is in the nesting. A BCPL
//! token sits inside a building: section → function → VALOF → block → IF →
//! expression → operand, sixteen levels and more (Crick 2004, "A GCC Front
//! End for BCPL", App. C: the BNF is a deep tree). So the learner tracks the
//! **stack of constructs it is currently inside** and predicts the next
//! token's role from that path, deepest levels first. `$(`/`$)` and `(`/`)`
//! open and close depth; VALOF, LET, IF, FOR, SWITCHON name each level.
//!
//! The verb lives on the noun: `bcpl::learn::Grammar`. It is fed function by
//! function and predicts before it is trained on each (prequential), so
//! every position is unseen when scored. Exact identifiers stay unknowable
//! (the open stratum, docs/157); the shape of the building is learned and
//! carries — which is what a grammar is.

use super::role;
use std::collections::HashMap;

/// Constructs that open a level of depth. Each keyword pushes a named level
/// onto the path; a block or paren pushes a bare level; closers pop.
fn opens(tok: &str) -> Option<&'static str> {
    match tok {
        "valof" => Some("valof"),
        "let" | "and" => Some("let"),
        "if" => Some("if"),
        "unless" => Some("unless"),
        "test" => Some("test"),
        "while" | "until" | "repeatwhile" | "repeatuntil" => Some("loop"),
        "for" => Some("for"),
        "switchon" => Some("switchon"),
        "case" => Some("case"),
        "$(" | "(" | "[" => Some("block"),
        _ => None,
    }
}

/// The path through the building: the stack of open constructs, grown online.
pub struct Grammar {
    depth: usize,
    ngram: HashMap<String, HashMap<&'static str, u32>>,
    pub max_seen: usize,
    pub depth_sum: u64,
    pub depth_n: u64,
}

impl Grammar {
    /// A learner that reads the `depth` deepest open constructs as context.
    pub fn new(depth: usize) -> Self {
        Self { depth: depth.max(1), ngram: HashMap::new(), max_seen: 0, depth_sum: 0, depth_n: 0 }
    }

    /// The context at a point: the deepest levels of the path plus the last
    /// role — the building this token hangs under.
    fn ctx(path: &[&str], last: &str, depth: usize) -> String {
        let from = path.len().saturating_sub(depth);
        let mut c = path[from..].join("/");
        c.push('|');
        c.push_str(last);
        c
    }

    /// Walk a body, maintaining the construct stack; call `f(ctx, role)` for
    /// each token, with the path *before* it.
    fn walk(&mut self, body: &str, mut f: impl FnMut(&str, &'static str)) {
        let toks = crate::prose::tokens(body);
        let mut path: Vec<&'static str> = vec!["fn"];
        let mut last = "fn";
        for t in &toks {
            let r = role::of(t);
            f(&Self::ctx(&path, last, self.depth), r);
            self.max_seen = self.max_seen.max(path.len());
            self.depth_sum += path.len() as u64;
            self.depth_n += 1;
            if matches!(t.as_str(), "$)" | ")" | "]" | "resultis") && path.len() > 1 {
                path.pop();
            }
            if let Some(level) = opens(t) {
                path.push(level);
            }
            last = r;
        }
    }

    /// Learn one function body's path→role evidence.
    pub fn learn(&mut self, body: &str) {
        let mut ins: Vec<(String, &'static str)> = Vec::new();
        self.walk(body, |ctx, r| ins.push((ctx.to_string(), r)));
        for (ctx, r) in ins {
            *self.ngram.entry(ctx).or_default().entry(r).or_insert(0) += 1;
        }
    }

    /// Predict every next role in `body` from what is known now; one bool
    /// per position, in order. Call before [`Self::learn`] on the same body.
    pub fn score(&mut self, body: &str) -> Vec<bool> {
        let ng = std::mem::take(&mut self.ngram);
        let mut hits = Vec::new();
        self.walk(body, |ctx, truth| {
            let g = ng.get(ctx).and_then(|m| m.iter().max_by_key(|x| *x.1).map(|(r, _)| *r));
            hits.push(g == Some(truth));
        });
        self.ngram = ng;
        hits
    }
}

/// The result of learning a stream of functions online: the one honest
/// curve ([`crate::study::Study`]) plus what only this subject knows —
/// how deep the buildings went.
pub struct Report {
    pub study: crate::study::Study,
    pub depth: usize,
    pub functions: usize,
    pub max_depth: usize,
    pub mean_depth: f64,
}

/// The floor a guesser hits knowing nothing: one role in [`role::COUNT`].
pub fn floor() -> f64 {
    100.0 / role::COUNT as f64
}

/// Learn a stream of function bodies online (predict-then-learn), reading
/// the `depth` deepest construct levels as context, over `slices` — the
/// shared protocol of [`crate::study`], nothing bespoke.
pub fn continuous(bodies: &[String], depth: usize, slices: usize) -> Report {
    let mut g = Grammar::new(depth);
    let mut study = crate::study::Study::new(&format!("BCPL grammar by depth {depth}"), slices, floor());
    for (i, body) in bodies.iter().enumerate() {
        for hit in g.score(body) {
            study.tick(i, bodies.len(), hit);
        }
        g.learn(body);
    }
    Report {
        study,
        depth,
        functions: bodies.len(),
        max_depth: g.max_seen,
        mean_depth: g.depth_sum as f64 / g.depth_n.max(1) as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_learns_the_building_not_the_leaf() {
        let bodies: Vec<String> = (0..40)
            .map(|i| format!("LET f{i}(x) = VALOF $( LET s = 0 FOR j = 1 TO x DO $( s := s + j*{i} $) RESULTIS s $)"))
            .collect();
        let r = continuous(&bodies, 16, 4);
        assert_eq!(r.functions, 40);
        assert!(r.max_depth >= 3, "the stack nests: {}", r.max_depth);
        assert!(r.study.beats_floor(), "{} vs floor {}", r.study.accuracy(), floor());
    }
}
