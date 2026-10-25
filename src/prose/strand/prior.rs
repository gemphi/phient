//! The distribution the whole story is read against: what a word is
//! expected to be before this story said anything. Read from a directory of
//! plain text (phiano's `data/text/emma`), counted, and Laplace-smoothed so
//! that a word the prior never saw still has a share — the share that makes
//! a story's own vocabulary informative at the top of the strand.

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct Prior {
    pub counts: HashMap<String, u64>,
    pub total: u64,
}

impl Prior {
    /// Every `.txt` under `dir`, tokenised as the story is (`prose::tokens`).
    pub fn read(dir: &Path) -> Self {
        let mut p = Prior::default();
        let mut files = Vec::new();
        collect(dir, &mut files);
        files.sort();
        for f in files {
            if let Ok(text) = std::fs::read_to_string(&f) {
                p.add(&text);
            }
        }
        p
    }

    /// Count `text` into the prior.
    pub fn add(&mut self, text: &str) {
        for t in super::super::tokens(text) {
            *self.counts.entry(t).or_default() += 1;
            self.total += 1;
        }
    }

    /// `p(w) = (c(w) + 1) / (total + V + 1)` with `V` the prior's
    /// vocabulary; the `+1` in the denominator is the share of all words
    /// it never saw, each of which gets `1 / (total + V + 1)`.
    pub fn p(&self, w: &str) -> f64 {
        let c = self.counts.get(w).copied().unwrap_or(0) as f64;
        (c + 1.0) / (self.total as f64 + self.counts.len() as f64 + 1.0)
    }
}

fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().is_some_and(|x| x == "txt") {
            out.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unseen_words_keep_a_share() {
        let mut p = Prior::default();
        p.add("the cat sat on the mat.");
        assert!(p.p("the") > p.p("cat"));
        assert!(p.p("dog") > 0.0 && p.p("dog") < p.p("cat"));
    }
}
