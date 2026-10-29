//! (`phient::train::kinds` — it was phiano's `kinds`.)

#![doc = include_str!("kinds/README.md")]

mod types;
mod estimator;
mod table;
mod modes;
mod relation;
mod saturation;
mod permute;
#[cfg(test)]
mod tests;
pub use types::*;
#[allow(unused_imports)]
use estimator::*;
#[allow(unused_imports)]
use table::*;
/// The standing-wave modes relations are read as: phiton's (Bentov).
pub use phiton::physics::standing::harmonic_rotation;
#[allow(unused_imports)]
use relation::*;
#[allow(unused_imports)]
use saturation::*;
#[allow(unused_imports)]
use permute::*;

use phiecs::phasor::facet::Facet;
use phiecs::phasor::spectral::SpectralPhasor;
use crate::train::roles::RoleDiscovery;
use phiton::units::PHASE_CHANNELS;
use std::collections::HashMap;
