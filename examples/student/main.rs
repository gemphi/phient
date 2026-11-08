//! `student` — one student, every method, one honest protocol (docs/172).
//!
//! Replaces `learn_grammar`, `learn_person`, `learn_bcpl`: the step-by-step
//! method is shared ([`phient::study::Study`] — predict first, learn after,
//! a curve over the stream) and the student takes three lessons with it:
//!
//! 1. **grammar** — Murphy's units as a concept tree: [`paragraph`] rows
//!    under each concept path, [`Knowing::tell`] one example at a time,
//!    the next word predicted UNDER its concept before each tell (Q2);
//!    then the practice questions against the book's Key.
//! 2. **person** — a Being told a person's lines one by one, the next
//!    word-choice predicted before each line (Q3, on the fly, no retrain).
//! 3. **code** — BCPL functions read online, the next role predicted from
//!    the construct nesting before each is learned (Q1, by depth).
//!
//! ```text
//! cargo run --release -p phient --example student -- [bcpl-dir]
//! ```

use phiecs::being::Being;
use phiecs::persona::Persona;
use phiecs::agent::Agent;
use phiecs::Row;
use phient::prose::{paragraph, tokens, Knowing};
use phient::relate::scopes;
use phient::study::Study;

mod book;
mod grown;
use book::{HELDOUT, PERSON, SHOWN, UNITS};

/// Lesson 1: the concepts. Predict each next word under its concept BEFORE
/// telling the example; the cast does the resolving, nothing bespoke.
fn grammar() -> (Knowing, Study) {
    let vocab: std::collections::HashSet<&str> =
        UNITS.iter().flat_map(|(_, exs)| exs.iter().flat_map(|e| e.split(' '))).collect();
    let mut study = Study::new("grammar under concepts", 5, 100.0 / vocab.len() as f64);
    let mut k = Knowing::fresh();
    let total: usize = UNITS.iter().map(|(_, e)| e.len()).sum();
    let mut i = 0;
    for (concept, exs) in UNITS {
        for ex in *exs {
            let words = tokens(ex);
            k.within = scopes(concept, 0);
            for j in 1..words.len() {
                let (_, ranked, _) = k.next(&words[..j]);
                study.tick(i, total, ranked.first().is_some_and(|(w, _)| *w == words[j]));
            }
            let path = format!("{concept}/e{}", i + 1);
            k.tell(&path, &paragraph(&path, &[ex.to_string()]));
            i += 1;
        }
    }
    (k, study)
}

/// The practice question, against the book's Key: under `concept`, after
/// `said`, what comes next?
fn practice(k: &mut Knowing, concept: &str, said: &[&str], key: &str) {
    k.within = scopes(concept, 0);
    let said: Vec<String> = said.iter().map(|s| s.to_string()).collect();
    let (_, ranked, _) = k.next(&said);
    let got = ranked.first().map(|(w, _)| w.as_str()).unwrap_or("—");
    println!("  under {:<20} \"{}\" → {got}   (key: {key}) {}",
        concept.trim_start_matches("grammar/"), said.join(" "),
        if got == key { "✓" } else { "✗" });
}

/// Lesson 2: the person. Predict their next word-choice BEFORE each line.
fn person() -> Study {
    let vocab: std::collections::HashSet<&str> = PERSON.iter().flat_map(|l| l.split(' ')).collect();
    let mut study = Study::new("a person, line by line", 5, 100.0 / vocab.len() as f64);
    let mut b = Being::new(Persona::from_md("---\nname: Friend\nseeds: think\nkinds: next\n---\n").unwrap());
    for (i, line) in PERSON.iter().enumerate() {
        let w = tokens(line);
        for p in w.windows(2) {
            study.tick(i, PERSON.len(), b.ask(&p[0], "next").first().is_some_and(|(t, _)| *t == p[1]));
        }
        b.tell(&w.windows(2).map(|p| Row::new(&p[0], "next", &p[1])).collect::<Vec<_>>());
    }
    study
}

/// Lesson 0: discovery as a DAG (docs/174). The examples WITHOUT their
/// concept labels: the student listens in the order its own interest picks,
/// then the scopes are its construction FAMILIES (`Knowing::families`) —
/// and a sentence belongs to EVERY family it shows, because its tense
/// construction and its collocations are different axes of the same words.
/// Conditions, fixed in advance: every Murphy unit the majority of at least
/// one family (coverage 4/4); member-weighted purity ≥ 85%; holding or
/// improving from round 1 (book) to round 2 (teacher's uses). The student
/// also names the constructions it is UNSURE of — the quest it would send.
fn discovery(shown: bool) {
    let mut pool: Vec<(usize, Vec<String>)> = UNITS.iter().enumerate()
        .flat_map(|(u, (_, exs))| exs.iter().map(move |e| (u, tokens(e)))).collect();
    if shown {
        pool.extend(SHOWN.iter().map(|(u, e)| (*u, tokens(e))));
    }
    let mut k = Knowing::fresh();
    let mut heard: Vec<usize> = Vec::new();
    while !pool.is_empty() {
        let words: Vec<Vec<String>> = pool.iter().map(|(_, w)| w.clone()).collect();
        let (unit, w) = pool.remove(k.interest(&words).unwrap_or(0));
        k.tell(&format!("heard/e{}", heard.len()), &paragraph("heard", &[w.join(" ")]));
        heard.push(unit);
    }
    let fams = k.families(2);
    let majority = |members: &[usize]| -> (usize, usize) {
        (0..UNITS.len())
            .map(|u| (u, members.iter().filter(|&&i| heard[i] == u).count()))
            .max_by_key(|&(_, n)| n)
            .unwrap_or((0, 0))
    };
    let (mut covered, mut weighted, mut mass) = (vec![false; UNITS.len()], 0usize, 0usize);
    for (_, members) in &fams {
        let (u, n) = majority(members);
        covered[u] = true;
        weighted += n;
        mass += members.len();
    }
    let round = if shown { "round 2, after the teacher showed more uses" } else { "round 1, the book alone" };
    println!("## discovery as a DAG ({round}) — no labels given:");
    println!("{} families over {} sentences; coverage {}/{} units; member-weighted purity {:.0}%",
        fams.len(), heard.len(), covered.iter().filter(|&&c| c).count(), UNITS.len(),
        100.0 * weighted as f64 / mass.max(1) as f64);
    let top: Vec<String> = fams.iter().take(6)
        .map(|((a, b), m)| format!("({a} {b})×{}", m.len())).collect();
    println!("largest: {}", top.join(", "));
    let unsure = k.unsure(2);
    println!("unsure of {} constructions — the quest would ask the teacher for uses of e.g. {}\n",
        unsure.len(),
        unsure.iter().take(3).map(|(a, b)| format!("\"{a} {b}\"")).collect::<Vec<_>>().join(", "));
}

fn main() {
    println!("# The student — every method, one protocol: predict first, learn after\n");

    discovery(false);
    discovery(true);

    let (mut k, g) = grammar();
    println!("{}", g.report());
    println!("Practice (the same cue resolved by CONCEPT — Q2, the Key from Murphy):");
    practice(&mut k, "grammar/present-continuous", &["it", "is", "not"], "raining");
    practice(&mut k, "grammar/present-simple", &["it", "does", "not"], "grow");
    practice(&mut k, "grammar/past-simple", &["it", "did", "not"], "rain");
    practice(&mut k, "grammar/present-perfect", &["it", "has", "not"], "rained");
    println!();

    println!("{}", person().report());

    // The DAG put back into the scope tree, read on held-out text.
    let mut listened = Knowing::fresh();
    for (i, line) in UNITS.iter().flat_map(|(_, e)| e.iter()).chain(SHOWN.iter().map(|(_, e)| e)).enumerate() {
        let path = format!("heard/e{i}");
        listened.tell(&path, &paragraph(&path, &[line.to_string()]));
    }
    grown::report(&listened, HELDOUT, 2);

    if let Some(dir) = std::env::args().skip(1).find(|a| a.starts_with("--corpus=")) {
        let dir = dir.trim_start_matches("--corpus=").to_string();
        let n = |k: &str, d: usize| std::env::args().skip(1).find_map(|a| a.strip_prefix(k).and_then(|v| v.parse().ok())).unwrap_or(d);
        grown::at_scale(&dir, n("--train=", 2000), n("--held=", 200), 3);
    }

    let root = std::env::args().nth(1).unwrap_or_else(|| "../ext/bcpl/cintcode/com".into());
    let bodies = phient::code::bcpl::read_bodies(std::path::Path::new(&root));
    if bodies.is_empty() {
        println!("## code — skipped: no BCPL at {root}");
    } else {
        let r = phient::code::bcpl::learn::continuous(&bodies, 16, 10);
        println!("{}", r.study.report());
        println!("({} functions; nesting max {} mean {:.1} — buildings, not leaves)", r.functions, r.max_depth, r.mean_depth);
    }
}