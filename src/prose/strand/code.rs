//! Description length: how many nats a run of tokens costs to say, and
//! what dividing a span saves. A segment is coded token by token with
//! Witten–Bell escape (PPM-C) to its parent's distribution:
//! `p(t) = (c(t) + T · p_parent(t)) / (i + T)`, `T` the types seen so far,
//! the first token at `p_parent`. Dividing a span of `n` tokens saves its
//! code length as one segment minus the halves' as two, and costs what it
//! takes to say the division: the two tree flags it adds, `2 ln 2`, and
//! where it falls. Where is said with the story's own marks as the
//! codebook: one bit for whether the cut follows a mark the story wrote
//! (a full stop, a comma, a question mark — any token that is not a word),
//! then which of those gaps, or which of the others, uniformly: `ln 2 +
//! ln m` after a mark, `ln 2 + ln(n − 1 − m)` elsewhere, `ln(n − 1)` when
//! one class is empty. A division is cheaper to say where the story
//! already marked one; nothing is assumed about which marks.

use super::cut::Cut;
use std::collections::HashMap;

/// Code length in nats of `ids` predicted in order with Witten–Bell escape
/// to `base` (one entry per token).
pub(super) fn code_length(ids: &[u32], base: &[f64], seen: &mut [u32], touched: &mut Vec<u32>) -> f64 {
    let (mut len, mut types) = (0.0, 0.0);
    for (i, (id, b)) in ids.iter().zip(base).enumerate() {
        let c = seen[*id as usize] as f64;
        let p = if i == 0 { *b } else { (c + types * b) / (i as f64 + types) };
        len -= p.ln();
        if c == 0.0 {
            types += 1.0;
            touched.push(*id);
        }
        seen[*id as usize] += 1;
    }
    for id in touched.drain(..) {
        seen[id as usize] = 0;
    }
    len
}

/// A span's Witten–Bell distribution over its words, given its parent's share of each.
pub(super) fn share_of(counts: &HashMap<u32, u32>, n: usize, above: &dyn Fn(u32) -> f64) -> HashMap<u32, f64> {
    let t = counts.len() as f64;
    counts.iter().map(|(id, c)| (*id, (*c as f64 + t * above(*id)) / (n as f64 + t))).collect()
}

impl Cut {
    /// The span's most informative division and its net gain, if it has two tokens.
    pub(super) fn best_division(&self, s: usize, prior: &dyn Fn(u32) -> f64, seen: &mut [u32], touched: &mut Vec<u32>) -> Option<(f64, usize)> {
        let sp = &self.spans[s];
        let n = sp.len();
        if n < 2 {
            return None;
        }
        let ids = &self.ids[sp.start..sp.end];
        let base: Vec<f64> = ids.iter().map(|id| self.share(sp.parent, *id, prior)).collect();
        let whole = code_length(ids, &base, seen, touched);
        let ln2 = std::f64::consts::LN_2;
        let k = n - 1;
        let m = (1..n).filter(|j| self.marks[sp.start + j - 1]).count();
        let cost = |j: usize| {
            2.0 * ln2
                + if m == 0 || m == k {
                    (k as f64).ln()
                } else if self.marks[sp.start + j - 1] {
                    ln2 + (m as f64).ln()
                } else {
                    ln2 + ((k - m) as f64).ln()
                }
        };
        (1..n)
            .map(|j| (whole - code_length(&ids[..j], &base[..j], seen, touched) - code_length(&ids[j..], &base[j..], seen, touched) - cost(j), sp.start + j))
            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
    }

}
