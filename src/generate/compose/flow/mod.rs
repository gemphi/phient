//! RiverFlow - generates text by flowing through color sectors.
//!
//! Based on the Flower-Hayes cognitive process model:
//! - **Planning**: the prompt determines the source sector (color)
//! - **Translating**: words are gathered from each sector along the flow
//!
//! The river starts at the prompt's sector, drifts through adjacent
//! sectors, reaches the opposite (tension), and resolves back.

mod river;
mod path;
#[allow(unused_imports)]
use river::*;
#[allow(unused_imports)]
use path::*;

mod compose;

use crate::generate::constants as config;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use phiecs::phasor::wave::Wave;

pub struct RiverFlow {
    /// The prompt that seeded the flow.
    pub prompt: String,
    /// The source sector where the river begins.
    pub source_sector: u16,
    /// The composed text for this sector variation.
    pub text: String,
}
