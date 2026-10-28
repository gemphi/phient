use super::*;

impl Harness {

    /// Runs the full comparison and returns a report.
    pub fn run(corpus: Vec<String>, epochs: usize) -> HarnessReport {
        let split = Self::split(corpus, 42);

        let kn = KneserNey::train(&split.train);
        let kn_ppl = kn.perplexity(&split.valid);

        let contrastive = Trainer::new(crate::train::constants::LEARNING_RATE);
        let (trained, log) = Self::train_and_measure(&split, &contrastive, epochs, true);
        let no_phase = Self::perplexity_no_phase(&trained, &split.valid);

        let attraction = Trainer::attraction_only(crate::train::constants::LEARNING_RATE);
        let (_, log_attr) = Self::train_and_measure(&split, &attraction, epochs, false);

        let (best_epoch, best) = log
            .iter()
            .enumerate()
            .min_by(|a, b| {
                a.1.valid_ppl
                    .partial_cmp(&b.1.valid_ppl)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, m)| (i, m.valid_ppl))
            .unwrap_or((0, f64::INFINITY));

        let attr_best = log_attr
            .iter()
            .map(|m| m.valid_ppl)
            .fold(f64::INFINITY, f64::min);

        let final_ppl = log.last().map(|m| m.valid_ppl).unwrap_or(f64::INFINITY);

        // Sweep the mixing weight on a facet trained for the best number of
        // epochs, so the contribution is measured where the model is strongest.
        let (best_facet, _) =
            Self::train_and_measure(&split, &contrastive, best_epoch + 1, true);
        let sweep = PhianoLM::with_gamma(&best_facet, 1.0).gamma_sweep(&split.valid);
        let best_gamma = sweep
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(g, _)| *g)
            .unwrap_or(0.0);

        let pred_facet = Self::train_predictive_heavy(&split, &contrastive, best_epoch + 1, 4);
        let sweep_pred = PhianoLM::with_gamma(&pred_facet, 1.0).gamma_sweep(&split.valid);
        let best_gamma_pred = sweep_pred
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(g, _)| *g)
            .unwrap_or(0.0);

        let best_m = &log[best_epoch];
        let phase_helps = best_m.valid_ppl < best_m.valid_ppl_no_phase;

        let verdict = match (best < kn_ppl, phase_helps) {
            (true, true) => format!(
                "Phiano beats Kneser-Ney ({:.2} vs {:.2}), and the phase manifold \
                 beats a unigram back-off on the same counts ({:.2} vs {:.2}).",
                best, kn_ppl, best_m.valid_ppl, best_m.valid_ppl_no_phase
            ),
            (false, true) => format!(
                "Phiano does not yet beat Kneser-Ney ({:.2} vs {:.2}), but the phase \
                 manifold DOES beat a unigram back-off on identical counts \
                 ({:.2} vs {:.2}) — the manifold is contributing; the n-gram \
                 smoothing is what lags.",
                best, kn_ppl, best_m.valid_ppl, best_m.valid_ppl_no_phase
            ),
            (_, false) => format!(
                "Phiano does not beat Kneser-Ney ({:.2} vs {:.2}), and the phase \
                 manifold does not beat a unigram back-off ({:.2} vs {:.2}). \
                 The phase layer is not paying for itself at this epoch.",
                best, kn_ppl, best_m.valid_ppl, best_m.valid_ppl_no_phase
            ),
        };

        HarnessReport {
            kn_trigram_ppl: kn_ppl,
            phiano_best_valid_ppl: best,
            phiano_final_valid_ppl: final_ppl,
            best_epoch,
            attraction_only_best_ppl: attr_best,
            no_phase_backoff_ppl: no_phase,
            gamma_sweep: sweep,
            best_gamma,
            gamma_sweep_predictive: sweep_pred,
            best_gamma_predictive: best_gamma_pred,
            verdict,
            epochs: log,
        }
    }
}

impl ContextKind {
    pub fn label(self) -> &'static str {
        match self {
            ContextKind::TwoWord => "2-word",
            ContextKind::Bound => "bound",
            ContextKind::Recurrent => "recurrent",
        }
    }

    pub fn is_recurrent(self) -> bool {
        matches!(self, ContextKind::Recurrent)
    }

    pub const ALL: [ContextKind; 3] =
        [ContextKind::TwoWord, ContextKind::Bound, ContextKind::Recurrent];
}
