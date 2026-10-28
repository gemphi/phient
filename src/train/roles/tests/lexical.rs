#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

fn content(w: &str) -> bool {
    w.len() > 2
        && !matches!(
            w,
            "the" | "a" | "an" | "and" | "or" | "of" | "for" | "in" | "with" | "by" | "to"
        )
}

/// The preposition names the relation, and different prepositions give
/// different relations for the same head.
#[test]
fn test_prepositions_name_distinct_relations() {
    let mut r = LexicalRule::new();
    r.extract("hammer", "an instrument for driving nails made of iron", content);
    r.extract("pneumonia", "a disease of the lungs", content);
    r.extract("pot", "a vessel used in cooking", content);

    let for_pairs = r.pairs_for_role("for");
    let of_pairs = r.pairs_for_role("of");
    let in_pairs = r.pairs_for_role("in");

    assert!(
        for_pairs.iter().any(|(h, f)| h == "hammer" && f == "driving"),
        "for(hammer, driving) expected, got {:?}",
        for_pairs
    );
    assert!(
        of_pairs.iter().any(|(h, f)| h == "pneumonia" && f == "lungs"),
        "of(pneumonia, lungs) expected, got {:?}",
        of_pairs
    );
    assert!(
        in_pairs.iter().any(|(h, f)| h == "pot" && f == "cooking"),
        "in(pot, cooking) expected — this is Searle's context C, and the \\
         enum extractor never found it once, got {:?}",
        in_pairs
    );
}

/// One gloss must yield several relations, which is the whole point: the
/// enum extractor averaged 1.06 per head and left the mechanism idle.
#[test]
fn test_one_gloss_yields_several_relations() {
    let mut r = LexicalRule::new();
    r.extract(
        "hammer",
        "an instrument for driving nails with a head of iron used by a smith",
        content,
    );
    assert!(
        r.len() >= 3,
        "a gloss with three prepositions must give three relations, got {}",
        r.len()
    );
    assert!(r.mean_per_head() >= 3.0);
}

/// A head must never be its own filler, and empty roles must be refused.
#[test]
fn test_degenerate_triples_are_refused() {
    let mut r = LexicalRule::new();
    r.add("cat", "of", "cat");
    r.add("cat", "", "animal");
    assert!(r.is_empty());
}
