#[allow(unused_imports)]
use super::*;

impl RoleDiscovery {
    /// The offset that takes `head` to `filler`, per channel.
    pub(crate) fn offset(facet: &Facet, head: &str, filler: &str) -> Option<Vec<f64>> {
        let (h, f) = (facet.lexicon.get(head)?, facet.lexicon.get(filler)?);
        Some((0..PHASE_CHANNELS).map(|k| f.theta(k) - h.theta(k)).collect())
    }

    /// Clusters `pairs` into `k` discovered relations.
    ///
    /// K-means on the torus ([`phical::circle::kmeans`]): assignment by offset
    /// agreement, update by circular mean. Seeding are chosen by farthest-point
    /// on the agreement metric rather than at random, so the result does not
    /// depend on a seed and two runs on the same data give the same types —
    /// which a discovery procedure has to, or the "types" are an artefact of
    /// initialisation.
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
        let offsets: Vec<&[f64]> = usable.iter().map(|(_, o)| o.as_slice()).collect();
        phical::circle::kmeans(&offsets, k, rounds, PHASE_CHANNELS)
            .into_iter()
            .map(|c| DiscoveredRole {
                members: c
                    .members
                    .iter()
                    .map(|i| (usable[*i].0 .0.clone(), usable[*i].0 .1.clone()))
                    .collect(),
                rotation: c.centroid,
                coherence: c.coherence,
            })
            .collect()
    }
}
