//! Generation: the context wave, the operator guide, the generator and its decoders.
//! With them: `learn` (the scorer's weights and the tuner that fits them),
//! `compose` (better/worse composition and the river flow) and `constants`.
//! It was phiano's engine (`generate`, `learn`, `compose`); phiano's apps
//! (persona, server, the core) read it from here.
//!
//! Split by concern: `context` (the context wave buffer), `guide` (the trained
//! operator as a guide), `generator` (the struct, its knobs and the candidate
//! scorer), `decode` (the beam and the greedy decoder), `select` (where a
//! step's candidates come from), `format` (speakability and output text).

mod beam;
pub mod compose;
pub mod constants;
pub mod learn;
mod candidates;
mod context;
mod decode;
mod format;
mod generator;
mod guide;
mod persona;
mod sampling;
mod select;
mod steer;
#[cfg(test)]
mod tests;

pub use context::{ContextWaveBuffer, CONTEXT_DECAY_BASE, CONTEXT_LAYERS_COUNT, CONTEXT_WINDOW_SIZE};
pub use generator::{Decode, Generator, RELATIONAL_PROPOSALS, RELATIONAL_WINDOW};
pub use persona::{PersonaPrior, PERSONA_CLIP};
pub use guide::{OperatorGuide, OPERATOR_PROPOSALS, OPERATOR_PROPOSAL_COUNT, OPERATOR_PROPOSAL_FLOOR, OPERATOR_WEIGHT, OPERATOR_WIDEN};

/// One partial sequence in a beam.
#[derive(Clone)]
pub(crate) struct Hypothesis {
    pub(crate) tokens: Vec<String>,
    pub(crate) prev: Option<String>,
    pub(crate) last: Option<String>,
    pub(crate) phase: f64,
    pub(crate) score: f64,
    pub(crate) recent: std::collections::HashSet<String>,
}
