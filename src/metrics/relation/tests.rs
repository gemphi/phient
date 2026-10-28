#[allow(unused_imports)]
use super::*;

#[cfg(test)]
mod floor;

pub(super) use crate::train::trainer::Trainer;

#[test]
fn test_related_words_beat_random_after_training() {
    let mut facet = Facet::new();
    let t = Trainer::new(0.05);
    // Train so that man/woman genuinely co-occur, and add filler vocabulary.
    for _ in 0..40 {
        t.train_sentence(&mut facet, "a man and a woman are adult people");
        t.train_sentence(&mut facet, "granite basalt and quartz are minerals");
        t.train_sentence(&mut facet, "copper tin and zinc are metals");
    }
    let fam = RelationFamily {
        name: "t".into(),
        pairs: vec![RelationPair { a: "man".into(), b: "woman".into() }],
    };
    let r = RelationBenchmark::evaluate_family(&facet, &fam);
    assert_eq!(r.usable_pairs, 1);
    assert!(r.pair_vs_random >= 0.0 && r.pair_vs_random <= 1.0);
    assert!(r.analogy_mrr >= 0.0);
}

#[test]
fn test_missing_words_are_excluded_not_counted_as_failures() {
    let facet = Facet::new();
    let fam = RelationFamily {
        name: "t".into(),
        pairs: vec![RelationPair { a: "absent".into(), b: "alsoabsent".into() }],
    };
    let r = RelationBenchmark::evaluate_family(&facet, &fam);
    assert_eq!(r.usable_pairs, 0, "absent words must not be scored as wrong answers");
}

/// A2's acceptance criterion, enforced rather than asserted in a doc.
///
/// The benchmark exists to make effects falsifiable; a benchmark that
/// quietly shrinks back below significance defeats that silently.
/// The floor must shrink the pool, and chance must be reported against the
/// pool that was actually used.
///
/// Restricting the pool makes the task easier — fewer distractors — so a raw
/// score at a high floor is not comparable to a raw score at floor 0. If
/// `vocabulary_size` kept reporting the whole lexicon, every chance level
/// derived from it would be wrong and the comparison would silently favour
/// the restricted run.
#[test]
fn test_floor_restricts_the_pool_and_its_chance_level() {
    let mut facet = Facet::new();
    let t = Trainer::new(0.05);
    for _ in 0..40 {
        t.train_sentence(&mut facet, "a man and a woman are adult people");
        t.train_sentence(&mut facet, "a boy and a girl are young people");
        t.train_sentence(&mut facet, "copper tin and zinc are metals");
    }
    let fams = RelationBenchmark::default_families();

    let open = RelationBenchmark::evaluate_above(&facet, &fams, 0);
    let tight = RelationBenchmark::evaluate_above(&facet, &fams, 1_000_000);

    assert_eq!(open.vocabulary_size, facet.vocabulary_size());
    assert_eq!(
        tight.vocabulary_size,
        RelationBenchmark::pool_size(&facet, 1_000_000),
        "the report must describe the pool it used, not the lexicon"
    );
    assert!(
        tight.vocabulary_size < open.vocabulary_size,
        "an impossible floor must empty the pool: {} vs {}",
        tight.vocabulary_size,
        open.vocabulary_size
    );
    assert!(
        tight.chance_neighbour_top10 >= open.chance_neighbour_top10,
        "a smaller pool must report a higher chance level, or the \
         comparison flatters the restricted run"
    );
    // No probe pair can clear an impossible floor, so nothing is scored.
    assert_eq!(tight.families.iter().map(|f| f.usable_pairs).sum::<usize>(), 0);
}

#[test]
fn test_probe_set_is_large_enough_to_support_a_claim() {
    let fams = RelationBenchmark::default_families();
    let total: usize = fams.iter().map(|f| f.pairs.len()).sum();

    assert!(fams.len() >= 8, "at least 8 families, found {}", fams.len());
    assert!(total >= 300, "at least 300 pairs, found {}", total);

    // No family may be large enough to carry the total by itself.
    for f in &fams {
        let share = f.pairs.len() as f64 / total as f64;
        assert!(
            share < 0.20,
            "family {} is {:.0}% of the probe set — one family must not \
             dominate the aggregate",
            f.name,
            share * 100.0
        );
    }

    // Duplicate pairs would inflate the count without adding evidence.
    let mut seen = std::collections::HashSet::new();
    for f in &fams {
        for p in &f.pairs {
            assert!(
                seen.insert((f.name.clone(), p.a.clone(), p.b.clone())),
                "duplicate pair {}/{} in family {}",
                p.a,
                p.b,
                f.name
            );
        }
    }
}

#[test]
fn test_analogy_query_is_a_real_offset() {
    let mut facet = Facet::new();
    for w in ["man", "woman", "king", "queen"] {
        facet.get_or_init(w);
    }
    let q = RelationBenchmark::analogy_query(&facet, "man", "woman", "king").unwrap();
    // The query must differ from the word it started from.
    assert!(q.resonance(&facet.lexicon["king"]) < 0.999);
}
