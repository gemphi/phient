//! `workshop` — teacher, student and compiler coupled on the English guide
//! (docs/192). The book is the curriculum: each unit is a lesson. For
//! every lesson the student (phient's learner) is scored on the unit's sentences
//! BEFORE it is told them — what it brought — then told them (on the fly),
//! then names the constructions it is unsure of (the quest's prompts), the
//! unit's exercises become the workshop prompts for a teacher, and one task
//! that is logic rather than memory is handed to the compiler: phiclm
//! counts, the judge verifies. Every page is written as it is played.
//!
//! Since docs/194 the KEY at the back of the book grades the student: the
//! Key is never told; after each unit's lesson the student must complete
//! the unit's answers word by word (top-1).
//!
//! Since docs/203 the student is the LEARNER: told a sentence, it predicts
//! each word, compares it with the word that came, and the error moves the
//! angles. It is corrected on the fly, as it is told; `--rehearse` adds one
//! pass over everything told after each page. The counted phasors (the old
//! student, `arena::Ecs`) and the counts are told the same sentences as the
//! contrast lines.
//!
//! ```text
//! cargo run --release -p phient --example workshop -- crates/phiano/data/person/murphy.txt --key crates/phiano/data/person/murphy_key.txt --pages 64 --out crates/phiano/docs/193_workshop.md [--rehearse]
//! ```

mod key;

use phient::lineup::Ecs;
use phient::lineup::learner::{Config, Learner};
use phient::lineup::Count;
use phient::prose::{family_prompt, paragraph, tokens, Knowing};
use phient::study::Reader;
use std::fmt::Write as _;

struct Unit {
    number: usize,
    title: String,
    lines: Vec<String>,
}

/// The book split at `Unit` / `N  Title`; the Key at the back is kept out.
fn units(text: &str) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(l) = lines.next() {
        if l.trim().starts_with("Key to Exercises") && !out.is_empty() {
            break;
        }
        if l.trim() == "Unit" {
            if let Some(head) = lines.peek() {
                let t = head.trim();
                let mut it = t.splitn(2, char::is_whitespace);
                if let (Some(n), Some(title)) = (it.next().and_then(|n| n.parse::<usize>().ok()), it.next()) {
                    lines.next();
                    out.push(Unit { number: n, title: title.trim().to_string(), lines: Vec::new() });
                    continue;
                }
            }
        }
        if let Some(u) = out.last_mut() {
            let t = l.trim();
            if !t.is_empty() {
                u.lines.push(t.to_string());
            }
        }
    }
    out
}

fn sentences(lines: &[String]) -> Vec<Vec<String>> {
    lines
        .join("\n")
        .split(|c| matches!(c, '.' | '!' | '?' | '\n'))
        .map(tokens)
        .map(|w| w.into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>())
        .filter(|w| w.len() >= 4)
        .collect()
}

/// Top-1 next-word over the sentences, without telling them.
fn score(student: &mut impl Reader, sents: &[Vec<String>]) -> (usize, usize) {
    let (mut hit, mut n) = (0, 0);
    for w in sents {
        for j in 1..w.len() {
            hit += (student.guess(&w[..j]).as_deref() == Some(w[j].as_str())) as usize;
            n += 1;
        }
    }
    (hit, n)
}

/// The compiler's task for a page: logic, not memory — count the negatives
/// among the unit's sentences from flags the student read, judged by phiclm.
fn counted(flags: &[bool]) -> (String, phiclm::Verdict) {
    let vec: Vec<String> = flags.iter().map(|f| if *f { "1".into() } else { "0".into() }).collect();
    let src = format!(
        "LET start() = VALOF $( LET v = VEC {n} AND s = 0\n{fill}\nFOR i = 0 TO {last} DO s := s + v!i\nRESULTIS s $)",
        n = flags.len(),
        last = flags.len().saturating_sub(1),
        fill = vec.iter().enumerate().map(|(i, f)| format!("v!{i} := {f}")).collect::<Vec<_>>().join("\n")
    );
    let v = phiclm::judge(&src, 200_000);
    (src, v)
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let path = a.first().cloned().unwrap_or_default();
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let pages: usize = flag("--pages").and_then(|s| s.parse().ok()).unwrap_or(64);
    let out = flag("--out").unwrap_or_else(|| "workshop.md".into());
    let rehearse = a.iter().any(|x| x == "--rehearse");

    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let book = units(&text);
    let key = flag("--key").map(|k| key::parse(&std::fs::read_to_string(k).unwrap_or_default())).unwrap_or_default();
    let mut contrast = Count::new();
    let (mut key_hit, mut key_n, mut key_hit_p, mut key_hit_c) = (0usize, 0usize, 0usize, 0usize);
    let per = (book.len() / pages.max(1)).max(1);
    let mut student = Learner::new(Config { block: 8, silence: 0.2, ..Config::default() });
    let mut phasors = Ecs::new();
    let mut knowing = Knowing::fresh();
    let mut doc = String::new();
    let _ = writeln!(doc, "# 193 · The workshop — {} units of the guide played as {} pages\n", book.len(), book.len().div_ceil(per));
    let _ = writeln!(doc, "In Phient's world, a word (w) is a seed: what it relates to, under what conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a word is 32–64 relations, each a complex phasor on a closed circle; meaning is a side effect of pairs of words moving under a condition, and they move by the error of what was expected — toward what came, away from what did not.\n");
    let _ = writeln!(doc, "The Key holds {} answers across {} units; it is never told — after each lesson the student (the learner) completes the unit's answers word by word, and is graded, beside the counted phasors and the counts as contrast lines.{}\n", key.len(), key.iter().map(|a| a.unit).collect::<std::collections::HashSet<_>>().len(), if rehearse { " After each page the learner rehearses everything told so far once." } else { " The learner is corrected on the fly only." });
    let _ = writeln!(doc, "Each page: what the student BROUGHT to the unit (next-word top-1 on its sentences before being told them), the unit told on the fly, the quest prompts it would send a teacher for the constructions it is unsure of, the unit's own exercises as workshop prompts, and one task of logic handed to the compiler and judged. Generated by `examples/workshop.rs`; nothing here is written by hand.\n");
    let (mut brought_all, mut n_all) = (0usize, 0usize);
    for (p, group) in book.chunks(per).enumerate() {
        let _ = writeln!(doc, "## Page {} — units {}", p + 1, group.iter().map(|u| format!("{} *{}*", u.number, u.title)).collect::<Vec<_>>().join("; "));
        for u in group {
            let sents = sentences(&u.lines);
            let (hit, n) = score(&mut student, &sents);
            brought_all += hit;
            n_all += n;
            for w in &sents {
                student.tell(w);
                phasors.tell(w);
                contrast.tell(w);
                let id = format!("unit{}/s{}", u.number, knowing.paragraphs.len());
                knowing.tell(&id, &paragraph(&id, &[w.join(" ")]));
            }
            // Graded against the Key: complete this unit's answers, word by
            // word, any alternative counting; the Key itself is never told.
            let (mut kh, mut kn, mut kp, mut kc) = (0usize, 0usize, 0usize, 0usize);
            for ans in key.iter().filter(|a| a.unit == u.number) {
                let first = &ans.alternative[0];
                for j in 1..first.len() {
                    kn += 1;
                    let g = student.guess(&first[..j]);
                    let p = phasors.guess(&first[..j]);
                    let c = contrast.guess(&first[..j]);
                    let ok = |g: &Option<String>| ans.alternative.iter().any(|alt| alt.get(j).is_some_and(|t| Some(t) == g.as_ref()));
                    kh += ok(&g) as usize;
                    kp += ok(&p) as usize;
                    kc += ok(&c) as usize;
                }
            }
            key_hit += kh;
            key_n += kn;
            key_hit_p += kp;
            key_hit_c += kc;
            // Unsure of WORDS, not exercise numbering; prompts are the
            // numbered instructions (`1.3 Write questions …`), not the items.
            let known = knowing.known_words(2);
            let mine: std::collections::HashSet<(String, String)> =
                sents.iter().flat_map(|w| Knowing::construction_with(w, &known)).collect();
            let unsure: Vec<(String, String)> = knowing
                .unsure(2)
                .into_iter()
                .filter(|p| mine.contains(p))
                .filter(|(a, b)| a.chars().all(char::is_alphabetic) && b.chars().all(char::is_alphabetic))
                .collect();
            let exercises: Vec<&String> = u
                .lines
                .iter()
                .filter(|l| {
                    let mut it = l.splitn(2, char::is_whitespace);
                    let head = it.next().unwrap_or("");
                    head.split_once('.').is_some_and(|(x, y)| x.parse::<usize>().is_ok() && y.parse::<usize>().is_ok()) && it.next().is_some_and(|rest| rest.len() > 12)
                })
                .take(4)
                .collect();
            let negatives: Vec<bool> = sents.iter().map(|w| w.iter().any(|t| t == "not" || t.ends_with("n't"))).collect();
            let (src, v) = counted(&negatives);
            let _ = writeln!(doc, "\n**Unit {} — {}** · {} sentences · brought {:.1}% of its next words ({hit}/{n}) before being told · **Key: {:.1}%** of its answers' words completed by the learner ({kh}/{kn}; counted phasors {:.1}%, counts {:.1}%).", u.number, u.title, sents.len(), 100.0 * hit as f64 / n.max(1) as f64, 100.0 * kh as f64 / kn.max(1) as f64, 100.0 * kp as f64 / kn.max(1) as f64, 100.0 * kc as f64 / kn.max(1) as f64);
            let _ = writeln!(doc, "\nQuest prompts (unsure of {} constructions; the first three):", unsure.len());
            for pair in unsure.iter().take(3) {
                let _ = writeln!(doc, "- `{}`", family_prompt(pair));
            }
            let _ = writeln!(doc, "\nWorkshop prompts (the unit's exercises):");
            for e in &exercises {
                let _ = writeln!(doc, "- {e}");
            }
            let _ = writeln!(doc, "\nCompiler task: count the negatives the student read among the {} sentences — phiclm ran {} O-code steps and returned **{}** ({}); the student's own count is {}.", sents.len(), v.steps, v.result, if v.compiled && v.error.is_none() { "judged" } else { "failed" }, negatives.iter().filter(|b| **b).count());
            let _ = writeln!(doc, "\n```text\n{}\n```", src.lines().take(2).chain(src.lines().skip(src.lines().count().saturating_sub(2))).collect::<Vec<_>>().join("\n… \n"));
        }
        if rehearse {
            student.epoch();
        }
    }
    let _ = writeln!(doc, "\n## The whole course\n\nOver {} units the student brought {:.1}% of each unit's next words before being told it ({brought_all}/{n_all}) — what it carried from every earlier lesson into the next, on the fly, with nothing retrained.\n\n**Graded against the Key** (never told): the learner completed **{:.1}%** of the answers' words ({key_hit}/{key_n}); the contrast lines, the counted phasors {:.1}% ({key_hit_p}/{key_n}) and the counts {:.1}% ({key_hit_c}/{key_n}). The floor is one word in the book's vocabulary.", book.len(), 100.0 * brought_all as f64 / n_all.max(1) as f64, 100.0 * key_hit as f64 / key_n.max(1) as f64, 100.0 * key_hit_p as f64 / key_n.max(1) as f64, 100.0 * key_hit_c as f64 / key_n.max(1) as f64);
    std::fs::write(&out, &doc).expect("write the pages");
    println!("{} units → {} pages → {out}", book.len(), book.len().div_ceil(per));
}
