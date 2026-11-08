//! Learning from the teacher in action (docs/103): the same utterances,
//! followed once by Phi-4's actions and once by the rule's; students
//! trained on each are tested against the game's rules. The gap is what
//! the teacher's use costs (or teaches) a student.

use super::experiments;
use super::split::Split;
use super::teacher;
use super::world::{Episode, ASPECTS};

/// A named test for which episodes belong to a group.
type Group = (&'static str, fn(&Episode) -> bool);

fn share(xs: &[(&Episode, &Episode)], f: impl Fn(&Episode, &Episode) -> bool) -> String {
    format!("{:.1}%", 100.0 * xs.iter().filter(|(r, t)| f(r, t)).count() as f64 / xs.len().max(1) as f64)
}

/// How often the teacher did what the rules say, by kind of episode.
fn agreement(rule: &[Episode], taught: &[Episode]) -> String {
    let mut s = String::from("| episodes | n | exact | act | what | colour | count | place |\n|---|---|---|---|---|---|---|---|\n");
    let groups: [Group; 3] = [
        ("orders", |e| e.game == "order" && e.action[0] == "bring"),
        ("no name (§27)", |e| e.game == "order" && e.action[0] != "bring"),
        ("reports (§21)", |e| e.game == "report"),
    ];
    for (name, g) in groups {
        let pairs: Vec<(&Episode, &Episode)> = rule.iter().zip(taught).filter(|(r, _)| g(r)).collect();
        let mut row = vec![name.to_string(), pairs.len().to_string(), share(&pairs, |r, t| r.action == t.action)];
        row.extend((0..ASPECTS.len()).map(|a| share(&pairs, |r, t| r.action[a] == t.action[a])));
        s += &format!("| {} |\n", row.join(" | "));
    }
    s
}

pub fn run(path: &str, rs: &[usize], epochs: usize, seed: u64) -> String {
    let lines = std::fs::read_to_string(path).unwrap_or_default();
    let read: Vec<(Episode, Option<Episode>)> = lines
        .lines()
        .filter_map(|l| {
            let one = |t: bool| teacher::load_line(l, t);
            Some((one(false)?, one(true)))
        })
        .collect();
    let unread = read.iter().filter(|(_, t)| t.is_none()).count();
    let (rule, taught): (Vec<Episode>, Vec<Episode>) = read.into_iter().filter_map(|(r, t)| Some((r, t?))).unzip();
    let sp = Split::new(seed);
    let tests: Vec<Episode> = [&sp.seen, &sp.combos, &sp.report, &sp.no_name].into_iter().flatten().cloned().collect();
    let mut md = format!("## Phi-4 in the game: {} episodes asked, {unread} replies unread\n\n### Did the teacher do what the rules say?\n\n", rule.len() + unread);
    md += &agreement(&rule, &taught);
    md += "\n### Students trained on the teacher's actions vs on the rule's, same utterances — exact match on all four rule tests\n\n| learner | from the rules | from Phi-4 |\n|---|---|---|\n";
    let mut rows: Vec<Vec<String>> = Vec::new();
    for train in [&rule, &taught] {
        for (i, l) in experiments::all(train, rs, epochs, seed).iter_mut().enumerate() {
            if rows.len() <= i {
                rows.push(vec![l.name()]);
            }
            let ok = tests.iter().filter(|e| l.act(e) == e.action).count() as f64 / tests.len() as f64;
            rows[i].push(format!("{:.1}%", 100.0 * ok));
        }
    }
    for r in rows {
        md += &format!("| {} |\n", r.join(" | "));
    }
    md
}
