//! `english` - an interactive English teacher built on the Murphy curriculum.
//!
//! Loads all curriculum units into a `Knowing` tree, then loops:
//!
//!   - A plain line: auto-detect tense, predict next word, then learn it.
//!   - `? <partial>`: predict next word (do not learn).
//!   - `! <sentence>`: teach this sentence (do not predict).
//!   - `/<concept> <...>`: force concept e.g. `/present-simple she works`
//!   - `quit` or empty EOF: exit.
//!
//! ```text
//! cargo run --release -p phient --example english
//! cargo run --release -p phient --example english -- grammar/present-continuous
//! ```

use phient::prose::{paragraph, tokens, Knowing};
use phient::relate::scopes;
use std::io::{self, BufRead, Write};

const UNITS: &[(&str, &[&str])] = &[
    ("grammar/present-continuous", &[
        "she is driving to work", "i am trying to work", "he is having a shower",
        "it is not raining any more", "are you enjoying it",
        "they are building their own house", "the population is increasing very fast",
        "the water is boiling can you turn it off", "my brother is looking for a job",
        "we are staying at a very nice hotel", "she is not working this week",
        "i am getting hungry", "they are not watching the film",
    ]),
    ("grammar/present-simple", &[
        "nurses look after patients", "the earth goes round the sun",
        "the cafe opens at seven thirty", "rice does not grow in cold climates",
        "i get up at eight every morning", "julie does not drink tea very often",
        "water boils at a hundred degrees", "banks lend money to people",
        "i do not drink coffee at night", "the museum closes at five",
        "she speaks four languages", "we do not watch television very often",
    ]),
    ("grammar/past-simple", &[
        "i did a lot of work yesterday", "we went to the cinema last night",
        "she worked in a bank for five years", "it did not rain last week",
        "we did not go out last night", "they arrived at the station an hour ago",
        "i saw her in town yesterday", "he did not play very well",
    ]),
    ("grammar/present-perfect", &[
        "i have done a lot of work today", "she has gone to the bank",
        "we have known each other for years", "it has not rained for a month",
        "they have lived here for ten years", "i have lost my keys",
        "she has not been to the cinema for ages", "we have just finished dinner",
    ]),
    ("grammar/past-continuous", &[
        "i was watching television when she arrived",
        "it was raining when i got up", "they were not listening to me",
        "what were you doing at ten o clock last night",
        "she was working in the garden all afternoon",
    ]),
    ("grammar/present-perfect-continuous", &[
        "i have been waiting for you for an hour",
        "she has been learning english for three years",
        "how long have you been living here",
        "they have been playing tennis all afternoon", "it has been raining all day",
    ]),
    ("grammar/will-future", &[
        "i will call you tomorrow", "do you think she will pass the exam",
        "it will probably rain later", "they will not be at home this evening",
        "i think he will be a good teacher",
    ]),
    ("grammar/modal-can", &[
        "can you speak any foreign languages", "i can come and see you tomorrow",
        "she cannot swim very well", "can i borrow your pen",
        "you can use my car if you want",
    ]),
    ("grammar/modal-must", &[
        "i must go to the bank today", "you must not tell anyone what i said",
        "she must be very tired", "we must leave now or we will be late",
        "you must try the food it is very good",
    ]),
    ("grammar/if-conditional", &[
        "if it rains we will stay at home", "if i had more money i would buy a car",
        "if she had studied harder she would have passed",
        "if you heat water it boils", "if i were you i would accept the offer",
    ]),
];

fn build_knowledge() -> Knowing {
    let mut k = Knowing::fresh();
    let mut i = 0usize;
    for (concept, exs) in UNITS {
        for ex in *exs {
            let path = format!("{concept}/e{i}");
            k.tell(&path, &paragraph(&path, &[ex.to_string()]));
            i += 1;
        }
    }
    k
}

fn best_concept(k: &mut Knowing, words: &[String]) -> String {
    let concepts: Vec<&str> = UNITS.iter().map(|(c, _)| *c).collect();
    let mut best = (0.0f64, concepts[0]);
    for concept in &concepts {
        let old = k.within.clone();
        k.within = scopes(concept, 0);
        let (_, ranked, _) = k.next(words);
        k.within = old;
        let score = ranked.first().map(|(_, s)| *s).unwrap_or(0.0);
        if score > best.0 {
            best = (score, concept);
        }
    }
    best.1.to_string()
}

fn complete(k: &mut Knowing, concept: &str, words: &[String]) -> Vec<String> {
    k.within = scopes(concept, 0);
    let (_, ranked, _) = k.next(words);
    ranked.into_iter().take(5).map(|(w, _)| w).collect()
}

fn main() {
    let mut k = build_knowledge();
    let forced: Option<String> = std::env::args().nth(1);
    let total: usize = UNITS.iter().map(|(_, e)| e.len()).sum();
    println!("English teacher - {} sentences across {} units loaded.", total, UNITS.len());
    println!("  <sentence>        auto-detect, predict, learn");
    println!("  ? <partial>       predict next word only");
    println!("  ! <sentence>      teach without predicting");
    println!("  /<concept> <...>  force concept (e.g. /past-simple she went)");
    println!("  quit              exit\n");

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 { break; }
        let line = line.trim();
        if line.is_empty() || line == "quit" || line == "exit" { break; }

        if let Some(rest) = line.strip_prefix('/') {
            let (concept_part, text) = rest.split_once(' ').unwrap_or((rest, ""));
            let concept = format!("grammar/{concept_part}");
            let words = tokens(text);
            if words.is_empty() { println!("  (give words after the concept)"); continue; }
            let c = complete(&mut k, &concept, &words);
            if c.is_empty() { println!("  -> (nothing yet - teach more sentences)"); }
            else { println!("  -> {}  [{}]", c.join(" / "), concept_part); }
            continue;
        }

        if let Some(text) = line.strip_prefix("? ") {
            let words = tokens(text);
            if words.is_empty() { continue; }
            let concept = forced.clone().unwrap_or_else(|| best_concept(&mut k, &words));
            let c = complete(&mut k, &concept, &words);
            let short = concept.trim_start_matches("grammar/");
            if c.is_empty() { println!("  -> (nothing yet)  [{}]", short); }
            else { println!("  -> {}  [{}]", c.join(" / "), short); }
            continue;
        }

        if let Some(text) = line.strip_prefix("! ") {
            let words = tokens(text);
            let concept = forced.clone().unwrap_or_else(|| best_concept(&mut k, &words));
            let short = concept.trim_start_matches("grammar/");
            let id = format!("{concept}/u{}", phistd::mix64(phistd::fnv1a(text)) & 0xFFFF);
            k.tell(&id, &paragraph(&id, &[words.join(" ")]));
            println!("  learned under {} ({} words)", short, words.len());
            continue;
        }

        let words = tokens(line);
        if words.is_empty() { continue; }
        let concept = forced.clone().unwrap_or_else(|| best_concept(&mut k, &words));
        let short = concept.trim_start_matches("grammar/");
        let candidates = if words.len() > 1 {
            complete(&mut k, &concept, &words[..words.len()-1])
        } else {
            complete(&mut k, &concept, &words)
        };
        if !candidates.is_empty() {
            let got = candidates.first().map(|s| s.as_str()).unwrap_or("-");
            let last = words.last().map(|s| s.as_str()).unwrap_or("-");
            let mark = if got == last { "correct" } else { "~" };
            println!("  {} predicted: {}  actual: {}  [{}]",
                mark, candidates.join(" / "), last, short);
        }
        let id = format!("{concept}/u{}", phistd::mix64(phistd::fnv1a(line)) & 0xFFFF);
        k.tell(&id, &paragraph(&id, &[words.join(" ")]));
        println!("  learned ({} rows in tree)", k.rows);
    }
    println!("Bye.");
}