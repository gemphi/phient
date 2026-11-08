//! Training and test episodes. Held out of training: numerals g and h with
//! beams, blue pillars, slabs brought to p5 (new combinations, §8); pillars
//! and beams in the report game, and `this` there (§21); the word `sepia`
//! everywhere but the pointing episodes (§28–30).

use super::world::{ostension, sentence, Dice, Episode, COLOURS};

pub struct Split {
    pub train: Vec<Episode>,
    pub seen: Vec<Episode>,
    pub combos: Vec<Episode>,
    pub report: Vec<Episode>,
    pub no_name: Vec<Episode>,
    pub sepia: Vec<Episode>,
    seed: u64,
}

fn new_combination(e: &Episode) -> bool {
    let has = |w: &str| e.tokens.iter().any(|t| t == w);
    let [_, what, colour, _, place] = &e.action;
    (what == "beam" && (has("g") || has("h"))) || (what == "pillar" && colour == "blue") || (what == "slab" && place == "p5" && e.tokens.len() > 3)
}

fn new_report(e: &Episode) -> bool {
    e.game == "report" && (e.action[1] == "pillar" || e.action[1] == "beam" || e.tokens.iter().any(|t| t == "this"))
}

/// `n` episodes from `make` that `keep` accepts.
fn draw(d: &mut Dice, n: usize, mut make: impl FnMut(&mut Dice) -> Episode, keep: impl Fn(&Episode) -> bool) -> Vec<Episode> {
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let e = make(d);
        if keep(&e) {
            out.push(e);
        }
    }
    out
}

impl Split {
    pub fn new(seed: u64) -> Self {
        let mut d = Dice(seed.wrapping_mul(0x2545_F491_4F6C_DD1D));
        let plain = |e: &Episode| !new_combination(e) && !new_report(e);
        let mut train = draw(&mut d, 3000, |d| sentence(d, "order", &COLOURS), plain);
        train.extend(draw(&mut d, 600, |d| sentence(d, "report", &COLOURS), plain));
        let seen = draw(&mut d, 500, |d| sentence(d, "order", &COLOURS), |e| plain(e) && e.action[0] == "bring");
        let combos = draw(&mut d, 300, |d| sentence(d, "order", &COLOURS), new_combination);
        let report = draw(&mut d, 300, |d| sentence(d, "report", &COLOURS), new_report);
        let no_name = draw(&mut d, 150, |d| sentence(d, "order", &COLOURS), |e| e.action[0] != "bring");
        let sepia = draw(&mut d, 300, |d| sentence(d, "order", &["sepia"]), |e| e.action[2] == "sepia");
        Split { train, seen, combos, report, no_name, sepia, seed }
    }

    /// The training episodes plus `k` pointings at sepia things, with the
    /// grammar word or without.
    pub fn with_ostension(&self, k: usize, grammar: bool) -> Vec<Episode> {
        let mut d = Dice(self.seed ^ 0x5E91A ^ k as u64);
        let mut t = self.train.clone();
        t.extend((0..k).map(|_| ostension(&mut d, "sepia", "sepia", grammar)));
        t
    }
}
