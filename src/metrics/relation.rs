//! Relation accuracy: does the manifold place related words in related positions?
//!
//! Perplexity says whether the model predicts text. It says nothing about
//! whether `woman` sits near `man`, or whether the step from `man` to `woman`
//! is the same step as from `grandfather` to `grandmother`. Those are the
//! questions a *semantic* representation has to answer, and they have clean
//! chance baselines, so the answers can be wrong.
//!
//! Three tests, in increasing difficulty:
//!
//! 1. **Pair versus random** — is `resonance(a, b)` greater than `resonance(a, r)`
//!    for a random `r`? Chance is 50%. Failing this means related words are not
//!    even loosely grouped.
//! 2. **Neighbourhood** — is `b` among the `k` nearest words to `a`?
//!    Chance is `k / V`.
//! 3. **Analogy** — `a : b :: c : d`. The relation is the per-channel phase
//!    offset `θ_b − θ_a`; applying it to `c` should land on `d`. This is
//!    unbind-then-bind in the phase domain, and it is the real test of whether
//!    a *relation* is represented rather than a mere cluster. Chance is `1 / V`.

mod benchmark;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use benchmark::*;

use phiecs::phasor::facet::Facet;
use rayon::prelude::*;
use phiecs::phasor::spectral::{fnv1a, SpectralPhasor};
use serde::{Deserialize, Serialize};

/// Random comparisons drawn per pair for the similarity test.
///
/// A single draw per pair gives ~23 Bernoulli trials across the whole probe
/// set — a standard error near 10%, enough for the figure to swing 30 points
/// between runs of the same experiment. Averaging many draws makes the number
/// worth comparing.
const RANDOM_DRAWS: usize = 64;

/// One ordered pair standing in some relation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationPair {
    pub a: String,
    pub b: String,
}

/// A group of pairs sharing the same relation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationFamily {
    pub name: String,
    pub pairs: Vec<RelationPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyResult {
    pub name: String,
    /// Pairs whose members were both in the lexicon.
    pub usable_pairs: usize,
    /// Fraction of (pair, random word) comparisons the pair wins. Chance = 0.5.
    /// Averaged over `RANDOM_DRAWS` draws per pair, so it is stable enough to
    /// compare between runs.
    pub pair_vs_random: f64,
    /// Fraction of pairs where `b` is among the 10 nearest to `a`.
    pub neighbour_top10: f64,
    /// Fraction of pairs where `b` is among the 50 nearest to `a`.
    pub neighbour_top50: f64,
    /// Analogy accuracy at rank 1 and rank 5, and mean reciprocal rank.
    pub analogy_top1: f64,
    pub analogy_top5: f64,
    pub analogy_mrr: f64,
    pub analogies_tested: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationReport {
    pub vocabulary_size: usize,
    pub families: Vec<FamilyResult>,
    /// Chance rate for the neighbourhood test at k = 10.
    pub chance_neighbour_top10: f64,
    /// Chance rate for analogy at rank 1.
    pub chance_analogy_top1: f64,
    pub overall_pair_vs_random: f64,
    pub overall_analogy_top1: f64,
}

pub struct RelationBenchmark;

/// Partner pairs sampled per pair when scoring analogies.
///
/// Exhaustive is n(n-1) full-vocabulary rankings per family. Sampling keeps the
/// benchmark affordable at 305 pairs; it is deterministic, so the comparison
/// between conditions is still exact.
const MAX_ANALOGY_PARTNERS: usize = 8;

/// A word's minimum training count for it to enter the candidate pool.
///
/// Every relational headline in this project was computed over the full 70k
/// vocabulary, of which most words were seen fewer than five times. A word seen
/// four times has phases essentially equal to its hash seed, so those words
/// contribute noise to every ranking and pull every average down.
///
/// Restricting the pool also *shrinks* it, which makes the task easier — so the
/// pool size and chance level are reported at every floor and the comparison is
/// ratio-to-chance, never a raw score against a raw score.
pub type CountFloor = u32;
