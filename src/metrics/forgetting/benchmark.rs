#[allow(unused_imports)]
use super::*;

impl ForgettingBenchmark {
    pub(super) fn feed(trainer: &Trainer, f: &mut Facet, s: &str, ranking: bool) {
        match ranking {
            true => { trainer.train(f, s); }
            false => { trainer.train_sentence(f, s); }
        }
    }

    /// Sequential exposure: all of the first set, then all of the second.
    pub(super) fn train_sequential(trainer: &Trainer, sets: &[&[String]], ranking: bool) -> Facet {
        let mut f = Facet::new();
        for set in sets {
            for s in set.iter() {
                Self::feed(trainer, &mut f, s, ranking);
            }
        }
        f
    }

    /// Joint exposure: the two domains **interleaved**.
    ///
    /// This is the distinction the whole benchmark turns on. Training all of A
    /// and then all of B *is* the sequential condition, so using it as the
    /// ceiling makes the two models identical and retention comes out at 100%
    /// by construction rather than by measurement. Round-robin interleaving is
    /// the standard joint-training upper bound: the same data, without the
    /// ordering that causes forgetting.
    pub(super) fn train_joint(trainer: &Trainer, a: &[String], b: &[String], ranking: bool) -> Facet {
        let mut f = Facet::new();
        let n = a.len().max(b.len());
        for i in 0..n {
            if let Some(s) = a.get(i) {
                Self::feed(trainer, &mut f, s, ranking);
            }
            if let Some(s) = b.get(i) {
                Self::feed(trainer, &mut f, s, ranking);
            }
        }
        f
    }

    /// Perplexity with the phase manifold as the back-off distribution.
    ///
    /// Measured at γ = 1 deliberately. At γ = 0 the score depends only on the
    /// n-gram counts, which are pure tallies that nothing ever overwrites — so
    /// they cannot forget, and a benchmark run there measures a property that
    /// is true by construction rather than anything about the manifold.
    pub(super) fn ppl(facet: &Facet, held_out: &[String]) -> f64 {
        PhianoLM::with_gamma(facet, 1.0).perplexity(held_out)
    }

    /// Perplexity from the counts alone, reported for reference.
    pub(super) fn ppl_counts(facet: &Facet, held_out: &[String]) -> f64 {
        PhianoLM::with_gamma(facet, 0.0).perplexity(held_out)
    }

    /// Runs the three-model comparison.
    pub fn run(
        trainer: &Trainer,
        a_name: &str,
        a_train: &[String],
        a_eval: &[String],
        b_name: &str,
        b_train: &[String],
        ranking: bool,
    ) -> ForgettingReport {
        let ceiling = Self::train_joint(trainer, a_train, b_train, ranking);
        let floor = Self::train_sequential(trainer, &[b_train], ranking);

        // Sequential: A first, then B, on the same model.
        let mut sequential = Self::train_sequential(trainer, &[a_train], ranking);
        let a_only_ppl = Self::ppl(&sequential, a_eval);
        for s in b_train {
            Self::feed(trainer, &mut sequential, s, ranking);
        }

        let ceiling_ppl = Self::ppl(&ceiling, a_eval);
        let floor_ppl = Self::ppl(&floor, a_eval);
        let sequential_ppl = Self::ppl(&sequential, a_eval);
        let counts_ppl = Self::ppl_counts(&sequential, a_eval);

        let retention = phistd::stats::log_share(floor_ppl, ceiling_ppl, sequential_ppl).map_or(f64::NAN, |r| r.clamp(0.0, 1.5));
        let degradation_pct = (sequential_ppl / a_only_ppl - 1.0) * 100.0;

        let verdict = match retention {
            r if r.is_nan() => "bounds collapsed — the domains are not distinct enough".to_string(),
            r if r >= 0.95 => format!(
                "retention {:.1}% — learning B cost domain A essentially nothing",
                r * 100.0
            ),
            r if r >= 0.7 => format!(
                "retention {:.1}% — mild forgetting, well short of catastrophic",
                r * 100.0
            ),
            r => format!(
                "retention {:.1}% — substantial forgetting; the 'zero catastrophic \
                 forgetting' claim is not supported",
                r * 100.0
            ),
        };

        ForgettingReport {
            domain_a_name: a_name.to_string(),
            domain_b_name: b_name.to_string(),
            n_train_a: a_train.len(),
            n_train_b: b_train.len(),
            n_eval_a: a_eval.len(),
            ceiling_ppl,
            sequential_ppl,
            floor_ppl,
            a_only_ppl,
            counts_only_ppl: counts_ppl,
            retention,
            degradation_pct,
            verdict,
        }
    }
}
