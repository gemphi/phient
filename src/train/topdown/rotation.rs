#[allow(unused_imports)]
use super::*;

/// Positional rotation per channel, for the order-sensitive encoding.
///
/// Shared across words, so position is a direction rather than a scramble — the
/// same reasoning as role rotation in [`crate::train::roles`].
#[inline]
pub(super) fn position_rotation(k: usize) -> f64 {
    let p = crate::train::constants::CHANNEL_PRIMES[k % crate::train::constants::CHANNEL_PRIMES.len()] as f64;
    TWO_PI * p / crate::train::constants::PRIME_MODULUS
}

impl Up {
    pub fn label(self) -> &'static str {
        match self {
            Up::Bag => "bag",
            Up::Bound => "bound",
        }
    }
}

impl Descent {
    /// Contribution of one word at one position to its sentence's state.
    #[inline]
    pub(super) fn contribution(p: &SpectralPhasor, pos: usize, k: usize, up: Up) -> c64 {
        let rot = match up {
            Up::Bag => 0.0,
            Up::Bound => pos as f64 * position_rotation(k),
        };
        c64::from_polar(p.amplitude, p.theta(k) + rot)
    }

    /// Runs the up-down cycle and keeps the result only if the guard allows it.
    ///
    /// `contrast` subtracts the corpus-mean sentence state from every target.
    /// It is a parameter rather than a constant because "does contrast prevent
    /// the collapse" is the question this module exists to answer, and an
    /// answer needs both arms.
    pub fn cycle(
        facet: &mut Facet,
        sentences: &[Vec<String>],
        up: Up,
        rounds: usize,
        step: f64,
        contrast: bool,
    ) -> DescentReport {
        Self::cycle_with(facet, sentences, up, rounds, step, contrast, DEFAULT_CONCENTRATION)
    }
}
