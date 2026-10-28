//! Benchmark report formatting: human-readable output and comparison.

use super::benchmark::runner::BenchmarkReport;
use std::fmt;

impl fmt::Display for BenchmarkReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "═══ Phiano Benchmark Report ═══")?;
        writeln!(f)?;

        match &self.baselines {
            None => {
                writeln!(f, "  Held-out perplexity: no evaluation corpus found")?;
                writeln!(f, "    (expected at {})", super::baseline::BaselineSet::CORPUS)?;
            }
            Some(b) => {
                writeln!(f, "  Held-out perplexity ({} sentences, lower is better):", b.n_heldout)?;
                writeln!(f, "    Uniform:            {:.2}", b.uniform_ppl)?;
                writeln!(f, "    Unigram:            {:.2}", b.unigram_ppl)?;
                writeln!(f, "    Kneser-Ney trigram: {:.2}", b.kn_trigram_ppl)?;
                writeln!(f, "    Phiano (counts):    {:.2}", b.phiano_counts_ppl)?;
                writeln!(f, "    Phiano (phase):     {:.2}", b.phiano_phase_ppl)?;
                writeln!(f)?;
                writeln!(f, "    beats Kneser-Ney:   {}", b.beats_kn())?;
                writeln!(f, "    phase back-off helps: {}", b.phase_helps())?;
                writeln!(f, "    phase signal recovered: {:.1}%", b.phase_signal_recovered() * 100.0)?;
            }
        }
        writeln!(f)?;

        writeln!(f, "  Manifold health:")?;
        writeln!(f, "    Phase dispersion: {:.4}  (1.0 spread, 0.0 collapsed)", self.phase_dispersion)?;
        writeln!(f, "    Sector Gini:      {:.4}", self.sector_gini)?;
        if self.phase_dispersion < 0.2 {
            writeln!(f, "    [WARN] dispersion below 0.2 — the lexicon is synchronising")?;
        }
        writeln!(f)?;

        writeln!(f, "  Robustness:")?;
        writeln!(f, "    Brittleness:      {:.4}", self.brittleness)?;
        writeln!(f, "    OOD Score:        {:.4}", self.ood_score)?;
        writeln!(f)?;
        writeln!(f, "  Adaptation:")?;
        writeln!(f, "    Efficiency:       {:.4}", self.adaptation_efficiency)?;
        writeln!(f, "    Novel Task Score: {:.4}", self.novel_task_score)?;
        writeln!(f)?;
        writeln!(f, "  Generalization (perplexity by vocabulary coverage):")?;
        writeln!(f, "    Local   ({:>4} sents): {:.2}", self.generalization.n_local, self.generalization.local_score)?;
        writeln!(f, "    Extreme ({:>4} sents): {:.2}", self.generalization.n_extreme, self.generalization.extreme_score)?;
        writeln!(f, "    Gap (log ratio):      {:.4}", self.generalization.gap)?;
        writeln!(f)?;

        if let Some(arc) = &self.arc_results {
            writeln!(f, "  Text-analogy proxy (NOT ARC-AGI):")?;
            writeln!(f, "    Tasks: {}  Exact: {}  Partial: {}  Failed: {}",
                arc.total, arc.exact, arc.partial, arc.failed)?;
            writeln!(f, "    Mean token F1: {:.4}", arc.mean_f1)?;
        }

        if !self.shortcut_warnings.is_empty() {
            writeln!(f, "  Shortcut Warnings:")?;
            for w in &self.shortcut_warnings {
                writeln!(f, "    [{}] {:.0}% — {}", w.shortcut_type, w.severity * 100.0, w.description)?;
            }
        }
        Ok(())
    }
}

impl BenchmarkReport {
    /// The headline number: held-out perplexity of the model's own counts.
    /// `None` when no evaluation corpus was available.
    pub fn headline_ppl(&self) -> Option<f64> {
        self.baselines.map(|b| b.phiano_counts_ppl)
    }

    /// Compares two reports, oldest first.
    pub fn compare(old: &BenchmarkReport, new: &BenchmarkReport) -> String {
        let mut lines = Vec::new();

        match (old.headline_ppl(), new.headline_ppl()) {
            (Some(o), Some(n)) => {
                let delta = n - o;
                lines.push(format!(
                    "Held-out perplexity: {:.2} → {:.2} ({:+.2}, {})",
                    o, n, delta,
                    if delta < 0.0 { "better" } else { "worse" }
                ));
            }
            _ => lines.push("Held-out perplexity: not measured".to_string()),
        }

        let disp = new.phase_dispersion - old.phase_dispersion;
        lines.push(format!(
            "Phase dispersion:    {:.4} → {:.4} ({:+.4}{})",
            old.phase_dispersion, new.phase_dispersion, disp,
            if disp < -0.05 { ", collapsing" } else { "" }
        ));

        let brit = new.brittleness - old.brittleness;
        lines.push(format!(
            "Brittleness:         {:.4} → {:.4} ({:+.4})",
            old.brittleness, new.brittleness, brit
        ));

        lines.join("\n  ")
    }
}
