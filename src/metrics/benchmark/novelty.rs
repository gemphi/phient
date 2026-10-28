//! Novel task benchmark: measures performance on tasks the model has never seen.

use crate::eval::Evaluator;
use phiecs::phasor::facet::Facet;
use crate::train::trainer::Trainer;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NovelTaskReport {
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub n_tasks: usize,
    pub scores: Vec<f64>,
}

#[derive(Debug, Default)]
pub struct NoveltyBenchmark;

impl NoveltyBenchmark {
    /// Presents a task the model has never seen and measures first-response quality.
    pub fn task_score(
        facet: &mut Facet,
        trainer: &Trainer,
        description: &str,
    ) -> f64 {
        let evaluator = Evaluator::new();

        trainer.train_online(facet, description);
        let result = evaluator.eval(facet, description);

        (result.coherence + result.resonance) / 2.0
    }

    /// Batch novel task evaluation.
    pub fn batch(
        facet: &mut Facet,
        trainer: &Trainer,
        tasks: &[String],
    ) -> NovelTaskReport {
        let mut scores = Vec::with_capacity(tasks.len());

        for task in tasks {
            let score = Self::task_score(facet, trainer, task);
            scores.push(score);
        }

        let mean = scores.iter().sum::<f64>() / scores.len().max(1) as f64;
        let max = scores.iter().cloned().fold(0.0f64, f64::max);
        let min = scores.iter().cloned().fold(1.0f64, f64::min);

        NovelTaskReport { mean, min, max, n_tasks: tasks.len(), scores }
    }
}
