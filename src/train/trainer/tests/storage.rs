//! The facet's files under phiano's trainer: the tests of
//! `phiecs::phasor::facet::storage` that train with `Trainer` stay here.

use crate::train::trainer::Trainer;
use phiecs::phasor::facet::storage::Storage;
use phiecs::phasor::facet::Facet;

#[test]
fn test_roundtrip_preserves_ngrams_and_lexicon() {
    let mut facet = Facet::new();
    let t = Trainer::new(0.05);
    for s in ["the cat sat on the mat", "the dog ran in the park"] {
        t.train_sentence(&mut facet, s);
    }

    let path = std::env::temp_dir().join("phiano_storage_roundtrip.chroma");
    let path = path.to_str().unwrap();
    Storage::save(&facet, path).unwrap();
    let back = Storage::load(path).unwrap();

    assert_eq!(back.vocabulary_size(), facet.vocabulary_size());
    assert_eq!(back.ngram_entries(), facet.ngram_entries());
    assert!((back.bigram_probability("the", "cat") - facet.bigram_probability("the", "cat")).abs() < 1e-12);
    for (w, p) in &facet.lexicon {
        let q = back.lexicon.get(w).expect("word survived the roundtrip");
        assert!((p.phase - q.phase).abs() < 1e-12);
        assert!(p.resonance(q) > 0.999);
    }
}

/// The v4 body carries evidence and the relation inventory; both must
/// survive a save/load.
#[test]
fn test_roundtrip_preserves_evidence_and_relations() {
    let mut facet = Facet::new();
    let t = Trainer::new(0.05);
    let corpus: Vec<String> = [
        "the cat sat on the mat of the house",
        "the dog ran in the park of the town",
        "a bird of prey sat in the tree",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    t.train_corpus(&mut facet, &corpus);
    assert!(facet.evidenced_words() > 0, "training deposits evidence");
    assert!(!facet.relations.is_empty(), "corpus training learns markers");

    let path = std::env::temp_dir().join("phiano_storage_evidence.chroma");
    let path = path.to_str().unwrap();
    Storage::save(&facet, path).unwrap();
    let back = Storage::load(path).unwrap();

    assert_eq!(back.relations.entries(), facet.relations.entries());
    for (w, p) in &facet.lexicon {
        let q = back.lexicon.get(w).expect("word survived");
        for k in 0..phiton::units::PHASE_CHANNELS {
            assert_eq!(p.evidence_count(k), q.evidence_count(k), "{w} channel {k}");
        }
        assert!((p.similarity(q) - 1.0).abs() < 1e-9);
    }
}
