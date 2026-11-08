//! Phi-4 taught a new word by pointing (§28–30, docs/116). Our students
//! learn `sepia` from one or three pointings at a group, with or without the
//! grammar word `colour`, and are then given orders that use it (paper 19:
//! counts 26% / 26% / 100% / 100%, bag of words 0%). Here Phi-4 gets the same
//! teaching in its prompt — the group pointed at, told as a scene, and what
//! A said — and the same orders.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c): one pointing gives a word its first relation.

use super::follow;
use super::learners::{Compose, GameCounts, Learner};
use super::split::Split;
use super::teacher;
use super::world::Episode;
use phiecs::quest::builders;
use std::collections::BTreeMap;

const CONDITIONS: [(usize, bool); 4] = [(1, false), (3, false), (1, true), (3, true)];

fn file(dir: &str, k: usize, grammar: bool, between: usize) -> String {
    format!("{dir}/phi4_sepia_k{k}{}{}.tsv", if grammar { "_colour" } else { "" }, if between > 0 { format!("_b{between}") } else { String::new() })
}

/// The needle (paper 24 §7): `n` other orders, answered as the game
/// answers them, between the pointing and the order that uses the word —
/// so the teaching sits `n` exchanges back in Phi-4's transcript.
fn exchanges(sp: &Split, n: usize, seed: u64) -> Vec<String> {
    let mut d = super::world::Dice(seed ^ 0xBE7);
    let orders: Vec<&Episode> = sp.train.iter().filter(|e| e.game == "order").collect();
    (0..n)
        .map(|_| {
            let e = orders[d.below(orders.len())];
            let (words, _) = teacher::said(e);
            format!("A said \"{words}\" and you answered \"{}\".", e.action.join(" | "))
        })
        .collect()
}

/// One pointing as the prompt tells it: the scene (every aspect at once, the
/// colour only as one there is no word for) and the words said.
fn shown(o: &Episode) -> String {
    let words: Vec<&str> = o.tokens.iter().filter(|t| !t.starts_with('⟨')).map(String::as_str).collect();
    let [_, what, _, count, place] = &o.action;
    format!("A pointed at a group: {count} {what}s at {place}, of a colour you have no word for, and said \"{}\".", words.join(" "))
}

/// `sepia-ask <dir> <n> [--between m] <oracle…>`: the four teachings, `n`
/// orders each; with `m` other exchanges between the pointing and the order.
pub fn ask(sp: &Split, dir: &str, n: usize, between: usize, seed: u64, oracle: &[String], parallel: usize) {
    for (k, grammar) in CONDITIONS {
        if between > 0 && !grammar {
            continue;
        }
        let train = sp.with_ostension(k, grammar);
        let mut shows: Vec<String> = train[train.len() - k..].iter().map(shown).collect();
        shows.extend(exchanges(sp, between, seed));
        println!("— taught by {k} pointing(s){}, {between} exchanges between: {}", if grammar { " with `colour`" } else { "" }, shows[..k].join(" / "));
        let prompt = |e: &Episode| {
            let (words, gestures) = teacher::said(e);
            builders::prompt_shown(&e.game, &words, &gestures, true, &shows)
        };
        teacher::ask(&sp.sepia, n, seed, &file(dir, k, grammar, between), oracle, parallel, &prompt);
    }
}

fn pct(n: usize, d: usize) -> String {
    format!("{:.1}%", 100.0 * n as f64 / d.max(1) as f64)
}

/// `sepia-report <dir> <out.md>`: how Phi-4 used the taught word.
pub fn report(sp: &Split, dir: &str) -> String {
    let mut s = String::from("## Phi-4 taught `sepia` by pointing, then given orders that use it\n\n| teaching | replies read | exact (game's terms) | colour = sepia | act | what | count | place | commonest colours answered | our counts student, same orders: exact / colour | student + Phi-4's reading (5 folds): exact |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
    for ((k, grammar), between) in CONDITIONS.iter().flat_map(|c| [0, 25, 100, 400].map(|b| (*c, b))) {
        let path = file(dir, k, grammar, between);
        if !std::path::Path::new(&path).exists() {
            continue;
        }
        let asked = follow::load(&path);
        let read = follow::read_of(&asked);
        let m = read.len();
        let ok = |a: usize| read.iter().filter(|x| x.2[a] == x.0.action[a]).count();
        let mut colours: BTreeMap<&str, usize> = BTreeMap::new();
        read.iter().for_each(|x| *colours.entry(x.2[2].as_str()).or_default() += 1);
        let mut c: Vec<_> = colours.into_iter().collect();
        c.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(y.0)));
        let top: Vec<String> = c.iter().take(3).map(|(v, n)| format!("{v} {n}")).collect();
        let exact = read.iter().filter(|x| x.2 == x.0.action).count();
        let mut student = GameCounts { compose: Compose::Movement, ..GameCounts::new(&sp.with_ostension(k, grammar)) };
        let ys: Vec<_> = asked.iter().map(|x| (student.act(&x.rule), &x.rule.action)).collect();
        // Distilled (stage B): in each fold, if Phi-4 answered the taught word
        // as the colour in most orders of the other folds, the student is told
        // what §29's grammar word tells it — this is a colour-word — and keeps
        // its own act, stone, count and place.
        let fold = |i: usize| i % 5;
        let mut sd = 0usize;
        for f in 0..5 {
            let others: Vec<&(&Episode, [String; 5], [String; 5])> = read.iter().filter(|x| asked.iter().position(|a| std::ptr::eq(&a.rule, x.0)).map(fold) != Some(f)).collect();
            let as_colour = others.iter().filter(|x| x.2[2] == "sepia").count() * 2 > others.len();
            let mut d = GameCounts { compose: Compose::Movement, ..GameCounts::new(&sp.with_ostension(k, grammar || as_colour)) };
            sd += asked.iter().enumerate().filter(|(i, _)| fold(*i) == f).filter(|(_, x)| d.act(&x.rule) == x.rule.action).count();
        }
        let (se, sc) = (ys.iter().filter(|(y, r)| y == *r).count(), ys.iter().filter(|(y, r)| y[2] == r[2]).count());
        let name = format!("{k} pointing{}{}{}", if k > 1 { "s" } else { "" }, if grammar { " + `colour`" } else { "" }, if between > 0 { format!(", {between} exchanges between") } else { String::new() });
        s += &format!("| {name} | {m} of {} | {} | {} | {} | {} | {} | {} | {} | {} / {} | {} |\n", asked.len(), pct(exact, m), pct(ok(2), m), pct(ok(0), m), pct(ok(1), m), pct(ok(3), m), pct(ok(4), m), top.join(", "), pct(se, ys.len()), pct(sc, ys.len()), pct(sd, asked.len()));
    }
    s
}
