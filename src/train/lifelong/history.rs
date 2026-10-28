//! Benchmark history tracking: records benchmark results over time.
//! Serializes to data/benchmark_history.json.

use crate::metrics::benchmark::runner::BenchmarkReport;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkEntry {
    pub timestamp: String,
    pub report: BenchmarkReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkHistory {
    pub entries: Vec<BenchmarkEntry>,
}

impl BenchmarkHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a benchmark report with a timestamp.
    pub fn record(&mut self, report: BenchmarkReport) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());
        self.entries.push(BenchmarkEntry { timestamp, report });
    }

    /// Held-out perplexity over time. Lower is better.
    ///
    /// This previously tracked the phase-baseline *coherence*, which is the
    /// Kuramoto order parameter — a quantity the training rule maximises and
    /// that rises as the manifold collapses. A trend line that improves while
    /// the model degrades is worse than no trend line.
    pub fn trend(&self) -> Vec<f64> {
        self.entries
            .iter()
            .filter_map(|e| e.report.headline_ppl())
            .collect()
    }

    /// Phase dispersion over time — the collapse trace, logged alongside.
    pub fn dispersion_trend(&self) -> Vec<f64> {
        self.entries.iter().map(|e| e.report.phase_dispersion).collect()
    }

    /// Saves history to a JSON file.
    pub fn save(&self, path: &str) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Loads history from a JSON file.
    pub fn load(path: &str) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Returns the most recent entry.
    pub fn latest(&self) -> Option<&BenchmarkEntry> {
        self.entries.last()
    }
}
