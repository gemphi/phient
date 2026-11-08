//! Paths and traces (docs/164): a story written into language sets. Every
//! scope on a story's path — the story, its chapters, their paragraphs —
//! holds a **vocabulary** (the set of words it says, with counts), a
//! **next** relation (the set of `(word, next word)` pairs within it) and a
//! **trace** (the walk through its words in order). A parent's sets are
//! the union of its children's, counts summed, so the folder is the tree
//! and the set algebra of [`phiclm::lisp::set`] runs on any two scopes in it.

use super::scopes;
use phiecs::Row;
use std::collections::BTreeMap;
use std::path::Path;

/// One scope's language sets.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scope {
    /// `scope:text/emma/ch01/p3`.
    pub path: String,
    /// The words it says, counted.
    pub says: BTreeMap<String, u32>,
    /// `(word, next word)` within it, counted.
    pub next: BTreeMap<(String, String), u32>,
    /// The walk: its words in order (a chapter's is its paragraphs' in order).
    pub trace: Vec<String>,
}

impl Scope {
    fn absorb(&mut self, toks: &[String]) {
        for t in toks {
            *self.says.entry(t.clone()).or_insert(0) += 1;
        }
        for w in toks.windows(2) {
            *self.next.entry((w[0].clone(), w[1].clone())).or_insert(0) += 1;
        }
        self.trace.extend(toks.iter().cloned());
    }

    /// The next relation as a set of pairs, for [`phiclm::lisp::set`].
    pub fn pairs(&self) -> phiclm::lisp::set::Relation {
        self.next.keys().cloned().collect()
    }
}

/// The words of one paragraph, in order, from its rows: the `has-word`
/// rows come one per occurrence, in reading order.
pub fn words_of(rows: &[Row]) -> Vec<String> {
    rows.iter().filter(|r| r.kind == "has-word").map(|r| r.other.clone()).collect()
}

/// Every scope of a tree of paragraphs, ancestors included, keyed by path.
pub fn scopes_of(tree: &[(String, Vec<Row>)]) -> BTreeMap<String, Scope> {
    let mut out: BTreeMap<String, Scope> = BTreeMap::new();
    for (path, rows) in tree {
        let toks = words_of(rows);
        for s in scopes(path, 0) {
            let sc = out.entry(s.clone()).or_default();
            sc.path = s;
            sc.absorb(&toks);
        }
    }
    out
}

/// Write the folder: `<dir>/<path>/{says.tsv, next.tsv, trace.txt}` for
/// every scope. Returns how many scopes were written.
pub fn write(dir: &Path, scopes: &BTreeMap<String, Scope>) -> std::io::Result<usize> {
    for (path, sc) in scopes {
        let d = dir.join(path.trim_start_matches("scope:"));
        std::fs::create_dir_all(&d)?;
        let mut says: Vec<(&String, &u32)> = sc.says.iter().collect();
        says.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        std::fs::write(d.join("says.tsv"), says.iter().map(|(w, c)| format!("{w}\t{c}\n")).collect::<String>())?;
        let mut next: Vec<(&(String, String), &u32)> = sc.next.iter().collect();
        next.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        std::fs::write(d.join("next.tsv"), next.iter().map(|((a, b), c)| format!("{a}\t{b}\t{c}\n")).collect::<String>())?;
        std::fs::write(d.join("trace.txt"), sc.trace.join(" ") + "\n")?;
    }
    Ok(scopes.len())
}

/// The children of `path` as a table: words, pairs, and what each child
/// shares with the one before it (meet) and holds with it (union).
pub fn summary(scopes: &BTreeMap<String, Scope>, path: &str) -> String {
    let prefix = format!("{path}/");
    let mut children: Vec<&Scope> = scopes.values().filter(|s| s.path.starts_with(&prefix) && !s.path[prefix.len()..].contains('/')).collect();
    // p2 before p10: the number at the end of a name sorts as a number.
    let key = |s: &Scope| {
        let name = &s.path[prefix.len()..];
        let stem = name.trim_end_matches(|c: char| c.is_ascii_digit());
        (stem.to_string(), name[stem.len()..].parse::<u64>().unwrap_or(0), name.to_string())
    };
    children.sort_by_key(|s| key(s));
    let mut out = format!("## {path}\n\n| scope | words said | distinct | next pairs | meet with previous | union with previous |\n|---|---|---|---|---|---|\n");
    let mut prev: Option<phiclm::lisp::set::Relation> = None;
    for c in children {
        let p = c.pairs();
        let (meet, union) = match &prev {
            Some(q) => (phiclm::lisp::set::meet(&p, q).len().to_string(), phiclm::lisp::set::union(&p, q).len().to_string()),
            None => ("–".into(), "–".into()),
        };
        out.push_str(&format!("| {} | {} | {} | {} | {meet} | {union} |\n", &c.path[prefix.len()..], c.trace.len(), c.says.len(), c.next.len()));
        prev = Some(p);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_story_is_its_sets_at_every_scope() {
        let tree = vec![
            ("text/tale/ch1/p1".to_string(), crate::prose::paragraph("text/tale/ch1/p1", &["the cat sat".to_string()])),
            ("text/tale/ch1/p2".to_string(), crate::prose::paragraph("text/tale/ch1/p2", &["the cat ran".to_string()])),
        ];
        let sc = scopes_of(&tree);
        assert_eq!(sc.len(), 5, "text, tale, ch1, p1, p2");
        let ch1 = &sc["scope:text/tale/ch1"];
        assert_eq!(ch1.says["the"], 2);
        assert_eq!(ch1.next[&("the".into(), "cat".into())], 2);
        assert_eq!(ch1.trace.len(), 6);
        let (p1, p2) = (sc["scope:text/tale/ch1/p1"].pairs(), sc["scope:text/tale/ch1/p2"].pairs());
        assert_eq!(phiclm::lisp::set::meet(&p1, &p2).len(), 1, "they share `the cat`");
        let dir = std::env::temp_dir().join(format!("phinet-sets-{}", std::process::id()));
        assert_eq!(write(&dir, &sc).unwrap(), 5);
        assert!(dir.join("text/tale/ch1/p2/trace.txt").exists());
        assert!(summary(&sc, "scope:text/tale/ch1").contains("| p2 | 3 | 3 | 2 | 1 | 3 |"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
