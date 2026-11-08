//! `crowd` — typing the closed stratum (docs/202). The 2,000 most-told
//! words are one type under the plain channel: reached from everywhere,
//! their one angle is the mean (docs/200). But each speaks on the answer
//! SLOTS of what it reaches — `the` on the nouns' slots, `is` on the
//! adjectives' — so its PROFILE (sixteen angles, one per slot, or silence)
//! keeps the mixture apart. Words with the same profile, quantised, are
//! one type. No part of speech is given.
//!
//! ```text
//! cargo run --release -p phient --example crowd -- <corpus> --top 300 --sectors 4
//! ```

use phient::lineup::Ecs;
use phient::prose::tokens;
use phient::study::Reader;
use std::collections::HashMap;

fn sentences(path: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .split(|c| matches!(c, '.' | '!' | '?' | '\n'))
        .map(tokens)
        .map(|w| w.into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>())
        .filter(|w| w.len() >= 4)
        .collect()
}

/// The profile as a string: one glyph per slot — `.` silent, else the
/// sector of its angle (0..k).
pub fn glyphs(profile: &[Option<f64>], k: usize) -> String {
    profile
        .iter()
        .map(|a| match a {
            None => '.',
            Some(t) => {
                let u = t.rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU;
                char::from_digit(((u * k as f64) as u32).min(k as u32 - 1), 36).unwrap_or('?')
            }
        })
        .collect()
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = a.first().cloned() else {
        eprintln!("which corpus?");
        return;
    };
    let flag = |k: &str, d: usize| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let (top, k, show) = (flag("--top", 300), flag("--sectors", 4), flag("--show", 14));

    let lines = sentences(&path);
    let mut person = Ecs::new();
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for w in &lines {
        person.tell(w);
        for x in w {
            *freq.entry(x).or_default() += 1;
        }
    }
    let mut crowd: Vec<(&str, usize)> = freq.iter().map(|(w, n)| (*w, *n)).collect();
    crowd.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    crowd.truncate(top);

    // The DIFFERENCE, not the angle: every crowd word carries nearly the
    // same slot angles (they are means over crowds too), so a word is typed
    // by where it stands against the crowd's mean on each slot — ahead `+`,
    // behind `-`, with it `0`, silent `.` — outside a dead band of `band`
    // turns (TURN = TAU/8).
    let band = a.iter().position(|x| x == "--band").and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(0.25) * phiecs::world::TURN;
    let profiles: Vec<Vec<Option<f64>>> = crowd.iter().map(|(w, _)| person.profile(w, "next")).collect();
    let slots = profiles.first().map_or(0, Vec::len);
    let mean: Vec<f64> = (0..slots)
        .map(|s| {
            let (x, y) = profiles.iter().filter_map(|p| p[s]).fold((0.0, 0.0), |(x, y), t| (x + t.cos(), y + t.sin()));
            y.atan2(x)
        })
        .collect();
    let mut group: HashMap<String, Vec<&str>> = HashMap::new();
    for ((w, _), p) in crowd.iter().zip(&profiles) {
        let g: String = p
            .iter()
            .zip(&mean)
            .map(|(a, m)| match a {
                None => '.',
                Some(t) => {
                    let d = (t - m).sin().atan2((t - m).cos());
                    if d > band { '+' } else if d < -band { '-' } else { '0' }
                }
            })
            .collect();
        group.entry(g).or_default().push(w);
    }
    let _ = (k, glyphs(&[], 1));
    let mut groups: Vec<(&String, &Vec<&str>)> = group.iter().collect();
    groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    let singles = groups.iter().filter(|(_, m)| m.len() == 1).count();

    println!("# the crowd typed by its slot profile — {path}\n");
    println!("the {} most-told words, profiled by their DIFFERENCE from the crowd's mean on each of the 16 answer slots of `next`: {} profiles, {} of them a word alone", crowd.len(), group.len(), singles);
    println!("\n## the groups (per slot: `+` ahead of the crowd, `-` behind, `0` with it, `.` silent)\n");
    for (g, members) in groups.iter().take(show) {
        println!("  {g}  ({:>3}): {}", members.len(), members.iter().take(16).copied().collect::<Vec<_>>().join(" "));
    }
}
