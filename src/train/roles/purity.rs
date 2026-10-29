#[allow(unused_imports)]
use super::*;

impl RoleDiscovery {

    /// How well discovered clusters line up with labels they never saw.
    ///
    /// Purity: for each cluster, the share taken by its most common true label,
    /// weighted by cluster size. This is the only honest test of a discovery
    /// procedure — tidy-looking clusters prove nothing, and the labels must play
    /// no part in producing them.
    ///
    /// Chance purity is roughly the largest label's share of the data, so purity
    /// must be read against that and not against 1.0.
    pub fn purity(clusters: &[DiscoveredRole], labels: &HashMap<(String, String), String>) -> (f64, f64) {
        let mut correct = 0usize;
        let mut total = 0usize;
        let mut label_counts: HashMap<&str, usize> = HashMap::new();

        for c in clusters {
            let mut here: HashMap<&str, usize> = HashMap::new();
            for m in &c.members {
                if let Some(l) = labels.get(m) {
                    *here.entry(l.as_str()).or_insert(0) += 1;
                    *label_counts.entry(l.as_str()).or_insert(0) += 1;
                    total += 1;
                }
            }
            correct += here.values().copied().max().unwrap_or(0);
        }

        let chance = label_counts.values().copied().max().unwrap_or(0) as f64
            / total.max(1) as f64;
        (correct as f64 / total.max(1) as f64, chance)
    }
}

impl Coherence {
    /// Mean agreement between each pair's offset and the group's circular mean.
    ///
    /// This is the metric, and it is cheap enough to run every epoch: one pass
    /// for the mean, one for the agreement.
    pub fn measure(facet: &Facet, pairs: &[(String, String)]) -> f64 {
        let offsets: Vec<Vec<f64>> = pairs
            .iter()
            .filter_map(|(h, f)| RoleDiscovery::offset(facet, h, f))
            .collect();
        if offsets.len() < 2 {
            return 0.0;
        }
        let mean = phical::circle::centroid(&offsets, PHASE_CHANNELS);
        phical::circle::mean_agreement(&offsets, &mean)
    }

    /// Pulls every pair in a group toward the group's shared offset.
    ///
    /// For each `(head, filler)`, the filler moves toward `head + mean_offset`.
    /// The head is deliberately left alone: a word is the head of few relations
    /// and the filler of many, so moving both ends lets a common filler be
    /// dragged by every group it appears in, which is how the composition work
    /// collapsed the manifold before the anchor was added.
    ///
    /// Jacobi, not Gauss-Seidel — every target is computed against frozen phases
    /// and applied afterwards — so the result does not depend on the order the
    /// pairs happen to be listed in.
    ///
    /// Returns the coherence after the update.
    pub fn align(facet: &mut Facet, pairs: &[(String, String)], rate: f64) -> f64 {
        let offsets: Vec<(usize, Vec<f64>)> = pairs
            .iter()
            .enumerate()
            .filter_map(|(i, (h, f))| RoleDiscovery::offset(facet, h, f).map(|o| (i, o)))
            .collect();
        if offsets.len() < 2 {
            return 0.0;
        }

        let refs: Vec<&Vec<f64>> = offsets.iter().map(|(_, o)| o).collect();
        let mean = phical::circle::centroid(&refs, PHASE_CHANNELS);

        // READ: every target against frozen phases.
        let mut targets: Vec<(String, Vec<f64>)> = Vec::with_capacity(offsets.len());
        for (i, _) in &offsets {
            let (head, filler) = &pairs[*i];
            let h = match facet.lexicon.get(head) {
                Some(h) => *h,
                None => continue,
            };
            targets.push((
                filler.clone(),
                (0..PHASE_CHANNELS).map(|k| h.theta(k) + mean[k]).collect(),
            ));
        }

        // WRITE.
        for (filler, target) in &targets {
            if let Some(p) = facet.lexicon.get_mut(filler) {
                for (k, &t) in target.iter().enumerate() {
                    let cur = p.theta(k);
                    let d = phical::circle::delta(cur, t);
                    p.set_theta(k, cur + rate * d);
                }
                p.sync_phase();
            }
        }

        Self::measure(facet, pairs)
    }

    /// Aligns several relation groups, rejecting the result if the manifold
    /// concentrates past `dispersion_floor`.
    ///
    /// The guard is not optional. Pulling offsets together is the same class of
    /// update as definition composition, which drove phase dispersion from 0.986
    /// to 0.305 before a floor was added — and a manifold that has collapsed has
    /// perfect coherence, because every offset is zero. Without the guard this
    /// objective's optimum is the degenerate solution.
    pub fn align_groups(
        facet: &mut Facet,
        groups: &[Vec<(String, String)>],
        rate: f64,
        rounds: usize,
        dispersion_floor: f64,
    ) -> (f64, bool) {
        let before = facet.clone();
        let mut last = 0.0;

        for _ in 0..rounds {
            let mut sum = 0.0;
            let mut n = 0usize;
            for g in groups {
                if g.len() >= 2 {
                    sum += Self::align(facet, g, rate);
                    n += 1;
                }
            }
            last = match n {
                0 => 0.0,
                _ => sum / n as f64,
            };
        }

        if facet.phase_dispersion() < dispersion_floor {
            *facet = before;
            return (last, false);
        }
        (last, true)
    }
}
