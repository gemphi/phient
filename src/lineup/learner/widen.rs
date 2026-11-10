//! Widening: more channels for a learner that has already learned, so
//! capacity can grow with the vocabulary and nothing learned is lost
//! (docs/210). A checkpoint fixes its width — its planes are that wide —
//! and 64 channels over 25,806 words was the constraint docs/209 found.
//!
//! The old channels are kept bit for bit. The new ones are born as a word's
//! channels are born at `intern`: at the word's seed angles, on the unit
//! circle. The operator rows for the new channels are born as `new` bears
//! them, the eighth-turn on the diagonal — but at a THOUSANDTH of a fresh
//! learner's amplitude, and so the pair's new entries: a fresh learner's
//! quiet is loud beside a trained learner's scores (born at it, the new
//! channels took a read of perplexity 197 to 336 before any training). At
//! `QUIET` they add nothing a read can see, the learner reads as it did,
//! and the error moves the new channels from there. The new operator rows
//! are born with their siblings' Adagrad experience — the mean accumulator
//! of the old rows at that distance — so their first steps are as small as
//! the trained rows', not a fresh row's leap. The word rows' accumulators
//! are per row and carry over.

use super::plane::Plane;
use super::Learner;

/// What a new channel is born at, as a share of a fresh learner's birth.
const QUIET: f64 = 1e-3;

impl Learner {
    /// Widens the learner to `channels`. Refused — `false`, nothing changed
    /// — unless that is wider than now and a multiple of the block.
    pub fn widen(&mut self, channels: usize) -> bool {
        let (d, b) = (self.config.channels, self.config.block);
        if channels <= d || channels % b != 0 {
            return false;
        }
        let n = channels;
        // Each word's three rows: the old d entries, then the new ones at
        // the word's seed angles for the wider width.
        let words = self.word.clone();
        let tie = self.config.tie;
        let grow = |plane: &mut Plane, role: usize| {
            let (mut re, mut im) = (Vec::with_capacity(words.len() * n), Vec::with_capacity(words.len() * n));
            for (i, w) in words.iter().enumerate() {
                re.extend_from_slice(&plane.re[i * d..(i + 1) * d]);
                im.extend_from_slice(&plane.im[i * d..(i + 1) * d]);
                let a = phistd::seed_angles(w, 3 * n);
                for &angle in &a[role * n + d..(role + 1) * n] {
                    re.push(angle.cos() as f32);
                    im.push(angle.sin() as f32);
                }
            }
            plane.re = re;
            plane.im = im;
            plane.d = n;
        };
        grow(&mut self.reach, 0);
        if !tie {
            grow(&mut self.reached, 1);
        }
        grow(&mut self.cond, 2);
        // The operator: row `dist · n + i` for every distance and channel,
        // the old rows carried over, the new born quiet.
        let mut op = Plane::new(b);
        let eighth = std::f64::consts::TAU / 8.0;
        let quiet = (4.0 / n as f64).sqrt() * QUIET;
        for dist in 0..self.config.context {
            let (c, s) = ((eighth * (dist + 1) as f64).cos(), (eighth * (dist + 1) as f64).sin());
            let amp = quiet / (dist + 1) as f64;
            let experience = self.op.acc[dist * d..(dist + 1) * d].iter().sum::<f32>() / d as f32;
            for i in 0..n {
                match i < d {
                    true => {
                        let (mr, mi) = self.op.row((dist * d + i) as u32);
                        op.re.extend_from_slice(mr);
                        op.im.extend_from_slice(mi);
                        op.acc.push(self.op.acc[dist * d + i]);
                    }
                    false => {
                        op.push((0..b).map(|_| 0.0), 0.0);
                        let at = (dist * n + i) * b + i % b;
                        op.re[at] = (amp * c) as f32;
                        op.im[at] = (amp * s) as f32;
                        *op.acc.last_mut().expect("the row just pushed") = experience;
                    }
                }
            }
        }
        // The new rows carry the old rows' experience, so they count as
        // touched now; the old rows' clocks go with them.
        op.touch_all(self.told.len() as u32);
        self.op = op;
        // The pair: its new entries at a thousandth of its birth amplitude.
        self.pair.re.extend(std::iter::repeat((0.1 * QUIET) as f32).take(n - d));
        self.pair.im.extend(std::iter::repeat(0.0).take(n - d));
        self.pair.d = n;
        self.config.channels = n;
        true
    }
}
