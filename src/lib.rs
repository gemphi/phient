//! # phient — Phient's Phasor Language Model
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and
//! composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical
//! stores no point: a word is 32–64 relations, each a complex phasor on a
//! closed circle with a count; meaning is a side effect of pairs of words
//! moving under a condition.
//!
//! The model itself, apart from any application (paper 25): a word is an
//! entity told once as a row `(about, kind, other)` under the conditions it
//! was used in — the sentence, the paragraph, the chapter; the function,
//! the file, the crate; the order, the scene, the session — and from then
//! on it lives by a few rules. What a Context Language Model
//! (`gemphi/ext/context-language-models`, Shao et al. 2026) keeps as one
//! text the model edits, this crate keeps as words with counts in a tree.
//!
//! | module | what |
//! |---|---|
//! | [`relate`] | `Tally` (D1 movement, D2 posts, the prior; docs/124), levels (D4 nested, `read_levels`; docs/126), `ScopeTree` and the cast (docs/126, docs/129), `Field` (scopes as phasors, cast by cosine; docs/136), kin (docs/138) |
//! | [`operator`] | what moves a word: `RelationModel` (the Joint FNO over a table of named functions; docs/91–97), `FnoWave` and `MirrorWave` (the filter read forward and backwards; docs/153) |
//! | [`prose`] | text as a tree of knowing: `Knowing` (learn, `learn_new`, `next`, `go`), the teacher cycle, what is not yet known, held-out scoring (docs/127–138) |
//! | [`baseline`] | the number to beat: interpolated Kneser-Ney over tokenized sentences |
//! | [`code`] | Rust read by its own grammar into rows (docs/108, docs/124) |
//! | [`study`] | step-by-step learning measured honestly: predict first, learn after, one curve for every method (docs/172) |
//!
//! phiano holds the applications — students, games, personas, the binaries —
//! and re-exports these three modules, so `phiano::relate` is this crate's.

pub mod baseline;
pub mod code;
pub mod operator;
pub mod prose;
pub mod relate;
pub mod study;
