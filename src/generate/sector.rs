//! Sector arithmetic and composition weights (moved from
//! `phiano::config::functions`, 14.2).
//!
//! Each function reads a constant the engine crates own. `sector_prime` and
//! `sector_per_color` followed `PRIME_*` and `COLOR_BANDS` into
//! `phistd::constant` (14.4).

use super::constants::{
    WEIGHT_ALIGNMENT, WEIGHT_COHERENCE, WEIGHT_COVERAGE, WEIGHT_DIVERSITY, WEIGHT_NOVELTY,
    WEIGHT_RESONANCE,
};
use phiecs::phasor::facet::{SECTOR_EXPONENT_MIN, SECTOR_RESOLUTION};
use phistd::constant::{COLOR_BANDS, PRIME_1024, PRIME_128, PRIME_256, PRIME_512, PRIME_64};

/// Returns the configured sector resolution as a `u16`.
///
/// # Panics
///
/// Panics if `SECTOR_RESOLUTION` is not a power of two with exponent
/// ≥ `SECTOR_EXPONENT_MIN`.
pub fn sector_resolution() -> u16 {
    let s = SECTOR_RESOLUTION;
    if s < (1 << SECTOR_EXPONENT_MIN) || !s.is_power_of_two() {
        panic!(
            "SECTOR_RESOLUTION must be a power of 2 with exponent >= {} (min {}), got {}",
            SECTOR_EXPONENT_MIN,
            1u16 << SECTOR_EXPONENT_MIN,
            s,
        );
    }
    s
}

/// Returns the exponent `n` such that `SECTOR_RESOLUTION = 2ⁿ`.
pub fn sector_exponent() -> u32 {
    sector_resolution().trailing_zeros()
}

/// Returns the prime mixing constant for the current sector resolution.
///
/// Primes are coprime to power-of-2 sector counts, preventing aliasing
/// artifacts in phase-to-sector mapping.
///
/// # Invariant
///
/// The returned value `p` satisfies `gcd(p, SECTOR_RESOLUTION) == 1`.
pub fn sector_prime() -> u64 {
    match SECTOR_RESOLUTION {
        64 => PRIME_64,
        128 => PRIME_128,
        256 => PRIME_256,
        512 => PRIME_512,
        1024 => PRIME_1024,
        _ => PRIME_64,
    }
}

/// Returns the number of sectors per colour band.
///
/// Each colour band spans `SECTOR_RESOLUTION / COLOR_BANDS` contiguous
/// sectors in the phase ring.
pub fn sector_per_color() -> u16 {
    sector_resolution() / COLOR_BANDS
}
/// Number of sector variations generated per composition round.
pub fn compose_variations() -> usize {
    sector_resolution() as usize
}

/// Total composition weight (for normalisation).
///
/// Returns the sum of all `WEIGHT_*` constants:
/// `WEIGHT_COHERENCE + WEIGHT_NOVELTY + WEIGHT_RESONANCE
///  + WEIGHT_DIVERSITY + WEIGHT_COVERAGE + WEIGHT_ALIGNMENT`.
pub fn total_compose_weight() -> f64 {
    WEIGHT_COHERENCE + WEIGHT_NOVELTY + WEIGHT_RESONANCE
        + WEIGHT_DIVERSITY + WEIGHT_COVERAGE + WEIGHT_ALIGNMENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sector_functions_read_the_resolution_constant() {
        assert_eq!(sector_resolution(), SECTOR_RESOLUTION);
        assert_eq!(1u16 << sector_exponent(), SECTOR_RESOLUTION);
        assert_eq!(compose_variations(), SECTOR_RESOLUTION as usize);
    }

    #[test]
    fn the_prime_is_odd_and_the_bands_divide_the_sectors() {
        assert_eq!(sector_prime() % 2, 1);
        assert_eq!(sector_per_color() * COLOR_BANDS, SECTOR_RESOLUTION);
    }

    #[test]
    fn the_weights_sum_to_one() {
        assert!((total_compose_weight() - 1.0).abs() < 1e-9);
    }
}
