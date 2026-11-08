//! `writer` — the modelless learning loop at two levels (docs/180–181).
//!
//! A programming language is also a person: the difference is KINDS only —
//! the rest is the same, doing things with words. Here the words are BCPL,
//! the doing is `phiclm::judge`, and the felicity condition is execution.
//! Three arms, one table, same rounds, same alphabet, same judge:
//!
//! - **soup** — no learning: the floor.
//! - **token** — depth-2 counts, novel successes told back (docs/180).
//! - **mold** — construction level: proven BUILDINGS reused, their open
//!   stratum varied, mutations minting new buildings (docs/181).
//! - **scribe** — the two levels COUPLED (docs/183) and, since docs/184,
//!   COMPOSING: one write in five wanders (minting), and one mold draw in
//!   roughly seven joins a second proven building on the end, bindings
//!   re-based — buildings composed, not leaves.
//!
//! What ran is told as rows to a Being — the person the language is — and
//! asked back like anyone: `(ask scribe p0 returned)`.
//!
//! ```text
//! cargo run --release -p phient --example writer -- [rounds]
//! ```

use phient::scribe::{coin, Memory, Scribe};
use phiecs::being::Being;
use phiecs::persona::Persona;
use phiecs::agent::Agent;
use phiclm::token::{lex, Token};
use phient::study::Study;

const SHOWN: &str = "LET f(n) = n = 0 -> 1, n * f(n - 1)\nLET start() = f(3)";
const ALSO: &str = "LET start() = VALOF $( LET x = 0 FOR n = 1 TO 9 DO x := x + n RESULTIS x $)";

fn alphabet() -> Vec<String> {
    let mut v: Vec<String> = phiclm::token::KEYWORDS.iter().map(|k| k.to_string()).collect();
    for p in ["(", ")", "=", ",", "+", "-", "*", "/", "!", "$(", "$)", ":=", "<", ">", "->"] {
        v.push(p.into());
    }
    for n in NAME.iter().chain(FIGURE) {
        v.push(n.to_string());
    }
    v
}

const NAME: &[&str] = &["start", "f", "n", "x", "v", "g", "s", "t"];
const FIGURE: &[&str] = &["0", "1", "2", "3", "4", "5", "7", "9", "100"];

fn lexemes(src: &str) -> Vec<String> {
    lex(src)
        .expect("the shown programs lex")
        .into_iter()
        .filter_map(|t| match t {
            Token::Num(n) => Some(n.to_string()),
            Token::Name(w) => Some(w),
            Token::Key(k) => Some(k.to_string()),
            Token::Pun(p) => Some(p.to_string()),
            _ => None,
        })
        .collect()
}

fn render(tok: &[String]) -> String {
    let mut s = String::new();
    for (i, t) in tok.iter().enumerate() {
        if i > 0 {
            s.push(if t == "let" { '\n' } else { ' ' });
        }
        s.push_str(t);
    }
    s
}

/// One arm: run rate, novelty curve, kept sources, new skeletons, and the
/// DISTINCT BEHAVIOURS among them — what was written and what came back.
/// Composition makes skeletons combinatorial, so structure alone stops
/// being scarce; behaviour is the stricter novelty (docs/184).
fn arm(mode: &str, rounds: usize, seed: u64) -> (Study, Study, Vec<String>, usize, usize) {
    let vocab = alphabet();
    let (name, figure): (Vec<String>, Vec<String>) =
        (NAME.iter().map(|s| s.to_string()).collect(), FIGURE.iter().map(|s| s.to_string()).collect());
    let mut scribe = Scribe::new();
    let mut rng = seed;
    let (mut rate, mut novel) = (Study::new(mode, 10, 0.0), Study::new("novel", 10, 0.0));
    let (mut kept, mut skeleton) = (Vec::new(), 0usize);
    let mut behaviour: std::collections::HashSet<(String, i64)> = Default::default();
    if mode != "soup" {
        for s in [SHOWN, ALSO] {
            scribe.keep(&lexemes(s));
        }
    }
    for i in 0..rounds {
        let wander = mode == "scribe" && coin(&mut rng) < 0.2;
        let tok = match mode {
            "soup" => Memory::new().write(&vocab, &mut rng, 1.0, 60),
            "token" => scribe.memory.write(&vocab, &mut rng, 0.08, 60),
            _ if wander => scribe.memory.write(&vocab, &mut rng, 0.08, 60),
            _ => scribe
                .mold
                .write(&scribe.memory, &name, &figure, &mut rng, 0.2, if mode == "scribe" { 0.15 } else { 0.0 })
                .unwrap_or_else(|| scribe.memory.write(&vocab, &mut rng, 0.08, 60)),
        };
        let source = render(&tok);
        let v = phiclm::judge(&source, 20_000);
        let ran = v.compiled && v.error.is_none();
        rate.tick(i, rounds, ran);
        // The interest rule one level up (docs/184): a success is NOVEL
        // only if it BEHAVES newly — writes or returns something no kept
        // program has. String novelty was farmable by composition; what a
        // program does is the scarcer thing, so it is what gets fed back.
        let new = ran && mode != "soup" && behaviour.insert((v.out.clone(), v.result));
        novel.tick(i, rounds, new);
        if new {
            skeleton += scribe.keep(&tok) as usize;
            kept.push(source);
        }
    }
    (rate, novel, kept, skeleton, behaviour.len())
}

fn main() {
    let rounds = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(10_000usize);
    const SEED: [u64; 5] = [7, 11, 13, 17, 19];
    println!("# The writer at two levels — a language is a person; the judge is its felicity condition\n");
    println!("Every number is the mean of {} seeded runs, with its range — a single run's", SEED.len());
    println!("count swings with the seed, and a swinging number proves nothing.\n");
    println!("| arm | ran (mean) | distinct kept (mean, range) | new skeletons (mean, range) | behaviours (mean) |\n|---|---|---|---|---|");
    let mut molded: Vec<String> = Vec::new();
    for mode in ["soup", "token", "mold", "scribe"] {
        let (mut rate, mut kept, mut skeleton, mut act) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (i, seed) in SEED.iter().enumerate() {
            let (r, _, k, sk, be) = arm(mode, rounds, *seed);
            rate.push(r.accuracy());
            skeleton.push(sk);
            act.push(be);
            if mode == "scribe" && i == 0 {
                molded = k.clone();
            }
            kept.push(k.len());
        }
        let mean = |v: &[usize]| v.iter().sum::<usize>() as f64 / v.len() as f64;
        println!(
            "| {mode} | {:.1}% | {:.0} ({}–{}) | {:.0} ({}–{}) | {:.0} |",
            rate.iter().sum::<f64>() / rate.len() as f64,
            mean(&kept),
            kept.iter().min().expect("runs"),
            kept.iter().max().expect("runs"),
            mean(&skeleton),
            skeleton.iter().min().expect("runs"),
            skeleton.iter().max().expect("runs"),
            mean(&act),
        );
    }
    println!("\nThe last two programs the coupled scribe wrote and the judge passed:");
    for k in molded.iter().rev().take(2) {
        println!("---\n{k}");
    }
    // The person the language is: its deeds told as rows, asked like anyone.
    let mut being = Being::new(Persona::from_md("---\nname: Scribe\nseeds: program\nkinds: compiled, returned, wrote, failed\n---\n").expect("persona"));
    for (i, src) in molded.iter().rev().take(8).enumerate() {
        being.tell(&phiclm::judge(src, 20_000).rows(&format!("p{i}")));
    }
    let answer = being.ask("p0", "returned");
    println!("---\ntold {} rows of its own deeds; (ask scribe p0 returned) → {:?}", being.told(), answer.first().map(|(w, _)| w.as_str()).unwrap_or("—"));
    let _ = coin(&mut 7u64);
}
