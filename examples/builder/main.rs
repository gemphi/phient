//! Language in action: the builders' game of Philosophical Investigations
//! §§1–32, played, and five ways of learning it measured side by side
//! (docs/102, papers/paper_6_descriptive_logic.md).
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c). Here that is taken literally: a learner sees only who
//! said what, in which game, and what followed. It is then asked to act.
//!
//! Tests: **seen** (new episodes of the kinds trained), **new combinations**
//! (§8: numeral × stone × place pairs never heard together), **report**
//! (§21: stones never reported, in the report game), **no name** (§27:
//! `help`, `away`, `no`), and **sepia** (§28–30: a colour-word met once or
//! three times by pointing, with or without the grammar word `colour`,
//! then used in orders).
//!
//! `cargo run --release -p phient --example builder -- [out.md] [--r 32] [--epochs 30] [--seed 7]`
//! `… builder -- ask episodes.tsv 400 --parallel 3 ollama run phi4` (Phi-4 plays B)
//! `… builder -- teacher episodes.tsv teacher.md` (students learn from Phi-4)

mod bow;
mod compose;
mod experiments;
mod follow;
mod compare;
mod graded;
mod learners;
mod lookup;
mod naming;
mod phient;
mod quest;
mod split;
mod student;
mod teacher;
mod weighted;
mod wordposts;
mod world;
#[cfg(test)]
mod tests;

use learners::{Compose, GameCounts, Learner};
use lookup::Lookup;
use split::Split;
use world::Episode;

use ::phient::study::lab::flag;

/// Share of episodes whose five aspects are all right.
fn exact(l: &mut dyn Learner, test: &[Episode]) -> f64 {
    test.iter().filter(|e| l.act(e) == e.action).count() as f64 / test.len().max(1) as f64
}

fn learners(train: &[Episode], r: usize, epochs: usize, seed: u64) -> (Vec<Box<dyn Learner>>, String) {
    let t0 = std::time::Instant::now();
    let p = std::rc::Rc::new(phient::Phient::new(train, r, epochs, seed, Compose::Movement));
    let note = format!("{} relations kept from {} episodes · {} numbers · functions agree with counts on {:.1}% · {:.0} s", p.relations, train.len(), p.size(), 100.0 * p.agreed_share(), t0.elapsed().as_secs_f64());
    let p_sum = phient::Phient::new(train, r, epochs, seed, Compose::Sum);
    let counts = GameCounts::new(train);
    let bow = bow::Bow::new(train, &counts, epochs, seed);
    let moved = GameCounts { compose: Compose::Movement, ..GameCounts::new(train) };
    (vec![Box::new(Lookup::new(train)), Box::new(naming::Naming::new(train)), Box::new(bow), Box::new(counts), Box::new(moved), Box::new(weighted::Weighted::new(train, epochs, seed)), Box::new(wordposts::WordPosts::new(train, epochs, seed, wordposts::SEEN)), Box::new(p_sum), Box::new(phient::TwoSpeed { p: p.clone(), consolidate: 0.0 }), Box::new(phient::TwoSpeed { p: p.clone(), consolidate: 8.0 }), Box::new(phient::TwoSpeed { p, consolidate: -1.0 })], note)
}

fn main() {
    let mut a: Vec<String> = std::env::args().skip(1).collect();
    let r: usize = flag(&mut a, "--r").and_then(|s| s.parse().ok()).unwrap_or(32);
    let epochs: usize = flag(&mut a, "--epochs").and_then(|s| s.parse().ok()).unwrap_or(30);
    let seed: u64 = flag(&mut a, "--seed").and_then(|s| s.parse().ok()).unwrap_or(7);
    let parallel: usize = flag(&mut a, "--parallel").and_then(|s| s.parse().ok()).unwrap_or(1);
    // `--prompt roles`: the scene also says who fetches and who takes note.
    let roles = flag(&mut a, "--prompt").as_deref() == Some("roles");
    // `ask <episodes.tsv> <n> <oracle…>`: Phi-4 plays the game (docs/103).
    if a.first().map(String::as_str) == Some("ask") && a.len() > 3 {
        let n = a[2].parse().unwrap_or(400);
        let prompt = |e: &Episode| {
            let (words, gestures) = teacher::said(e);
            phiecs::quest::builders::prompt_with(&e.game, &words, &gestures, roles)
        };
        teacher::ask(&Split::new(seed).train, n, seed, &a[1], &a[3..], parallel, &prompt);
        return;
    }
    // `sepia-ask <dir> <n> <oracle…>` / `sepia-report <dir> <out.md>`: Phi-4 taught by pointing (docs/116).
    match (a.first().map(String::as_str), a.len()) {
        (Some("sepia-ask"), 4..) => {
            let mut rest: Vec<String> = a[3..].to_vec();
            let between: usize = rest.iter().position(|x| x == "--between").and_then(|i| { let v = rest.get(i + 1)?.parse().ok(); rest.drain(i..i + 2); v }).unwrap_or(0);
            return quest::ask(&Split::new(seed), &a[1], a[2].parse().unwrap_or(50), between, seed, &rest, parallel);
        }
        (Some("sepia-report"), 3..) => return std::fs::write(&a[2], quest::report(&Split::new(seed), &a[1])).expect("write report"),
        _ => {}
    }
    // `teacher <episodes.tsv> [out.md]`: students learn from Phi-4's actions.
    if a.first().map(String::as_str) == Some("teacher") && a.len() > 1 {
        let md = student::run(&a[1], &[r], epochs, seed);
        let out = a.get(2).cloned().unwrap_or_else(|| "teacher.md".into());
        std::fs::write(&out, &md).expect("write report");
        println!("{md}\nwrote {out}");
        return;
    }
    // `follow <episodes.tsv> <out.md>`: Phi-4 as the other player (§§185–242, docs/107).
    if a.first().map(String::as_str) == Some("follow") && a.len() > 1 {
        let md = follow::report(&follow::load(&a[1]), &[r], epochs, seed);
        let out = a.get(2).cloned().unwrap_or_else(|| "follow.md".into());
        std::fs::write(&out, &md).expect("write report");
        println!("{md}\nwrote {out}");
        return;
    }
    // `graded <episodes.tsv> <out.md>`: τ swept on a teacher of tendencies (docs/111).
    if a.first().map(String::as_str) == Some("graded") && a.len() > 2 {
        let md = graded::sweep(&follow::read_of(&follow::load(&a[1])), &[0.0, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0], epochs, seed);
        std::fs::write(&a[2], &md).expect("write report");
        return println!("{md}\nwrote {}", a[2]);
    }
    // `compare <out.md> <episodes.tsv>…`: Phi-4 runs side by side (docs/108).
    if a.first().map(String::as_str) == Some("compare") && a.len() > 2 {
        let md = compare::compare(&a[2..], epochs, seed);
        std::fs::write(&a[1], &md).expect("write report");
        println!("{md}\nwrote {}", a[1]);
        return;
    }
    // `posts <out.md>`: the post threshold swept (§88, docs/105).
    if a.first().map(String::as_str) == Some("posts") {
        let md = experiments::posts(&Split::new(seed), &[0.0, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 4.0], seed);
        let out = a.get(1).cloned().unwrap_or_else(|| "posts.md".into());
        std::fs::write(&out, &md).expect("write report");
        println!("{md}\nwrote {out}");
        return;
    }
    // `mistakes <out.md>`: learning from a teacher who errs (§54, docs/104).
    if a.first().map(String::as_str) == Some("mistakes") {
        let md = experiments::mistakes(&Split::new(seed), &[0.0, 0.1, 0.2, 0.3, 0.4], &[r], epochs, seed);
        let out = a.get(1).cloned().unwrap_or_else(|| "mistakes.md".into());
        std::fs::write(&out, &md).expect("write report");
        println!("{md}\nwrote {out}");
        return;
    }
    let out = a.first().cloned().unwrap_or_else(|| "builders.md".into());
    let sp = Split::new(seed);
    let mut md = format!(
        "# Language in action — the builders' game (seed {seed}, R = {r}, {epochs} epochs)\n\n{} training episodes ({} orders, {} reports). Exact match: all five aspects of the action right.\n\n",
        sp.train.len(),
        sp.train.iter().filter(|e| e.game == "order").count(),
        sp.train.iter().filter(|e| e.game == "report").count()
    );
    let (mut ls, note) = learners(&sp.train, r, epochs, seed);
    md += &format!("Phient: {note}.\n\n| learner | seen | new combinations (§8) | report, new stones (§21) | no name (§27) |\n|---|---|---|---|---|\n");
    for l in ls.iter_mut() {
        let row = [&sp.seen, &sp.combos, &sp.report, &sp.no_name].map(|t| format!("{:.1}%", 100.0 * exact(l.as_mut(), t)));
        md += &format!("| {} | {} |\n", l.name(), row.join(" | "));
        println!("{} {}", l.name(), row.join(" "));
        if std::env::var("PHIANO_DEBUG").is_ok() {
            for set in [&sp.seen, &sp.combos, &sp.report] {
                let per: Vec<String> = (0..5).map(|a| format!("{:.0}%", 100.0 * set.iter().filter(|e| l.act(e)[a] == e.action[a]).count() as f64 / set.len() as f64)).collect();
                let bad: Vec<String> = set.iter().filter_map(|e| { let y = l.act(e); (y != e.action).then(|| format!("{} ⇒ {:?}", e.tokens.join(" "), y)) }).take(2).collect();
                println!("   {} · {}", per.join(" "), bad.join(" ; "));
            }
        }
    }
    if std::env::var("PHIANO_QUICK").is_ok() {
        std::fs::write(&out, &md).expect("write report");
        println!("wrote {out}");
        return;
    }
    md += "\n## Sepia (§28–30): a new colour-word, met only by pointing, then used in orders\n\n| learner | 1 pointing | 3 pointings | 1 pointing + `colour` | 3 pointings + `colour` |\n|---|---|---|---|---|\n";
    let mut cells: Vec<Vec<String>> = Vec::new();
    for (k, grammar) in [(1, false), (3, false), (1, true), (3, true)] {
        let train = sp.with_ostension(k, grammar);
        let (mut ls, _) = learners(&train, r, epochs, seed);
        for (i, l) in ls.iter_mut().enumerate() {
            if cells.len() <= i {
                cells.push(vec![l.name()]);
            }
            cells[i].push(format!("{:.1}%", 100.0 * exact(l.as_mut(), &sp.sepia)));
            if std::env::var("PHIANO_DEBUG").is_ok() {
                let per: Vec<String> = (0..5).map(|a| format!("{:.0}%", 100.0 * sp.sepia.iter().filter(|e| l.act(e)[a] == e.action[a]).count() as f64 / sp.sepia.len() as f64)).collect();
                let bad: Vec<String> = sp.sepia.iter().filter_map(|e| { let y = l.act(e); (y != e.action).then(|| format!("{} ⇒ {:?}", e.tokens.join(" "), y)) }).take(2).collect();
                println!("   sepia {k} {grammar} {} · {} · {}", l.name(), per.join(" "), bad.join(" ; "));
            }
        }
        println!("sepia k={k} grammar={grammar} done");
    }
    for c in cells {
        md += &format!("| {} |\n", c.join(" | "));
    }
    md += "\n## The toolbox (§11–17): where each word holds a post (order game, Phical counts)\n\n";
    md += &compose::posts(&GameCounts::new(&sp.with_ostension(3, true)));
    md += "\n";
    md += &experiments::curve(&sp, &[20, 40, 80, 160, 320, 640], &[8, 32], epochs, seed);
    md += "\n";
    md += &experiments::stranger(&sp, &[10, 20, 40, 80], &[8, 32], epochs, seed);
    std::fs::write(&out, &md).expect("write report");
    println!("\n{md}\nwrote {out}");
}
