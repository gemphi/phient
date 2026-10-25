//! The scope tree, and casting into it (docs/126). The conditions a word is
//! read under form a tree — the path of its scopes — and the tree is built
//! from `within` rows alone, so the quest can tell it and the file system
//! can hold it (`data/<quest>/<family>/<task>.rs` is one). A word is not
//! aware of where it belongs: it is *drawn* into scopes by what it says,
//! the way a ray is drawn through a bounding-volume hierarchy. `cast` starts
//! at the roots and enters the children that attract the word's conditions
//! — those whose functions said them more than the whole tree does (D4 on
//! the `says` kind; a function is told `within` every scope on its path, so
//! a scope says what its subtree says) — pruning the rest, and returns the
//! paths it reached.
//! Reading under a found path is then `Tally::read_levels`.

use super::counts::Tally;
use phiecs::quest::Row;
use std::collections::HashMap;

/// The kind under which a scope's functions say their conditions, so a
/// scope has a distribution over words a cast can be drawn by.
pub use phiecs::scope::SAYS;

/// Where a cast reached: the path, how many of the asked conditions were
/// said along it, how many by its last scope alone, and the graded
/// attraction along it.
#[derive(Clone, Debug, PartialEq)]
pub struct Cast {
    pub path: Vec<String>,
    pub said: usize,
    pub own: usize,
    pub attraction: f64,
}

/// How the scopes a cast reached are ranked. `Told`: most of the asked
/// conditions said along the path, then the deepest, then the scope told
/// later (phiecs's `Condition`: now over was), then the most attracted.
/// `Attraction`: the attraction alone — the phasor's cosine, with no time
/// in it (docs/136).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Order {
    #[default]
    Told,
    Attraction,
}

/// Ranks `found` best first by `by`.
pub fn order(found: &mut [Cast], by: Order) {
    match by {
        Order::Told => found.sort_by(|a, b| b.said.cmp(&a.said).then(b.path.len().cmp(&a.path.len())).then(later(&b.path).cmp(&later(&a.path))).then(b.attraction.total_cmp(&a.attraction))),
        Order::Attraction => found.sort_by(|a, b| b.attraction.total_cmp(&a.attraction)),
    }
}

#[derive(Default, Clone, Debug)]
pub struct ScopeTree {
    /// Parent scope ⇒ its children, in the order told.
    children: HashMap<String, Vec<String>>,
    roots: Vec<String>,
}

impl ScopeTree {
    /// From the `within` rows between scopes (`scope:… within scope:…`);
    /// what a scope holds is told by `within` too but is not a scope. A
    /// scope never a child is a root.
    pub fn from_rows(rows: &[Row]) -> Self {
        let mut t = ScopeTree::default();
        t.add_rows(rows);
        t
    }

    /// Adds the scope-within-scope rows of `rows`.
    pub fn add_rows(&mut self, rows: &[Row]) {
        rows.iter().filter(|r| r.kind == "within" && r.about.starts_with(super::levels::SCOPE)).for_each(|r| self.add(&r.other, &r.about));
    }

    pub fn add(&mut self, parent: &str, child: &str) {
        let v = self.children.entry(parent.to_string()).or_default();
        if !v.contains(&child.to_string()) {
            v.push(child.to_string());
        }
        self.roots.retain(|r| r != child);
        let is_child = self.children.values().any(|v| v.iter().any(|c| c == parent));
        if !is_child && !self.roots.contains(&parent.to_string()) {
            self.roots.push(parent.to_string());
        }
    }

    pub fn children(&self, scope: &str) -> &[String] {
        self.children.get(scope).map_or(&[], Vec::as_slice)
    }

    pub fn roots(&self) -> &[String] {
        &self.roots
    }

    /// How `scope` attracts `conds` beyond its parent: how many of them it
    /// says — a word said by at most half of the siblings (D3: a
    /// condition is local; `is` is said by every turn and draws nowhere)
    /// and said here — and the sum over them of `ln(dist_scope(c) /
    /// base(c))`, the scope's `says` counts smoothed toward the parent's
    /// distribution `base` (D4 nested); `None` for a scope whose words said
    /// nothing. Returns the scope's own distribution with them, the base
    /// for its children.
    fn attraction(counts: &Tally, siblings: &[&String], scope: &str, conds: &[(String, Option<usize>)], base: &[f64]) -> Option<(usize, f64)> {
        let total = counts.total(Some(scope), SAYS);
        if total == 0.0 {
            return None;
        }
        let local = |c: &String| siblings.len() < 2 || siblings.iter().filter(|s| counts.count(Some(s), SAYS, c) > 0.0).count() * 2 <= siblings.len();
        let said = conds.iter().filter(|(c, _)| counts.count(Some(scope), SAYS, c) >= 1.0 && local(c)).count();
        // The smoothed share at each condition, as `dist` would give it,
        // without building the whole distribution.
        let graded: f64 = conds.iter().filter_map(|(c, i)| i.map(|i| ((counts.count(Some(scope), SAYS, c) + base[i]) / (total + 1.0) / base[i]).ln())).sum();
        Some((said, graded))
    }

    /// The scopes `conds` are drawn into, best first: from the roots down,
    /// entering at each step the `beam` children that say most of the
    /// conditions beyond their parent — among equals the one told later
    /// (phiecs's `Condition`: `now` over `was`; scopes are named in the
    /// order told, `t0001`, `t0002`, …, `p1`, `p2`, …), then the one saying
    /// them more — and keeping every scope entered with its path. A child
    /// whose attraction is below `min` nats is not entered. The leaves of
    /// the search are where the word is read; `read_levels` on a path does
    /// the reading.
    pub fn cast(&self, counts: &Tally, conds: &[String], min: f64, beam: usize) -> Vec<Cast> {
        self.cast_under(counts, &[], conds, min, beam)
    }

    /// `cast` from a scope already entered (`under`, a path from a root):
    /// the search stays in that subtree, the way a story goes on inside its
    /// own chapter while the words draw on the whole tree. Each result is
    /// `(path, said)` with `said` the conditions said along it, summed, and
    /// the graded attraction in its fraction.
    pub fn cast_under(&self, counts: &Tally, under: &[String], conds: &[String], min: f64, beam: usize) -> Vec<Cast> {
        let vp = counts.values_and_prior(SAYS);
        let (values, prior) = (&vp.0, vp.1.clone());
        let conds: Vec<(String, Option<usize>)> = conds.iter().map(|c| (c.clone(), values.binary_search(c).ok())).collect();
        // (path, conditions said along it, said by its last scope, graded attraction along it, this scope's distribution)
        type Entry = (Vec<String>, usize, usize, f64, Vec<f64>);
        let mut base = prior;
        for s in under {
            if let Some(d) = counts.dist(s, SAYS, values, &base) {
                base = d;
            }
        }
        let mut found: Vec<Entry> = Vec::new();
        let mut frontier: Vec<Entry> = vec![(under.to_vec(), 0, 0, 0.0, base)];
        while let Some((path, said, _, graded, base)) = frontier.pop() {
            let (children, floor): (Vec<&String>, f64) = match path.last() {
                Some(s) => (self.children(s).iter().collect(), min),
                None => (self.roots.iter().collect(), f64::MIN),
            };
            let siblings = children.clone();
            let mut next: Vec<(String, usize, usize, f64)> = children
                .into_iter()
                .filter_map(|s| Self::attraction(counts, &siblings, s, &conds, &base).filter(|(_, a)| *a >= floor).map(|(n, a)| (s.clone(), said + n, n, graded + a)))
                .collect();
            next.sort_by(|a, b| b.1.cmp(&a.1).then(later(std::slice::from_ref(&b.0)).cmp(&later(std::slice::from_ref(&a.0)))).then(b.3.total_cmp(&a.3)));
            next.truncate(beam);
            // Only the scopes entered get their whole distribution (the base
            // for their children).
            for (s, n, own, g) in next {
                let d = counts.dist(&s, SAYS, values, &base).unwrap_or_else(|| base.clone());
                let e: Entry = (path.iter().cloned().chain([s]).collect(), n, own, g, d);
                found.push(e.clone());
                frontier.push(e);
            }
        }
        if !under.is_empty() && found.is_empty() {
            return vec![Cast { path: under.to_vec(), said: 0, own: 0, attraction: 0.0 }];
        }
        if std::env::var("PHIANO_CAST_DEBUG").is_ok() {
            for f in &found {
                eprintln!("cast {:?} said {} (own {}) graded {:.3}", f.0, f.1, f.2, f.3);
            }
        }
        let mut casts: Vec<Cast> = found.into_iter().map(|(path, said, own, attraction, _)| Cast { path, said, own, attraction }).collect();
        order(&mut casts, Order::Told);
        casts
    }
}

/// A path's order of telling: each component's trailing number, so `p10`
/// comes after `p9` and `t0100` after `t0099`.
fn later(path: &[String]) -> Vec<(u64, String)> {
    path.iter()
        .map(|s| {
            let digits: String = s.chars().rev().take_while(char::is_ascii_digit).collect::<Vec<_>>().into_iter().rev().collect();
            (digits.parse().unwrap_or(0), s.clone())
        })
        .collect()
}
