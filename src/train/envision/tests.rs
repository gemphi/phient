#[allow(unused_imports)]
use super::*;

use crate::train::trainer::Trainer;

#[test]
fn test_stops_asking_after_the_limit() {
    let mut e = Envision::new();
    let mut f = Facet::new();
    let t = Trainer::new(0.05);
    t.train_sentence(&mut f, "the parser reads code");

    for _ in 0..MAX_ASKS {
        assert!(e.detect_gaps(&mut f, None, "the tokenizer reads code").is_some());
    }
    // Beyond the limit it should stop pestering about the same word.
    assert!(
        e.detect_gaps(&mut f, None, "the tokenizer reads code").is_none(),
        "an unanswered gap must not be asked about forever"
    );
}

#[test]
fn test_ledger_ranks_by_frequency() {
    let mut e = Envision::new();
    let mut f = Facet::new();
    let t = Trainer::new(0.05);
    t.train_sentence(&mut f, "known words only");

    let _ = e.detect_gaps(&mut f, None, "known words only zzz");
    let _ = e.detect_gaps(&mut f, None, "known words only zzz");
    let _ = e.detect_gaps(&mut f, None, "known words only qqq");

    let ranked = e.ledger.unresolved_ranked();
    assert_eq!(ranked[0].0, "zzz", "the most-encountered gap should lead the agenda");
    assert_eq!(e.ledger.unresolved_count(), 2);
}

#[test]
fn test_suggestions_use_context_not_only_spelling() {
    let mut f = Facet::new();
    let t = Trainer::new(0.05);
    for _ in 0..5 {
        t.train_sentence(&mut f, "the parser reads source code tokens");
    }
    let ctx = Wave::text(&f, "the parser reads source code");
    let s = Envision::suggest(&f, ctx, "tokenizer");
    assert!(!s.is_empty());
    assert!(
        s.iter().any(|(_, _, kind)| *kind == "context" || *kind == "both"),
        "context must contribute, not spelling alone"
    );
}

#[test]
fn test_known_input_produces_no_vision() {
    let mut e = Envision::new();
    let mut f = Facet::new();
    let t = Trainer::new(0.05);
    t.train_sentence(&mut f, "everything here is known");
    assert!(e.detect_gaps(&mut f, None, "everything here is known").is_none());
}
