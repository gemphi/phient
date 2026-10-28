#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

/// The floor has to reach the ranker, not just the probe filter.
///
/// This is the bug the sweep shipped with: `evaluate_above` filtered which
/// pairs were scored and which pool the chance level was computed from, but
/// every ranking still ran over the whole lexicon. A rank drawn from 70,000
/// distractors reported against a 4,000-word chance level is not a
/// measurement of anything.
#[test]
fn test_the_floor_restricts_the_ranking_pool() {
    let mut facet = Facet::new();

    // The query sits exactly on the rare words and a little off the target,
    // so every rare word is a *strictly* closer match. Above the floor the
    // target stands alone and ranks first; at floor 0 the rare words are in
    // the pool and bury it.
    let mut query = SpectralPhasor::new(0.0, 1.0, 1);
    for k in 0..SpectralPhasor::channels() {
        query.set_theta(k, 0.0);
    }
    query.sync_phase();

    {
        let target = facet.get_or_init("target");
        target.count = 100;
        for k in 0..SpectralPhasor::channels() {
            target.set_theta(k, 0.5);
        }
        target.sync_phase();
    }

    for i in 0..200 {
        let p = facet.get_or_init(&format!("rare{i}"));
        p.count = 1;
        for k in 0..SpectralPhasor::channels() {
            p.set_theta(k, 0.0);
        }
        p.sync_phase();
    }

    let (rank_all, _) =
        RelationBenchmark::rank_of_above(&facet, &query, "target", &[], 0).unwrap();
    let (rank_floored, _) =
        RelationBenchmark::rank_of_above(&facet, &query, "target", &[], 50).unwrap();

    assert!(
        rank_all > rank_floored,
        "the rare words should be distractors at floor 0 ({rank_all}) and \
         absent at floor 50 ({rank_floored})"
    );
    assert_eq!(rank_floored, 1, "above the floor the target stands alone");
}

#[test]
fn test_pool_size_tracks_the_floor() {
    let mut facet = Facet::new();
    for i in 0..10 {
        let p = facet.get_or_init(&format!("w{i}"));
        p.count = i as u32 + 1;
    }
    assert_eq!(RelationBenchmark::pool_size(&facet, 0), 10);
    assert_eq!(RelationBenchmark::pool_size(&facet, 5), 6);
    assert_eq!(RelationBenchmark::pool_size(&facet, 11), 0);
}
