use crate::generate::compose::better::{Evaluator as ComposerEvaluator, KillerWord, SectorScore};
use crate::generate::compose::flow::RiverFlow;
use crate::generate::compose::worse::Discarder;
use crate::generate::compose::{SectorPalette, Composition};
use crate::generate::constants as config;
use phiecs::phasor::facet::Facet;
use crate::train::trainer::Trainer;

/// CompositionTuner - the "Monitor" in the Flower-Hayes model.
///
/// The monitor orchestrates the recursive writing process:
///
///   1. PROPOSE: Generate 64 sector variations (flow.rs)
///   2. EVALUATE: Score each variation (better.rs)
///   3. DISCARD + TRAIN: Keep the better, discard the worse,
///      train on the better (worse.rs + Kuramoto)
///   4. RECURSE: Generate again with the re-tuned facet
///   5. CONVERGE: Stop when the top score stops improving
///
/// This is how a child learns to write:
/// - The teacher gives a prompt and examples
/// - The child tries many approaches (64 sectors)
/// - The teacher says "this is better, this is worse"
/// - The child internalizes the better patterns
/// - Next attempt is improved because the internal model shifted
/// - Repeat until the composition is good
///
/// In Phiano, "internalizing" = Kuramoto phase relaxation on the
/// winning texts. The facet literally re-tunes itself.
pub struct CompositionTuner {
    /// Maximum recursive refinement rounds.
    pub max_rounds: usize,
    /// How many sectors to visit in each river flow (context depth).
    pub depth: usize,
    /// The evaluator (better.rs).
    pub evaluator: ComposerEvaluator,
    /// The discarder (worse.rs).
    pub discarder: Discarder,
    /// Cross-round memory of words from winning sectors (Huberman's killer heuristic).
    pub killers: KillerWord,
    /// Reject a round whose winner does not beat the previous round's under a
    /// frozen referee (Samuel's Alpha/Beta). Rounds rejected so far.
    pub rejected_rounds: usize,
}

impl CompositionTuner {
    /// Creates a new tuner with the given max rounds.
    ///
    /// Defaults: depth=8 sectors per flow, keep top 16, discard bottom 16.
    pub fn new(max_rounds: usize) -> Self {
        Self {
            max_rounds,
            depth: 8,
            evaluator: ComposerEvaluator::new(),
            discarder: Discarder::new(),
            killers: KillerWord::new(),
            rejected_rounds: 0,
        }
    }

    /// Runs the full recursive refinement loop.
    ///
    /// This is the heart of the compose system. It cycles through
    /// propose → evaluate → discard → train → recurse until convergence.
    pub fn refine(
        &mut self,
        facet: &mut Facet,
        trainer: &Trainer,
        prompt: &str,
        examples: &[String],
    ) -> Composition {
        let mut prev_best: f64 = 0.0;
        let mut best_score: SectorScore;
        let mut all_scores: Vec<SectorScore>;
        let mut rounds_completed = 0;

        for round in 0..self.max_rounds {
            println!("  [round {}/{}] generating 64 sector variations...", round + 1, self.max_rounds);

            // Phase 1: PROPOSE - generate 64 variations
            let flows = RiverFlow::generate_variations(facet, prompt, self.depth);

            // Phase 2: EVALUATE - score all 64
            all_scores = self.evaluator.evaluate_variations(facet, &flows, &self.killers);

            best_score = all_scores[0].clone();
            let avg = self.evaluator.average_score(&all_scores);
            let spread = self.evaluator.score_spread(&all_scores);

            println!(
                "  [round {}/{}] best: {:.4} (stage {}, sector {} {}) avg: {:.4} spread: {:.4}",
                round + 1,
                self.max_rounds,
                best_score.score,
                best_score.stage,
                best_score.sector,
                best_score.color,
                avg,
                spread,
            );

            // A vanishing spread is degeneracy, not success: the variants have
            // become indistinguishable, so selection has nothing left to act on.
            if spread < config::SPREAD_ALARM && round > 0 {
                eprintln!(
                    "  [WARN] sector spread {:.4} below {:.4} — variants are degenerate; \
                     the manifold may be collapsing (see docs/how/08_self_scoring.md)",
                    spread, config::SPREAD_ALARM
                );
            }

            // Alpha/Beta: freeze the manifold before the round trains on it, so
            // the round can be judged by something it has not altered.
            let beta = facet.clone();
            let beta_best = all_scores
                .first()
                .map(|s| ComposerEvaluator::referee_score(&beta, prompt, &s.text))
                .unwrap_or(0.0);

            // Phase 3: GUARD + SELECT + REINFORCE + PENALISE
            let result = self.discarder.discard_and_train(facet, trainer, &all_scores);
            self.discarder.print_summary(&result);

            // Judge the round against the frozen manifold. If training on this
            // round's winners did not produce a better composition by a referee
            // it cannot influence, reject the round and restore Beta - which is
            // exactly how Samuel decided whether Alpha's changes were kept.
            let alpha_flows = RiverFlow::generate_variations(facet, prompt, self.depth);
            let alpha_scores =
                self.evaluator.evaluate_variations(facet, &alpha_flows, &self.killers);
            let alpha_best = alpha_scores
                .first()
                .map(|s| ComposerEvaluator::referee_score(&beta, prompt, &s.text))
                .unwrap_or(0.0);

            if alpha_best < beta_best {
                *facet = beta;
                self.rejected_rounds += 1;
                println!(
                    "  [reject] round {} rolled back: referee {:.4} < {:.4}",
                    round + 1, alpha_best, beta_best
                );
            } else {
                // Remember what won, for next round's candidate ordering.
                for text in &result.trained_texts {
                    self.killers.record_winner(text);
                }
            }

            // Phase 4: CONVERGE - check if we're done. Degeneracy is reported
            // separately so it is never mistaken for convergence.
            if self.evaluator.has_converged(&all_scores, prev_best) && round > 0 {
                match self.evaluator.is_degenerate(&all_scores) {
                    true => println!("  [halted] population degenerate — stopping, not converged"),
                    false => println!("  [converged] improvement below threshold"),
                }
                rounds_completed = round + 1;
                break;
            }

            prev_best = best_score.score;
            rounds_completed = round + 1;
        }

        // Final evaluation with the re-tuned facet
        let final_flows = RiverFlow::generate_variations(facet, prompt, self.depth);
        let final_scores = self.evaluator.evaluate_variations(facet, &final_flows, &self.killers);

        let winner = &final_scores[0];
        let sector_scores: Vec<(u16, f64)> = final_scores
            .iter()
            .map(|s| (s.sector, s.score))
            .collect();

        let eval = crate::eval::Evaluator::new().eval(facet, &winner.text);

        Composition {
            prompt: prompt.to_string(),
            text: winner.text.clone(),
            winning_sector: winner.sector,
            winning_color: SectorPalette::color(winner.sector),
            rounds: rounds_completed,
            eval,
            sector_scores,
            examples: examples.to_vec(),
        }
    }
}
