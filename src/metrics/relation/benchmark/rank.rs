#[allow(unused_imports)]
use super::super::*;

impl RelationBenchmark {

    /// Ranks the whole vocabulary against a query phasor, returning the rank of
    /// `target` (1-based) and the top matches. Excluded words are skipped.
    pub(in crate::metrics::relation) fn rank_of(
        facet: &Facet,
        query: &SpectralPhasor,
        target: &str,
        exclude: &[&str],
    ) -> Option<(usize, Vec<String>)> {
        Self::rank_of_above(facet, query, target, exclude, 0)
    }

    pub(in crate::metrics::relation) fn rank_of_above(
        facet: &Facet,
        query: &SpectralPhasor,
        target: &str,
        exclude: &[&str],
        floor: CountFloor,
    ) -> Option<(usize, Vec<String>)> {
        // Only the target's rank and the top few are wanted, so counting how
        // many words beat the target is enough — no full sort of the vocabulary,
        // which at 70k words inside a doubly-nested analogy loop was the reason
        // the expanded probe set wedged the experiment.
        let tp = facet.lexicon.get(target)?;
        let target_score = query.resonance(tp);

        let beating = facet
            .lexicon
            .par_iter()
            // The floor defines the pool the rank is *against*. Without this the
            // sweep filtered only the probe pairs and the chance denominator
            // while every ranking still ran over the whole lexicon, so the
            // reported ratios compared a rank from one pool to a chance level
            // from another.
            .filter(|(_, p)| p.count >= floor)
            .filter(|(w, _)| w.as_str() != target && !exclude.contains(&w.as_str()))
            .filter(|(_, p)| query.resonance(p) > target_score)
            .count();

        Some((beating + 1, Vec::new()))
    }

    /// As [`RelationBenchmark::rank_of`], but also returns the nearest words.
    ///
    /// Split out because the ranking path runs millions of times and does not
    /// need the names, while inspection runs rarely and does.
    pub fn nearest(facet: &Facet, query: &SpectralPhasor, exclude: &[&str], k: usize) -> Vec<String> {
        let mut scored: Vec<(&str, f64)> = facet
            .lexicon
            .iter()
            .filter(|(w, _)| !exclude.contains(&w.as_str()))
            .map(|(w, p)| (w.as_str(), query.resonance(p)))
            .collect();
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(b.0))
        });
        scored.into_iter().take(k).map(|(w, _)| w.to_string()).collect()
    }

    /// Builds the analogy query `c + (b − a)`, per channel.
    pub(in crate::metrics::relation) fn analogy_query(
        facet: &Facet,
        a: &str,
        b: &str,
        c: &str,
    ) -> Option<SpectralPhasor> {
        let (pa, pb, pc) = (
            facet.lexicon.get(a)?,
            facet.lexicon.get(b)?,
            facet.lexicon.get(c)?,
        );
        let mut q = *pc;
        for k in 0..SpectralPhasor::channels() {
            q.set_theta(k, pc.theta(k) + pb.theta(k) - pa.theta(k));
        }
        q.sync_phase();
        Some(q)
    }

    /// Evaluates one family.
    /// Words eligible for the candidate pool at a given floor.
    pub fn pool_size(facet: &Facet, floor: CountFloor) -> usize {
        match floor {
            0 => facet.vocabulary_size(),
            f => facet.lexicon.values().filter(|p| p.count >= f).count(),
        }
    }

    pub fn evaluate_family(facet: &Facet, family: &RelationFamily) -> FamilyResult {
        Self::evaluate_family_above(facet, family, 0)
    }
}
