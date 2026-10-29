mod discarder;
#[allow(unused_imports)]
use discarder::*;

use crate::generate::compose::better::SectorScore;
use crate::generate::compose::SectorPalette;
use crate::generate::constants as config;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use crate::train::trainer::Trainer;

/// Discarder — the "worse" half of the reviewing process.
///
/// In Huberman's 1968 design `worse` is a **guard**: the search is never
/// expanded through a position that gives ground, and stage 0 is unconditionally
/// worse. In MENACE, losing removes beads. In Samuel's checkers player, a losing
/// Alpha has its weight changes rejected. In every case the negative half is the
/// mechanism, not a refinement — remove it from MENACE and the machine converges
/// on uniform random play.
///
/// This discarder therefore does two things the previous version did not:
///
/// 1. **Guards.** A stage-0 variant is never a survivor, whatever it scored.
/// 2. **Penalises.** Losing compositions are rotated *away* from their own
///    centroid, so the manifold separates instead of concentrating.
///
/// The earlier implementation trained only on winners and reasoned that losers
/// would "naturally drift apart". They do not: `train_sentence`'s attraction
/// term has no counterpart, so unreinforced words are not pushed anywhere — they
/// merely stop being pulled, while winners are pulled together every round.
pub struct Discarder {
    /// Number of top sectors to keep and train on.
    pub keep_count: usize,
    /// Number of bottom sectors to discard and penalise.
    pub discard_count: usize,
    /// Repulsion applied to losers, relative to the trainer's learning rate.
    pub repulsion: f64,
}

/// DiscardResult - the outcome of a discard round.
#[derive(Debug, Clone)]
pub struct DiscardResult {
    /// Sectors that were kept (trained on).
    pub kept: Vec<u16>,
    /// Sectors that were discarded and penalised.
    pub discarded: Vec<u16>,
    /// Sectors rejected by the stage guard before scoring mattered.
    pub guarded: Vec<u16>,
    /// The texts that were trained on.
    pub trained_texts: Vec<String>,
    /// Number of tokens updated during positive training.
    pub tokens_updated: usize,
    /// Number of tokens pushed away during negative training.
    pub tokens_penalised: usize,
}
