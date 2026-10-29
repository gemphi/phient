use super::*;

mod context;
mod affine;
mod advance;
mod sweep;
mod gamma;

impl<'a> PhianoLM<'a> {
    pub fn new(facet: &'a Facet, use_phase: bool) -> Self {
        Self::with_gamma(facet, if use_phase { 1.0 } else { 0.0 })
    }

    pub fn with_gamma(facet: &'a Facet, gamma: f64) -> Self {
        let n = facet.lexicon.len();
        let mut index = std::collections::HashMap::with_capacity(n);
        let mut vecs = Vec::with_capacity(n * 2 * LM_CHANNELS);
        let mut counts = Vec::with_capacity(n);
        let mut sectors = Vec::with_capacity(n);

        // Sorted, so the index a word receives does not depend on HashMap
        // iteration order. The readout's negative sampling draws by index, so a
        // shifting index made readout fitting differ between runs on identical
        // data.
        let mut words: Vec<(&String, &phiecs::phasor::spectral::SpectralPhasor)> =
            facet.lexicon.iter().collect();
        words.sort_unstable_by(|a, b| a.0.cmp(b.0));

        for (w, p) in words {
            index.insert(w.clone(), counts.len());
            for k in 0..LM_CHANNELS {
                let t = p.theta(k);
                vecs.push(t.cos());
                vecs.push(t.sin());
            }
            sectors.push(crate::metrics::nonlinear::SectorReadout::target_sector(p.theta(0)));
            counts.push(p.count.max(1) as f64);
        }

        let total: f64 = counts.iter().sum();
        let unigram: Vec<f64> = counts
            .iter()
            .map(|c| if total > 0.0 { c / total } else { 0.0 })
            .collect();

        Self { facet, index, vecs, unigram, gamma, sectors, readout: None }
    }

    /// Angles of a context vector, for keying the readout table.
    pub(in crate::metrics::harness) fn ctx_angles(ctx: &[f64]) -> Vec<f64> {
        (0..LM_CHANNELS)
            .map(|k| ctx[2 * k + 1].atan2(ctx[2 * k]))
            .collect()
    }

    /// Fits the conditional readout on training text.
    ///
    /// For every trigram position the context cell is rewarded for the sector
    /// the true next word occupies and penalised for the sector of a frequency
    /// sampled negative — the same contrastive shape the trainer uses, applied
    /// to the discretised readout instead of to the phases.
    ///
    /// Deliberately fitted on `train` only. Fitting a lookup table on the same
    /// text it is scored against would measure memorisation.
    pub fn fit_readout(&mut self, sentences: &[String], lr: f64, recurrent: bool) {
        self.fit_readout_shaped(
            sentences,
            lr,
            recurrent,
            crate::metrics::nonlinear::KEY_CHANNELS,
            crate::metrics::nonlinear::KEY_SECTORS,
        );
    }

    /// As [`PhianoLM::fit_readout`], at an explicit key resolution.
    ///
    /// The resolution is the whole trade-off. A fine key distinguishes more
    /// contexts but misses on almost every held-out one, and a table that never
    /// hits cannot change a held-out score however well it fits the training
    /// split. Sweeping it is how that gets measured rather than assumed.
    pub fn fit_readout_shaped(
        &mut self,
        sentences: &[String],
        lr: f64,
        recurrent: bool,
        key_channels: usize,
        key_sectors: usize,
    ) {
        let mut table = crate::metrics::nonlinear::SectorReadout::with_shape(key_channels, key_sectors);
        let mut rng: u64 = 0x9E3779B97F4A7C15;
        let v = self.unigram.len();
        if v == 0 {
            return;
        }

        for sentence in sentences {
            let toks = Tokenizer::tokenize(sentence);
            if toks.len() < 3 {
                continue;
            }
            let mut h = vec![phistd::c64::new(0.0, 0.0); LM_CHANNELS];
            if recurrent {
                self.advance(&mut h, &toks[0]);
                self.advance(&mut h, &toks[1]);
            }
            for i in 2..toks.len() {
                let (a, b, c) = (&toks[i - 2], &toks[i - 1], &toks[i]);
                let idx = match self.index.get(c) {
                    Some(x) => *x,
                    None => {
                        if recurrent {
                            self.advance(&mut h, c);
                        }
                        continue;
                    }
                };
                let ctx = match recurrent {
                    true => Self::state_to_ctx(&h),
                    false => match self.context_vec(a, b) {
                        Some(x) => x,
                        None => {
                            if recurrent {
                                self.advance(&mut h, c);
                            }
                            continue;
                        }
                    },
                };
                // xorshift64, so the negative sample is deterministic per run.
                rng = phistd::random::xorshift64(rng);
                let neg = (rng % v as u64) as usize;

                let key = table.key(&Self::ctx_angles(&ctx));
                table.learn(key, self.sectors[idx], self.sectors[neg], lr);

                if recurrent {
                    self.advance(&mut h, c);
                }
            }
        }
        self.readout = Some(table);
    }

    /// Number of context cells the fitted readout holds; 0 when there is none.
    pub fn readout_cells(&self) -> usize {
        self.readout.as_ref().map(|r| r.cells()).unwrap_or(0)
    }

    /// Fraction of readout lookups since the last reset that hit a fitted cell.
    pub fn readout_coverage(&self) -> f64 {
        self.readout.as_ref().map(|r| r.coverage()).unwrap_or(0.0)
    }

    pub fn reset_readout_coverage(&self) {
        if let Some(r) = &self.readout {
            r.reset_coverage();
        }
    }

    /// Discards the fitted readout, so the same model can be scored both ways.
    pub fn clear_readout(&mut self) {
        self.readout = None;
    }
}
