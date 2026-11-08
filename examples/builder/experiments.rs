//! Two more trials against the other methods: how many episodes each
//! learner needs (§5–6: language is learned by training, not by being
//! told), and the stranger of §32, who learns a new vocabulary for a game
//! he already plays.

use super::learners::{Compose, GameCounts, Learner};
use super::lookup::Lookup;
use super::split::Split;
use super::world::{Dice, Episode};
use super::{bow, naming, phient};

pub fn all(train: &[Episode], rs: &[usize], epochs: usize, seed: u64) -> Vec<Box<dyn Learner>> {
    let counts = GameCounts::new(train);
    let mut v: Vec<Box<dyn Learner>> = vec![
        Box::new(Lookup::new(train)),
        Box::new(naming::Naming::new(train)),
        Box::new(bow::Bow::new(train, &counts, epochs, seed)),
        Box::new(GameCounts { compose: Compose::Movement, ..GameCounts::new(train) }),
        Box::new(super::weighted::Weighted::new(train, epochs, seed)),
        Box::new(super::wordposts::WordPosts::new(train, epochs, seed, super::wordposts::SEEN)),
    ];
    for &r in rs {
        let p = std::rc::Rc::new(phient::Phient::new(train, r, epochs, seed, Compose::Movement));
        for n in [0.0, 8.0, 32.0, -1.0] {
            v.push(Box::new(phient::TwoSpeed { p: p.clone(), consolidate: n }));
        }
    }
    v
}

fn exact(l: &mut dyn Learner, test: &[Episode]) -> f64 {
    test.iter().filter(|e| l.act(e) == e.action).count() as f64 / test.len().max(1) as f64
}

fn table(title: &str, cols: &[String], rows: Vec<Vec<String>>) -> String {
    let mut s = format!("{title}\n\n| learner | {} |\n|---|{}\n", cols.join(" | "), "---|".repeat(cols.len()));
    for r in rows {
        s += &format!("| {} |\n", r.join(" | "));
    }
    s
}

/// Exact match on every test set together, trained on the first `n`
/// episodes (orders and reports in their training share).
pub fn curve(sp: &Split, sizes: &[usize], rs: &[usize], epochs: usize, seed: u64) -> String {
    let tests: Vec<Episode> = [&sp.seen, &sp.combos, &sp.report, &sp.no_name].into_iter().flatten().cloned().collect();
    let mut d = Dice(seed ^ 0xC0FFEE);
    let mut order: Vec<usize> = (0..sp.train.len()).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, d.below(i + 1));
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    for &n in sizes {
        let train: Vec<Episode> = order.iter().take(n).map(|&i| sp.train[i].clone()).collect();
        for (i, l) in all(&train, rs, epochs, seed).iter_mut().enumerate() {
            if rows.len() <= i {
                rows.push(vec![l.name()]);
            }
            rows[i].push(format!("{:.1}%", 100.0 * exact(l.as_mut(), &tests)));
        }
        println!("curve n={n} done");
    }
    let cols: Vec<String> = sizes.iter().map(|n| format!("{n} episodes")).collect();
    table("## Training (§5–6): exact match on all four test sets, by episodes seen", &cols, rows)
}

/// §32: the same game in a new vocabulary (every word replaced; pointing
/// and the game itself unchanged). The stranger already plays the game in
/// the old words; the child meets only the new ones.
fn foreign(e: &Episode) -> Episode {
    let tokens = e.tokens.iter().map(|t| if t.starts_with('⟨') { t.clone() } else { format!("ξ{t}") }).collect();
    Episode { tokens, ..e.clone() }
}

pub fn stranger(sp: &Split, ks: &[usize], rs: &[usize], epochs: usize, seed: u64) -> String {
    let tests: Vec<Episode> = sp.combos.iter().chain(&sp.seen).map(foreign).collect();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut cols = Vec::new();
    for &k in ks {
        let new: Vec<Episode> = sp.train.iter().filter(|e| e.game == "order").take(k).map(foreign).collect();
        let mut both = sp.train.clone();
        both.extend(new.iter().cloned());
        for (who, train) in [("child", &new), ("stranger", &both)] {
            for (i, l) in all(train, rs, epochs, seed).iter_mut().enumerate() {
                if rows.len() <= i {
                    rows.push(vec![l.name()]);
                }
                rows[i].push(format!("{:.1}%", 100.0 * exact(l.as_mut(), &tests)));
            }
            cols.push(format!("{who}, {k}"));
        }
        println!("stranger k={k} done");
    }
    table("## The stranger (§32): new words for a game already played — exact match on new-vocabulary orders", &cols, rows)
}

/// §54: a teacher who makes mistakes. With probability `p` one aspect of
/// what followed is replaced by another value that aspect takes. *Silent*:
/// nothing marks the mistake. *Corrected*: the mistake is seen corrected
/// (A says no), so the episode is played in a game of its own,
/// `corrected` — a condition, not a deletion.
pub fn mistakes(sp: &Split, rates: &[f64], rs: &[usize], epochs: usize, seed: u64) -> String {
    let tests: Vec<Episode> = [&sp.seen, &sp.combos, &sp.report, &sp.no_name].into_iter().flatten().cloned().collect();
    let counts = GameCounts::new(&sp.train);
    let values: Vec<Vec<String>> = (0..super::world::ASPECTS.len()).map(|a| counts.values(a)).collect();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut cols = Vec::new();
    for &p in rates {
        for corrected in [false, true] {
            if p == 0.0 && corrected {
                continue;
            }
            let mut d = Dice(seed ^ 0xBAD ^ (p * 1000.0) as u64);
            let train: Vec<Episode> = sp
                .train
                .iter()
                .map(|e| {
                    let mut e = e.clone();
                    if d.chance(p) {
                        let a = d.below(values.len());
                        let others: Vec<&String> = values[a].iter().filter(|v| **v != e.action[a]).collect();
                        if !others.is_empty() {
                            e.action[a] = others[d.below(others.len())].clone();
                            if corrected {
                                e.game = "corrected".into();
                            }
                        }
                    }
                    e
                })
                .collect();
            for (i, l) in all(&train, rs, epochs, seed).iter_mut().enumerate() {
                if rows.len() <= i {
                    rows.push(vec![l.name()]);
                }
                rows[i].push(format!("{:.1}%", 100.0 * exact(l.as_mut(), &tests)));
            }
            cols.push(format!("{:.0}% {}", 100.0 * p, if corrected { "corrected" } else { "silent" }));
            println!("mistakes p={p} corrected={corrected} done");
        }
    }
    table("## Mistakes (§54): the teacher errs in one aspect of a share of episodes — exact match on all four rule tests", &cols, rows)
}

/// §§69–71, §88: how exact must a post be? The post threshold τ swept for
/// Phical counts (movement): the four rule tests, sepia met once with the
/// grammar word, and the rule tests after 160 episodes.
pub fn posts(sp: &Split, taus: &[f64], seed: u64) -> String {
    let tests: Vec<Episode> = [&sp.seen, &sp.combos, &sp.report, &sp.no_name].into_iter().flatten().cloned().collect();
    let mut d = Dice(seed ^ 0xC0FFEE);
    let mut order: Vec<usize> = (0..sp.train.len()).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, d.below(i + 1));
    }
    let small: Vec<Episode> = order.iter().take(160).map(|&i| sp.train[i].clone()).collect();
    let sepia = sp.with_ostension(1, true);
    let mut s = String::from("## How exact must a post be? (§§69–71, §88) — Phical counts, movement\n\n| τ (nats) | four rule tests | sepia, 1 pointing + `colour` | 160 episodes |\n|---|---|---|---|\n");
    for &tau in taus {
        let at = |train: &[Episode], test: &[Episode]| {
            let mut c = GameCounts { compose: Compose::Movement, ..GameCounts::new(train) };
            c.post = tau;
            100.0 * exact(&mut c, test)
        };
        s += &format!("| {tau} | {:.1}% | {:.1}% | {:.1}% |\n", at(&sp.train, &tests), at(&sepia, &sp.sepia), at(&small, &tests));
    }
    s
}
