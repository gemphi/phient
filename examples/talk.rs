//! `talk` — ask the person (docs/192, docs/203). The whole corpus is told
//! to the learner (phient's `lineup::Learner`): each sentence once, on the
//! fly, then `--epochs` passes in which every reading is a prediction and
//! the error moves the angles, toward the word that came and away from the
//! words expected instead. Then each question is a CONDITION: the learner
//! goes on from its last words, the likeliest word each time, and stops
//! where the line is likelier to end. A word of the question it was never
//! told is read as the unknown word, not refused.
//!
//! `--counted` answers from the counted phasors instead (`arena::Ecs`, each
//! angle moved to the mean of what it was seen with): the contrast line of
//! docs/203, never the default.
//!
//! ```text
//! cargo run --release -p phient --example talk -- <corpus> "question one" "question two" …
//! cargo run --release -p phient --example talk -- <corpus> --words 14 --epochs 7 "when do we use the present perfect"
//! cargo run --release -p phient --example talk -- <corpus> --read "i have been"
//! ```

use phient::lineup::Ecs;
use phient::lineup::learner::{Config, Learner};
use phient::prose::tokens;
use phient::study::Reader;

fn sentences(path: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .split(|c| matches!(c, '.' | '!' | '?' | '\n'))
        .map(tokens)
        .map(|w| w.into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>())
        .filter(|w| w.len() >= 4)
        .collect()
}

fn words_of(s: &str) -> Vec<String> {
    tokens(s).into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect()
}

/// The learner's answer: it goes on from the whole question.
fn answer(person: &Learner, question: &str, words: usize) -> String {
    let q = words_of(question);
    match person.speak(&q, words).as_slice() {
        [] => format!("[{}] — (the line ends there)", q.join(" ")),
        said => format!("[{}] {}", q.join(" "), said.join(" ")),
    }
}

/// The counted contrast: go on from the end of the question, stopping when
/// the voice starts repeating itself; back off over words never spoken.
fn answer_counted(person: &mut Ecs, question: &str, words: usize, tau: f64) -> String {
    let q = words_of(question);
    let Some(from) = (1..=q.len()).rev().find(|&n| person.guess(&q[..n]).is_some()) else {
        return "— (nothing of that is known yet)".into();
    };
    let mut said = q[..from].to_vec();
    let mut seen: Vec<(String, String)> = Vec::new();
    while said.len() < from + words {
        let read = person.ranked(&said);
        // A crowded word reads as a near-tie: the condition does not
        // determine what follows, and a voice that speaks anyway drifts
        // into the crowd (docs/198).
        if Ecs::clearance(&read) < tau {
            break;
        }
        let Some((next, _)) = read.first().cloned() else { break };
        let pair = (said.last().cloned().unwrap_or_default(), next.clone());
        if seen.contains(&pair) {
            break;
        }
        seen.push(pair);
        said.push(next);
    }
    format!("[{}] {}", q[..from].join(" "), said[from..].join(" "))
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = a.first().cloned() else {
        eprintln!("which corpus?");
        return;
    };
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let tau: f64 = flag("--clear").and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let words: usize = flag("--words").and_then(|s| s.parse().ok()).unwrap_or(12);
    let epochs: usize = flag("--epochs").and_then(|s| s.parse().ok()).unwrap_or(7);
    let counted = a.iter().any(|x| x == "--counted");
    let dump = flag("--read");
    let questions: Vec<&String> = a.iter().skip(1).filter(|x| !x.starts_with("--") && x.parse::<f64>().is_err() && Some(*x) != dump.as_ref()).collect();

    let lines = sentences(&path);
    let t0 = std::time::Instant::now();
    println!("# Talking to {path}\n");
    if counted {
        let mut person = Ecs::new();
        for w in &lines {
            person.tell(w);
        }
        println!("{} sentences told to the counted phasors in {:.1} s (the contrast line).\n", lines.len(), t0.elapsed().as_secs_f64());
        for q in questions {
            println!("Q: {q}\nA: {}\n", answer_counted(&mut person, q, words, tau));
        }
        return;
    }
    let mut person = Learner::new(Config { block: 8, silence: 0.2, epochs, ..Config::default() });
    for w in &lines {
        person.tell(w);
    }
    person.ready();
    println!("{} sentences told to the learner, then {epochs} epochs corrected by the error, in {:.1} s.\n", lines.len(), t0.elapsed().as_secs_f64());
    if let Some(d) = dump {
        // The read itself, for a condition: what the learner expects next,
        // and by how much.
        let said = words_of(&d);
        println!("read after [{}]:", said.join(" "));
        for (w, s) in person.ranked(&said, 10) {
            println!("  {s:>10.5}  {w}");
        }
        return;
    }
    for q in questions {
        println!("Q: {q}\nA: {}\n", answer(&person, q, words));
    }
}
