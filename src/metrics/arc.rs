//! Text-analogy benchmark.
//!
//! **This is not ARC-AGI.** ARC-AGI is a program-synthesis benchmark over grid
//! transformations; nothing here manipulates grids, and a score from this module
//! must not be reported as an ARC score. It is a proxy: given input→output text
//! pairs, can the model produce the right continuation for a held-out input?
//!
//! The previous implementation set
//! `predicted = format!("{} relates to the pattern", task.test_input)` — a fixed
//! template with no inference in the path — and marked a task correct when the
//! first word of the expected answer happened to appear inside that template.
//! Whatever that measured, it was not the model.
//!
//! Here the prediction actually comes from the trained facet, and scoring is
//! token overlap against the expected answer. The number will be low. It will
//! also be real, and it can go down.

use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use crate::train::trainer::Trainer;
use phiecs::phasor::wave::Wave;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArcTask {
    pub id: String,
    pub input_pairs: Vec<(String, String)>,
    pub test_input: String,
    pub expected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcResult {
    pub total: usize,
    /// Every expected token recovered.
    pub exact: usize,
    /// Some expected tokens recovered.
    pub partial: usize,
    pub failed: usize,
    /// Mean token-level F1 across tasks — the honest headline.
    pub mean_f1: f64,
    /// Reminder attached to every serialised result.
    pub note: String,
    pub details: Vec<ArcTaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcTaskResult {
    pub task_id: String,
    pub predicted: String,
    pub expected: String,
    pub f1: f64,
}

#[derive(Debug, Default)]
pub struct ArcBenchmark;

impl ArcBenchmark {
    /// Loads tasks from a JSON file.
    pub fn load_tasks(path: &str) -> Vec<ArcTask> {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    /// Predicts a continuation for `input` using the trained manifold.
    ///
    /// The input's bound wave is projected into the lexicon by ray casting, and
    /// the nearest content words that are not already in the input become the
    /// prediction. This is a weak predictor; it is at least a predictor.
    fn predict(facet: &Facet, input: &str, n_words: usize) -> String {
        let wave = Wave::text_bound(facet, input);
        if wave.norm() < 1e-12 {
            return String::new();
        }
        let seen: HashSet<String> = Tokenizer::tokenize(input).into_iter().collect();
        Wave::ray_cast(facet, wave, n_words * 6)
            .into_iter()
            .map(|(w, _)| w)
            .filter(|w| !seen.contains(w) && !Tokenizer::is_function_word(w) && w.len() > 1)
            .take(n_words)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Token-level F1 between a prediction and the expected answer.
    fn f1(predicted: &str, expected: &str) -> f64 {
        let p: HashSet<String> = Tokenizer::content_words(predicted).into_iter().collect();
        let e: HashSet<String> = Tokenizer::content_words(expected).into_iter().collect();
        if p.is_empty() || e.is_empty() {
            return 0.0;
        }
        let hit = p.intersection(&e).count() as f64;
        let precision = hit / p.len() as f64;
        let recall = hit / e.len() as f64;
        match precision + recall > 0.0 {
            true => 2.0 * precision * recall / (precision + recall),
            false => 0.0,
        }
    }

    /// Trains on each task's demonstration pairs, then predicts the held-out input.
    pub fn evaluate(facet: &mut Facet, trainer: &Trainer, tasks: &[ArcTask]) -> ArcResult {
        let mut details = Vec::new();
        let (mut exact, mut partial) = (0usize, 0usize);
        let mut f1_sum = 0.0f64;

        for task in tasks {
            for (input, output) in &task.input_pairs {
                trainer.train_sentence(facet, input);
                trainer.train_sentence(facet, output);
                trainer.train_sentence(facet, &format!("{} {}", input, output));
            }

            let expected_len = Tokenizer::content_words(&task.expected).len().max(1);
            let predicted = Self::predict(facet, &task.test_input, expected_len);
            let f1 = Self::f1(&predicted, &task.expected);

            match f1 {
                x if x >= 0.999 => exact += 1,
                x if x > 0.0 => partial += 1,
                _ => {}
            }
            f1_sum += f1;

            details.push(ArcTaskResult {
                task_id: task.id.clone(),
                predicted,
                expected: task.expected.clone(),
                f1,
            });
        }

        let total = tasks.len();
        ArcResult {
            total,
            exact,
            partial,
            failed: total - exact - partial,
            mean_f1: match total { 0 => 0.0, n => f1_sum / n as f64 },
            note: "Text-analogy proxy. NOT ARC-AGI: no grid transformations are \
                   attempted, and this score must not be reported as an ARC score."
                .to_string(),
            details,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction_is_not_a_template() {
        let mut facet = Facet::new();
        let t = Trainer::new(0.05);
        let tasks = vec![ArcTask {
            id: "t1".into(),
            input_pairs: vec![("hot".into(), "cold".into())],
            test_input: "big".into(),
            expected: "small".into(),
        }];
        let r = ArcBenchmark::evaluate(&mut facet, &t, &tasks);
        assert_eq!(r.total, 1);
        // whatever it predicts, it must not be the old fixed template
        assert!(!r.details[0].predicted.contains("relates to the pattern"));
        assert!(r.mean_f1 >= 0.0 && r.mean_f1 <= 1.0);
    }

    #[test]
    fn test_f1_is_symmetric_on_exact_match() {
        assert!((ArcBenchmark::f1("small stone", "stone small") - 1.0).abs() < 1e-9);
        assert_eq!(ArcBenchmark::f1("alpha", "beta"), 0.0);
    }
}
