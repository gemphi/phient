#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

#[test]
fn test_same_length_words_diverge() {
    let mut f = Facet::new();
    let t = Trainer::new(0.05);
    t.train_sentence(&mut f, "cat dog war");
    let c = f.lexicon.get("cat").unwrap().phase;
    let d = f.lexicon.get("dog").unwrap().phase;
    assert!(phistd::wrap(c - d).abs() > 0.01, "identity seeding must separate same-length words");
}
