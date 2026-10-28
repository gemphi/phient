#![allow(dead_code, unused_imports)]
//! Lifelong learning coordinator: component reuse, meta-learning, and benchmark tracking.
//!
//! Implements François Chollet's vision of lifelong learning (Ch 14.5):
//! accumulating reusable abstractions and programs across sequential tasks,
//! adapting rapidly via meta-learned phase priors, and monitoring distribution drift.
//! All operations are encapsulated in [`LifelongLearner`], following the Diem
//! convention that all public symbols belong to named types.
//!
//! # Architecture
//!
//! ```text
//! Incoming Task
//!   │
//!   ▼
//! LifelongLearner::learn_task()
//!   ├─▶ 1. Query ComponentLibrary (Phase-signature similarity)
//!   │       └─▶ If matched: apply & increment reuse_count
//!   ├─▶ 2. Online Task Adaptation (Meta-learning priors)
//!   └─▶ 3. Extract & Register Novel Patterns in ComponentLibrary
//! ```

pub mod meta;
pub mod history;
pub mod reuse;
pub mod monitor;

use crate::eval::Evaluator;
use phiecs::phasor::facet::Facet;
use crate::train::synthesis::library::ComponentLibrary;
use phiton::p64::words::Tokenizer;
use crate::train::trainer::Trainer;
pub use history::{BenchmarkEntry, BenchmarkHistory};
pub use meta::{MetaLearner, MetaModel};
pub use monitor::{Alert, ModelMonitor};
pub use reuse::{FeatureReuse, FeatureSet};
use serde::Serialize;

/// Outcome of a lifelong learning task evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct LearnResult {
    /// Identifier or prompt of the task.
    pub task: String,
    /// Whether an existing component from the library was reused.
    pub reused_component: Option<String>,
    /// Coherence score after learning. Never read alone: it is the Kuramoto
    /// order parameter, which rises as the manifold collapses.
    pub coherence: f64,
    /// Phase dispersion after learning, reported beside coherence so the two
    /// can be read together.
    pub dispersion: f64,
    /// Word positions warm-started from the reused component.
    pub warm_started: usize,
    /// Number of training iterations performed.
    pub iterations: usize,
}

/// Outcome of cross-domain knowledge transfer between facets.
#[derive(Debug, Clone, Serialize)]
pub struct TransferResult {
    pub source_label: String,
    pub target_label: String,
    pub features_transferred: usize,
    pub post_transfer_coherence: f64,
}

/// Central coordinator for lifelong learning, component synthesis, and meta-adaptation.
#[derive(Debug, Default)]
pub struct LifelongLearner {
    /// Persistent library of synthesized program abstractions.
    pub library: ComponentLibrary,
    /// Chronological history of benchmark reports.
    pub history: BenchmarkHistory,
}

impl LifelongLearner {
    /// Creates a new, empty [`LifelongLearner`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Solves an incoming task by querying the library first, then adapting online.
    pub fn learn_task(
        &mut self,
        facet: &mut Facet,
        trainer: &Trainer,
        task: &str,
    ) -> LearnResult {
        // 1. Look for a component whose phase shape matches this task.
        let matched: Option<(String, Vec<(String, f64)>)> = self
            .library
            .find_reusable(facet, task)
            .map(|c| (c.name.clone(), c.word_phases.clone()));

        // 2. Warm-start from it.
        //
        // Reuse previously only lowered the iteration count from 16 to 4: the
        // matched component's program was never executed and its positions were
        // never applied, so nothing was transferred and the saving was asserted
        // rather than earned. Here the component's learned positions are pulled
        // into the facet first, which is what makes fewer iterations sufficient.
        let mut warm_started = 0usize;
        if let Some((name, word_phases)) = &matched {
            for (word, target) in word_phases {
                if let Some(p) = facet.lexicon.get_mut(word) {
                    let d = phistd::wrap(target - p.theta(0));
                    p.nudge(0, 0.3 * d);
                    p.sync_phase();
                    warm_started += 1;
                }
            }
            self.library.mark_used(name);
        }

        // 3. Online training on the task.
        let iterations = match matched.is_some() {
            true => 4,
            false => 16,
        };
        for _ in 0..iterations {
            trainer.train_sentence(facet, task);
        }

        let evaluator = Evaluator::new();
        LearnResult {
            task: task.to_string(),
            reused_component: matched.map(|(n, _)| n),
            coherence: evaluator.eval(facet, task).coherence,
            dispersion: facet.phase_dispersion(),
            warm_started,
            iterations,
        }
    }

    /// Transfers the relational structure of a source domain onto a target.
    ///
    /// `features_transferred` now counts **words actually moved**. It previously
    /// counted extracted feature sets while the apply step created one synthetic
    /// `meta_sector_N` token per set and overwrote it in a loop — so the number
    /// reported a transfer that had not happened.
    pub fn transfer_knowledge(
        &mut self,
        facet: &mut Facet,
        source_label: &str,
        target_label: &str,
    ) -> TransferResult {
        let source: Vec<String> = Tokenizer::content_words(source_label);
        let target: Vec<String> = Tokenizer::content_words(target_label);
        let moved = FeatureReuse::apply_relational(facet, &source, &target, 0.4);

        let evaluator = Evaluator::new();
        TransferResult {
            source_label: source_label.to_string(),
            target_label: target_label.to_string(),
            features_transferred: moved,
            post_transfer_coherence: evaluator.eval(facet, target_label).coherence,
        }
    }
}
