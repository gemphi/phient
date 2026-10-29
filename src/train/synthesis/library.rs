//! Reusable component library: stores synthesized programs for reuse.
//!
//! Matching is by **phase histogram**, not by a positional list of angles.
//! The previous similarity compared position *i* of one signature to position
//! *i* of another, so `"sort the list"` and `"please sort the list"` — the same
//! task, offset by one word — were compared entirely out of alignment. It also
//! scored ~0.5 on average for two unrelated signatures (the mean of
//! `1 − |Δ|/π` for uniform Δ), against a 0.6 acceptance threshold: roughly
//! 0.7σ above chance for a four-token task, and 1.0 for everything once the
//! manifold synchronised.

use super::program::Program;
use phiecs::phasor::facet::SECTOR_RESOLUTION;
use phiton::units::TWO_PI;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use serde::Serialize;

/// Cosine similarity above which two tasks are treated as the same shape.
/// Sparse L2-normalised histograms of unrelated word sets score far below this.
const MATCH_THRESHOLD: f64 = 0.6;

#[derive(Debug, Clone, Serialize)]
pub struct Component {
    pub name: String,
    pub program: Program,
    /// L2-normalised occupancy histogram over phase sectors. Order-invariant.
    pub histogram: Vec<f32>,
    /// The component's learned word positions, used to warm-start a new task.
    pub word_phases: Vec<(String, f64)>,
    pub reuse_count: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ComponentLibrary {
    pub components: Vec<Component>,
}

impl ComponentLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a reusable component, keyed on the phase shape of `text`.
    pub fn register(&mut self, name: &str, program: Program, facet: &Facet, text: &str) {
        self.components.push(Component {
            name: name.to_string(),
            program,
            histogram: Self::histogram(facet, text),
            word_phases: Self::word_phases(facet, text),
            reuse_count: 0,
        });
    }

    /// Finds the component whose phase shape best matches a task.
    pub fn find_reusable(&self, facet: &Facet, task: &str) -> Option<&Component> {
        let target = Self::histogram(facet, task);
        if target.iter().all(|v| *v == 0.0) {
            return None;
        }

        let (best, score) = self.components.iter().fold((None, 0.0f64), |acc, comp| {
            let sim = Self::cosine(&comp.histogram, &target);
            match sim > acc.1 {
                true => (Some(comp), sim),
                false => acc,
            }
        });

        match score > MATCH_THRESHOLD {
            true => best,
            false => None,
        }
    }

    /// Increments reuse count for a component.
    pub fn mark_used(&mut self, name: &str) {
        if let Some(c) = self.components.iter_mut().find(|c| c.name == name) {
            c.reuse_count += 1;
        }
    }

    /// L2-normalised histogram of a text's words over phase sectors.
    fn histogram(facet: &Facet, text: &str) -> Vec<f32> {
        let n = SECTOR_RESOLUTION as usize;
        let width = TWO_PI / n as f64;
        let mut h = vec![0.0f32; n];

        for token in Tokenizer::content_words(text) {
            if let Some(p) = facet.lexicon.get(&token) {
                let s = (p.theta(0) / width).floor() as usize % n;
                h[s] += 1.0;
            }
        }

        phistd::real::normalise_f32(&mut h);
        h
    }

    fn word_phases(facet: &Facet, text: &str) -> Vec<(String, f64)> {
        Tokenizer::content_words(text)
            .into_iter()
            .filter_map(|t| facet.lexicon.get(&t).map(|p| (t.clone(), p.theta(0))))
            .collect()
    }

    fn cosine(a: &[f32], b: &[f32]) -> f64 {
        phistd::real::dot_f32(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::train::synthesis::program::Program;
    use crate::train::trainer::Trainer;

    fn setup() -> (Facet, ComponentLibrary) {
        let mut facet = Facet::new();
        let t = Trainer::new(0.05);
        for s in [
            "sort the numeric list ascending",
            "braise the beef with onions",
        ] {
            t.train_sentence(&mut facet, s);
        }
        let mut lib = ComponentLibrary::new();
        lib.register("sorter", Program::identity(), &facet, "sort the numeric list ascending");
        (facet, lib)
    }

    /// The failure the positional signature had: the same task, one word longer.
    #[test]
    fn test_matching_is_order_and_length_invariant() {
        let (facet, lib) = setup();
        let m = lib.find_reusable(&facet, "please sort the list numeric ascending");
        assert!(m.is_some(), "a re-ordered, longer phrasing of the same task must still match");
        assert_eq!(m.unwrap().name, "sorter");
    }

    #[test]
    fn test_unrelated_task_does_not_match() {
        let (facet, lib) = setup();
        assert!(
            lib.find_reusable(&facet, "braise the beef with onions").is_none(),
            "an unrelated task must not match above chance"
        );
    }
}
