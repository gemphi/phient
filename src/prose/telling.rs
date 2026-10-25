//! Telling (docs/132): a tree grows paragraph by paragraph, on the fly —
//! nothing is retrained, the counts and the scopes grow in place. This is
//! the incremental half of [`Knowing`]: `fresh`, `tell`, `learn_tree`.

use super::knowing::{Knowing, KINDS};
use crate::relate::{Order, ScopeTree, Tally};

impl Knowing {
    /// As [`Self::learn_except`], but over a tree already read into rows by
    /// any source (prose, BCPL, Rust) — so a language learns the same way a
    /// story does. `held(path)` keeps a paragraph out, for scoring.
    pub fn learn_tree(tree: &[(String, Vec<phiecs::quest::Row>)], held: &dyn Fn(&str) -> bool) -> Self {
        let mut k = Self::fresh();
        for (path, rows) in tree {
            if held(path) {
                k.held.push((path.clone(), Self::words(rows)));
            } else {
                k.tell(path, rows);
            }
        }
        k
    }

    /// A tree that knows nothing yet — told paragraph by paragraph.
    pub fn fresh() -> Self {
        Knowing {
            counts: Tally::new(KINDS),
            tree: ScopeTree::default(),
            paragraphs: Vec::new(),
            rows: 0,
            within: Vec::new(),
            flat: false,
            held: Vec::new(),
            field: None,
            order: Order::Told,
            base: None,
            kin: false,
        }
    }

    /// Tells one more paragraph — on the fly, no re-reading (docs/132): the
    /// counts and the scope tree grow in place, so a question asked between
    /// two tells is answered by everything told so far.
    pub fn tell(&mut self, path: &str, rows: &[phiecs::quest::Row]) {
        self.counts.learn(rows);
        self.tree.add_rows(rows);
        self.paragraphs.push((path.to_string(), Self::words(rows)));
        self.rows += rows.len();
    }

    pub(super) fn words(rows: &[phiecs::quest::Row]) -> Vec<String> {
        rows.iter().filter(|r| r.kind == "has-word").map(|r| r.other.clone()).collect()
    }
}
