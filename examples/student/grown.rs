//! Lesson: does the discovered DAG earn its keep? (docs/175 — the condition
//! docs/170 fixed for Q2.)
//!
//! Everything told is re-filed under the construction families the student
//! found (`Knowing::refile`), and held-out sentences — never told — are
//! read twice at the same budget: once FLAT (the last word and the last
//! two, no scopes) and once UNDER the concept the words said so far point
//! at (`scope_said` + the cast). Same positions, same counts, same floor.
//!
//! Two position sets, both fixed before reading: ALL positions, and the
//! DISCRIMINATING ones — those whose cue (the last word said) is shown by
//! two or more families, so the concept has something to choose between.
//! The subset is computed from what was told, never from the answer.

use phient::prose::{tokens, Knowing};
use phient::study::Study;

/// The cues a concept could act on: last words shown by ≥2 families.
fn ambiguous(k: &Knowing, at_least: usize) -> std::collections::HashSet<String> {
    let mut in_fams: std::collections::HashMap<String, usize> = Default::default();
    for (pair, _) in k.families(at_least) {
        *in_fams.entry(pair.0).or_default() += 1;
        *in_fams.entry(pair.1).or_default() += 1;
    }
    in_fams.into_iter().filter(|(_, n)| *n >= 2).map(|(w, _)| w).collect()
}

/// `(flat all, under all, flat on discriminating cues, under on same)`.
pub fn compare(k: &Knowing, held: &[&str], at_least: usize) -> [Study; 4] {
    let vocab: std::collections::HashSet<&String> = k.paragraphs.iter().flat_map(|(_, w)| w).collect();
    let floor = 100.0 / vocab.len().max(1) as f64;
    let cues = ambiguous(k, at_least);
    let mut s = [
        Study::new("flat, all positions", 4, floor),
        Study::new("under its concept, all positions", 4, floor),
        Study::new("flat, discriminating cues", 4, floor),
        Study::new("under its concept, discriminating cues", 4, floor),
    ];
    let mut deep = k.refile(at_least);
    let mut shallow = k.refile(at_least);
    shallow.flat = true;
    // The known words and family paths taken ONCE, so each position costs
    // a lookup rather than a scan of everything told.
    let known = deep.known_words(at_least);
    let paths: std::collections::HashSet<String> =
        deep.families(at_least).into_iter().map(|(pr, _)| phient::prose::family_path(&pr)).collect();

    for (i, line) in held.iter().enumerate() {
        let w = tokens(line);
        for j in 1..w.len() {
            let said = &w[..j];
            shallow.within = Vec::new();
            let f = shallow.next(said).1.first().is_some_and(|(x, _)| *x == w[j]);
            deep.within = match deep.scope_said_with(said, &known, &paths) {
                Some(p) => phient::relate::scopes(&p, 0),
                None => Vec::new(),
            };
            let u = deep.next(said).1.first().is_some_and(|(x, _)| *x == w[j]);
            s[0].tick(i, held.len(), f);
            s[1].tick(i, held.len(), u);
            if cues.contains(&w[j - 1]) {
                s[2].tick(i, held.len(), f);
                s[3].tick(i, held.len(), u);
            }
        }
    }
    s
}

/// Prints both readings on both position sets, and the verdict.
pub fn report(k: &Knowing, held: &[&str], at_least: usize) {
    let s = compare(k, held, at_least);
    println!("## does the discovered DAG earn its keep? ({} held-out sentences, never told)\n", held.len());
    println!("| positions | flat | under its concept | lift |\n|---|---|---|---|");
    let row = |name: &str, a: &Study, b: &Study| {
        println!("| {name} ({}) | {:.1}% | {:.1}% | {:+.1} pts |", a.positions, a.accuracy(), b.accuracy(), b.accuracy() - a.accuracy());
    };
    row("all", &s[0], &s[1]);
    row("discriminating cues", &s[2], &s[3]);
    let lift = s[3].accuracy() - s[2].accuracy();
    println!("\nfloor {:.1}%. Condition (docs/170 Q2): the concept path must beat the flat", s[0].floor);
    println!("window at the same budget, where a concept can act at all.");
    println!("{}\n", if lift > 0.0 { "MET on the discriminating cues." } else { "NOT MET — reported straight." });
}

/// The same question on a real corpus (docs/175): sentences from `dir`,
/// the first `train` told, the next `held` never told. The toy curriculum
/// cannot decide Q2 — every family there holds two or three sentences —
/// so this runs it where families hold hundreds.
pub fn at_scale(dir: &str, train: usize, held: usize, at_least: usize) {
    let mut sentences: Vec<String> = Vec::new();
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map(|r| r.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "txt")).collect())
        .unwrap_or_default();
    files.sort();
    for f in files {
        for s in std::fs::read_to_string(&f).unwrap_or_default().split(". ") {
            let s = s.trim();
            if s.split(' ').count() >= 5 {
                sentences.push(s.to_string());
            }
            if sentences.len() >= train + held {
                break;
            }
        }
        if sentences.len() >= train + held {
            break;
        }
    }
    if sentences.len() < train + held {
        println!("## at scale — skipped: only {} sentences at {dir}\n", sentences.len());
        return;
    }
    let mut k = Knowing::fresh();
    for (i, line) in sentences[..train].iter().enumerate() {
        let path = format!("read/e{i}");
        k.tell(&path, &phient::prose::paragraph(&path, &[line.clone()]));
    }
    let fams = k.families(at_least);
    println!("## at scale — {train} sentences told, {} families found (largest {} members)",
        fams.len(), fams.first().map(|(_, m)| m.len()).unwrap_or(0));
    let held: Vec<&str> = sentences[train..train + held].iter().map(String::as_str).collect();
    report(&k, &held, at_least);
}
