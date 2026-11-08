//! Descriptive Logic II, §§46–64 in action: the squares game of §48 with
//! the composite names of §64, and four learners (docs/104, paper 20).
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c). Here the condition of a word is where the words before it
//! left the reader. Training sentences use at most one composite name, and
//! never at a held-out place; the tests ask for new places and for several
//! composites in one sentence.
//!
//! `cargo run --release -p phient --example square -- [out.md] [--seed 7]`

mod baselines;
mod reader;
mod world;

use baselines::{Learner, Lookup, Naming, Positional};
use world::{episode, Dice, Episode};

/// Held out of training: composites beginning in the middle column.
fn held_place(e: &Episode) -> bool {
    e.composites_at.iter().any(|c| c % 3 == 1)
}

fn draw(d: &mut Dice, n: usize, use_pair: f64, max: usize, keep: impl Fn(&Episode) -> bool) -> Vec<Episode> {
    let mut v = Vec::new();
    while v.len() < n {
        let e = episode(d, use_pair, max);
        if keep(&e) {
            v.push(e);
        }
    }
    v
}

impl Learner for reader::Reader {
    fn name(&self) -> String {
        "Phient reader (relations under the reader's place)".into()
    }
    fn read(&self, words: &[String]) -> Vec<String> {
        reader::Reader::read(self, words)
    }
}

fn exact(l: &dyn Learner, test: &[Episode]) -> f64 {
    test.iter().filter(|e| l.read(&e.words) == e.grid).count() as f64 / test.len().max(1) as f64
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let seed: u64 = a.iter().position(|x| x == "--seed").and_then(|i| a.get(i + 1)?.parse().ok()).unwrap_or(7);
    let out = a.first().filter(|x| !x.starts_with("--")).cloned().unwrap_or_else(|| "squares.md".into());
    let mut d = Dice(seed.wrapping_mul(0x2545_F491_4F6C_DD1D));
    let one = |e: &Episode| e.composites_at.len() <= 1 && !held_place(e);
    let tests: Vec<(&str, Vec<Episode>)> = vec![
        ("plain names only", draw(&mut d, 300, 0.0, 0, |_| true)),
        ("one composite, trained place", draw(&mut d, 300, 1.0, 1, |e| e.composites_at.len() == 1 && !held_place(e))),
        ("one composite, new place (§64)", draw(&mut d, 300, 1.0, 1, |e| e.composites_at.len() == 1 && held_place(e))),
        ("two or more composites", draw(&mut d, 300, 1.0, 9, |e| e.composites_at.len() >= 2)),
    ];
    let mut md = format!("# The squares game (§48, §64), seed {seed}\n\nExact match: all nine squares right.\n\n| learner | training sentences | {} |\n|---|---|{}\n", tests.iter().map(|t| t.0).collect::<Vec<_>>().join(" | "), "---|".repeat(tests.len()));
    let mut posts = String::new();
    for n in [50, 200, 1000] {
        let train = draw(&mut d, n, 0.5, 1, one);
        let r = reader::Reader::learn(&train, 12);
        let ls: Vec<Box<dyn Learner>> = vec![Box::new(Lookup::new(&train)), Box::new(Naming::new(&train)), Box::new(Positional::new(&train, 20))];
        for l in ls.iter().map(|b| b.as_ref()).chain([&r as &dyn Learner]) {
            let row: Vec<String> = tests.iter().map(|(_, t)| format!("{:.1}%", 100.0 * exact(l, t))).collect();
            md += &format!("| {} | {n} | {} |\n", l.name(), row.join(" | "));
            println!("{n} {} {}", l.name(), row.join(" "));
        }
        posts = r.posts();
    }
    md += &format!("\n## What each word came to relate to (Phient reader, 1,000 sentences)\n\n{posts}");
    std::fs::write(&out, &md).expect("write report");
    println!("\n{md}\nwrote {out}");
}
