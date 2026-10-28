//! The helix round trip under the trainer: `phiecs::phasor::facet::helix`
//! writes and reads back a lexicon the trainer has coupled.

use crate::train::constants::LEARNING_RATE;
use crate::train::trainer::Trainer;
use phiecs::phasor::facet::helix::{read, write};
use phiecs::phasor::facet::Facet;

#[test]
fn a_helix_round_trips_the_lexicon() {
    let docs: Vec<Vec<String>> = ["the cat sat on the mat", "a dog eats a donut", "cat of mat"]
        .iter()
        .map(|s| s.split(' ').map(String::from).collect())
        .collect();
    let mut facet = Facet::new();
    facet.learn_relations(&docs);
    for d in &docs {
        for w in d {
            facet.get_or_init(w);
        }
    }
    let t = Trainer::new(LEARNING_RATE);
    for _ in 0..3 {
        for d in &docs {
            t.couple_relations(&mut facet, d);
        }
    }
    let dir = std::env::temp_dir().join(format!("phiano-helix-{}", std::process::id()));
    let n = write(&facet, &dir).unwrap();
    assert_eq!(n, facet.lexicon.len());
    let back = read(&dir).unwrap();
    assert_eq!(back.lexicon.len(), facet.lexicon.len());
    for (w, p) in &facet.lexicon {
        let q = &back.lexicon[w];
        assert_eq!(p.packed(), q.packed(), "{w} source side");
        assert_eq!(p.packed_target(), q.packed_target(), "{w} target side");
        assert_eq!(p.packed_evidence(), q.packed_evidence(), "{w} evidence");
        assert_eq!(p.count, q.count);
    }
    assert_eq!(back.relations.entries(), facet.relations.entries());
    assert!(dir.join("codons").join("AAA").join("words.tsv").exists());
    let _ = std::fs::remove_dir_all(&dir);
}
