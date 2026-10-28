use super::super::*;

impl<'a> PhianoLM<'a> {

    pub(in crate::metrics::harness) fn advance(&self, h: &mut [phistd::c64], token: &str) {
        for (k, z) in h.iter_mut().enumerate() {
            *z *= Self::kernel(k);
        }
        if let Some(p) = self.facet.lexicon.get(token) {
            for (k, z) in h.iter_mut().enumerate().take(LM_CHANNELS) {
                *z += phistd::c64::from_polar(p.amplitude, p.theta(k));
            }
        }
    }

    /// Unit-normalises a recurrent state into a context vector.
    pub(in crate::metrics::harness) fn state_to_ctx(h: &[phistd::c64]) -> Vec<f64> {
        let mut out = vec![0.0f64; 2 * LM_CHANNELS];
        for k in 0..LM_CHANNELS {
            let n = h[k].norm();
            if n > 1e-12 {
                out[2 * k] = h[k].re / n;
                out[2 * k + 1] = h[k].im / n;
            } else {
                out[2 * k] = 1.0;
            }
        }
        out
    }

    /// Raw coherence score of every vocabulary item against a context vector.
    pub(in crate::metrics::harness) fn scores(&self, ctx: &[f64], buf: &mut Vec<f64>) {
        buf.clear();
        let v = self.unigram.len();
        for i in 0..v {
            let base = i * 2 * LM_CHANNELS;
            let mut dot = 0.0;
            for k in 0..LM_CHANNELS {
                dot += self.vecs[base + 2 * k] * ctx[2 * k]
                    + self.vecs[base + 2 * k + 1] * ctx[2 * k + 1];
            }
            // Mean cosine across channels, not the sum. A sum over 16 channels
            // spans [-16, 16]; exponentiating a spread that wide saturates the
            // softmax, every non-top candidate underflows the probability
            // floor, and the phase distribution degenerates into a constant.
            // A constant back-off can only ever lose to a unigram, which is how
            // a saturated scale masquerades as "the manifold contributes
            // nothing". Dividing by the channel count puts the score in
            // [-1, 1] and hands the temperature back to beta, where it belongs.
            buf.push(dot / LM_CHANNELS as f64);
        }

        // The non-linear correction, when fitted. It is added *after* the linear
        // dot, and it depends on both the context cell and the candidate's
        // sector — a bias depending on only one of the two could not reorder
        // anything.
        if let Some(r) = &self.readout {
            let key = r.key(&Self::ctx_angles(ctx));
            for (i, s) in buf.iter_mut().enumerate() {
                *s += READOUT_WEIGHT * r.bias_for(key, self.sectors[i]);
            }
        }
    }

    /// Full experiment sweep: context construction × temperature × mixing weight.
    ///
    /// One pass per context kind. The coherence scores are computed once per
    /// position and reused across every temperature, and the trigram probability
    /// is affine in the base distribution, so every mixing weight comes out of
    /// the same pass.
    pub fn sweep(&self, sentences: &[String], betas: &[f64], recurrent: bool) -> Vec<SweepRow> {
        self.sweep_against(sentences, betas, recurrent, false)
    }

    /// [`PhianoLM::sweep`] over an explicit context construction.
    pub fn sweep_kind(
        &self,
        sentences: &[String],
        betas: &[f64],
        kind: ContextKind,
        against_uniform: bool,
    ) -> Vec<SweepRow> {
        self.sweep_kinded(sentences, betas, kind, against_uniform)
    }

    /// As [`PhianoLM::sweep`], but the thing the phase distribution is mixed
    /// against can be a **uniform** distribution instead of a unigram.
    ///
    /// This is the control that says how much information the manifold carries
    /// at all. Against a unigram, phase has to beat word frequency — a strong
    /// opponent. Against uniform it only has to beat knowing nothing. If γ = 1
    /// loses to γ = 0 even here, the phase distribution is no better than chance.
    pub fn sweep_against(
        &self,
        sentences: &[String],
        betas: &[f64],
        recurrent: bool,
        against_uniform: bool,
    ) -> Vec<SweepRow> {
        let kind = match recurrent {
            true => ContextKind::Recurrent,
            false => ContextKind::TwoWord,
        };
        self.sweep_kinded(sentences, betas, kind, against_uniform)
    }
}
