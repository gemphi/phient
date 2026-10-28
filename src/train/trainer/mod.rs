mod build;
mod sentence;
mod predictive;
mod ngrams;
mod chain;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use build::*;
#[allow(unused_imports)]
use sentence::*;
#[allow(unused_imports)]
use predictive::*;
#[allow(unused_imports)]
use ngrams::*;
#[allow(unused_imports)]
use chain::*;

pub mod metrics;
pub use metrics::{TrainingMetric, MultiEpochResult};

use phiton::units::{TWO_PI, PHASE_CHANNELS};
use crate::train::constants::{PHASE_REPULSION, CONVERGENCE_THRESHOLD, CORRECTION_FLOOR, CHANNELS_PER_UPDATE, NEG_SAMPLES, NEG_RATE, RARE_TARGET_FLOOR, RARE_TARGET_BOOST, HINGE_MARGIN, FUNCTION_WORD_WEIGHT, RELATION_RATE};
use phiecs::phasor::spectral::AMPLITUDE_MAX;
use phiecs::phasor::facet::{AMPLITUDE_INITIAL, BAND_N_INITIAL, RELATION_WINDOW};
use phiecs::phasor::facet::Facet;
use phiecs::phasor::spectral::{fnv1a, SpectralPhasor};
use phiton::p64::words::Tokenizer;

/// Trainer — contrastive Kuramoto-Sakaguchi learning on the phase torus.
///
/// Each training step has two halves:
///
/// * **Attraction** — words co-occurring in a sentence are pulled toward the
///   sentence's per-channel centroid, with an asymmetric lag encoding word order.
/// * **Repulsion** — sampled non-co-occurring words are pushed away.
///
/// The second half is not an optimisation. Kuramoto coupling with all-positive
/// coupling has one globally stable attractor — total synchronisation — so an
/// attraction-only rule drives the lexicon to a single point, at which the
/// Kuramoto order parameter (reported as `coherence`) reads 1.0 for every input
/// including noise. The negative term is what makes the fixed point track
/// pointwise mutual information instead.
#[derive(Clone)]
pub struct Trainer {
    /// Kuramoto learning rate - controls how fast phases converge.
    pub learning_rate: f64,
    /// Negative samples drawn per token. Zero disables repulsion.
    pub neg_samples: usize,
    /// When present, negative samples are filtered against it so a word from
    /// the anchor's own definition is never used as a negative. `None` keeps
    /// uniform sampling, which is the control the filter has to beat.
    pub definitions: Option<std::sync::Arc<crate::train::conception::DefinitionGraph>>,
    /// Mixed into every stochastic decision so a run can be repeated exactly
    /// *and* varied deliberately.
    ///
    /// Training became reproducible when the sample pool stopped depending on
    /// HashMap order, but reproducible is only half of what a measurement needs:
    /// one number from one deterministic run has no error bar, and the effects
    /// now being reported are small enough that the interval decides them.
    /// Varying this and holding everything else fixed is what turns a single
    /// figure into a distribution.
    pub seed: u64,
    /// Couple each observed pair on the channel of the relation it was observed
    /// under, and deposit evidence there. See [`Trainer::couple_relations`].
    /// Off reproduces the pre-evidence manifold exactly, which is the control.
    pub relations: bool,
    /// How relation-channel nudges land on their byte. Nearest by default;
    /// [`Rounding::Dither`] lets sub-quantum steps accumulate (docs/83).
    pub rounding: phiecs::phasor::spectral::Rounding,
}
