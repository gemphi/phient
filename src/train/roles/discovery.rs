#[allow(unused_imports)]
use super::*;

impl RoleDiscovery {
    /// The offset that takes `head` to `filler`, per channel.
    pub(crate) fn offset(facet: &Facet, head: &str, filler: &str) -> Option<Vec<f64>> {
        let (h, f) = (facet.lexicon.get(head)?, facet.lexicon.get(filler)?);
        Some((0..PHASE_CHANNELS).map(|k| f.theta(k) - h.theta(k)).collect())
    }

    /// Mean cosine between two offset vectors — 1.0 when the two pairs stand in
    /// the same relation, 0 when they are unrelated.
    pub(crate) fn agreement(a: &[f64], b: &[f64]) -> f64 {
        let mut s = 0.0;
        for (x, y) in a.iter().zip(b.iter()) {
            s += (x - y).cos();
        }
        s / a.len().max(1) as f64
    }

    /// Circular mean of a set of offset vectors.
    pub(crate) fn centroid(offsets: &[&Vec<f64>]) -> Vec<f64> {
        (0..PHASE_CHANNELS)
            .map(|k| {
                let (mut x, mut y) = (0.0f64, 0.0f64);
                for o in offsets {
                    x += o[k].cos();
                    y += o[k].sin();
                }
                match x.hypot(y) > 1e-12 {
                    true => y.atan2(x),
                    false => 0.0,
                }
            })
            .collect()
    }

    /// Clusters `pairs` into `k` discovered relations.
    ///
    /// K-means on the torus: assignment by offset agreement, update by circular
    /// mean. Seeding are chosen by farthest-point on the agreement metric rather
    /// than at random, so the result does not depend on a seed and two runs on
    /// the same data give the same types — which a discovery procedure has to,
    /// or the "types" are an artefact of initialisation.
    pub fn discover(
        facet: &Facet,
        pairs: &[(String, String)],
        k: usize,
        rounds: usize,
    ) -> Vec<DiscoveredRole> {
        let usable: Vec<(&(String, String), Vec<f64>)> = pairs
            .iter()
            .filter_map(|p| Self::offset(facet, &p.0, &p.1).map(|o| (p, o)))
            .collect();
        Self::cluster(&usable, k, rounds)
    }

    /// [`RoleDiscovery::discover`] over offsets that have already been computed.
    ///
    /// Split out because the offset is not always the plain difference. Head
    /// domination — pairs clustering because they share a word rather than a
    /// relation — is a *main effect* of the head, and removing it means
    /// centering the offsets before they are clustered. The clustering itself
    /// does not change, and must not: the comparison between estimators is only
    /// fair if everything downstream of the offset is identical.
    pub fn cluster(
        usable: &[(&(String, String), Vec<f64>)],
        k: usize,
        rounds: usize,
    ) -> Vec<DiscoveredRole> {
        if usable.len() < k || k == 0 {
            return Vec::new();
        }

        // Farthest-point seeding: start at the first pair, then repeatedly take
        // the pair least like everything chosen so far.
        let mut seeds: Vec<usize> = vec![0];
        while seeds.len() < k {
            let next = (0..usable.len())
                .filter(|i| !seeds.contains(i))
                .min_by(|a, b| {
                    let sim = |i: &usize| {
                        seeds
                            .iter()
                            .map(|s| Self::agreement(&usable[*i].1, &usable[*s].1))
                            .fold(f64::NEG_INFINITY, f64::max)
                    };
                    sim(a).partial_cmp(&sim(b)).unwrap_or(std::cmp::Ordering::Equal)
                });
            match next {
                Some(n) => seeds.push(n),
                None => break,
            }
        }

        let mut centroids: Vec<Vec<f64>> = seeds.iter().map(|s| usable[*s].1.clone()).collect();
        let mut assign: Vec<usize> = vec![0; usable.len()];

        for _ in 0..rounds {
            let mut moved = false;
            for (i, (_, off)) in usable.iter().enumerate() {
                let best = centroids
                    .iter()
                    .enumerate()
                    .max_by(|a, b| {
                        Self::agreement(off, a.1)
                            .partial_cmp(&Self::agreement(off, b.1))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(c, _)| c)
                    .unwrap_or(0);
                if assign[i] != best {
                    assign[i] = best;
                    moved = true;
                }
            }
            for (c, cen) in centroids.iter_mut().enumerate() {
                let members: Vec<&Vec<f64>> = usable
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| assign[*i] == c)
                    .map(|(_, (_, o))| o)
                    .collect();
                if !members.is_empty() {
                    *cen = Self::centroid(&members);
                }
            }
            if !moved {
                break;
            }
        }

        centroids
            .into_iter()
            .enumerate()
            .map(|(c, rotation)| {
                let members: Vec<(String, String)> = usable
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| assign[*i] == c)
                    .map(|(_, (p, _))| (p.0.clone(), p.1.clone()))
                    .collect();
                let coherence = match members.is_empty() {
                    true => 0.0,
                    false => {
                        let sum: f64 = usable
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| assign[*i] == c)
                            .map(|(_, (_, o))| Self::agreement(o, &rotation))
                            .sum();
                        sum / members.len() as f64
                    }
                };
                DiscoveredRole { rotation, members, coherence }
            })
            .collect()
    }
}
