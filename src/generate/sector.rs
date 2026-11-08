//! Sector arithmetic and composition weights (moved from
//! `phiano::config::functions`, 14.2).
//!
//! Each function reads a constant the engine crates own. `sector_prime` and
//! `sectors_per_color` stay in `phiano::config` until `PRIME_*` and
//! `COLOR_BANDS` reach `phistd` (14.4).

use super::constants::{
    WEIGHT_ALIGNMENT, WEIGHT_COHERENCE, WEIGHT_COVERAGE, WEIGHT_DIVERSITY, WEIGHT_NOVELTY,
    WEIGHT_RESONANCE,
};
use phiecs::phasor::facet::{SECTOR_EXPONENT_MIN, SECTOR_RESOLUTION};

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
    fn the_weights_sum_to_one() {
        assert!((total_compose_weight() - 1.0).abs() < 1e-9);
    }
}
