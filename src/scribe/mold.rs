//! The molds: a kept program's SKELETON with its open stratum as holes
//! (docs/157 read for writing — names and numbers are the open stratum of
//! BCPL; keywords and punctuation are the building). Writing from a mold
//! reuses a whole PROVEN building and varies its leaves: construction-level
//! composition, not token-level wandering. A mutation sometimes splices a
//! mold through the token memory, which is how new buildings are minted.

use super::memory::{coin, Memory};

/// A lexeme's place in a mold: kept, or a typed hole. A name-hole carries
/// its binding CLASS — every occurrence of one original name is the same
/// class, so filling renames CONSISTENTLY (substitution: the leaves are
/// bound by their declarations, docs/181, and independent filling breaks
/// the binding — measured first, fixed second). `start` stays fixed: the
/// machine's own convention, part of the runtime's closed stratum.
#[derive(Clone, Debug, PartialEq)]
pub enum Slot {
    Fixed(String),
    /// A number stood here; numbers are free.
    Figure,
    /// A name stood here; the class binds its occurrences together.
    Hole(usize),
}

/// A program's mold, holes classed by binding.
fn of(tok: &[String]) -> Vec<Slot> {
    let mut class: std::collections::HashMap<&str, usize> = Default::default();
    tok.iter()
        .map(|t| {
            if t.parse::<i64>().is_ok() {
                Slot::Figure
            } else if t != "start"
                && t.chars().next().is_some_and(|c| c.is_alphabetic())
                && !phiclm::token::KEYWORDS.contains(&t.as_str())
            {
                let n = class.len();
                Slot::Hole(*class.entry(t).or_insert(n))
            } else {
                Slot::Fixed(t.to_string())
            }
        })
        .collect()
}

/// The molds kept so far, each with how often it has produced a keep —
/// and, since docs/185, a library of CALLEE fragments: each top-level
/// `LET name(...)` of a kept program, with the binding class of its
/// declared name, so a composition can LINK a caller to it instead of
/// concatenating dead weight (docs/184's finding).
#[derive(Default)]
pub struct MoldSet {
    pub mold: Vec<Vec<Slot>>,
    pub weight: Vec<u32>,
    pub callee: Vec<(Vec<Slot>, Option<usize>)>,
}

/// The top-level fragments of a token stream: split where `let` stands at
/// section depth 0 — structure read off the brackets, nothing guessed.
fn fragments(tok: &[String]) -> Vec<Vec<String>> {
    let (mut out, mut depth): (Vec<Vec<String>>, i32) = (Vec::new(), 0);
    for t in tok {
        if t == "let" && depth == 0 {
            out.push(Vec::new());
        }
        match t.as_str() {
            "$(" | "{" => depth += 1,
            "$)" | "}" => depth -= 1,
            _ => {}
        }
        if let Some(last) = out.last_mut() {
            last.push(t.clone());
        }
    }
    out
}

impl MoldSet {
    /// A kept program's mold: counted again if known, minted if new.
    /// Returns true when the SKELETON itself is new — structural novelty.
    /// Its callee fragments (top-level LETs that are not `start`) join the
    /// library with the class of their declared name.
    pub fn keep(&mut self, tok: &[String]) -> bool {
        for f in fragments(tok) {
            let m = of(&f);
            let named = match m.get(1) {
                Some(Slot::Hole(c)) => Some(*c),
                _ => None, // `start`, or nothing — not a callee
            };
            if named.is_some() && !self.callee.iter().any(|(x, _)| *x == m) {
                self.callee.push((m, named));
            }
        }
        let m = of(tok);
        match self.mold.iter().position(|x| *x == m) {
            Some(i) => {
                self.weight[i] += 1;
                false
            }
            None => {
                self.mold.push(m);
                self.weight.push(1);
                true
            }
        }
    }

    /// A building drawn by weight.
    fn pick(&self, rng: &mut u64) -> usize {
        phistd::random::draw_by_weight(&self.weight, coin(rng))
    }

    /// One program written from the molds: a proven building drawn by
    /// weight — and at rate `compose`, a SECOND building joined on the end,
    /// its binding classes re-based past the first's so the two stay
    /// separate unless the filling draws them together (composition at the
    /// construction level: buildings, not leaves). Holes filled from `name`
    /// and `figure`; at rate `mutate` one place spliced through the token
    /// memory — a new building proposed.
    pub fn write(&self, memory: &Memory, name: &[String], figure: &[String], rng: &mut u64, mutate: f64, compose: f64) -> Option<Vec<String>> {
        if self.mold.is_empty() {
            return None;
        }
        let which = self.pick(rng);
        let mut drawn = self.mold[which].clone();
        // Linked composition (docs/185): a callee fragment PREPENDED, its
        // classes re-based past the program's, and ONE of the program's
        // classes tied to the callee's declared name — so the building is
        // not dead weight (docs/184) but something the program can call.
        let mut tie: Option<(usize, usize)> = None;
        let classes_of = |m: &[Slot]| m.iter().filter_map(|s| match s { Slot::Hole(c) => Some(*c + 1), _ => None }).max().unwrap_or(0);
        if coin(rng) < compose && !self.callee.is_empty() {
            let own = classes_of(&drawn);
            let (frag, named) = &self.callee[(coin(rng) * self.callee.len() as f64) as usize % self.callee.len()];
            if own > 0 {
                if let Some(named) = named {
                    tie = Some(((coin(rng) * own as f64) as usize % own, own + named));
                }
            }
            let mut joined: Vec<Slot> = frag
                .iter()
                .map(|s| match s {
                    Slot::Hole(c) => Slot::Hole(c + own),
                    keep => keep.clone(),
                })
                .collect();
            joined.extend(drawn);
            drawn = joined;
        }
        let any = |pool: &[String], rng: &mut u64| pool[(coin(rng) * pool.len() as f64) as usize % pool.len()].clone();
        // One fresh name per binding class, drawn once — substitution.
        let classes = drawn
            .iter()
            .filter_map(|s| match s {
                Slot::Hole(c) => Some(*c),
                _ => None,
            })
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        let mut fresh: Vec<String> = (0..classes).map(|_| any(name, rng)).collect();
        if let Some((caller, callee)) = tie {
            fresh[caller] = fresh[callee].clone();
        }
        let mut tok: Vec<String> = drawn
            .iter()
            .map(|s| match s {
                Slot::Fixed(t) => t.clone(),
                Slot::Figure => any(figure, rng),
                Slot::Hole(c) => fresh[*c].clone(),
            })
            .collect();
        if coin(rng) < mutate && !tok.is_empty() {
            let at = (coin(rng) * tok.len() as f64) as usize % tok.len();
            let grown = memory.write(name, rng, 0.0, 3);
            match (coin(rng) * 3.0) as u32 {
                0 => tok[at] = grown.last().cloned().unwrap_or_else(|| "1".into()),
                1 => tok.insert(at, grown.last().cloned().unwrap_or_else(|| "1".into())),
                _ => {
                    tok.remove(at);
                }
            }
        }
        Some(tok)
    }
}
