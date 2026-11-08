//! How Phiano learns what words mean, and answers from it.
//! `cargo run --release --example meaning_demo`
//!
//! A tiny world: people, animals, places, times, and a few gender pairs.
//! Trained on the recommended path (`train_corpus_ranking`): the corpus picks
//! the relation words, then every observed pair moves one relation channel
//! and deposits evidence there.
use phiecs::phasor::facet::Facet;
use phient::train::trainer::Trainer;

const WORLD: &[&str] = &[
    "the baby played with the dog on the lawn in the evening",
    "the baby played with the cat on the lawn in the summer",
    "the girl played with the dog on the grass in the evening",
    "the boy played with the ball on the lawn in the summer",
    "the dog ran on the lawn in the evening",
    "the cat slept on the sofa in the night",
    "the mother took a photo of the baby and the dog",
    "the father took a photo of the boy and the ball",
    "the king ruled the country from the palace",
    "the queen ruled the country from the palace",
    "the king is a man and the queen is a woman",
    "the prince is a boy and the princess is a girl",
    "the father is a man and the mother is a woman",
    "the man walked the dog in the park in the morning",
    "the woman walked the dog in the park in the evening",
    "the baby slept in the cot in the night",
    "the boy ran in the park in the summer",
    "the girl ran on the grass in the morning",
];

fn main() {
    let sentences: Vec<String> = WORLD.iter().map(|s| s.to_string()).collect();
    let mut facet = Facet::new();
    let trainer = Trainer::new(0.08).with_seed(7);
    for _ in 0..30 {
        trainer.train_corpus_ranking(&mut facet, &sentences);
    }

    println!("== 1. Register the corpus named (one channel each)");
    let names: Vec<String> = (8..8 + facet.relations.markers()).map(|k| facet.relation_name(k)).collect();
    println!("   offset+1..offset+8, then markers: {}", names.join(" "));

    println!("\n== 2. What a word is: its strongest relations");
    for w in ["baby", "dog", "lawn", "evening", "queen"] {
        let p = facet.get_phasor(w).unwrap();
        let mut ev: Vec<(usize, u32)> = (0..64).map(|k| (k, p.evidence_count(k))).filter(|x| x.1 > 0 && x.0 >= 8).collect();
        ev.sort_by(|a, b| b.1.cmp(&a.1));
        let top: Vec<String> = ev.iter().take(4).map(|(k, c)| format!("{}:{}", facet.relation_name(*k), c)).collect();
        println!("   {:<8} {}", w, top.join("  "));
    }

    let ask = |q: &str, blank: usize, pool: &[&str]| {
        let toks: Vec<String> = q.split_whitespace().map(|s| s.to_string()).collect();
        let mut scored: Vec<(&str, f64, usize)> = pool
            .iter()
            .map(|c| {
                let (s, n) = facet.relational_fit(&toks, blank, c);
                (*c, s, n)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let row: Vec<String> = scored.iter().take(4).map(|(w, s, n)| format!("{w} {s:+.2}({n})")).collect();
        println!("   {:<46} {}", q, row.join("  "));
    };
    let pool = ["baby", "dog", "lawn", "evening", "park", "photo", "queen", "sofa", "summer", "ball"];

    println!("\n== 3. Questions as a blank the relations must fill (score, evidenced pairs)");
    ask("the ? played with the dog", 1, &pool);
    ask("the baby played with the ?", 5, &pool);
    ask("the baby played on the ?", 5, &pool);
    ask("the baby played in the ?", 5, &pool);
    ask("the mother took a ? of the baby", 4, &pool);

    println!("\n== 4. Meaning as shared relations (evidence-weighted similarity)");
    for (a, b) in [("king", "queen"), ("man", "woman"), ("dog", "cat"), ("lawn", "grass"), ("king", "lawn"), ("dog", "evening")] {
        println!("   {:<6} ~ {:<6} {:+.3}", a, b, facet.similarity(a, b));
    }

    println!("\n== 5. Analogy: king:queen :: man:?  (offset difference, per channel)");
    let (k, q, m) = (facet.get_phasor("king").unwrap(), facet.get_phasor("queen").unwrap(), facet.get_phasor("man").unwrap());
    let mut best: Vec<(&String, f64)> = facet
        .lexicon
        .iter()
        .filter(|(w, _)| !["king", "queen", "man"].contains(&w.as_str()))
        .map(|(w, p)| {
            let s: f64 = (0..64)
                .map(|c| (m.theta(c) + q.theta(c) - k.theta(c) - p.theta(c)).cos())
                .sum::<f64>() / 64.0;
            (w, s)
        })
        .collect();
    best.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let top: Vec<String> = best.iter().take(4).map(|(w, s)| format!("{w} {s:+.2}")).collect();
    println!("   {}", top.join("  "));
}
