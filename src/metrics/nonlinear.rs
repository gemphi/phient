//! Non-linear readout — the fourth requirement in HOW 16.
//!
//! Every operation in the engine is a sum or a multiplication by a constant,
//! followed by argmax. A composition of linear maps is linear, so the model's
//! function class is linear however many tiers the architecture diagram shows,
//! and neither capacity nor training fixes that.
//!
//! The readout is a **conditional** table: given the sector pattern of the
//! context, a learned bias per *target* sector. That conditioning is what makes
//! it non-linear and what makes it useful — a bias that depended only on the
//! context would add the same constant to every candidate and could not change
//! a ranking at all.
//!
//! Discretisation is the non-linearity, and a lookup table over discretised
//! inputs is a universal approximator. It also stays interpretable: the table
//! can be printed.

mod default;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use default::*;

use phiecs::phasor::wave::Wave;
use phistd::c64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default channels hashed into the context key.
pub const KEY_CHANNELS: usize = 4;

/// Default key resolution: buckets per channel.
///
/// The full sector grid is 64 wide, and 64⁴ is 16.7 million cells — a table
/// that fine memorises the training split and is a cache miss on every held-out
/// context, so it cannot change a held-out score at all. The key resolution is
/// therefore a *separate*, coarser quantisation from the target sector grid,
/// and it is the parameter that trades expressiveness against coverage.
pub const KEY_SECTORS: usize = 8;

/// Target sectors the bias vector covers.
const TARGET_SECTORS: usize = 64;

#[derive(Debug, Serialize, Deserialize)]
pub struct SectorReadout {
    /// context cell → bias per target sector
    pub(super) bias: HashMap<u64, Vec<f32>>,
    pub(super) hits: HashMap<u64, u32>,
    /// Channels hashed into the key, and buckets per channel. Stored so a table
    /// can only ever be queried at the resolution it was fitted at.
    pub(super) key_channels: usize,
    pub(super) key_sectors: usize,
    /// Held-out diagnostics: lookups made, and lookups that found a cell.
    ///
    /// Atomic because scoring runs under rayon; relaxed ordering because these
    /// are a diagnostic ratio, not a synchronisation point.
    #[serde(skip)]
    pub(super) lookups: std::sync::atomic::AtomicU64,
    #[serde(skip)]
    pub(super) found: std::sync::atomic::AtomicU64,
}
