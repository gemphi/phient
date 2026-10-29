#[allow(unused_imports)]
use super::*;

impl Discarder {
    /// Creates a discarder that keeps the top 16 and penalises the bottom 16.
    pub fn new() -> Self {
        Self {
            keep_count: 16,
            discard_count: 16,
            repulsion: config::LOSER_REPULSION,
        }
    }

    /// Performs the guard → select → reinforce → penalise cycle.
    pub fn discard_and_train(
        &self,
        facet: &mut Facet,
        trainer: &Trainer,
        scores: &[SectorScore],
    ) -> DiscardResult {
        // 1. GUARD — stage 0 never survives, whatever its measure.
        let guarded: Vec<u16> = scores.iter().filter(|s| s.is_worse()).map(|s| s.sector).collect();
        let eligible: Vec<&SectorScore> = scores.iter().filter(|s| !s.is_worse()).collect();

        // 2. SELECT
        let keep_end = self.keep_count.min(eligible.len());
        let kept: Vec<u16> = eligible[..keep_end].iter().map(|s| s.sector).collect();

        // Losers are the tail of the eligible set plus everything the guard cut.
        let discard_start = eligible.len().saturating_sub(self.discard_count).max(keep_end);
        let mut losers: Vec<&SectorScore> = eligible[discard_start..].to_vec();
        losers.extend(scores.iter().filter(|s| s.is_worse()));
        let discarded: Vec<u16> = losers.iter().map(|s| s.sector).collect();

        // 3. REINFORCE the winners.
        let mut trained_texts = Vec::new();
        let mut tokens_updated = 0;
        for score in eligible.iter().take(keep_end) {
            tokens_updated += trainer.train_sentence(facet, &score.text);
            trained_texts.push(score.text.clone());
        }

        // 4. PENALISE the losers.
        let tokens_penalised = self.penalise(facet, trainer, &losers);

        DiscardResult {
            kept,
            discarded,
            guarded,
            trained_texts,
            tokens_updated,
            tokens_penalised,
        }
    }

    /// Rotates the content words of losing compositions away from their centroid.
    ///
    /// Content words only: closed-class words appear in winners and losers alike,
    /// so penalising them degrades the model globally to express a judgement
    /// about one composition.
    pub(super) fn penalise(&self, facet: &mut Facet, trainer: &Trainer, losers: &[&SectorScore]) -> usize {
        let rate = self.repulsion * trainer.learning_rate;
        let mut touched = 0usize;

        for score in losers {
            let tokens = Tokenizer::tokenize(&score.text);
            let mut sum = phical::circle::Resultant::new();
            for t in &tokens {
                if let Some(p) = facet.lexicon.get(t) {
                    sum.add(p.amplitude, p.theta(0));
                }
            }
            if sum.vanishes() {
                continue;
            }
            let centroid = sum.direction();

            for token in Tokenizer::content_words(&score.text) {
                if let Some(p) = facet.lexicon.get_mut(&token) {
                    let away = -(centroid - p.theta(0)).sin();
                    p.nudge(0, rate * away);
                    p.sync_phase();
                    touched += 1;
                }
            }
        }

        touched
    }

    /// Reinforces a generation trajectory with decayed, per-step credit.
    ///
    /// Reward was applied to a whole composition: if a twenty-word output won
    /// because of three good words, all twenty were reinforced equally,
    /// including the seventeen that were mediocre. Samuel's central technical
    /// contribution was the opposite — propagate credit back through the
    /// sequence so the move responsible receives it.
    ///
    /// The trace this needs has been collected all along:
    /// `PhaseFlow::record_step` stores each step's word, resonance and novelty
    /// and nothing ever read it. Credit decays by `lambda` with distance from
    /// the end and scales with each step's own resonance.
    ///
    /// Returns the number of tokens credited.
    pub fn reinforce_trajectory(
        &self,
        facet: &mut Facet,
        trainer: &Trainer,
        flow: &phiecs::phasor::flow::PhaseFlow,
        lambda: f64,
    ) -> usize {
        let n = flow.trajectory.len();
        if n == 0 {
            return 0;
        }
        let target = flow.collective_phase;
        let mut credited = 0usize;

        for (i, step) in flow.trajectory.iter().enumerate() {
            let word = match &step.selected_word {
                Some(w) => w,
                None => continue,
            };
            // Recency-decayed eligibility, weighted by that step's own contribution.
            let decay = lambda.powi((n - 1 - i) as i32);
            let credit = decay * step.resonance_score.max(0.0);
            if credit > 1e-6 && trainer.nudge_token(facet, word, target, credit) {
                credited += 1;
            }
        }
        credited
    }

    /// Prints a summary of the discard round.
    pub fn print_summary(&self, result: &DiscardResult) {
        print!("  [keep]    sectors: ");
        for (i, &sector) in result.kept.iter().enumerate().take(8) {
            if i > 0 { print!(", "); }
            print!("{} ({})", sector, SectorPalette::color(sector));
        }
        if result.kept.len() > 8 {
            print!(", ... ({} total)", result.kept.len());
        }
        println!();

        if !result.guarded.is_empty() {
            println!(
                "  [guard]   {} sector(s) rejected at stage 0 before scoring",
                result.guarded.len()
            );
        }

        print!("  [discard] sectors: ");
        for (i, &sector) in result.discarded.iter().enumerate().take(8) {
            if i > 0 { print!(", "); }
            print!("{} ({})", sector, SectorPalette::color(sector));
        }
        if result.discarded.len() > 8 {
            print!(", ... ({} total)", result.discarded.len());
        }
        println!();

        println!(
            "  [train]   +{} tokens reinforced across {} texts, -{} tokens pushed away",
            result.tokens_updated,
            result.trained_texts.len(),
            result.tokens_penalised,
        );
    }
}

impl Default for Discarder {
    fn default() -> Self {
        Self::new()
    }
}
