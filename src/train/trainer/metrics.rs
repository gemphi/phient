use std::fmt;
use std::time::Duration;

/// Training metrics - tracks throughput and progress during ingestion.
#[derive(Debug, Clone, Default)]
pub struct TrainingMetric {
    pub epochs_completed: usize,
    pub words_learned: usize,
    pub total_time: Duration,
}

impl TrainingMetric {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn words_per_sec(&self) -> f64 {
        let secs = self.total_time.as_secs_f64();
        if secs > 0.0 {
            self.words_learned as f64 / secs
        } else {
            0.0
        }
    }

    pub fn report(&self) {
        println!(
            "  [metrics] {} epochs, {} words, {:?} total, {:.0} words/sec",
            self.epochs_completed,
            self.words_learned,
            self.total_time,
            self.words_per_sec(),
        );
    }
}

impl fmt::Display for TrainingMetric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} epochs, {} words, {:?} ({:.0} w/s)",
            self.epochs_completed,
            self.words_learned,
            self.total_time,
            self.words_per_sec(),
        )
    }
}

/// Result of a multi-epoch training run.
#[derive(Debug, Clone)]
pub struct MultiEpochResult {
    pub epochs: usize,
    pub tokens_learned: usize,
    pub converged: bool,
}

impl fmt::Display for MultiEpochResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} epochs, {} tokens, {}",
            self.epochs,
            self.tokens_learned,
            if self.converged { "converged" } else { "not converged" },
        )
    }
}
