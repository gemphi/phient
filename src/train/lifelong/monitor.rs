//! Deployment monitoring: detects drift, regression and manifold collapse.

use phiecs::phasor::facet::Facet;
use crate::train::lifelong::history::BenchmarkHistory;
use crate::metrics::benchmark::runner::BenchmarkReport;
use crate::metrics::detection::ood::OodDetector;
use serde::Serialize;

/// Relative perplexity increase that counts as a regression.
const REGRESSION_FRACTION: f64 = 0.05;
/// Phase dispersion below which the lexicon is treated as collapsing.
const COLLAPSE_DISPERSION: f64 = 0.2;

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub alert_type: String,
    pub message: String,
    pub severity: f64,
}

pub struct ModelMonitor {
    pub history: BenchmarkHistory,
    pub alerts: Vec<Alert>,
}

impl Default for ModelMonitor {
    fn default() -> Self {
        Self { history: BenchmarkHistory::new(), alerts: Vec::new() }
    }
}

impl ModelMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Detects if recent inputs are drifting from the training distribution.
    pub fn check_drift(&self, facet: &Facet, recent_inputs: &[String]) -> Option<Alert> {
        let max_ood = recent_inputs
            .iter()
            .map(|i| OodDetector::score(facet, i))
            .fold(0.0f64, f64::max);

        match max_ood > 0.7 {
            true => Some(Alert {
                alert_type: "distribution_drift".to_string(),
                message: format!("Input drift detected: OOD score {:.4}", max_ood),
                severity: max_ood,
            }),
            false => None,
        }
    }

    /// Detects a regression in held-out perplexity.
    ///
    /// Previously this watched the phase-baseline coherence and fired when it
    /// *fell*. Since coherence rises as the manifold collapses, that alarm
    /// reported health during the system's main failure mode, and would have
    /// fired on any fix that restored phase dispersion.
    pub fn check_regression(&self, current: &BenchmarkReport) -> Option<Alert> {
        let prev = self.history.latest()?;
        let (before, after) = (prev.report.headline_ppl()?, current.headline_ppl()?);
        if before <= 0.0 {
            return None;
        }

        let fraction = (after - before) / before;
        match fraction > REGRESSION_FRACTION {
            true => Some(Alert {
                alert_type: "performance_regression".to_string(),
                message: format!(
                    "Held-out perplexity worsened: {:.2} → {:.2} ({:+.1}%)",
                    before, after, fraction * 100.0
                ),
                severity: fraction.min(1.0),
            }),
            false => None,
        }
    }

    /// Detects the manifold synchronising toward a point.
    ///
    /// Kuramoto coupling is attraction-dominated, so a lexicon losing phase
    /// dispersion is the architecture's characteristic failure. It is invisible
    /// to every score that is itself a function of synchronisation, which is why
    /// it needs its own alarm.
    /// The alarm is raised on the *frequent band*, not the whole lexicon. A rare
    /// word that never trains keeps its initial angle, so on a long-tailed
    /// vocabulary the global mean stays near 1.0 no matter what the trained
    /// words do — the global figure measures the part of the model that cannot
    /// fail.
    pub fn check_collapse(&self, current: &BenchmarkReport) -> Option<Alert> {
        let worst = current.phase_dispersion.min(current.band_dispersion);
        match worst < COLLAPSE_DISPERSION {
            true => Some(Alert {
                alert_type: "manifold_collapse".to_string(),
                message: format!(
                    "Phase dispersion {:.4} global / {:.4} frequent-band, below {:.2} — \
                     the lexicon is synchronising \
                     (see docs/how/02_the_kuramoto_step.md)",
                    current.phase_dispersion, current.band_dispersion, COLLAPSE_DISPERSION
                ),
                severity: 1.0 - worst,
            }),
            false => None,
        }
    }

    /// Records a benchmark and returns any alerts it raised.
    pub fn update(&mut self, report: BenchmarkReport) -> Vec<Alert> {
        let mut new_alerts = Vec::new();
        if let Some(a) = self.check_regression(&report) {
            new_alerts.push(a);
        }
        if let Some(a) = self.check_collapse(&report) {
            new_alerts.push(a);
        }
        self.history.record(report);
        self.alerts.extend(new_alerts.clone());
        new_alerts
    }
}
