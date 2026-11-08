//! The counts moved to phient's `relate::Tally`; the game's numbers must not
//! move with them. Recount the training episodes the way the game's own
//! map did and compare every `(token, kind) ⇒ value` exactly.

use super::learners::{kind, told, GameCounts};
use super::split::Split;
use std::collections::HashMap;

#[test]
fn phient_holds_exactly_what_the_game_told() {
    for seed in [7, 11] {
        let train = Split::new(seed).train;
        let mut n: HashMap<(String, String), HashMap<String, f64>> = HashMap::new();
        for e in &train {
            for (t, a, v) in told(e) {
                for k in [kind(a, Some(&e.game)), kind(a, None)] {
                    *n.entry((t.clone(), k)).or_default().entry(v.clone()).or_default() += 1.0;
                }
            }
        }
        let c = GameCounts::new(&train);
        let pairs = c.pairs();
        assert_eq!(pairs.len(), n.len(), "the same (token, kind) pairs");
        for (t, k) in &pairs {
            assert_eq!(c.table(t, k).as_ref(), n.get(&(t.clone(), k.clone())), "{t} under {k}");
        }
        for a in 0..super::world::ASPECTS.len() {
            let mut v: Vec<String> = n.iter().filter(|((_, k), _)| k == super::world::ASPECTS[a]).flat_map(|(_, m)| m.keys().cloned()).collect();
            v.sort();
            v.dedup();
            assert_eq!(c.values(a), v, "the values of aspect {a}");
        }
    }
}
