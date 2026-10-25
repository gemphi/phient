//! BCPL read as rows (docs/166, docs/168): the 65-keyword language whose
//! open stratum is 2%, told the way prose is told — each function a
//! paragraph under its file's scope — plus the three relations the
//! language itself declares: `declares` (LET/AND bindings), `gets` (GET
//! headers) and `calls` (a name applied). Nothing here is a new
//! convention: the rows are [`crate::prose::paragraph`]'s, so the sets
//! folder, the Lisp and every counter run on BCPL unchanged.

pub mod learn;
pub mod role;

use crate::prose::{paragraph, tokens};
use phiecs::Row;
use std::path::Path;

/// A `.b` file cut into functions: `LET name(...)` or `AND name(...)` at
/// the start of a line opens one; everything to the next opener is its
/// body. Text before the first opener is the file's own `p0`.
pub fn functions(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut cur = ("p0".to_string(), String::new());
    for line in text.lines() {
        let t = line.trim_start();
        let opener = (line.starts_with("LET ") || line.starts_with("AND "))
            && t.split_whitespace().nth(1).is_some_and(|n| n.chars().next().is_some_and(char::is_alphabetic));
        if opener {
            if !cur.1.trim().is_empty() {
                out.push(cur);
            }
            let word = t.split_whitespace().nth(1).unwrap_or("fn");
            let name: String = word.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.').collect::<String>().to_lowercase();
            cur = (name, String::new());
        }
        cur.1.push_str(line);
        cur.1.push(' ');
    }
    if !cur.1.trim().is_empty() {
        out.push(cur);
    }
    out
}

/// What `name` declares, gets and calls in `body` — the function-level
/// relations alone.
fn relations(name: &str, body: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    let toks = tokens(body);
    for w in toks.windows(2) {
        match (w[0].as_str(), w[1].as_str()) {
            ("let" | "and" | "manifest" | "global" | "static", x) if x.chars().next().is_some_and(char::is_alphabetic) => {
                rows.push(Row::new(name, "declares", x));
            }
            (f, "(") if f.chars().next().is_some_and(char::is_alphabetic) && f != name => {
                rows.push(Row::new(name, "calls", f));
            }
            _ => {}
        }
    }
    for (i, t) in toks.iter().enumerate() {
        if t == "get" {
            if let Some(h) = toks.get(i + 2).filter(|h| h.chars().next().is_some_and(char::is_alphanumeric)) {
                rows.push(Row::new(name, "gets", h));
            }
        }
    }
    rows
}

/// One function told in full: the prose convention for its body, plus the
/// function-level relations.
pub fn tell(path: &str, name: &str, body: &str) -> Vec<Row> {
    let mut rows = paragraph(path, &[body.to_string()]);
    rows.extend(relations(name, body));
    rows
}

/// One function told at a persona's grain — the path's nesting and the
/// function-level relations, without the prose reader's occurrence rows:
/// ~50 rows a function instead of ~5,000.
pub fn tell_coarse(path: &str, name: &str, body: &str) -> Vec<Row> {
    let mut rows = crate::relate::nest(path, 0);
    rows.push(Row::new(name, "within", &format!("scope:{}", path.rsplit_once('/').map_or(path, |x| x.0))));
    rows.extend(relations(name, body));
    rows
}

/// Every function body under `root`, in reading order — the examples the
/// learner reads. One shared reader (docs/170); nothing duplicates `walk`.
pub fn read_bodies(root: &Path) -> Vec<String> {
    walk(root).into_iter().map(|(_, _, body)| body).collect()
}

/// Every function of every `.b` file under `root`: `(path, name, body)`,
/// paths `bcpl/<file stem>/<function>`, files in sorted order.
fn walk(root: &Path) -> Vec<(String, String, String)> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for p in rd.filter_map(|e| e.ok().map(|e| e.path())) {
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "b") {
                files.push(p);
            }
        }
    }
    files.sort();
    let mut out = Vec::new();
    for p in files {
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        let stem = p.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
        for (name, body) in functions(&text) {
            out.push((format!("bcpl/{stem}/{name}"), name, body));
        }
    }
    out
}

/// Every `.b` file under `root` at the corpus grain: the same `(path,
/// rows)` shape as [`crate::prose::read_tree`].
pub fn read_tree(root: &Path) -> Vec<(String, Vec<Row>)> {
    walk(root)
        .into_iter()
        .map(|(path, name, body)| {
            let rows = tell(&path, &name, &body);
            (path, rows)
        })
        .collect()
}

/// Every `.b` file under `root` at the persona grain: one flat table of
/// what every function declares, gets and calls.
pub fn read_coarse(root: &Path) -> Vec<Row> {
    walk(root).iter().flat_map(|(path, name, body)| tell_coarse(path, name, body)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "GET \"libhdr\"\n\nLET start() = VALOF\n{ LET sum = 0\n  FOR i = 1 TO 9 DO sum := sum + i\n  writef(\"%n*n\", sum)\n  RESULTIS 0\n}\n\nAND twice(x) = x + x\n";

    #[test]
    fn a_bcpl_file_is_functions_told_as_rows() {
        let fns = functions(SRC);
        assert_eq!(fns.iter().map(|f| f.0.as_str()).collect::<Vec<_>>(), ["p0", "start", "twice"]);
        let rows = tell("bcpl/t/start", "start", &fns[1].1);
        assert!(rows.iter().any(|r| r.about == "start" && r.kind == "declares" && r.other == "sum"));
        assert!(rows.iter().any(|r| r.about == "start" && r.kind == "calls" && r.other == "writef"));
        assert!(rows.iter().any(|r| r.kind == "within" && r.other == "scope:bcpl"), "the prose convention holds");
        // The coarse grain carries the same relations with no occurrences.
        let coarse = tell_coarse("bcpl/t/start", "start", &fns[1].1);
        assert!(coarse.len() < rows.len() / 2);
        assert!(coarse.iter().any(|r| r.about == "start" && r.kind == "calls" && r.other == "writef"));
        assert!(coarse.iter().all(|r| !r.about.contains('@')));
        assert!(coarse.iter().any(|r| r.about == "start" && r.kind == "within" && r.other == "scope:bcpl/t"));
        // Simple by simple: a world told this answers the simplest question.
        let w = phiecs::world::World::from_rows(&coarse);
        let (s, k) = (w.lexicon.get("start").unwrap(), w.relations.get("calls").unwrap());
        assert_eq!(w.spelling(w.calls_for(s, k)[0].0), "writef");
    }
}
