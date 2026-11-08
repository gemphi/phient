//! `template` — TRN for prose (docs/200): a sentence compressed to its TYPE
//! sequence. No part of speech is given. A word's type is where the phasors
//! hold it under `next>` — the range channel, where it sits as the thing
//! reached — quantised into `K` sectors of the circle. A sentence is then
//! a string over K symbols, and the question is BCPL's question about a
//! program: how many distinct shapes does the corpus need, and do the
//! commonest 1024 cover it?
//!
//! ```text
//! PHI_READ=sure cargo run --release -p phient --example template -- <corpus> --sectors 16
//! ```

use phient::lineup::Ecs;
use phient::prose::tokens;
use phient::study::Reader;
use std::collections::HashMap;

const KEEP: usize = 1024;

fn sentences(path: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .split(|c| matches!(c, '.' | '!' | '?' | '\n'))
        .map(tokens)
        .map(|w| w.into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>())
        .filter(|w| w.len() >= 4)
        .collect()
}

/// The sector a phase falls in, 0..k.
fn sector(theta: f64, k: usize) -> usize {
    let t = theta.rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU;
    ((t * k as f64) as usize).min(k - 1)
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = a.first().cloned() else {
        eprintln!("which corpus?");
        return;
    };
    let flag = |k: &str, d: usize| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let k = flag("--sectors", 16);
    let show = flag("--show", 6);
    // --pair: a type is BOTH angles — the sector a word is reached at (next>)
    // and the sector it reaches from (next). `the` and `is` are reached from
    // everywhere alike, but reach different things.
    let pair = a.iter().any(|x| x == "--pair");

    let lines = sentences(&path);
    let mut person = Ecs::new();
    for w in &lines {
        person.tell(w);
    }

    // Every word's type: its sector under the range channel of `next`.
    let mut kind_of: HashMap<&str, usize> = HashMap::new();
    for w in lines.iter().flatten() {
        if !kind_of.contains_key(w.as_str()) {
            if let Some(t) = person.angle(w, "next>") {
                let r = sector(t, k);
                let d = if pair { person.angle(w, "next").map_or(k, |t| sector(t, k)) } else { 0 };
                kind_of.insert(w, d * (k + 1) + r);
            }
        }
    }
    let typed = kind_of.len();

    // Each sentence as its template: the sequence of types ('?' when unknown).
    let mut count: HashMap<String, usize> = HashMap::new();
    for w in &lines {
        let t: Vec<String> = w.iter().map(|x| kind_of.get(x.as_str()).map_or("?".into(), |s| s.to_string())).collect();
        *count.entry(t.join(" ")).or_default() += 1;
    }
    let mut ranked: Vec<(&String, &usize)> = count.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let covered: usize = ranked.iter().take(KEEP).map(|(_, n)| **n).sum();
    let share = 100.0 * covered as f64 / lines.len() as f64;

    println!("# the template — {path} — {k} sectors");
    println!("{} sentences, {typed} words typed by their angle under next>; {} distinct templates", lines.len(), count.len());
    println!("the commonest {KEEP} templates cover {share:.1}% of the sentences ({covered}/{})", lines.len());
    println!("\n## the commonest templates\n");
    for (t, n) in ranked.iter().take(show) {
        println!("  ×{n:<4} {t}");
    }
    println!("\n## the sectors, by their members (most-told first)\n");
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for w in lines.iter().flatten() {
        *freq.entry(w).or_default() += 1;
    }
    let types: usize = if pair { (k + 1) * (k + 1) } else { k };
    for s in 0..types {
        let mut members: Vec<(&str, usize)> = kind_of.iter().filter(|(_, t)| **t == s).map(|(w, _)| (*w, freq[w])).collect();
        members.sort_by(|a, b| b.1.cmp(&a.1));
        if members.is_empty() {
            continue;
        }
        let names: Vec<&str> = members.iter().take(12).map(|(w, _)| *w).collect();
        let label = if pair { format!("reaches {:>2} · reached {:>2}", s / (k + 1), s % (k + 1)) } else { format!("sector {s:>2}") };
        println!("  {label} ({:>4} words): {}", members.len(), names.join(" "));
    }
}
