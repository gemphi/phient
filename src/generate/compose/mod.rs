pub mod flow;
pub mod better;
pub mod worse;
pub mod tune;

use phiecs::phasor::facet::Facet;
use crate::train::trainer::Trainer;
use std::fmt;

/// Color mapping palette for phase-space sectors.
///
/// Uses the phiton/gemgum physics-derived spectral mapping instead of
/// hard-coded color names. Each sector maps to a wavelength on the
/// visible spectrum, producing a natural color from the fine-structure
/// constant and golden ratio.
pub struct SectorPalette;

impl SectorPalette {
    /// Maps sector indices to physics-derived color names.
    ///
    /// Delegates to [`phiecs::phasor::gemgum::Gemgum::sector_color_name`], which
    /// maps the sector through the phiton spectral domain.
    pub fn color(sector: u16) -> String {
        let n = phiecs::phasor::wave::Wave::sector_count();
        phiecs::phasor::gemgum::Gemgum::sector_color_name(sector, n)
    }

    /// Maps sector indices to full [`phiton::PhitonColor`] with RGB.
    #[allow(dead_code)]
    pub fn phiton_color(sector: u16) -> phiton::PhitonColor {
        let n = phiecs::phasor::wave::Wave::sector_count();
        phiecs::phasor::gemgum::Gemgum::sector_color(sector, n)
    }
}

/// Composition - the result of a full recursive compose cycle.
///
/// Based on the Flower-Hayes (1981) cognitive process model:
///   Planning → Translating → Reviewing (evaluating + revising)
///
/// The model generates 64 variations (one per sector), evaluates them
/// (better.rs), discards the worst (worse.rs), trains on the best
/// (Kuramoto re-tuning), and recurses. The final composition is the
/// one that survived the tournament.
pub struct Composition {
    /// The prompt that seeded the composition.
    pub prompt: String,
    /// The final winning text.
    pub text: String,
    /// The sector that produced the winner.
    pub winning_sector: u16,
    /// The color of the winning sector.
    pub winning_color: String,
    /// Number of recursive refinement rounds completed.
    pub rounds: usize,
    /// The evaluation scores of the winner.
    pub eval: crate::eval::Eval,
    /// All sector scores from the final round: (sector, score) pairs.
    pub sector_scores: Vec<(u16, f64)>,
    /// Examples that were learned from before composing.
    pub examples: Vec<String>,
}

impl Composition {
    /// Runs the full recursive compose cycle.
    ///
    /// 1. Learn from examples (teacher's specimens)
    /// 2. Generate 64 variations - one per sector (Planning + Translating)
    /// 3. Evaluate each variation (Reviewing → Evaluating)
    /// 4. Discard the worse, keep the better (Reviewing → Revising)
    /// 5. Train on the better (Kuramoto re-tunes the facet)
    /// 6. Recurse with the improved facet
    /// 7. Repeat until convergence or max rounds
    pub fn compose(
        facet: &mut Facet,
        trainer: &Trainer,
        prompt: &str,
        examples: &[String],
        max_rounds: usize,
    ) -> Self {
        let mut tuner = tune::CompositionTuner::new(max_rounds);

        // Phase 1: Learn from the teacher's examples
        for example in examples {
            trainer.train_sentence(facet, example);
        }

        // Phase 2-7: Recursive refinement via the monitor
        tuner.refine(facet, trainer, prompt, examples)
    }
}

impl fmt::Display for Composition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  ── composition ──")?;
        writeln!(f, "  prompt: \"{}\"", self.prompt)?;
        if !self.examples.is_empty() {
            writeln!(f, "  learned from {} examples", self.examples.len())?;
        }
        writeln!(
            f,
            "  winner: sector {} ({}) after {} rounds",
            self.winning_sector, self.winning_color, self.rounds,
        )?;
        writeln!(f)?;
        // Indent each line of the composition for readability
        for line in self.text.lines() {
            writeln!(f, "    {}", line)?;
        }
        writeln!(f)?;
        writeln!(f, "  ── evaluation ──")?;
        write!(f, "{}", self.eval)?;

        // Show top 8 sectors from the tournament
        writeln!(f)?;
        writeln!(f, "  ── sector tournament (top 8) ──")?;
        let mut ranked = self.sector_scores.clone();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (i, (sector, score)) in ranked.iter().take(8).enumerate() {
            let color = SectorPalette::color(*sector);
            writeln!(f, "    #{}: sector {} ({}) score {:.4}", i + 1, sector, color, score)?;
        }

        Ok(())
    }
}
