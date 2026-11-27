use super::super::*;

/// What a context word brings to the sum: the share of its amplitude, and
/// the roll its place puts on its phase. A weight leaves the two words in
/// one superposition; a roll binds each to where it stands.
struct Brought {
    weight: f64,
    roll: f64,
}

impl Brought {
    /// A share of the word's amplitude, its phase as it stands.
    fn weighed(weight: f64) -> Self {
        Self { weight, roll: 0.0 }
    }

    /// The whole of it, rolled by the golden angle for a place `at` words
    /// before the one being predicted.
    fn at(place: f64) -> Self {
        Self { weight: 1.0, roll: place * phistd::GOLDEN_ANGLE }
    }
}

impl<'a> PhianoLM<'a> {

    /// Per-channel context direction from the two preceding words.
    pub(in crate::metrics::harness) fn context_vec(&self, a: &str, b: &str) -> Option<Vec<f64>> {
        self.summed([(a, Brought::weighed(0.4)), (b, Brought::weighed(1.0))])
    }

    /// Order-bound context vector.
    ///
    /// [`PhianoLM::context_vec`] weights the two context words 0.4 and 1.0 and
    /// sums them. That is a recency weighting and not an encoding of order -
    /// the two words land in one superposition - but it is not order-blind
    /// either, as this was once documented to be: each channel is normalised
    /// back to the unit circle afterwards, which keeps only a direction, and
    /// the heavier word turns it. Measured by `order_sensitivity` on the toy
    /// corpus it reads 0.453, where blindness would read 1.0. Order is in
    /// there as a by-product of the normalisation, not as anything the
    /// construction means.
    ///
    /// Here each word is *rotated* by its offset from the prediction point
    /// before it is summed — position −2 by −2·φ, position −1 by −1·φ, where φ
    /// is the golden angle. Rotation by an irrational multiple never collides
    /// across positions, so the sum is a binding rather than a blur, and it is
    /// relative to the target, so the encoding does not drift with sentence
    /// length. Measured the same way it reads -0.481: a swap gives a context
    /// pointing the other way, where the weighting gives one merely turned.
    ///
    /// This is the same construction [`phiecs::phasor::wave::Wave::sentence_channels`]
    /// uses with `bound = true`. It was in the codebase and not in the scoring
    /// path.
    pub(in crate::metrics::harness) fn context_vec_bound(&self, a: &str, b: &str) -> Option<Vec<f64>> {
        self.summed([(a, Brought::at(-2.0)), (b, Brought::at(-1.0))])
    }

    /// The two context words summed into the channels, each channel then
    /// normalised back to the unit circle. Both constructions above are
    /// this sum; they differ only in what each word brings to it.
    fn summed(&self, words: [(&str, Brought); 2]) -> Option<Vec<f64>> {
        let mut acc = vec![0.0f64; 2 * LM_CHANNELS];
        let mut any = false;
        for (w, brought) in words {
            let Some(p) = self.facet.lexicon.get(w) else {
                continue;
            };
            any = true;
            let m = p.amplitude * brought.weight;
            for k in 0..LM_CHANNELS {
                let t = p.theta(k) + brought.roll;
                acc[2 * k] += m * t.cos();
                acc[2 * k + 1] += m * t.sin();
            }
        }
        match any {
            false => None,
            true => {
                for k in 0..LM_CHANNELS {
                    let (x, y) = (acc[2 * k], acc[2 * k + 1]);
                    let n = x.hypot(y);
                    let (cos, sin) = match n > 1e-12 {
                        true => (x / n, y / n),
                        false => (1.0, 0.0),
                    };
                    (acc[2 * k], acc[2 * k + 1]) = (cos, sin);
                }
                Some(acc)
            }
        }
    }

    /// How much a context construction changes when the two words are swapped.
    ///
    /// Returns mean cosine similarity between `ctx(a, b)` and `ctx(b, a)` over
    /// the given pairs, averaged across channels. 1.0 means order is not
    /// represented at all; lower means a swap produces a different context.
    ///
    /// This is the diagnostic that separates "order is encoded" from "order is
    /// weighted": a recency weight leaves the two words in the same
    /// superposition and scores near 1.0 however the weights are set.
    pub fn order_sensitivity(&self, pairs: &[(String, String)], kind: ContextKind) -> f64 {
        let build = |a: &str, b: &str| match kind {
            ContextKind::Bound => self.context_vec_bound(a, b),
            _ => self.context_vec(a, b),
        };
        let mut sum = 0.0;
        let mut n = 0usize;
        for (a, b) in pairs {
            if let (Some(x), Some(y)) = (build(a, b), build(b, a)) {
                let mut c = 0.0;
                for k in 0..LM_CHANNELS {
                    c += x[2 * k] * y[2 * k] + x[2 * k + 1] * y[2 * k + 1];
                }
                sum += c / LM_CHANNELS as f64;
                n += 1;
            }
        }
        match n {
            0 => 1.0,
            _ => sum / n as f64,
        }
    }

    /// Softmax over the vocabulary of mean phase coherence with the context.
    /// Returns the probability assigned to `target_idx`.
    pub(in crate::metrics::harness) fn p_phase(&self, ctx: &[f64], target_idx: usize) -> f64 {
        let v = self.unigram.len();
        let mut scores = Vec::with_capacity(v);
        let mut max = f64::NEG_INFINITY;

        let rk = self
            .readout
            .as_ref()
            .map(|r| r.key(&Self::ctx_angles(ctx)));

        for i in 0..v {
            let base = i * 2 * LM_CHANNELS;
            let mut dot = 0.0;
            for k in 0..LM_CHANNELS {
                dot += self.vecs[base + 2 * k] * ctx[2 * k]
                    + self.vecs[base + 2 * k + 1] * ctx[2 * k + 1];
            }
            let mut dot = dot / LM_CHANNELS as f64;
            if let (Some(r), Some(key)) = (&self.readout, rk) {
                dot += READOUT_WEIGHT * r.bias_for(key, self.sectors[i]);
            }
            let s = PHASE_BETA * dot;
            if s > max {
                max = s;
            }
            scores.push(s);
        }

        let z = phistd::prob::softmax_z(scores.iter().copied(), max, 1.0);
        if z <= 0.0 {
            return 1.0 / v as f64;
        }
        phistd::prob::softmax_p(scores[target_idx], max, 1.0, z)
    }
}
