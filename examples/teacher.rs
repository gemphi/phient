//! `teacher` — the quest closes the loop on the person (docs/191).
//!
//! A person is learned in chunks (docs/186), on the phasors (docs/190).
//! After each chunk the learner names the constructions it is UNSURE of —
//! shown once, not yet a family (docs/174) — and asks a teacher for uses
//! of exactly those; the answers are told into the World like anything
//! else. Two arms on the same chunks, same positions, scored before each
//! chunk is told: ALONE, and TAUGHT. The teacher never sees a scored chunk.
//!
//! The teacher is any [`Oracle`]: `--oracle ollama:<model>` on a machine
//! with ollama; here, the RESERVE — the last fifth of the corpus, held out
//! of the stream entirely — answers the same prompts with the person's own
//! sentences. One code path, two teachers.
//!
//! ```text
//! cargo run --release -p phient --example teacher -- <corpus> [--chunks 8] [--ask 24] [--oracle ollama:phi4]
//! ```

use phient::lineup::Ecs;
use phient::lineup::Count;
use phiecs::quest::{Oracle, OracleError, Spawned};
use phient::prose::{family_prompt, paragraph, tokens, Knowing};
use phient::study::{Reader, Study};

/// The reserve as a teacher: answers a family prompt with up to three of
/// its own sentences that use the pair adjacently.
struct Reserve(Vec<Vec<String>>);

impl Oracle for Reserve {
    fn ask(&mut self, prompt: &str) -> Result<String, OracleError> {
        let pair: Vec<&str> = prompt.split('"').nth(1).unwrap_or("").split(' ').collect();
        let hit = |w: &Vec<String>| w.windows(2).any(|p| pair.len() == 2 && p[0] == pair[0] && p[1] == pair[1]);
        Ok(self.0.iter().filter(|w| hit(w)).take(3).map(|w| w.join(" ")).collect::<Vec<_>>().join("\n"))
    }
}

fn sentences(path: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .split(|c| matches!(c, '.' | '!' | '?' | '\n'))
        .map(tokens)
        .map(|w| w.into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>())
        .filter(|w| w.len() >= 4)
        .collect()
}

fn answer(reader: &mut dyn Reader, prompt: &str) -> String {
    let mut said = tokens(prompt);
    let (start, mut seen) = (said.len(), Vec::new());
    while said.len() < start + 8 {
        let Some(next) = reader.guess(&said) else { break };
        let pair = (said.last().cloned().unwrap_or_default(), next.clone());
        if seen.contains(&pair) {
            break;
        }
        seen.push(pair);
        said.push(next);
    }
    said.join(" ")
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let path = a.first().cloned().unwrap_or_default();
    let flag = |k: &str, d: usize| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let (chunks, ask) = (flag("--chunks", 8), flag("--ask", 24));
    let prompts = ["you can", "it is not", "the compiler", "she has been"];

    let all = sentences(&path);
    let cut = all.len() * 4 / 5;
    let (stream, reserve) = (&all[..cut], &all[cut..]);
    let mut oracle: Box<dyn Oracle> = match a.iter().position(|x| x == "--oracle").and_then(|i| a.get(i + 1)) {
        Some(o) if o.starts_with("ollama:") => Box::new(Spawned::ollama(&o[7..])),
        _ => Box::new(Reserve(reserve.to_vec())),
    };
    let vocab: std::collections::HashSet<&String> = stream.iter().flatten().collect();
    let floor = 100.0 / vocab.len().max(1) as f64;

    let (mut alone, mut taught, mut count) = (Ecs::new(), Ecs::new(), Count::new());
    let (mut s_alone, mut s_taught, mut s_count) =
        (Study::new("phasors, alone", chunks, floor), Study::new("phasors, TAUGHT by the quest", chunks, floor), Study::new("counts, alone (contrast)", chunks, floor));
    let mut knowing = Knowing::fresh();
    let (mut asked, mut told) = (0usize, 0usize);

    println!("# The teacher — the quest closes the loop on the person: {path}\n");
    println!("{} sentences streamed in {chunks} chunks; {} held in reserve as the teacher's book; floor {floor:.2}%.\n", stream.len(), reserve.len());
    let size = stream.len().div_ceil(chunks);
    for (k, chunk) in stream.chunks(size).enumerate() {
        let (mut ha, mut ht, mut hc, mut n) = (0, 0, 0, 0);
        for w in chunk {
            for j in 1..w.len() {
                let truth = w[j].as_str();
                let (x, y, z) = (alone.guess(&w[..j]).as_deref() == Some(truth), taught.guess(&w[..j]).as_deref() == Some(truth), count.guess(&w[..j]).as_deref() == Some(truth));
                s_alone.tick(k, chunks, x);
                s_taught.tick(k, chunks, y);
                s_count.tick(k, chunks, z);
                ha += x as usize;
                ht += y as usize;
                hc += z as usize;
                n += 1;
            }
            alone.tell(w);
            taught.tell(w);
            count.tell(w);
            let p = format!("stream/c{k}/s{}", knowing.paragraphs.len());
            knowing.tell(&p, &paragraph(&p, &[w.join(" ")]));
        }
        // The quest: what is the taught one unsure of? Ask, and tell the uses.
        let mut uses = 0;
        for pair in knowing.unsure(2).into_iter().take(ask) {
            asked += 1;
            if let Ok(reply) = oracle.ask(&family_prompt(&pair)) {
                for line in reply.lines().map(tokens).filter(|w| w.len() >= 3) {
                    taught.tell(&line);
                    let p = format!("taught/c{k}/u{uses}");
                    knowing.tell(&p, &paragraph(&p, &[line.join(" ")]));
                    uses += 1;
                }
            }
        }
        told += uses;
        let pct = |h: usize| 100.0 * h as f64 / n.max(1) as f64;
        println!("## chunk {}/{chunks} unseen — phasors alone {:.1}%, TAUGHT {:.1}%, counts {:.1}%; then asked {} constructions, told {uses} uses", k + 1, pct(ha), pct(ht), pct(hc), ask.min(asked));
        for p in prompts.iter().take(2) {
            println!("   \"{p}\" → {}", answer(&mut taught, p));
        }
    }
    println!("\n{}\n{}\n{}", s_alone.report(), s_taught.report(), s_count.report());
    let lift = s_taught.accuracy() - s_alone.accuracy();
    println!("Asked {asked} constructions, told {told} uses from the teacher (never a scored sentence).");
    println!("Condition (docs/191): taught beats alone on the same unseen positions by a clear margin.");
    println!("Lift {lift:+.1} pts: {}", if lift >= 0.5 { "MET." } else if lift > -0.5 { "NO EFFECT at this dose — reported straight." } else { "NOT MET — reported straight." });
}
