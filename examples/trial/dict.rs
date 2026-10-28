//! Webster's dictionary as lines to be told: a word, then the first
//! sentence of its definition, so the word stands next to what it means.

use phient::prose::tokens;
use std::collections::HashSet;

/// The definitions of the words in `wanted`, read from the dictionary's
/// one-entry-per-line JSON (`    "word": "definition",`).
pub fn definitions(path: &str, wanted: &HashSet<String>) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let l = l.trim().strip_prefix('"')?;
            let (word, rest) = l.split_once("\": \"")?;
            let def = rest.trim_end_matches(',').trim_end_matches('"').replace("\\\"", "\"");
            let word = word.to_lowercase();
            wanted.contains(&word).then(|| {
                let first = def.split(". ").next().unwrap_or(&def);
                let mut line = tokens(&word);
                line.extend(tokens(first));
                line
            })
        })
        .filter(|w| w.len() >= 3)
        .collect()
}
