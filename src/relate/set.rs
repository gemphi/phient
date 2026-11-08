//! Language as a set game (docs/162): what a being holds under a kind is a
//! relation in Russell and Whitehead's sense — a set of ordered pairs —
//! and two beings think together by the algebra of relations: union (what
//! either holds), meet (what both hold), composition (`a R b`, `b S c` ⇒
//! `a R∘S c`) and the converse. Nothing is inferred that is not a pair
//! someone was told; the algebra only says which pairs stand together.
//! The algebra is `phiclm::lisp::set`; what reads a being stays here.

use phiecs::agent::Agent;
use phiecs::being::Being;

use phiclm::lisp::set::{compose, meet, union, Relation};

/// Every pair `who` holds under `kind`.
pub fn held(who: &Being, kind: &str) -> Relation {
    let w = who.world();
    let Some(k) = w.relations.get(kind) else { return Relation::new() };
    w.lexicon
        .iter()
        .flat_map(|(word, s)| {
            w.edges.out_of(word).iter().filter(|e| e.kind == k).map(move |e| (s.to_string(), w.spelling(e.to).to_string()))
        })
        .collect()
}

/// One kind thought over by two beings.
#[derive(Debug, Clone, PartialEq)]
pub struct Thought {
    pub kind: String,
    pub a: usize,
    pub b: usize,
    pub union: usize,
    pub meet: usize,
    /// `A ∘ B`: a's pairs chained into b's — what follows from the two together.
    pub composed: Relation,
}

/// Two beings think over every kind either acts under.
pub fn think(a: &Being, b: &Being) -> Vec<Thought> {
    let mut kinds: Vec<&String> = a.persona.intents.iter().chain(b.persona.intents.iter()).collect();
    kinds.sort();
    kinds.dedup();
    kinds
        .into_iter()
        .map(|k| {
            let (ra, rb) = (held(a, k), held(b, k));
            Thought { kind: k.clone(), a: ra.len(), b: rb.len(), union: union(&ra, &rb).len(), meet: meet(&ra, &rb).len(), composed: compose(&ra, &rb) }
        })
        .collect()
}

/// The thoughts as a markdown table, with the first few composed pairs.
pub fn render(a: &str, b: &str, thoughts: &[Thought]) -> String {
    let mut out = format!("| kind | {a} | {b} | union | meet | {a}∘{b} |\n|---|---|---|---|---|---|\n");
    for t in thoughts {
        let eg: Vec<String> = t.composed.iter().take(3).map(|(x, z)| format!("{x}→{z}")).collect();
        out.push_str(&format!("| {} | {} | {} | {} | {} | {} {} |\n", t.kind, t.a, t.b, t.union, t.meet, t.composed.len(), eg.join(", ")));
    }
    out
}

#[cfg(test)]
mod tests;
