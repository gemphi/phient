#[allow(unused_imports)]
use super::*;

impl Descent {

    /// [`Descent::cycle`] with the concentration threshold exposed.
    pub fn cycle_with(
        facet: &mut Facet,
        sentences: &[Vec<String>],
        up: Up,
        rounds: usize,
        step: f64,
        contrast: bool,
        concentration: f64,
    ) -> DescentReport {
        let (dispersion_before, band_before) = crate::train::grounding::dispersion_pair(facet);

        // Only sentences whose words the lexicon knows contribute anything, and
        // a sentence of one known word has an empty leave-one-out state.
        let usable: Vec<Vec<(String, usize)>> = sentences
            .iter()
            .map(|s| {
                s.iter()
                    .enumerate()
                    .filter(|(_, w)| facet.lexicon.contains_key(*w))
                    .map(|(i, w)| (w.clone(), i))
                    .collect::<Vec<_>>()
            })
            .filter(|s: &Vec<(String, usize)>| s.len() >= 2)
            .collect();

        let words_seen = {
            let mut set = std::collections::HashSet::new();
            for s in &usable {
                for (w, _) in s {
                    set.insert(w.as_str());
                }
            }
            set.len()
        };

        let mut candidate = facet.clone();
        let mut words_moved = 0usize;

        for _ in 0..rounds.max(1) {
            let votes = Self::votes(&candidate, &usable, up);
            words_moved = Self::settle(&mut candidate, &votes, usable.len(), step, contrast, concentration);
        }

        let (dispersion_after, band_after) = crate::train::grounding::dispersion_pair(&candidate);
        let floor = crate::train::grounding::DISPERSION_FLOOR;
        let rejected = dispersion_after < floor || band_after < floor;

        if !rejected {
            *facet = candidate;
        }

        DescentReport {
            rounds: rounds.max(1),
            sentences: usable.len(),
            words_moved,
            words_seen,
            dispersion_before,
            dispersion_after,
            band_before,
            band_after,
            rejected,
        }
    }
}

