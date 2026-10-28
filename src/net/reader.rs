//! Reads the existing model files and says what is in them.
//!
//! Two kinds of file sit in `data/`:
//!
//! * `*.chroma` written by [`phiecs::phasor::facet::storage::Storage`] — a facet. The shipped
//!   `manifold.chroma` is format v1 with a 20-byte phasor (`phase`, `amplitude`,
//!   `band_n`): one angle per word, no channels, no evidence. Until the v1
//!   reader was rewritten to parse by hand it did not load at all.
//! * `memory.chroma` written by [`crate::memory::Memo`] — a context-wave log,
//!   not a facet, despite the extension.
//!
//! [`inspect`] opens either, and for a facet reports what the promoted lexicon
//! can see in it: how many words carry evidence, how the angles are spread per
//! channel, what the relation inventory holds, and what the n-gram tables hold.

mod version;
pub use version::*;

use phiton::units::PHASE_CHANNELS;
use phiecs::phasor::facet::Facet;
use crate::memory::Memo;
use phiecs::phasor::facet::storage::Storage;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, Clone)]
pub enum ModelKind {
    Facet,
    Memo,
    Unknown,
}

/// What a model file holds, at the level the operator cares about.
#[derive(Debug, Clone)]
pub struct ModelReport {
    pub path: String,
    pub kind: ModelKind,
    pub bytes: u64,
    pub format_version: u32,
    pub words: usize,
    pub evidenced_words: usize,
    pub bigram_contexts: usize,
    pub trigram_contexts: usize,
    pub phase_lags: usize,
    pub marker_channels: usize,
    /// Dispersion of channel 0 (the one angle a v1 model has) over the whole
    /// lexicon, and over words seen 25+ times.
    pub dispersion_ch0: f64,
    pub dispersion_ch0_frequent: f64,
    /// Mean dispersion over channels 1..64, which for a v1 model measures the
    /// identity seeding and nothing learned.
    pub dispersion_other: f64,
    pub amplitude_mean: f64,
    pub amplitude_max: f64,
    pub band_n_max: u32,
    pub memo_entries: usize,
    pub notes: Vec<String>,
}
