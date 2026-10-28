#[allow(unused_imports)]
use super::super::*;

impl RelationBenchmark {

    /// [`RelationBenchmark::evaluate_family`] with a pool floor.
    pub fn evaluate_family_above(
        facet: &Facet,
        family: &RelationFamily,
        floor: CountFloor,
    ) -> FamilyResult {
        let above = |w: &str| {
            facet.lexicon.get(w).is_some_and(|p| p.count >= floor.max(1))
        };
        // A probe pair whose own words are below the floor cannot be scored
        // against a pool that excludes them.
        let usable: Vec<&RelationPair> = family
            .pairs
            .iter()
            .filter(|p| above(&p.a) && above(&p.b))
            .collect();

        let mut wins_frac = 0.0f64;
        let mut wins_n = 0usize;
        let mut top10 = 0usize;
        let mut top50 = 0usize;

        for (i, p) in usable.iter().enumerate() {
            let pa = &facet.lexicon[&p.a];
            let pb = &facet.lexicon[&p.b];

            // 1. pair versus random, averaged over many deterministic draws
            let target = pa.resonance(pb);
            // Distractors come from the pool, not the lexicon, and in a fixed
            // order — a HashMap's iteration order is not stable across runs, and
            // this vector is indexed by a deterministic hash.
            let mut vocab: Vec<&String> = facet
                .lexicon
                .iter()
                .filter(|(_, p)| p.count >= floor)
                .map(|(w, _)| w)
                .collect();
            vocab.sort_unstable();
            let (mut hits, mut draws) = (0usize, 0usize);
            for d in 0..RANDOM_DRAWS {
                let r = (fnv1a(&p.a) ^ ((i as u64) << 32) ^ (d as u64).wrapping_mul(0x9E3779B9))
                    as usize;
                let cand = vocab[r % vocab.len().max(1)];
                if cand == &p.a || cand == &p.b {
                    continue;
                }
                draws += 1;
                if target > pa.resonance(&facet.lexicon[cand]) {
                    hits += 1;
                }
            }
            if draws > 0 {
                wins_frac += hits as f64 / draws as f64;
                wins_n += 1;
            }

            // 2. neighbourhood
            if let Some((rank, _)) = Self::rank_of_above(facet, pa, &p.b, &[&p.a], floor) {
                if rank <= 10 {
                    top10 += 1;
                }
                if rank <= 50 {
                    top50 += 1;
                }
            }
        }

        // 3. analogy, over ordered pairs of distinct pairs in the family.
        //
        // Every ordered pair is n(n-1), which at 35 pairs per family across ten
        // families is ~11,000 full-vocabulary rankings *per condition* — the
        // expanded probe set made the honest exhaustive version unaffordable.
        // The stride below samples a fixed, deterministic subset so the cost is
        // linear in family size while the sample stays identical between runs;
        // an experiment whose probe set moves cannot support a comparison.
        let stride = (usable.len() / MAX_ANALOGY_PARTNERS.max(1)).max(1);
        let (mut a1, mut a5, mut mrr, mut tested) = (0usize, 0usize, 0.0f64, 0usize);
        for (i, p) in usable.iter().enumerate() {
            for (j, q) in usable.iter().enumerate() {
                if i == j || j % stride != i % stride {
                    continue;
                }
                let query = match Self::analogy_query(facet, &p.a, &p.b, &q.a) {
                    Some(x) => x,
                    None => continue,
                };
                let ranked = Self::rank_of_above(facet, &query, &q.b, &[&p.a, &p.b, &q.a], floor);
                if let Some((rank, _)) = ranked {
                    tested += 1;
                    if rank == 1 {
                        a1 += 1;
                    }
                    if rank <= 5 {
                        a5 += 1;
                    }
                    mrr += 1.0 / rank as f64;
                }
            }
        }

        let n = usable.len().max(1) as f64;
        let t = tested.max(1) as f64;

        FamilyResult {
            name: family.name.clone(),
            usable_pairs: usable.len(),
            pair_vs_random: match wins_n {
                0 => 0.0,
                k => wins_frac / k as f64,
            },
            neighbour_top10: top10 as f64 / n,
            neighbour_top50: top50 as f64 / n,
            analogy_top1: a1 as f64 / t,
            analogy_top5: a5 as f64 / t,
            analogy_mrr: mrr / t,
            analogies_tested: tested,
        }
    }

    /// Evaluates every family and summarises.
    pub fn evaluate(facet: &Facet, families: &[RelationFamily]) -> RelationReport {
        Self::evaluate_above(facet, families, 0)
    }

    /// [`RelationBenchmark::evaluate`] over a candidate pool restricted to words
    /// seen at least `floor` times.
    ///
    /// `vocabulary_size` in the returned report is the **pool** size, not the
    /// lexicon's, so the chance levels beside it are the ones that actually
    /// apply.
    pub fn evaluate_above(
        facet: &Facet,
        families: &[RelationFamily],
        floor: CountFloor,
    ) -> RelationReport {
        let results: Vec<FamilyResult> = families
            .iter()
            .map(|f| Self::evaluate_family_above(facet, f, floor))
            .collect();

        let v = Self::pool_size(facet, floor).max(1) as f64;
        let usable_total: usize = results.iter().map(|r| r.usable_pairs).sum();
        let tested_total: usize = results.iter().map(|r| r.analogies_tested).sum();

        let overall_pair = match usable_total {
            0 => 0.0,
            _ => results
                .iter()
                .map(|r| r.pair_vs_random * r.usable_pairs as f64)
                .sum::<f64>()
                / usable_total as f64,
        };
        let overall_analogy = match tested_total {
            0 => 0.0,
            _ => results
                .iter()
                .map(|r| r.analogy_top1 * r.analogies_tested as f64)
                .sum::<f64>()
                / tested_total as f64,
        };

        RelationReport {
            vocabulary_size: Self::pool_size(facet, floor),
            families: results,
            chance_neighbour_top10: 10.0 / v,
            chance_analogy_top1: 1.0 / v,
            overall_pair_vs_random: overall_pair,
            overall_analogy_top1: overall_analogy,
        }
    }
}
