//! How learnable is BCPL, against English (docs/166)? The same reader on
//! both: every line is a sentence, `phient::prose::tokens` cuts it, one
//! tenth is held out by hash, and two numbers are read on the held-out
//! part — the interpolated Kneser-Ney perplexity (`phient::baseline`) and
//! the top-1 next-token hit rate of the plain bigram count (what the
//! ladder's rung 2 is). Lower perplexity and a higher hit rate mean a
//! language whose next word follows more from its last.
//!
//! ```text
//! cargo run --release -p phient --example bcpl -- <dir of .b files> <dir of .txt files> [--max-lines N]
//! ```

use phient::study::lab::flag;
use phient::baseline::KneserNey;
use phient::prose::tokens;
use std::collections::HashMap;
use std::path::Path;

fn lines_under(dir: &Path, ext: &str, max: usize) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        let mut ps: Vec<_> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        ps.sort();
        for p in ps {
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == ext) {
                if let Ok(t) = std::fs::read_to_string(&p) {
                    for l in t.lines() {
                        let toks = tokens(l);
                        if toks.len() >= 2 {
                            out.push(toks);
                        }
                        if out.len() >= max {
                            return out;
                        }
                    }
                }
            }
        }
    }
    out
}

fn measure(name: &str, sents: Vec<Vec<String>>) {
    let held = |i: usize| phistd::mix64(phistd::fnv1a(&format!("{name}{i}"))) % 10 == 0;
    let (train, test): (Vec<_>, Vec<_>) = sents.iter().enumerate().partition(|(i, _)| !held(*i));
    let train: Vec<Vec<String>> = train.into_iter().map(|(_, s)| s.clone()).collect();
    let test: Vec<Vec<String>> = test.into_iter().map(|(_, s)| s.clone()).collect();
    let toks: usize = train.iter().map(Vec::len).sum();
    let mut vocab: HashMap<&str, usize> = HashMap::new();
    let mut bigram: HashMap<(&str, &str), usize> = HashMap::new();
    for s in &train {
        for w in s {
            *vocab.entry(w).or_insert(0) += 1;
        }
        for w in s.windows(2) {
            *bigram.entry((&w[0], &w[1])).or_insert(0) += 1;
        }
    }
    // The commonest successor of each word, from the training part alone.
    let mut best: HashMap<&str, (&str, usize)> = HashMap::new();
    for ((a, b), c) in &bigram {
        let e = best.entry(a).or_insert((b, 0));
        if *c > e.1 {
            *e = (b, *c);
        }
    }
    let (mut n, mut hit, mut told) = (0usize, 0usize, 0usize);
    for s in &test {
        for w in s.windows(2) {
            n += 1;
            hit += (best.get(w[0].as_str()).map(|x| x.0) == Some(w[1].as_str())) as usize;
            told += bigram.contains_key(&(w[0].as_str(), w[1].as_str())) as usize;
        }
    }
    let kn = KneserNey::train(&train);
    println!(
        "| {name} | {} | {} | {toks} | {} | {:.1}% | {:.1}% | {:.1} |",
        train.len(),
        test.len(),
        vocab.len(),
        100.0 * told as f64 / n.max(1) as f64,
        100.0 * hit as f64 / n.max(1) as f64,
        kn.perplexity(&test)
    );
}

fn main() {
    let mut a: Vec<String> = std::env::args().skip(1).collect();
    let max: usize = flag(&mut a, "--max-lines").and_then(|s| s.parse().ok()).unwrap_or(60_000);
    let (Some(code), Some(text)) = (a.first(), a.get(1)) else {
        eprintln!("usage: bcpl <dir of .b> <dir of .txt> [--max-lines N]");
        std::process::exit(2);
    };
    println!("| corpus | train lines | held-out lines | train tokens | vocabulary | held-out pairs already told | bigram top-1 | KN perplexity |\n|---|---|---|---|---|---|---|---|");
    measure("BCPL", lines_under(Path::new(code), "b", max));
    measure("English (Emma)", lines_under(Path::new(text), "txt", max));
}
