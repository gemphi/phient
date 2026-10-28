//! `memory` — the 16-layer interaction log: every interaction recorded
//! with its wave, organised by depth (surface, pattern, semantic, deep),
//! consolidated by count. It was phiano's `memory`.

mod band;
mod memo;
mod consolidate;
#[allow(unused_imports)]
use band::*;
#[allow(unused_imports)]
use memo::*;
#[allow(unused_imports)]
use consolidate::*;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Error, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Total number of memory layers in the system.
pub const MEMORY_LAYERS: usize = 16;

/// Memory band - a group of four layers representing a depth of understanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryBand {
    Surface,
    Pattern,
    Semantic,
    Deep,
}

/// A single recorded interaction in the memory log.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContextWaveEntry {
    pub timestamp_ms: u64,
    pub superposition_wave: (f64, f64),
    pub text_hash: u64,
    #[serde(default)]
    pub text: String,
    pub layer: usize,
    /// Times this interaction has been seen. Raised by consolidation.
    #[serde(default)]
    pub count: u32,
}

/// 16-layer memory log - records every interaction and organizes it by depth.
///
/// `layers` holds *indices* into `entries` rather than clones of them. The
/// previous layout stored every interaction twice, including its full text.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Memo {
    pub entries: Vec<ContextWaveEntry>,
    #[serde(default)]
    pub layers: [Vec<usize>; MEMORY_LAYERS],
}
