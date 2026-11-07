//! The English words phient's rules name, read from files under `data/lex/`
//! (the ruling of 2026-11-07: no data in code). The code keeps the rule and
//! the file keeps which words the rule is about, so a run can append one.
//!
//! | file | what |
//! |---|---|
//! | `structural.txt` | words in so many definitions that binding them washes the manifold out |
//! | `relational_prepositions.txt` | the prepositions that mark a relation between a head and a noun |

use std::path::PathBuf;

/// Where the files live: the crate's own `data/lex/`, or `$PHIENT_LEX`.
pub fn lex_dir() -> PathBuf {
    match std::env::var_os("PHIENT_LEX") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/lex")),
    }
}

/// The words of `name`, read once from `data/lex/<name>.txt`.
pub fn words(name: &str) -> &'static [String] {
    phistd::lexfile::cached(lex_dir().join(format!("{name}.txt")))
}

/// Whether `word` is in the list `name`.
pub fn is(name: &str, word: &str) -> bool {
    words(name).iter().any(|w| w == word)
}
