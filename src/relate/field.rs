//! A field of phasors over the scope tree (docs/136): the paradigm's other
//! half beside the counts. Every word has a function — its sixty-four
//! angles on the relation circle (phiecs's [`Phasor`](phiecs::phasor::Phasor)
//! seeded from its spelling) as a unit phasor per channel, or a function the
//! operator learned — and a scope's phasor is the sum of what it said beyond
//! the whole tree, each word by its D4 post: `Σ_w ln⁺(p_scope(w) / p(w)) ·
//! f(w)` (the same `ln(dist / prior)` the counts post; a word the scope says
//! no more than the tree does adds nothing, and a thousand common words do
//! not drown the one rare word a scope said). The asked words are summed by
//! how much each says, `ln(1 / p(w))`. A cast is then a cosine between the
//! two functions (phinet's `cosine`): the word is drawn into the scopes whose
//! phasor points its way, with R the length of the function as the knob. No
//! table is consulted in the descent; `said`/`own` on the result are the
//! counts' locality, kept so the same gate reads the result.

use super::{Cast, Tally, Order, ScopeTree, SAYS};
use phistd::{c64, cosine, Function};
use std::collections::HashMap;

pub struct Field {
    space: usize,
    words: HashMap<String, Function>,
    scopes: HashMap<String, Function>,
    /// Each word's share of what the whole tree says.
    prior: HashMap<String, f64>,
}

/// A word's function from its spelling alone: at R = 64 its phiecs
/// [`Phasor`](phiecs::phasor::Phasor) angles, otherwise a seeded unit function.
pub fn seeded(word: &str, space: usize) -> Function {
    if space == phiecs::relation::MAX_KINDS {
        return phiecs::phasor::Phasor::seeded(word).angles().iter().map(|t| c64::from_polar(1.0, *t)).collect();
    }
    phistd::unit_phase(space, phistd::fnv1a(word))
}

impl Field {
    /// Every word seeded from its spelling.
    pub fn seeded(counts: &Tally, tree: &ScopeTree, space: usize) -> Self {
        Self::with_words(counts, tree, space, HashMap::new())
    }

    /// Words with the functions given (a learned table); the rest seeded.
    pub fn with_words(counts: &Tally, tree: &ScopeTree, space: usize, mut words: HashMap<String, Function>) -> Self {
        let told = counts.said(None, SAYS);
        let total: f64 = told.iter().map(|x| x.1).sum::<f64>().max(1.0);
        for (w, _) in &told {
            words.entry(w.clone()).or_insert_with(|| seeded(w, space));
        }
        let prior: HashMap<String, f64> = told.into_iter().map(|(w, n)| (w, n / total)).collect();
        let mut f = Field { space, words, scopes: HashMap::new(), prior };
        let mut stack: Vec<String> = tree.roots().to_vec();
        while let Some(s) = stack.pop() {
            stack.extend(tree.children(&s).iter().cloned());
            let said = counts.said(Some(&s), SAYS);
            let own: f64 = said.iter().map(|x| x.1).sum::<f64>().max(1.0);
            let mut g = vec![c64::new(0.0, 0.0); space];
            for (w, n) in &said {
                let post = (n / own / f.prior[w]).ln();
                if post > 0.0 {
                    axpy(&mut g, post, &f.words[w]);
                }
            }
            f.scopes.insert(s, g);
        }
        f
    }

    pub fn space(&self) -> usize {
        self.space
    }

    /// The asked words as one function, each by how much it says
    /// (`ln(1 / p(w))`: a rare word says more than `the`); a word never told
    /// adds nothing.
    pub fn function(&self, conds: &[String]) -> Function {
        let mut g = vec![c64::new(0.0, 0.0); self.space];
        for c in conds {
            if let (Some(f), Some(p)) = (self.words.get(c), self.prior.get(c)) {
                axpy(&mut g, -p.ln(), f);
            }
        }
        g
    }

    /// How `scope` points the way of `conds`: the cosine of the two functions.
    pub fn attraction(&self, scope: &str, asked: &Function) -> f64 {
        self.scopes.get(scope).map_or(0.0, |g| cosine(asked, g))
    }

    /// The scopes `conds` are drawn into by cosine alone, best first: from
    /// `under` (or the roots) down, entering at each step the `beam`
    /// children whose phasor points the asked way (cosine above zero).
    /// `said` and `own` are the counts' locality on the result (`Cast`), so
    /// the same gate reads it; the result is ranked `by` (`Order::Told` adds
    /// the base's now-over-was, which a phasor does not carry).
    pub fn cast_under(&self, counts: &Tally, tree: &ScopeTree, under: &[String], conds: &[String], beam: usize, by: Order) -> Vec<Cast> {
        let asked = self.function(conds);
        let mut found: Vec<(Vec<String>, f64)> = Vec::new();
        let mut frontier: Vec<Vec<String>> = vec![under.to_vec()];
        while let Some(path) = frontier.pop() {
            // The roots are entered whatever they say (a root says the
            // whole tree, so its phasor points nowhere in particular).
            let (children, floor): (&[String], f64) = match path.last() {
                Some(s) => (tree.children(s), 0.0),
                None => (tree.roots(), f64::MIN),
            };
            let mut next: Vec<(&String, f64)> = children.iter().map(|s| (s, self.attraction(s, &asked))).filter(|x| x.1 > floor).collect();
            next.sort_by(|a, b| b.1.total_cmp(&a.1));
            next.truncate(beam);
            for (s, a) in next {
                let p: Vec<String> = path.iter().cloned().chain([s.clone()]).collect();
                found.push((p.clone(), a));
                frontier.push(p);
            }
        }
        if !under.is_empty() && found.is_empty() {
            found.push((under.to_vec(), 0.0));
        }
        let mut casts: Vec<Cast> = found
            .into_iter()
            .map(|(path, attraction)| {
                let leaf = path.last().map(String::as_str);
                let own = conds.iter().filter(|c| leaf.is_some_and(|s| counts.count(Some(s), SAYS, c) >= 1.0)).count();
                let said = path.iter().map(|s| conds.iter().filter(|c| counts.count(Some(s), SAYS, c) >= 1.0).count()).sum();
                Cast { path, said, own, attraction }
            })
            .collect();
        super::order(&mut casts, by);
        if std::env::var("PHIANO_CAST_DEBUG").is_ok() {
            for c in &casts {
                eprintln!("field {:?} said {} (own {}) cosine {:.3}", c.path, c.said, c.own, c.attraction);
            }
        }
        casts
    }
}

/// `y += a · x`.
fn axpy(y: &mut [c64], a: f64, x: &[c64]) {
    for (y, x) in y.iter_mut().zip(x) {
        *y += x * a;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phiecs::quest::Row;

    fn row(a: &str, k: &str, o: &str) -> Row {
        Row::new(a, k, o)
    }

    #[test]
    fn a_word_is_drawn_to_the_scope_whose_phasor_points_its_way() {
        let mut rows = vec![row("scope:sum", "within", "scope:data"), row("scope:count", "within", "scope:data")];
        for (f, scope, w) in [("sum_even", "scope:sum", "even"), ("sum_abs", "scope:sum", "abs"), ("count_even", "scope:count", "even"), ("count_below", "scope:count", "below"), ("count_odd", "scope:count", "odd")] {
            rows.push(row(f, "within", scope));
            rows.push(row(f, "within", "scope:data"));
            rows.push(row(f, SAYS, w));
            rows.push(row(f, SAYS, f.split('_').next().unwrap()));
        }
        let mut c = Tally::new(&["within"]);
        c.learn(&rows);
        let t = ScopeTree::from_rows(&rows);
        for space in [64, 256] {
            let f = Field::seeded(&c, &t, space);
            let found = f.cast_under(&c, &t, &[], &["count".into(), "odd".into()], 2, Order::Attraction);
            assert_eq!(found[0].path, ["scope:data", "scope:count"], "R = {space}: {found:?}");
            assert_eq!(found[0].own, 2);
            let found = f.cast_under(&c, &t, &[], &["sum".into()], 1, Order::Attraction);
            assert_eq!(found[0].path, ["scope:data", "scope:sum"], "R = {space}: {found:?}");
        }
    }
}
