#[allow(unused_imports)]
use super::*;

/// Permutes fillers among pairs, destroying the relations and keeping everything
/// else.
///
/// A derangement is attempted — a pair mapped to its own filler would leave a
/// real relation intact inside the null — but a single-pair input has no
/// derangement, and that case is returned unchanged rather than pretended about.
pub(super) fn permute_fillers(pairs: &[(String, String)], seed: u64) -> Vec<(String, String)> {
    let mut fillers: Vec<String> = pairs.iter().map(|p| p.1.clone()).collect();
    let mut r = seed | 1;
    for i in (1..fillers.len()).rev() {
        r ^= r << 13;
        r ^= r >> 7;
        r ^= r << 17;
        let j = (r % (i as u64 + 1)) as usize;
        fillers.swap(i, j);
    }
    pairs
        .iter()
        .zip(fillers)
        .map(|(p, f)| (p.0.clone(), f))
        .collect()
}
