//! Recursive composition: words as functions within functions.
//!
//! `roles.rs` composes a word's identity from its fillers, each bound to a
//! role by phase addition, and recovers a filler by subtracting the role.
//! That works once. It was never made to work *twice* — an identity whose
//! fillers are themselves identities — and the reason is arithmetic: each
//! level of superposition adds the crosstalk of every other filler to the
//! recovered one, and after two levels the recovered function is mostly noise.
//! Holographic memories solve this with a **cleanup**: after every unbind,
//! snap the noisy result to the nearest clean item. Here the cleanup is
//! learned — the Fourier operator, trained on `(noisy unbind → clean filler)`
//! pairs — so it can remove crosstalk the way the crosstalk is actually shaped
//! on the relation circle, rather than by nearest-neighbour lookup in a table
//! it would have to be given.
//!
//! The roles are real corpus structure: role `d` is "the word that most often
//! stands `d` positions after this one", for `d = 1..4`. A word with all four
//! fillers is composable; its level-1 identity binds those four fillers, its
//! level-2 identity binds their level-1 identities, and so on. Recovery at
//! level `L` for role `d` is `unbind(I_L(w), d)` ranked against the level
//! `L − 1` objects of every composable word.

use super::space::{cosine, FunctionSpace, WordFunction};
use phiton::units::PHASE_CHANNELS;
use phical::Fno;
pub use phiton::hrr::{bind, compose, unbind};
use std::collections::HashMap;

/// Number of offset roles in the offset-keyed composition (`PairKey::Offset`).
pub const ROLES: usize = 4;
/// Fillers a word needs on distinct keys to be composable.
pub const MIN_FILLERS: usize = 4;
/// Most fillers a channel-keyed identity binds. Twenty superposed terms on a
/// 64-sample circle is noise (level-1 recovery 6× chance against 59× with
/// four — docs/74); the identity keeps the `MAX_FILLERS` best-evidenced keys.
pub const MAX_FILLERS: usize = 4;

/// The role for key `k`, as a unit function. Under `PairKey::Channel` the
/// key *is* the relation channel, so the role of "of" is the role of channel
/// "of" everywhere in the engine.
pub fn role(k: usize) -> WordFunction {
    phiton::hrr::role(PHASE_CHANNELS, 0x0FF5E7 + k as u64)
}

/// How a pair of words is keyed.
///
/// The engine has one definition of a pair's key — `RelationVocab::channel`:
/// the marker word between the two if there is one, else the offset — and
/// the trainer, `Facet::relational_score` and (now) composition all use it.
/// `Offset` is the older keying, offsets 1..=ROLES only, kept so the
/// numbers in docs/66–68 stay reproducible.
#[derive(Clone, Copy)]
pub enum PairKey<'a> {
    Offset,
    Channel(&'a phiecs::relation::RelationVocab),
}

/// The composable words and their fillers per key, from the corpus.
pub struct Composition {
    /// Composable word (space index) → `(key, filler)` pairs, key-sorted.
    pub fillers: HashMap<usize, Vec<(usize, usize)>>,
    /// Composable words in a fixed order.
    pub words: Vec<usize>,
    /// Role function per key index (offset `d` → index `d − 1`; channel `k` → `k`).
    pub roles: Vec<WordFunction>,
}

impl Composition {
    /// Offset-keyed: key `d`'s filler for `w` is the pool word most often
    /// found `d` after it, `d = 1..=ROLES`; composable = all four present.
    pub fn from_corpus(space: &FunctionSpace, sentences: &[Vec<String>], pool: &[usize]) -> Self {
        Self::from_corpus_keyed(space, sentences, pool, PairKey::Offset)
    }

    /// Composition under a chosen pair key. Under `Channel`, the filler for
    /// `(w, k)` is the pool word most often standing in relation `k` after
    /// `w` — "of" is a role, offset+3 is a role — and a word is composable
    /// with `MIN_FILLERS` or more keys filled.
    pub fn from_corpus_keyed(space: &FunctionSpace, sentences: &[Vec<String>], pool: &[usize], key: PairKey) -> Self {
        let in_pool: std::collections::HashSet<usize> = pool.iter().copied().collect();
        let mut counts: HashMap<(usize, usize), HashMap<usize, u32>> = HashMap::new();
        let window = match key {
            PairKey::Offset => ROLES,
            PairKey::Channel(_) => phiecs::phasor::facet::RELATION_WINDOW,
        };
        for s in sentences {
            let ids: Vec<Option<usize>> = s.iter().map(|w| space.index.get(w).copied()).collect();
            for i in 0..s.len() {
                let wi = match ids[i] {
                    Some(w) if in_pool.contains(&w) => w,
                    _ => continue,
                };
                for (j, id) in ids.iter().enumerate().take((i + 1 + window).min(s.len())).skip(i + 1) {
                    let wj = match *id {
                        Some(w) if in_pool.contains(&w) && w != wi => w,
                        _ => continue,
                    };
                    let k = match key {
                        PairKey::Offset => j - i - 1,
                        PairKey::Channel(v) => match v.channel(s, i, j) {
                            Some(k) => k,
                            None => continue,
                        },
                    };
                    *counts.entry((wi, k)).or_default().entry(wj).or_insert(0) += 1;
                }
            }
        }
        let n_roles = match key {
            PairKey::Offset => ROLES,
            PairKey::Channel(_) => PHASE_CHANNELS,
        };
        let mut fillers: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
        let mut words = Vec::new();
        for &w in pool {
            let mut f: Vec<(usize, usize, u32)> = Vec::new();
            for k in 0..n_roles {
                if let Some(m) = counts.get(&(w, k)) {
                    // Most frequent; ties broken by index so the choice is stable.
                    if let Some((b, c)) = m.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0))).map(|(x, c)| (*x, *c)) {
                        f.push((k, b, c));
                    }
                }
            }
            if matches!(key, PairKey::Channel(_)) && f.len() > MAX_FILLERS {
                // Keep the best-evidenced keys; stable on ties by key index.
                f.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
                f.truncate(MAX_FILLERS);
                f.sort_by_key(|x| x.0);
            }
            let f: Vec<(usize, usize)> = f.into_iter().map(|(k, b, _)| (k, b)).collect();
            let enough = match key {
                PairKey::Offset => f.len() == ROLES,
                PairKey::Channel(_) => f.len() >= MIN_FILLERS,
            };
            if enough {
                fillers.insert(w, f);
                words.push(w);
            }
        }
        let roles = (0..n_roles).map(role).collect();
        Self { fillers, words, roles }
    }

}

mod recover;
#[cfg(test)]
mod tests;
