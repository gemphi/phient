//! Ask a relation model (`.relate`, from `kind_operator` or
//! `persona_from_quest`) what a word calls for under a kind.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c). `word · kind · ?` is read by the model's operator for that
//! kind, ranked over every word the model was told as an answer.
//!
//! `cargo run --release --bin relate_ask -- <model.relate> <word> <kind> [n]`
//! `cargo run --release --bin relate_ask -- <model.relate> <word>` asks every kind.

use phient::relate::{kind_token, RelationModel};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (Some(path), Some(word)) = (a.first(), a.get(1)) else {
        eprintln!("usage: relate_ask <model.relate> <word> [kind] [n]");
        std::process::exit(2);
    };
    let mut m = RelationModel::load(path).unwrap_or_else(|e| panic!("{e}"));
    let n: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
    let kind_rows: Vec<usize> = m.kinds.iter().filter_map(|k| m.index.get(&kind_token(k)).copied()).collect();
    let pool: Vec<usize> = (0..m.names.len()).filter(|i| !kind_rows.contains(i)).collect();
    if !m.index.contains_key(word) {
        println!("{word}: not in this model ({} words)", pool.len());
        return;
    }
    let kinds: Vec<String> = match a.get(2) {
        Some(k) => vec![k.clone()],
        None => m.kinds.clone(),
    };
    for k in kinds {
        let got: Vec<String> = m.answers(word, &k, &pool, n).into_iter().map(|(w, s)| format!("{w} ({s:.2})")).collect();
        println!("{word} · {k} ⇒ {}", got.join(", "));
    }
}
