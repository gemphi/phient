//! Feature and architecture reuse: extracts and applies reusable phase patterns.
//!
//! A transferable feature is a **shape**, not a location: the arrangement of a
//! group of words relative to their own centre, which can be carried to a
//! different region of the manifold and re-imposed on different words.
//!
//! The previous implementation created one synthetic token per feature set,
//! named `meta_sector_N`, and then overwrote its phase once per element of the
//! pattern — so the token ended at the last phase in the vector and every
//! earlier one was discarded. Nothing was transferred, and
//! `TransferResult.features_transferred` counted the feature sets anyway.

use phiecs::phasor::facet::SECTOR_RESOLUTION;
use phiton::units::TWO_PI;
use phiecs::phasor::facet::Facet;
use serde::Serialize;

/// A group of words and the shape of their arrangement.
#[derive(Debug, Clone, Serialize)]
pub struct FeatureSet {
    pub label: String,
    /// Member words, in a stable order.
    pub words: Vec<String>,
    /// Each member's signed phase offset from the group centroid — the shape.
    pub offsets: Vec<f64>,
    /// Where the group currently sits.
    pub centroid: f64,
}

#[derive(Debug, Default)]
pub struct FeatureReuse;

impl FeatureReuse {
    /// Extracts the arrangement of each populated phase sector.
    pub fn extract(facet: &Facet) -> Vec<FeatureSet> {
        let n = SECTOR_RESOLUTION as usize;
        let width = TWO_PI / n as f64;
        let mut buckets: Vec<Vec<String>> = vec![Vec::new(); n];

        for (word, phasor) in &facet.lexicon {
            let s = (phasor.theta(0) / width).floor() as usize % n;
            buckets[s].push(word.clone());
        }

        buckets
            .into_iter()
            .enumerate()
            .filter(|(_, w)| w.len() >= 3)
            .filter_map(|(idx, mut words)| {
                words.sort();
                let centroid = Self::centroid(facet, &words)?;
                let offsets = words
                    .iter()
                    .filter_map(|w| facet.lexicon.get(w).map(|p| phistd::wrap(p.theta(0) - centroid)))
                    .collect();
                Some(FeatureSet { label: format!("sector_{}", idx), words, offsets, centroid })
            })
            .collect()
    }

    /// Circular centroid of a group of words on channel 0.
    fn centroid(facet: &Facet, words: &[String]) -> Option<f64> {
        let mut r = phical::circle::Resultant::new();
        let mut n = 0;
        for w in words {
            if let Some(p) = facet.lexicon.get(w) {
                r.add(p.amplitude, p.theta(0));
                n += 1;
            }
        }
        match n {
            0 => None,
            _ => Some(r.direction()),
        }
    }

    /// Re-imposes each feature set's own shape on its own words.
    ///
    /// A consolidation pass: it sharpens arrangements the model has already
    /// found rather than inventing tokens. Returns how many words moved.
    pub fn apply(facet: &mut Facet, features: &[FeatureSet], strength: f64) -> usize {
        let mut moved = 0;
        for fs in features {
            for (w, off) in fs.words.iter().zip(&fs.offsets) {
                let target = fs.centroid + off;
                if let Some(p) = facet.lexicon.get_mut(w) {
                    let d = phistd::wrap(target - p.theta(0));
                    p.nudge(0, strength * d);
                    p.sync_phase();
                    moved += 1;
                }
            }
        }
        moved
    }

    /// Transfers the *relational structure* of a source domain onto a target.
    ///
    /// The source arrangement is translated so its centre lands on the target's
    /// centre, then each target word is drawn toward the corresponding source
    /// position. This is an analogy operation — "arrange the target domain the
    /// way the source domain is arranged" — and it is what transfer between
    /// tasks actually requires.
    ///
    /// Returns the number of target words moved.
    pub fn apply_relational(
        facet: &mut Facet,
        source: &[String],
        target: &[String],
        strength: f64,
    ) -> usize {
        let (src_c, dst_c) = match (Self::centroid(facet, source), Self::centroid(facet, target)) {
            (Some(a), Some(b)) => (a, b),
            _ => return 0,
        };
        let shift = phistd::wrap(dst_c - src_c);

        let mut moved = 0;
        for (s, d) in source.iter().zip(target) {
            let want = match facet.lexicon.get(s) {
                Some(p) => p.theta(0) + shift,
                None => continue,
            };
            if let Some(p) = facet.lexicon.get_mut(d) {
                let diff = phistd::wrap(want - p.theta(0));
                p.nudge(0, strength * diff);
                p.sync_phase();
                moved += 1;
            }
        }
        moved
    }

    /// Shape similarity between two feature sets, independent of where they sit.
    pub fn similarity(a: &FeatureSet, b: &FeatureSet) -> f64 {
        phical::circle::shape_similarity(&a.offsets, &b.offsets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::train::trainer::Trainer;

    #[test]
    fn test_apply_creates_no_synthetic_tokens() {
        let mut facet = Facet::new();
        let t = Trainer::new(0.05);
        for s in ["alpha beta gamma delta", "epsilon zeta eta theta"] {
            t.train_sentence(&mut facet, s);
        }
        let before = facet.vocabulary_size();
        let feats = FeatureReuse::extract(&facet);
        FeatureReuse::apply(&mut facet, &feats, 0.3);
        assert_eq!(facet.vocabulary_size(), before, "transfer must not invent vocabulary");
        assert!(!facet.lexicon.keys().any(|k| k.starts_with("meta_")));
    }

    #[test]
    fn test_relational_transfer_moves_target_words() {
        let mut facet = Facet::new();
        for w in ["cat", "kitten", "dog", "puppy"] {
            facet.get_or_init(w);
        }
        let src = vec!["cat".to_string(), "kitten".to_string()];
        let dst = vec!["dog".to_string(), "puppy".to_string()];
        let before = facet.lexicon["puppy"].phase;
        let moved = FeatureReuse::apply_relational(&mut facet, &src, &dst, 0.5);
        assert_eq!(moved, 2);
        assert!((facet.lexicon["puppy"].phase - before).abs() > 1e-9);
    }
}
