//! The context kernel: each channel's decay and frequency, the frequency
//! set by the channel primes. It was phiano's `config::channel_kernel`;
//! phiano's `config` re-exports it and its settings from here.

use crate::train::constants::{CHANNEL_PRIMES, PRIME_MODULUS};
use phiton::units::TWO_PI;

/// Base decay per token for the recurrent context state, |λ| < 1.
pub const CONTEXT_LAMBDA: f64 = 0.92;

/// Base rotation per token for the recurrent context state (ω).
/// Channel k uses λ_k and ω_k spread geometrically from these bases, giving
/// each channel its own timescale — a diagonal complex linear recurrence.
pub const CONTEXT_OMEGA: f64 = 0.11;

/// Use prime-spaced channel frequencies instead of the linear ramp.
pub const PRIME_CHANNEL_SPACING: bool = true;

/// The recurrence kernel for channel `k`: decay and rotation per step.
///
/// One definition, used by the language model, the sentence benchmark and the
/// generation buffer, so the three cannot silently drift apart — they had three
/// copies of the same formula before this.
///
/// The rotation is where [`CHANNEL_PRIMES`] enters. A linear frequency ramp
/// makes channel *k* and channel *2k* commensurate: they share periods and
/// carry overlapping information, so the effective channel count is smaller
/// than 64. Prime-proportional frequencies are pairwise coprime, so no two
/// channels share a period until their product.
///
/// Decay stays a smooth ramp: `lambda` sets how far back a channel remembers,
/// and there is no aliasing argument for spacing memory horizons by primes.
pub fn channel_kernel(k: usize, channels: usize) -> phistd::c64 {
    let frac = k as f64 / channels.max(1) as f64;
    let lambda = CONTEXT_LAMBDA.powf(1.0 + 3.0 * frac);
    let omega = match PRIME_CHANNEL_SPACING {
        true => {
            let p = CHANNEL_PRIMES[k % CHANNEL_PRIMES.len()] as f64;
            TWO_PI * p / PRIME_MODULUS
        }
        false => CONTEXT_OMEGA * (1.0 + 4.0 * frac),
    };
    phistd::c64::from_polar(lambda, omega)
}
