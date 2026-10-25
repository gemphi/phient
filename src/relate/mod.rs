//! The quest's rows as relation operators: where phiecs, phiano and phinet
//! meet.
//!
//! In Phient's world, a word (w) is a seed of what it relates to, under what
//! conditions (c). The three crates have one job each here (docs/85, docs/93):
//!
//! - **phiecs** asks: a quest writes `relations.tsv`, read with
//!   [`phiecs::quest::parse_tsv`] — the one parser for that table.
//! - **phinet** learns, knowing no words: [`phical::relate`] trains one
//!   operator for every kind over rows of a function table, counts the laws
//!   the kinds obey, and trains closure.
//! - **phiano** names: [`RelationModel`] maps words and kinds to table rows,
//!   can start the table from a facet's own angles (so the quest facet
//!   `quest_ingest` wrote is the operator's first guess), reads a word's
//!   answers back by name, and saves the whole thing as `.relate`.
//! - [`Tally`] reads the same rows by their counts alone (D2, D4): no
//!   operator and no epochs, so one told row is learned at once.
//! - [`Wave`] reads either learned reader as a matched filter, forward and
//!   backwards: [`FnoWave`] for the Joint FNO, [`MirrorWave`] for the
//!   phase-conjugate mirror (docs/153). The operators themselves live in
//!   [`crate::operator`]; this module is the counting.

pub mod counts;
mod dist;
pub mod field;
mod kin;
mod table;
pub mod levels;
pub mod tree;
mod weights;
mod path;
mod quests;
#[cfg(test)]
mod tests;

pub mod kit;
pub use counts::{Tally, ValuesAndPrior};
pub use kit::Casting;
pub use field::Field;
pub use levels::{nest, scopes, SCOPE};
pub use tree::{order, Cast, Order, ScopeTree, SAYS};
pub use weights::{left_out_weight, mixture_weights, Weighting};
// The operators moved to [`crate::operator`] (docs/159); their names stay
// readable here so every caller of `phient::relate` reads unchanged.
pub use crate::operator::{inner, kind_token, offset_kind, FnoWave, MirrorWave, RelationModel, Seeding, Wave};
pub use quests::{held_out, read_quests};
pub use phical::relate::{find_laws, Law, Score, Triple};
