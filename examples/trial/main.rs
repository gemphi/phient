//! `trial` — learning as trial and error, on one book (docs/204).
//!
//! Every assumption about what decides the next word is a reader in the
//! arena (`phient::study::Arena`): it guesses at every position before the
//! line is told, so each assumption is tested on the same positions and the
//! ones the numbers refute are refuted, stated straight.
//!
//! ```text
//! cargo run --release -p phient --example trial -- [book.txt] [--lines N] [--round 1..8] [--seam N]
//! cargo run --release -p phient --example trial -- [book.txt] --parse
//! cargo run --release -p phient --example trial -- --round 10 [--train dir] [--test book.txt]
//! ```

mod classes;
mod dict;
mod hedge;
mod mix;
mod parse;
mod parts;
mod readers;
mod report;
mod rounds;
mod scene;
mod table;

use phient::prose::tokens;
use phient::study::Arena;

/// The book as sentences, split as the arena example splits them.
fn sentences(path: &str, lines: usize) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    text.split(". ").map(tokens).filter(|w| w.len() >= 5).take(lines).collect()
}

/// Every `.txt` under `dir` but its READMEs, as sentences, in name order.
fn shelf(dir: &str) -> Vec<Vec<String>> {
    let mut files: Vec<_> = std::fs::read_dir(dir).map(|r| r.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    files.retain(|p: &std::path::PathBuf| p.extension().is_some_and(|x| x == "txt") && !p.to_string_lossy().contains("README"));
    files.sort();
    files.iter().flat_map(|p| sentences(&p.to_string_lossy(), usize::MAX)).collect()
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let path = a.first().filter(|x| !x.starts_with("--")).cloned().unwrap_or_else(|| "crates/phiano/data/corpora/austen/emma.txt".into());
    let lines = flag("--lines").and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
    let round: u32 = flag("--round").and_then(|s| s.parse().ok()).unwrap_or(2);
    let seam: u32 = flag("--seam").and_then(|s| s.parse().ok()).unwrap_or(8);
    let reports: Vec<mix::Report> = (0..4).map(|_| Default::default()).collect();

    if a.iter().any(|x| x == "--parse") {
        return report::priorities(&sentences(&path, lines), &path);
    }
    if (10..=13).contains(&round) {
        // Read one shelf unscored, then score a book by another hand.
        let train = shelf(&flag("--train").unwrap_or_else(|| "crates/phiano/data/corpora/austen".into()));
        let test_path = flag("--test").unwrap_or_else(|| "crates/phiano/data/corpora/contrast/carroll-alice.txt".into());
        let test = sentences(&test_path, lines);
        // Round 13: Webster's definitions of every word the reader will meet,
        // told before the shelf.
        let defs = match round {
            13 => {
                let wanted: std::collections::HashSet<String> = train.iter().chain(&test).flatten().cloned().collect();
                dict::definitions("crates/phiano/data/websters_dictionary.json", &wanted)
            }
            _ => Vec::new(),
        };
        println!("Told {} definitions before the shelf.
", defs.len());
        let train: Vec<Vec<String>> = defs.into_iter().chain(train).collect();
        let set = match round {
            13 => 10,
            _ => round,
        };
        let mut arena = Arena::new(rounds::readers(set, seam, &reports), 4, 0.0);
        arena.read_split(&train, train.len());
        for r in &reports {
            let mut t = r.borrow_mut();
            t.bits = 0.0;
            t.words = 0;
        }
        let n = arena.read(&test);
        println!("# Transfer: told {} lines of one shelf, then scored on {test_path} ({n} positions, never told before scored)\n", train.len());
        println!("{}", arena.table());
        return match round {
            12 => {
                println!("Both phasor learners learn online as they are told; the second also trains 3 epochs on the shelf before the book.");
            }
            11 => report::parts(&[("A8 scene", &["the scene"]), ("A11 scene and phasors", &["the scene", "the phasor learner"])], &reports),
            _ => report::surprise(&reports),
        };
    }

    let stream = sentences(&path, lines);
    let mut vocab: Vec<&String> = stream.iter().flatten().collect();
    vocab.sort();
    vocab.dedup();
    let floor = 100.0 / vocab.len().max(1) as f64;
    let t = std::time::Instant::now();
    let mut arena = Arena::new(rounds::readers(round, seam, &reports), 4, floor);
    let n = arena.read(&stream);
    println!("# Trial and error on one book: {path}\n");
    println!("{} sentences, {} words of vocabulary, {n} positions scored online (each guessed before its line is told), floor {floor:.3}%.\n", stream.len(), vocab.len());
    println!("{}", arena.table());
    println!("Read in {:.1} s.", t.elapsed().as_secs_f64());
    match round {
        8 => report::surprise(&reports),
        3 | 6 | 7 => report::weights(round, &reports[0]),
        5 => {
            report::weights(round, &reports[0]);
            report::unexpected(&stream, seam);
        }
        _ => report::situated(&stream),
    }
}
