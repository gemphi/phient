//! Phi-4 in the game (docs/103): the teacher is put in the builders' game
//! through the phiecs quest kind `builders` and asked what it does on each
//! call; its actions become the episodes a student learns from. The rule
//! action is kept beside it, so the teacher can be read against the game as
//! well as the student against the teacher.
//!
//! File: one episode a line — `game \t tokens \t rule action \t teacher
//! action \t reply \t index`, actions as `act|what|colour|count|place`, `?` when the
//! reply could not be read.

use super::world::{Dice, Episode};
use phiecs::quest::builders;
use phiecs::quest::{Oracle, Spawned};
use std::sync::Mutex;

fn gesture(t: &str) -> Option<String> {
    let x = t.strip_prefix("⟨point:")?.strip_suffix('⟩')?;
    Some(match x.starts_with('p') && x.len() == 2 {
        true => format!("pointing at place {x}"),
        false => format!("pointing at a {x}"),
    })
}

/// The prompt's words and gestures for an episode.
pub fn said(e: &Episode) -> (String, Vec<String>) {
    let words: Vec<&str> = e.tokens.iter().filter(|t| !t.starts_with('⟨')).map(String::as_str).collect();
    (words.join(" "), e.tokens.iter().filter_map(|t| gesture(t)).collect())
}

/// Asks `oracle` (a command, e.g. `ollama run phi4`) about `n` of the
/// `train` episodes, `parallel` at a time, each put by `prompt`; lines already in `out` are kept, so a
/// stopped run resumes.
pub fn ask(train: &[Episode], n: usize, seed: u64, out: &str, oracle: &[String], parallel: usize, prompt: &(dyn Fn(&Episode) -> String + Sync)) {
    let mut order: Vec<usize> = (0..train.len()).collect();
    let mut d = Dice(seed ^ 0x7EAC);
    for i in (1..order.len()).rev() {
        order.swap(i, d.below(i + 1));
    }
    // Each line ends with the episode's place in the asking order, so a run
    // stopped mid-way (answers arrive out of order) resumes exactly.
    let asked: std::collections::HashSet<usize> = std::fs::read_to_string(out).unwrap_or_default().lines().filter_map(|l| l.rsplit('\t').next()?.parse().ok()).collect();
    let done = asked.len();
    let todo: Vec<(usize, &Episode)> = order.iter().take(n).enumerate().filter(|(j, _)| !asked.contains(j)).map(|(j, &i)| (j, &train[i])).collect();
    println!("{done} already asked · {} to ask · {parallel} at a time · {}", todo.len(), oracle.join(" "));
    let file = Mutex::new(std::fs::OpenOptions::new().create(true).append(true).open(out).expect("open episodes file"));
    let next = Mutex::new(0usize);
    std::thread::scope(|s| {
        for _ in 0..parallel.max(1) {
            s.spawn(|| {
                let args: Vec<&str> = oracle[1..].iter().map(String::as_str).collect();
                let mut o = Spawned::new(&oracle[0], &args);
                loop {
                    let i = {
                        let mut g = next.lock().unwrap();
                        *g += 1;
                        *g - 1
                    };
                    let Some(&(j, e)) = todo.get(i) else { break };
                    let t0 = std::time::Instant::now();
                    let reply = o.ask(&prompt(e)).unwrap_or_else(|x| format!("error: {x:?}"));
                    let act = builders::parse(&reply).map_or("?".to_string(), |a| a.join("|"));
                    let line = format!("{}\t{}\t{}\t{}\t{}\t{j}\n", e.game, e.tokens.join(" "), e.action.join("|"), act, builders::strip_escapes(reply.trim()).replace('\t', " ").replace('\n', "⏎"));
                    use std::io::Write;
                    let _ = file.lock().unwrap().write_all(line.as_bytes());
                    println!("{:>4}. {:<7} {:<36} ⇒ {:<34} (rule {}) {:.1}s", done + i + 1, e.game, e.tokens.join(" "), act, e.action.join("|"), t0.elapsed().as_secs_f64());
                }
            });
        }
    });
}

/// One line of an episodes file, with the teacher's action (`teacher`) or
/// the rule's as what followed; `None` for an unread reply.
pub fn load_line(l: &str, teacher: bool) -> Option<Episode> {
    let f: Vec<&str> = l.split('\t').collect();
    let act = if teacher { *f.get(3)? } else { *f.get(2)? };
    let action: [String; 5] = act.split('|').map(String::from).collect::<Vec<_>>().try_into().ok()?;
    Some(Episode { game: f[0].to_string(), tokens: f.get(1)?.split(' ').map(String::from).collect(), action })
}
