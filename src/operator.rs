//! The operators: what moves a word, as against [`crate::relate`], which
//! counts it (docs/159). Three of them, one per field the tree draws on:
//!
//! - [`RelationModel`] — the Joint FNO over a table of named functions
//!   (Fourier; phical's `Fno`, `Joint`), trained one operator per kind
//!   (docs/91–97);
//! - [`FnoWave`] — that operator read as a matched filter, forward and
//!   backwards, through [`Wave`];
//! - [`MirrorWave`] — the phase-conjugate mirror (`phiton::conjugate`): the
//!   hologram read backwards (docs/153).
//!
//! `phient::relate` re-exports every name here, so a caller that counts and
//! moves imports from one place.

mod mirror;
mod model;
mod wave;

pub use mirror::MirrorWave;
pub use model::{kind_token, RelationModel, Seeding};
pub use wave::{inner, offset_kind, FnoWave, Wave};
