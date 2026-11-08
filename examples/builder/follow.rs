//! Following a rule, with Phi-4 as the other player (Philosophical
//! Investigations §§185–242, docs/107, paper 23). Phi-4 was put in the
//! builders' game (`builders ask`) and told the set-up, never the
//! rules. How far does it go on as our rules go on, where does it go on
//! differently, does it go on the same way twice — and can a student learn
//! to go on as *Phi-4* does?

use super::experiments;
use super::world::{Episode, ASPECTS};
use phiecs::quest::builders;
use std::collections::BTreeMap;

/// One asked call: the rule's action and the teacher's reply.
pub struct Asked {
    pub rule: Episode,
    pub raw: Option<[String; 5]>,
}

/// The reply again, with the parser of today: newlines were written as `⏎`
/// (or, in the first files, flattened to spaces, where runs of three spaces
/// stand in for them).
fn reread(reply: &str) -> Option<[String; 5]> {
    let text = if reply.contains('⏎') { reply.replace('⏎', "\n") } else { reply.split("   ").collect::<Vec<_>>().join("\n") };
    builders::parse(&text)
}

pub fn load(path: &str) -> Vec<Asked> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let rule = super::teacher::load_line(l, false)?;
            let reply = f.get(4).copied().unwrap_or("").to_string();
            Some(Asked { raw: reread(&reply), rule })
        })
        .collect()
}

/// The teacher's notation read in the game's terms, only where the call
/// had no word for the aspect: `-` for "no colour said" is the game's `any`,
/// for "no numeral" its `1`, for "no place" its `here`. Nothing else is
/// changed: a wrong act or stone stays wrong.
pub fn notation(_call: &Episode, a: &[String; 5]) -> [String; 5] {
    let mut out = a.clone();
    let doing = matches!(out[0].as_str(), "bring" | "record");
    if doing {
        for (i, default) in [(2, "any"), (3, "1"), (4, "here")] {
            if out[i] == "-" {
                out[i] = default.to_string();
            }
        }
    }
    out
}

fn pct(n: usize, d: usize) -> String {
    format!("{:.1}%", 100.0 * n as f64 / d.max(1) as f64)
}

/// The read replies: the call, Phi-4's notation, and the game's reading of it.
pub fn read_of(asked: &[Asked]) -> Vec<(&Episode, [String; 5], [String; 5])> {
    asked.iter().filter_map(|x| x.raw.clone().map(|r| (&x.rule, r.clone(), notation(&x.rule, &r)))).collect()
}

/// Pairs of answers to the same call, and how many of them are identical.
pub fn consistency(read: &[(&Episode, [String; 5], [String; 5])]) -> (usize, usize) {
    let mut by: BTreeMap<String, Vec<&[String; 5]>> = BTreeMap::new();
    for x in read {
        by.entry(x.0.key()).or_default().push(&x.2);
    }
    let (mut pairs, mut same) = (0usize, 0usize);
    for v in by.values() {
        for i in 0..v.len() {
            for j in i + 1..v.len() {
                pairs += 1;
                same += (v[i] == v[j]) as usize;
            }
        }
    }
    (same, pairs)
}

pub fn report(asked: &[Asked], rs: &[usize], epochs: usize, seed: u64) -> String {
    let read = read_of(asked);
    let mut s = format!("## Phi-4 in the builders' game: {} calls, {} replies read\n\n", asked.len(), read.len());
    s += "### How far it goes on as our rules go on\n\n| reading | exact | act | what | colour | count | place |\n|---|---|---|---|---|---|---|\n";
    for (name, k) in [("its own notation", 1usize), ("notation read in the game's terms", 2)] {
        let get = |x: &(&Episode, [String; 5], [String; 5])| if k == 1 { x.1.clone() } else { x.2.clone() };
        let mut row = vec![name.to_string(), pct(read.iter().filter(|x| get(x) == x.0.action).count(), read.len())];
        row.extend((0..5).map(|a| pct(read.iter().filter(|x| get(x)[a] == x.0.action[a]).count(), read.len())));
        s += &format!("| {} |\n", row.join(" | "));
    }
    s += "\n### Where it goes on differently (game's terms): the commonest divergences\n\n| aspect | rule ⇒ Phi-4 | calls |\n|---|---|---|\n";
    for (a, aspect) in ASPECTS.iter().enumerate() {
        let mut c: BTreeMap<(String, String), usize> = BTreeMap::new();
        for x in read.iter().filter(|x| x.2[a] != x.0.action[a]) {
            *c.entry((x.0.action[a].clone(), x.2[a].clone())).or_default() += 1;
        }
        let mut v: Vec<_> = c.into_iter().collect();
        v.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
        for ((r, t), n) in v.into_iter().take(3) {
            s += &format!("| {aspect} | {r} ⇒ {t} | {n} |\n");
        }
    }
    // Does it go on the same way twice?
    let mut by: BTreeMap<String, Vec<[String; 5]>> = BTreeMap::new();
    for x in &read {
        by.entry(x.0.key()).or_default().push(x.2.clone());
    }
    let (mut pairs, mut same) = (0usize, 0usize);
    for v in by.values().filter(|v| v.len() > 1) {
        for i in 0..v.len() {
            for j in i + 1..v.len() {
                pairs += 1;
                same += (v[i] == v[j]) as usize;
            }
        }
    }
    s += &format!("\n### Does it go on the same way twice?\n\nCalls asked more than once: {}. Pairs of answers to the same call that are identical (game's terms): {} of {} ({}).\n\n", by.values().filter(|v| v.len() > 1).count(), same, pairs, pct(same, pairs));
    s += &student(&read, rs, epochs, seed).0;
    s
}

/// Can a student learn to go on as Phi-4 does? Five folds by call (a call
/// asked twice is never on both sides); each learner trained on Phi-4's
/// actions in four folds predicts its action in the fifth. Our rules are
/// scored the same way, as one more predictor of Phi-4.
pub fn student(read: &[(&Episode, [String; 5], [String; 5])], rs: &[usize], epochs: usize, seed: u64) -> (String, Vec<(String, f64)>) {
    let taught: Vec<Episode> = read.iter().map(|x| Episode { action: x.2.clone(), ..x.0.clone() }).collect();
    let fold = |e: &Episode| (phistd::mix64(phistd::fnv1a(&e.key()) ^ seed) % 5) as usize;
    let mut rows: Vec<(String, usize, [usize; 5])> = Vec::new();
    let (mut rule_exact, mut rule_asp, mut n) = (0usize, [0usize; 5], 0usize);
    for f in 0..5 {
        let train: Vec<Episode> = taught.iter().filter(|e| fold(e) != f).cloned().collect();
        let test: Vec<(&Episode, &Episode)> = read.iter().zip(&taught).filter(|(_, e)| fold(e) == f).map(|(x, e)| (x.0, e)).collect();
        n += test.len();
        for (rule, t) in &test {
            rule_exact += (rule.action == t.action) as usize;
            for (hits, (r, w)) in rule_asp.iter_mut().zip(rule.action.iter().zip(&t.action)) {
                *hits += (r == w) as usize;
            }
        }
        for (i, l) in experiments::all(&train, rs, epochs, seed).iter_mut().enumerate() {
            if rows.len() <= i {
                rows.push((l.name(), 0, [0; 5]));
            }
            for (_, t) in &test {
                let y = l.act(t);
                rows[i].1 += (y == t.action) as usize;
                for (hits, (p, w)) in rows[i].2.iter_mut().zip(y.iter().zip(&t.action)) {
                    *hits += (p == w) as usize;
                }
            }
        }
        println!("follow fold {f} done");
    }
    let mut s = String::from("### Can a student learn to go on as Phi-4 does? (5 folds by call)\n\n| predictor of Phi-4's action | exact | act | what | colour | count | place |\n|---|---|---|---|---|---|---|\n");
    let line = |name: &str, ex: usize, asp: [usize; 5]| format!("| {name} | {} | {} |\n", pct(ex, n), asp.iter().map(|x| pct(*x, n)).collect::<Vec<_>>().join(" | "));
    s += &line("our rules (the game as we wrote it)", rule_exact, rule_asp);
    let mut exact = vec![("our rules".to_string(), rule_exact as f64 / n.max(1) as f64)];
    for (name, ex, asp) in rows {
        s += &line(&name, ex, asp);
        exact.push((name, ex as f64 / n.max(1) as f64));
    }
    (s, exact)
}
