//! A word's sector against the lexicon mean, read from the facet alone:
//! what a persona's fingerprint is a histogram of (phiano's `persona`), and
//! what the generator's persona prior reads a candidate by. They sat on
//! phiano's `Fingerprint`; they need only the lexicon, so they live here,
//! beside `encode_side`.

use super::encode_side;
use phiecs::phasor::facet::Facet;
use phiecs::phasor::wave::Wave;
use phistd::c64;
use phiton::units::PHASE_CHANNELS;

/// The mean target-side function of the evidenced lexicon: the bias every
/// word shares. Sectors are read on the *centred* function (word minus
/// this mean), because the raw centroids of a whole lexicon bunch into a
/// few sectors (45–49 on Austen, docs/74) and a histogram over them
/// cannot tell authors apart. Centring is what a channel-balanced sector
/// means: what this word does that the average word does not.
pub fn lexicon_mean(facet: &Facet) -> Vec<c64> {
    let mut sum = vec![c64::new(0.0, 0.0); PHASE_CHANNELS];
    let mut n = 0usize;
    for p in facet.lexicon.values() {
        if p.has_evidence() {
            for (a, b) in sum.iter_mut().zip(encode_side(p, true)) {
                *a += b;
            }
            n += 1;
        }
    }
    if n > 0 {
        for z in sum.iter_mut() {
            *z /= n as f64;
        }
    }
    sum
}

/// The sector a word occupies on the relation circle: the Q64 sector of
/// the centroid of its evidence-weighted *target-side* function — the
/// side a writer's choices put it under — centred on the lexicon mean,
/// when the lexicon carries relation evidence; the legacy single phase
/// otherwise. The single phase (channel 0, source side) collapsed for
/// frequent words (docs/69) and every persona read the same four sectors
/// from it; the uncentred centroid bunched into sectors 45–49 (docs/74).
pub fn relational_sector(facet: &Facet, word: &str, mean: &[c64]) -> Option<u16> {
    let p = facet.lexicon.get(word)?;
    if !p.has_evidence() {
        return Wave::word_sector(facet, word);
    }
    let f: Vec<c64> = encode_side(p, true).iter().zip(mean).map(|(a, m)| a - m).collect();
    Some(phistd::hexagram::hexagram_of(&f) as u16)
}
