#[allow(unused_imports)]
use super::*;

impl Groundable for ChunkStore {
    fn entries(&self) -> Vec<(String, String)> {
        self.load_all()
    }
    fn source_name(&self) -> &str {
        "dictionary"
    }
}

/// Global and frequent-band dispersion, in that order.
pub fn dispersion_pair(facet: &Facet) -> (f64, f64) {
    (facet.phase_dispersion(), facet.dispersion_top(GUARD_BAND_TOP))
}
