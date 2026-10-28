//! What each round prints after the arena's table.

use super::hedge::Hedge;
use super::mix::Report;
use super::parse;
use super::rounds::experts;
use super::scene::Scene;
use phient::study::Reader;

/// Round 9: BCPL's priorities, learned on the first four fifths, scored on
/// the last.
pub fn priorities(stream: &[Vec<String>], path: &str) {
    let cut = stream.len() * 4 / 5;
    let ops = parse::operators(&stream[..cut], 30);
    let flat: parse::Prio = ops.iter().map(|o| (o.clone(), 3)).collect();
    let (prio, kept) = parse::learn(stream, cut, &ops, 2);
    let held = |p: Option<&parse::Prio>| parse::surprise(stream, cut, stream.len(), p);
    println!("# BCPL's priorities learned on English: {path}\n");
    println!("Learned on the first {cut} lines ({} changes kept of {} tried); scored on the last {} lines.\n", kept.len(), ops.len() * 4 * 2, stream.len() - cut);
    println!("| reading | bits per word, held out |\n|---|---|");
    println!("| the word before | {:.3} |", held(None));
    println!("| the open operator, every priority equal (A4) | {:.3} |", held(Some(&flat)));
    println!("| the open operator, priorities learned by trial and error | {:.3} |", held(Some(&prio)));
    println!("\n## The priorities it learned (5 closes the most)\n");
    for level in (1..=5u8).rev() {
        let mut at: Vec<&String> = prio.iter().filter(|(_, p)| **p == level).map(|(w, _)| w).collect();
        at.sort();
        println!("- {level}: {}", at.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(", "));
    }
}

/// Rounds 8 and 10: each mix's surprise and its extra part's weights.
pub fn surprise(reports: &[Report]) {
    println!("\n## The unexpected, measured: bits of surprise per word at what came (lower is learned)\n");
    println!("| mix | bits per word | the extra part's weight, by how often the two words were seen (never · once · 2-4 · 5-19 · 20+) |\n|---|---|---|");
    for (name, r) in ["A6", "A8 scene", "A9 classes", "A10 open operator"].iter().zip(reports) {
        let t = r.borrow();
        let mut rows: Vec<_> = t.weights.iter().collect();
        rows.sort_by_key(|(b, _)| **b);
        let extra: Vec<String> = rows.iter().map(|(_, w)| w.get(3).map_or("-".into(), |x| format!("{x:.2}"))).collect();
        println!("| {name} | {:.3} | {} |", t.bits / t.words.max(1) as f64, extra.join(" · "));
    }
}

/// Round 11: each mix's surprise and every extra part's weight per bucket.
pub fn parts(mixes: &[(&str, &[&str])], reports: &[Report]) {
    println!("
## The unexpected, measured, and the weight each extra part earned (never · once · 2-4 · 5-19 · 20+)
");
    for ((name, extra), r) in mixes.iter().zip(reports) {
        let t = r.borrow();
        println!("- {name}: {:.3} bits per word", t.bits / t.words.max(1) as f64);
        let mut rows: Vec<_> = t.weights.iter().collect();
        rows.sort_by_key(|(b, _)| **b);
        for (k, part) in extra.iter().enumerate() {
            let w: Vec<String> = rows.iter().map(|(_, w)| w.get(3 + k).map_or("-".into(), |x| format!("{x:.2}"))).collect();
            println!("  - {part}: {}", w.join(" · "));
        }
    }
}

/// Rounds 3 to 7: the weights the error gave each part of one mix.
pub fn weights(round: u32, report: &Report) {
    println!("\n## The weights the error gave each part, by how often the two words were seen\n");
    let mut rows: Vec<_> = report.borrow().weights.clone().into_iter().collect();
    rows.sort_by_key(|(b, _)| *b);
    let last = match round {
        3 => "phasor learner",
        5 => "the scene",
        7 => "the open operator",
        _ => "classes",
    };
    for (b, w) in rows {
        let seen = ["never", "once", "2-4 times", "5-19 times", "20+ times"][b as usize];
        let parts = ["two words", "the word before", "the commonest", last];
        let row: Vec<String> = parts.iter().zip(&w).map(|(p, x)| format!("{p} {x:.2}")).collect();
        println!("- two words seen {seen}: {}", row.join(" · "));
    }
}

/// Round 5: how many lines reached no scene.
pub fn unexpected(stream: &[Vec<String>], seam: u32) {
    let mut s = Scene::new(seam);
    stream.iter().for_each(|w| s.tell(w));
    println!("\nThe unexpected: {} of {} lines reached no scene through their edges (seam: a pair held by {seam} lines or more).", s.unexpected.0, s.unexpected.1);
}

/// Rounds 1 and 2: what the situated trial and error came to trust, where —
/// the same stream read once more by that reader alone.
pub fn situated(stream: &[Vec<String>]) {
    let mut h = Hedge::new("situated", experts(), 0.9, true);
    for words in stream {
        for j in 1..words.len() {
            h.guess(&words[..j]);
        }
        h.tell(words);
    }
    println!("\n## What the situated reader trusted, where\n");
    for (s, w) in h.trust() {
        let row: Vec<String> = w.iter().map(|(e, x)| format!("{e} {:.2}", x)).collect();
        println!("- {s}: {}", row.join(" · "));
    }
}
