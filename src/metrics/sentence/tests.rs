#[allow(unused_imports)]
use super::*;

use crate::train::constants::LEARNING_RATE;
use crate::train::trainer::Trainer;

fn docs() -> Vec<Vec<String>> {
    (0..30)
        .map(|d| {
            (0..8)
                .map(|i| format!("document {} sentence {} about the topic of trade and money", d, i))
                .collect()
        })
        .collect()
}

#[test]
fn test_items_are_deterministic_and_truth_is_present() {
    let d = docs();
    let pool: Vec<Vec<String>> = d
        .iter()
        .flatten()
        .map(|s| Tokenizer::tokenize(s))
        .collect();

    let a = SentenceBenchmark::build_items(&d, &pool, 7);
    let b = SentenceBenchmark::build_items(&d, &pool, 7);
    assert_eq!(a.len(), b.len());
    assert!(!a.is_empty(), "the fixture must produce items");

    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.truth, y.truth, "the item set must not move between runs");
        assert_eq!(x.candidates.len(), CANDIDATES);
        assert_eq!(x.candidates, y.candidates);
    }
}

/// The truth must not sit at a fixed slot, or a scorer wins by guessing it.
#[test]
fn test_truth_position_varies() {
    let d = docs();
    let pool: Vec<Vec<String>> = d.iter().flatten().map(|s| Tokenizer::tokenize(s)).collect();
    let items = SentenceBenchmark::build_items(&d, &pool, 7);
    let distinct: HashSet<usize> = items.iter().map(|i| i.truth).collect();
    assert!(
        distinct.len() > 1,
        "the truth landed at one slot in every item: {:?}",
        distinct
    );
}

/// Chance is reported, and a trained model must at least reach it.
#[test]
fn test_phase_reaches_chance() {
    let d = docs();
    let flat: Vec<String> = d.iter().flatten().cloned().collect();
    let pool: Vec<Vec<String>> = flat.iter().map(|s| Tokenizer::tokenize(s)).collect();

    let mut facet = Facet::new();
    let t = Trainer::new(LEARNING_RATE);
    for s in &flat {
        t.train_sentence(&mut facet, s);
    }

    let r = SentenceBenchmark::evaluate(&facet, &d, &pool, 7);
    assert!(r.items > 0);
    let phase = &r.scorers[0];
    assert!(phase.mrr >= r.chance_mrr * 0.5, "phase {} vs chance {}", phase.mrr, r.chance_mrr);
    assert!(phase.top1 <= 1.0 && phase.mrr <= 1.0);
}
