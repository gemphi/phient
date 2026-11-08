//! A graded teacher (docs/111). Phi-4 at temperature 0 answers the same
//! call the same way, but its act is a tendency, not a rule: `slab` and
//! `pillar` lean it to `record`, colour-words never do, and no single word
//! decides. A post (D2) asks a word to move an aspect by τ nats; a tendency
//! of 47% against 26% moves it by about 0.1 nats, below τ = 0.5, so the
//! counts student never hears it while bag-of-words, which weighs every word,
//! does. Here τ is swept on Phi-4's own actions, with the same five folds
//! by call as `follow::student`.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c): how strongly it must relate before it is heard is itself
//! a condition, and a teacher who teaches by tendency sets it lower (§§88, 107).

use super::bow::Bow;
use super::learners::{Compose, GameCounts, Learner};
use super::weighted::Weighted;
use super::wordposts::WordPosts;
use super::world::Episode;

const SEENS: &[f64] = &[4.0, 8.0, 32.0, f64::INFINITY];

fn pct(n: usize, d: usize) -> String {
    format!("{:.1}%", 100.0 * n as f64 / d.max(1) as f64)
}

/// Per aspect, the τ with most right answers in `inner` folds of `train`
/// (ties to the larger τ, the side of D2 as written).
fn choose(train: &[Episode], taus: &[f64], seed: u64) -> Vec<f64> {
    let fold = |e: &Episode| (phistd::mix64(phistd::fnv1a(&e.key()) ^ seed ^ 0x1AA) % 4) as usize;
    let mut right = vec![vec![0usize; taus.len()]; 5];
    for f in 0..4 {
        let tr: Vec<Episode> = train.iter().filter(|e| fold(e) != f).cloned().collect();
        for (i, &tau) in taus.iter().enumerate() {
            let mut l = GameCounts { compose: Compose::Movement, post: tau, ..GameCounts::new(&tr) };
            for t in train.iter().filter(|e| fold(e) == f) {
                let y = l.act(t);
                (0..5).for_each(|a| right[a][i] += (y[a] == t.action[a]) as usize);
            }
        }
    }
    (0..5).map(|a| taus[(0..taus.len()).rev().max_by_key(|&i| right[a][i]).unwrap()]).collect()
}

pub fn sweep(read: &[(&Episode, [String; 5], [String; 5])], taus: &[f64], epochs: usize, seed: u64) -> String {
    let taught: Vec<Episode> = read.iter().map(|x| Episode { action: x.2.clone(), ..x.0.clone() }).collect();
    let fold = |e: &Episode| (phistd::mix64(phistd::fnv1a(&e.key()) ^ seed) % 5) as usize;
    // rows: bag of words, counts at each τ, counts with τ per aspect chosen
    // inside the training folds; (exact, per aspect)
    let mut rows = vec![(0usize, [0usize; 5], 0usize); taus.len() + 3 + SEENS.len()];
    let (mut n, mut n_known, mut chosen, mut table, mut moved) = (0usize, 0usize, Vec::new(), String::new(), String::new());
    for f in 0..5 {
        let train: Vec<Episode> = taught.iter().filter(|e| fold(e) != f).cloned().collect();
        let test: Vec<&Episode> = taught.iter().filter(|e| fold(e) == f).collect();
        n += test.len();
        // A call none of whose words was heard in training (`help`, `away`,
        // `no`: each is one call, so all its askings fall in one fold).
        let heard: std::collections::HashSet<&String> = train.iter().flat_map(|e| &e.tokens).collect();
        let known = |t: &Episode| t.tokens.iter().any(|w| !w.starts_with('⟨') && heard.contains(w));
        n_known += test.iter().filter(|t| known(t)).count();
        let counts = GameCounts::new(&train);
        let mut ls: Vec<Box<dyn Learner>> = vec![Box::new(Bow::new(&train, &counts, epochs, seed))];
        for &tau in taus {
            ls.push(Box::new(GameCounts { compose: Compose::Movement, post: tau, ..GameCounts::new(&train) }));
        }
        let by = choose(&train, taus, seed);
        chosen.push(format!("{by:?}"));
        ls.push(Box::new(GameCounts { compose: Compose::Movement, post_by: by, ..GameCounts::new(&train) }));
        let w = Weighted::new(&train, epochs, seed);
        table = w.table();
        ls.push(Box::new(w));
        for &n in SEENS {
            let wp = WordPosts::new(&train, epochs, seed, n);
            if n == 8.0 {
                moved = wp.moved(12);
            }
            ls.push(Box::new(wp));
        }
        for (i, l) in ls.iter_mut().enumerate() {
            for t in &test {
                let y = l.act(t);
                rows[i].0 += (y == t.action) as usize;
                rows[i].2 += (y == t.action && known(t)) as usize;
                (0..5).for_each(|a| rows[i].1[a] += (y[a] == t.action[a]) as usize);
            }
        }
    }
    let mut s = String::from("### How strongly must a word move an answer to be heard? (τ swept, students of Phi-4, 5 folds by call)\n\n| student | exact | exact, calls with a word heard in training | act | what | colour | count | place |\n|---|---|---|---|---|---|---|---|\n");
    let mut names = vec!["bag of words + softmax".to_string()];
    names.extend(taus.iter().map(|t| format!("Phical counts, movement, τ = {t}")));
    names.push("Phical counts, movement, τ per aspect (chosen in the training folds)".into());
    names.push("Phical counts, movement, weights learned per (aspect, post strength) (docs/112)".into());
    names.extend(SEENS.iter().map(|n| format!("Phical counts, movement, two-speed posts: own weight after {n} sightings (docs/113)")));
    for (name, (ex, asp, exk)) in names.iter().zip(&rows) {
        s += &format!("| {name} | {} | {} | {} |\n", pct(*ex, n), pct(*exk, n_known), asp.iter().map(|x| pct(*x, n)).collect::<Vec<_>>().join(" | "));
    }
    s += &format!("\n{n} calls, {n_known} with a word heard in training.\n");
    s += &format!("\nτ chosen per aspect (act, what, colour, count, place), fold by fold: {}\n", chosen.join(" · "));
    s += &format!("\nLearned weights (last fold):\n\n{table}");
    s += &format!("\nTwo-speed posts, the words that moved most from their D2 post (last fold, 8 sightings): {moved}\n");
    s
}
