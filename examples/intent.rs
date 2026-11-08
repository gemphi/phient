//! `intent` — the hierarchy does not end at the sentence (docs/201): word
//! → sentence → paragraph → INTENTION → the compiler. The English guide
//! hands over ground truth for free — a unit's prose EXPLAINS, an exercise
//! line INSTRUCTS, the Key ANSWERS — so an intention discovered from the
//! types alone can be checked against what the text was for. Each
//! intention is then a kind on BCPL's ladder (tell = `s_ass`, do =
//! `s_rtap`, answer = `s_resultis`), and every instruction becomes a
//! program the judge runs: the ladder ends in the compiler.
//!
//! ```text
//! PHI_READ=sure cargo run --release -p phient --example intent -- murphy.txt --key murphy_key.txt
//! ```

#[path = "workshop/key.rs"]
mod key;

use phient::lineup::Ecs;
use phiclm::manifest::S;
use phient::prose::tokens;
use phient::study::Reader;
use std::collections::HashMap;

const K: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Intent {
    Explain,
    Instruct,
    Answer,
}

impl Intent {
    /// The rung on the ladder the intention is: telling, doing, answering.
    fn rung(self) -> S {
        match self {
            Intent::Explain => S::Ass,
            Intent::Instruct => S::Rtap,
            Intent::Answer => S::Resultis,
        }
    }
}

fn words(s: &str) -> Vec<String> {
    tokens(s).into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect()
}

fn is_instruction(l: &str) -> bool {
    let mut it = l.splitn(2, char::is_whitespace);
    let head = it.next().unwrap_or("");
    head.split_once('.').is_some_and(|(x, y)| x.parse::<usize>().is_ok() && y.parse::<usize>().is_ok()) && it.next().is_some_and(|r| r.len() > 12)
}

/// The labelled sentences, with the unit each came from (0 for the Key).
fn corpus(text: &str, key: &str) -> Vec<(usize, Intent, Vec<String>)> {
    let mut out = Vec::new();
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut unit = 0usize;
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        i += 1;
        if l.starts_with("Key to Exercises") && unit > 0 {
            break;
        }
        // The layout puts `Unit` on one line and `N  Title` on the next.
        if l == "Unit" {
            if let Some(n) = lines.get(i).and_then(|h| h.split_whitespace().next()).and_then(|n| n.parse().ok()) {
                unit = n;
                i += 1;
                continue;
            }
        }
        if unit == 0 || l.is_empty() {
            continue;
        }
        let (intent, body) = if is_instruction(l) { (Intent::Instruct, l.splitn(2, char::is_whitespace).nth(1).unwrap_or("")) } else { (Intent::Explain, l) };
        for s in body.split(|c| matches!(c, '.' | '!' | '?')) {
            let w = words(s);
            if w.len() >= 4 {
                out.push((unit, intent, w));
            }
        }
    }
    for a in key::parse(key) {
        if let Some(w) = a.alternative.first().filter(|w| w.len() >= 4) {
            out.push((a.unit, Intent::Answer, w.clone()));
        }
    }
    out
}

fn sector(theta: f64) -> usize {
    let t = theta.rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU;
    ((t * K as f64) as usize).min(K - 1)
}

/// An instruction as a program: its template counted into a vector and
/// summed — what the compiler is handed is the SHAPE, the names dropped.
fn program(template: &[usize]) -> String {
    let mut n = vec![0usize; K + 1];
    for &t in template {
        n[t] += 1;
    }
    let fills: Vec<String> = n.iter().enumerate().map(|(i, c)| format!("v!{i} := {c}")).collect();
    format!("LET start() = VALOF $( LET v = VEC {K} AND s = 0\n{}\nFOR i = 0 TO {K} DO s := s + v!i\nRESULTIS s $)", fills.join("\n"))
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let (Some(path), Some(keypath)) = (a.first().cloned(), flag("--key")) else {
        eprintln!("usage: intent <murphy.txt> --key <murphy_key.txt>");
        return;
    };
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let keytext = std::fs::read_to_string(&keypath).unwrap_or_default();
    let rows = corpus(&text, &keytext);

    let mut person = Ecs::new();
    for (_, _, w) in &rows {
        person.tell(w);
    }
    let mut kind_of: HashMap<&str, usize> = HashMap::new();
    for w in rows.iter().flat_map(|r| r.2.iter()) {
        if !kind_of.contains_key(w.as_str()) {
            if let Some(t) = person.angle(w, "next>") {
                kind_of.insert(w, sector(t));
            }
        }
    }
    let template = |w: &[String]| -> Vec<usize> { w.iter().map(|x| kind_of.get(x.as_str()).copied().unwrap_or(K)).collect() };

    // The intention signature: how a sentence OPENS and how it CLOSES, and
    // whether it is short — the shape of what it is for, not what it says.
    let sig = |t: &[usize]| (t[0], t[1], t[2], t.len() < 8);
    let mut by_sig: HashMap<(usize, usize, usize, bool), HashMap<Intent, usize>> = HashMap::new();
    let mut by_label: HashMap<Intent, usize> = HashMap::new();
    for (_, label, w) in &rows {
        *by_sig.entry(sig(&template(w))).or_default().entry(*label).or_default() += 1;
        *by_label.entry(*label).or_default() += 1;
    }
    let total = rows.len();
    let majority = by_label.values().max().copied().unwrap_or(0);
    let pure: usize = by_sig.values().map(|m| m.values().max().copied().unwrap_or(0)).sum();

    println!("# the intention — {path}\n");
    println!("{total} sentences: {} explain, {} instruct, {} answer; {} signatures (three opening types, short)", by_label.get(&Intent::Explain).copied().unwrap_or(0), by_label.get(&Intent::Instruct).copied().unwrap_or(0), by_label.get(&Intent::Answer).copied().unwrap_or(0), by_sig.len());
    println!("purity of the discovered intentions: {:.1}% (baseline, the majority label: {:.1}%)", 100.0 * pure as f64 / total as f64, 100.0 * majority as f64 / total as f64);

    // The paragraph and the chapter: a unit as its sequence of sentence shapes.
    let mut unit_shape: HashMap<usize, Vec<String>> = HashMap::new();
    for (u, _, w) in &rows {
        unit_shape.entry(*u).or_default().push(template(w).iter().map(ToString::to_string).collect::<Vec<_>>().join(""));
    }
    let mut shapes: HashMap<String, usize> = HashMap::new();
    for v in unit_shape.values() {
        *shapes.entry(v.join("|")).or_default() += 1;
    }
    println!("{} units as sequences of sentence shapes: {} distinct unit shapes", unit_shape.len(), shapes.len());

    // The compiler end: every instruction is a program, judged.
    let (mut compiled, mut steps, mut asked) = (0usize, 0usize, 0usize);
    for (_, label, w) in rows.iter().filter(|r| r.1 == Intent::Instruct) {
        let v = phiclm::judge(&program(&template(w)), 50_000);
        asked += 1;
        if v.compiled && v.result == w.len() as i64 {
            compiled += 1;
            steps += v.steps;
        }
        let _ = label;
    }
    println!("\nthe ladder ends in the compiler: explain = {}, instruct = {}, answer = {}", Intent::Explain.rung().name(), Intent::Instruct.rung().name(), Intent::Answer.rung().name());
    println!("{compiled}/{asked} instructions compiled to a program whose result equals the sentence's length ({} steps each, mean)", steps / compiled.max(1));
    let mut top: Vec<_> = by_sig.iter().collect();
    top.sort_by_key(|(_, m)| std::cmp::Reverse(m.values().sum::<usize>()));
    println!("\nthe commonest signatures and what they were for:");
    for (s, m) in top.iter().take(8) {
        let mut l: Vec<_> = m.iter().collect();
        l.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
        println!("  {:?}  {}", s, l.iter().map(|(i, n)| format!("{i:?} {n}")).collect::<Vec<_>>().join(", "));
    }
}
