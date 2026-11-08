//! `person` — learn a person from a little of their words, on the fly, and
//! talk to it (docs/186: the user's own condition of satisfaction).
//!
//! Any corpus — a book, a feed, a transcript — is read in chunks. Before
//! each chunk is told, the next word of every sentence in it is predicted
//! (prequential: nothing is ever scored on what it was trained on), so the
//! curve is the pickup of this person's word-choices as more is fed, with
//! no retraining. After each chunk the learner ANSWERS: it continues the
//! prompts you give it, in the voice it has learned so far.
//!
//! Three readers from the arena, same positions: the counts alone, the
//! concepts (the construction family, backing off), and — the model, not
//! a contrast line — the PHASORS of phiecs read under the condition
//! (docs/187–189): the person's word-choices as angles moved by hearing.
//!
//! ```text
//! cargo run --release -p phient --example person -- <file or dir> [--chunks 10] [--prompt "you can" …]
//! ```

use phient::lineup::Ecs;
use phient::lineup::{Concept, Count};
use phient::prose::tokens;
use phient::study::{Reader, Study};

/// Sentences from a file or every .txt under a directory.
fn sentences(path: &str) -> Vec<Vec<String>> {
    let mut text = String::new();
    let p = std::path::Path::new(path);
    if p.is_dir() {
        let mut files: Vec<_> = std::fs::read_dir(p).map(|r| r.flatten().map(|e| e.path()).collect()).unwrap_or_default();
        files.sort();
        for f in files.iter().filter(|f| f.extension().is_some_and(|x| x == "txt")) {
            text.push_str(&std::fs::read_to_string(f).unwrap_or_default());
            text.push('\n');
        }
    } else {
        text = std::fs::read_to_string(p).unwrap_or_default();
    }
    text.split(|c| matches!(c, '.' | '!' | '?' | '\n'))
        .map(tokens)
        .map(|w| w.into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>())
        .filter(|w| w.len() >= 4)
        .collect()
}

/// The learner's answer: the prompt continued word by word from its own
/// guesses, stopping at eight words or when it starts repeating itself.
fn answer(reader: &mut dyn Reader, prompt: &str) -> String {
    let mut said = tokens(prompt);
    let start = said.len();
    let mut seen: Vec<(String, String)> = Vec::new();
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
    let path = a.first().cloned().unwrap_or_else(|| "corpus.txt".into());
    let chunks = a.iter().position(|x| x == "--chunks").and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(10usize);
    let mut prompts: Vec<String> = a.iter().enumerate().filter(|(_, x)| *x == "--prompt").filter_map(|(i, _)| a.get(i + 1).cloned()).collect();
    if prompts.is_empty() {
        prompts = ["you can", "it is not", "i have", "we use"].iter().map(|s| s.to_string()).collect();
    }

    let lines = sentences(&path);
    if lines.is_empty() {
        eprintln!("nothing to read at {path}");
        return;
    }
    let mut vocab: Vec<&String> = lines.iter().flatten().collect();
    vocab.sort();
    vocab.dedup();
    let floor = 100.0 / vocab.len() as f64;
    let words: usize = lines.iter().map(Vec::len).sum();

    let mut count = Count::new();
    let mut concept = Concept::new();
    let mut ecs = Ecs::new();
    let mut flat = Study::new("counts (the word before)", chunks, floor);
    let mut deep = Study::new("concepts (the family, backing off)", chunks, floor);
    let mut wave = Study::new("phasors (phiecs, read under the condition)", chunks, floor);

    println!("# Learning a person from their words — {path}\n");
    println!("{} sentences, {words} words, {} words of vocabulary, floor {floor:.2}%; read in {chunks} chunks,", lines.len(), vocab.len());
    println!("every next word predicted BEFORE its chunk is told, nothing retrained.\n");

    let size = lines.len().div_ceil(chunks);
    for (k, chunk) in lines.chunks(size).enumerate() {
        let (mut hit_f, mut hit_d, mut hit_w, mut n) = (0usize, 0usize, 0usize, 0usize);
        for w in chunk {
            for j in 1..w.len() {
                let f = count.guess(&w[..j]).as_deref() == Some(w[j].as_str());
                let d = concept.guess(&w[..j]).as_deref() == Some(w[j].as_str());
                let e = ecs.guess(&w[..j]).as_deref() == Some(w[j].as_str());
                flat.tick(k, chunks, f);
                deep.tick(k, chunks, d);
                wave.tick(k, chunks, e);
                hit_f += f as usize;
                hit_d += d as usize;
                hit_w += e as usize;
                n += 1;
            }
            count.tell(w);
            concept.tell(w);
            ecs.tell(w);
        }
        let pct = |h: usize| 100.0 * h as f64 / n.max(1) as f64;
        println!("## after chunk {}/{chunks} — this chunk, unseen: counts {:.1}%, concepts {:.1}%, PHASORS {:.1}% ({n} words)", k + 1, pct(hit_f), pct(hit_d), pct(hit_w));
        for p in &prompts {
            println!("   \"{p}\" → {}", answer(&mut ecs, p));
        }
        println!();
    }

    println!("{}", flat.report());
    println!("{}", deep.report());
    println!("{}", wave.report());
    println!("Condition (docs/186, on the PHASORS): the person's next word picked up above the");
    println!("floor from the first chunk, rising as more is fed, with no retraining.");
    let met = wave.beats_floor() && wave.rises();
    println!("{}", if met { "MET." } else { "NOT MET — reported straight." });
}
