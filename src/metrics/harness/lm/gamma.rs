use super::super::*;

impl<'a> PhianoLM<'a> {

    /// [`PhianoLM::gamma_sweep`] with the phase term supplied from outside.
    ///
    /// `scorer(toks, i)` returns one score per vocabulary index (in
    /// [`PhianoLM::vocabulary`] order) for the word at position `i` given the
    /// sentence prefix `toks[..i]`, or `None` to fall back to the unigram. The
    /// scores go through the same `softmax(β·s)` as the manifold's own coherence,
    /// the same trigram affine, and the same γ grid — so an operator's
    /// distribution is measured on exactly the footing the manifold's is.
    pub fn gamma_sweep_scored<F>(&self, sentences: &[String], mut scorer: F) -> Vec<(f64, f64)>
    where
        F: FnMut(&[String], usize) -> Option<Vec<f64>>,
    {
        let floor = 1.0 / (self.unigram.len().max(1) as f64 * 100.0);
        let mut sums = [0.0f64; GAMMA_GRID.len()];
        let mut n = 0usize;
        let v = self.unigram.len();

        for sentence in sentences {
            let toks = Tokenizer::tokenize(sentence);
            if toks.len() < 3 {
                continue;
            }
            for i in 2..toks.len() {
                let (a, b, c) = (&toks[i - 2], &toks[i - 1], &toks[i]);
                let idx = match self.index.get(c) {
                    Some(i) => *i,
                    None => continue,
                };
                let (k, coef) = self.affine(a, b, c);
                let p_uni = self.unigram[idx].max(floor);
                let p_ph = match scorer(&toks, i) {
                    Some(scores) if scores.len() == v => {
                        let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        let z: f64 = scores.iter().map(|s| (PHASE_BETA * (s - max)).exp()).sum();
                        match z > 0.0 {
                            true => (PHASE_BETA * (scores[idx] - max)).exp() / z,
                            false => 1.0 / v as f64,
                        }
                    }
                    _ => p_uni,
                };
                for (gi, g) in GAMMA_GRID.iter().enumerate() {
                    let base = g * p_ph + (1.0 - g) * p_uni;
                    sums[gi] += (k + coef * base).max(1e-12).ln();
                }
                n += 1;
            }
        }

        GAMMA_GRID
            .iter()
            .zip(sums.iter())
            .map(|(g, s)| {
                (*g, match n { 0 => f64::INFINITY, _ => (-s / n as f64).exp() })
            })
            .collect()
    }

    /// Returns `(gamma, perplexity)` pairs. γ = 0 is the model with the phase
    /// manifold removed; γ = 1 is the manifold alone as the back-off.
    pub fn gamma_sweep(&self, sentences: &[String]) -> Vec<(f64, f64)> {
        let floor = 1.0 / (self.unigram.len().max(1) as f64 * 100.0);
        let mut sums = [0.0f64; GAMMA_GRID.len()];
        let mut n = 0usize;

        for sentence in sentences {
            let toks = Tokenizer::tokenize(sentence);
            if toks.len() < 3 {
                continue;
            }
            for w in toks.windows(3) {
                let idx = match self.index.get(&w[2]) {
                    Some(i) => *i,
                    None => continue,
                };
                let (k, coef) = self.affine(&w[0], &w[1], &w[2]);
                let p_uni = self.unigram[idx].max(floor);
                let p_ph = match self.context_vec(&w[0], &w[1]) {
                    Some(ctx) => self.p_phase(&ctx, idx),
                    None => p_uni,
                };
                for (gi, g) in GAMMA_GRID.iter().enumerate() {
                    let base = g * p_ph + (1.0 - g) * p_uni;
                    sums[gi] += (k + coef * base).max(1e-12).ln();
                }
                n += 1;
            }
        }

        GAMMA_GRID
            .iter()
            .zip(sums.iter())
            .map(|(g, s)| {
                (*g, match n { 0 => f64::INFINITY, _ => (-s / n as f64).exp() })
            })
            .collect()
    }
}
