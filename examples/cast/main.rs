//! Q2: hierarchy (read_levels over construct path) vs flat (depth-16 n-gram).
//!
//! Same prequential protocol as Q1. For each function, predict every
//! next-role BEFORE training on it:
//!
//!   flat  -- depth-16 context n-gram (Q1 baseline, 37.1%)
//!   hier  -- Tally::read_levels over the construct-path hierarchy,
//!            outermost construct first, backing off level by level
//!
//! Q2 condition of satisfaction: hier overall accuracy > flat overall.
//!
//! cargo run --release -p phient --example cast -- <bcpl-dir>

use phient::code::bcpl::{self, learn::Grammar, role};
use phient::relate::{Tally, levels::SCOPE};
use phient::study::Study;
use phient::prose::tokens;
use phiecs::quest::Row;
use std::path::Path;

const DEPTH: usize = 16;
const TAU: f64 = 0.5;
const SLICES: usize = 10;

/// Walk a body, returning per-token: (construct-path, role-of-token, next-role).
/// Path is outermost-first list of construct names.
fn walk_with_path(body: &str) -> Vec<(Vec<String>, &'static str, &'static str)> {
    let toks = tokens(body);
    let mut path: Vec<String> = vec!["fn".into()];
    let mut last_role = "fn";
    let mut out = Vec::new();
    for t in &toks {
        let r = role::of(t);
        out.push((path.clone(), last_role, r));
        // maintain path
        match t.as_str() {
            "$)" | ")" | "]" | "resultis" if path.len() > 1 => { path.pop(); }
            _ => {}
        }
        let opens = match t.as_str() {
            "valof" => Some("valof"), "let" | "and" => Some("let"),
            "if" => Some("if"), "unless" => Some("unless"),
            "test" => Some("test"),
            "while" | "until" | "repeatwhile" | "repeatuntil" => Some("loop"),
            "for" => Some("for"), "switchon" => Some("switchon"),
            "case" => Some("case"), "$(" | "(" | "[" => Some("block"),
            _ => None,
        };
        if let Some(lv) = opens { path.push(lv.into()); }
        last_role = r;
    }
    out
}

fn main() {
    let root = std::env::args().nth(1)
        .unwrap_or_else(|| "../ext/bcpl/cintcode/com".into());
    let bodies = bcpl::read_bodies(Path::new(&root));
    if bodies.is_empty() {
        eprintln!("No BCPL .b files at {root}"); std::process::exit(1);
    }
    println!("Q2 -- hierarchy vs flat on {} BCPL functions\n", bodies.len());

    let values: Vec<String> = ["keyword","name","number","string","bracket","operator"]
        .iter().map(|s| s.to_string()).collect();
    let floor = 100.0 / values.len() as f64;

    let mut flat_g = Grammar::new(DEPTH);
    let mut counts  = Tally::new(&["within"]);
    let mut flat_study = Study::new("flat depth-16 (Q1 baseline)", SLICES, floor);
    let mut hier_study = Study::new("hier read_levels  (Q2)", SLICES, floor);

    let total: usize = bodies.iter().map(|b| tokens(b).len().saturating_sub(1)).sum();
    let mut global_pos = 0usize;

    for body in &bodies {
        let positions = walk_with_path(body);
        if positions.is_empty() { continue; }

        // -- flat: predict from depth-16 n-gram BEFORE learning --
        for hit in flat_g.score(body) {
            flat_study.tick(global_pos, total, hit);
            global_pos += 1;
        }
        flat_g.learn(body);

        // -- hier: predict from read_levels BEFORE telling counts --
        // We already advanced global_pos; use a local offset.
        let hier_start = global_pos - positions.len();
        for (i, (path, _last, next_role)) in positions.iter().enumerate() {
            // Build levels: outermost scope first, each is a vec of one condition.
            let levels: Vec<Vec<String>> = path.iter()
                .enumerate()
                .map(|(j, _)| vec![format!("{}{}", SCOPE, path[..=j].join("/"))])
                .collect();
            let (scores, _) = counts.read_levels(&levels, "next_role", &values, TAU);
            let best_idx = scores.iter().enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(i, _)| i);
            let hit = best_idx.map(|bi| values[bi] == *next_role).unwrap_or(false);
            hier_study.tick(hier_start + i, total, hit);
        }

        // Tell the hier counts AFTER prediction (prequential).
        for (path, last, next_r) in &positions {
            let scope = format!("{}{}", SCOPE, path.join("/"));
            counts.learn(&[Row::new(&scope, "within", &scope)]);
            counts.learn(&[Row::new(last, "next_role", next_r)]);
            counts.learn(&[Row::new(&format!("{}|{}", scope, last), "next_role", next_r)]);
        }
    }

    println!("{}", flat_study.report());
    println!();
    println!("{}", hier_study.report());

    let fa = flat_study.accuracy();
    let ha = hier_study.accuracy();
    println!("\nQ2 result:");
    println!("  flat  {fa:.1}%  ({:.2}x floor)", fa / floor);
    println!("  hier  {ha:.1}%  ({:.2}x floor)", ha / floor);
    println!("  lift  {:+.1}%  =>  Q2 {}",
        ha - fa,
        if ha > fa { "MET" } else { "not yet -- hierarchy did not beat flat" });
}