//! `net` — the lexicon read as a function space, beside the operators.
//!
//! A word's [`SpectralPhasor`](phiecs::phasor::SpectralPhasor) is a function
//! on the relation circle; the Fourier operator (`phical::Fno`, on phistd's
//! tape) learns over those functions. This module is what that needs:
//!
//! * [`space`]   — a facet's words as functions, contexts as functions, and
//!   `(context → next word)` examples.
//! * [`joint`]   — examples for joint training, and writing learned functions
//!   back into a facet.
//! * [`compose`] — nested role binding from corpus offsets, with the operator
//!   as the learned cleanup that makes recursive unbinding recoverable.
//! * [`reader`]  — what is in the existing model files.
//! * [`lm`]      — the operator as a scorer inside the perplexity harness.
//!
//! It was phiano's `phinet` (and then `phiano::net`, for the reader and the
//! scorer, until the storage, memory and metrics they read moved too).

pub mod compose;
pub mod joint;
pub mod lm;
pub mod reader;
pub mod space;

pub use space::{FunctionSpace, WordFunction};
