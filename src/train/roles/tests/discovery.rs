#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

/// Discovery must recover relations it was never told about.
///
/// Two synthetic relations are planted by construction — filler = head
/// rotated by a fixed offset — and the clusterer is given the pairs with no
/// labels. If it cannot separate two relations that are exactly separable,
/// it will not find real ones.
#[test]
fn test_discovery_separates_planted_relations() {
    let mut f = Facet::new();
    let heads: Vec<String> = (0..24).map(|i| format!("h{}", i)).collect();
    let rot_a: Vec<f64> = (0..PHASE_CHANNELS).map(|k| (k as f64 * 0.37) % TWO_PI).collect();
    let rot_b: Vec<f64> = (0..PHASE_CHANNELS).map(|k| (k as f64 * 2.11 + 1.7) % TWO_PI).collect();

    let mut pairs = Vec::new();
    let mut labels: HashMap<(String, String), String> = HashMap::new();
    for (i, h) in heads.iter().enumerate() {
        let hp = SpectralPhasor::seeded(h, 1.0, 1);
        f.lexicon.insert(h.clone(), hp);

        let (rot, name) = match i % 2 {
            0 => (&rot_a, "alpha"),
            _ => (&rot_b, "beta"),
        };
        let fname = format!("f{}", i);
        let mut fp = hp;
        for (k, r) in rot.iter().enumerate() {
            fp.set_theta(k, hp.theta(k) + r);
        }
        fp.sync_phase();
        f.lexicon.insert(fname.clone(), fp);

        pairs.push((h.clone(), fname.clone()));
        labels.insert((h.clone(), fname), name.to_string());
    }

    let clusters = RoleDiscovery::discover(&f, &pairs, 2, 20);
    assert_eq!(clusters.len(), 2);

    let (purity, chance) = RoleDiscovery::purity(&clusters, &labels);
    assert!(
        purity > 0.9,
        "two exactly-separable relations must be separated: purity {} vs chance {}",
        purity,
        chance
    );
    for c in &clusters {
        assert!(c.coherence > 0.9, "a recovered relation must be tight: {}", c.coherence);
    }
}

/// Discovery must be reproducible: seeded by farthest point, not by chance.
#[test]
fn test_discovery_is_deterministic() {
    let mut f = Facet::new();
    let mut pairs = Vec::new();
    for i in 0..20 {
        let (h, fl) = (format!("head{}", i), format!("fill{}", i));
        f.lexicon.insert(h.clone(), SpectralPhasor::seeded(&h, 1.0, 1));
        f.lexicon.insert(fl.clone(), SpectralPhasor::seeded(&fl, 1.0, 1));
        pairs.push((h, fl));
    }
    let a = RoleDiscovery::discover(&f, &pairs, 3, 10);
    let b = RoleDiscovery::discover(&f, &pairs, 3, 10);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.members, y.members, "clustering must not move between runs");
    }
}
