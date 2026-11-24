//! The files: every `.rs` under a directory read into one `Reader`, or
//! each on its own with its place in the tree told as `within` rows.

use super::extract::{row, Reader};
use phiecs::quest::Row;
use syn::visit::Visit;

/// Every `.rs` file under `dir` read into one `Reader`.
pub fn read_dir(dir: &std::path::Path, r: &mut Reader, files: &mut usize) {
    for p in phistd::tree::under(dir, "rs") {
        let Some(file) = parse(&p) else { continue };
        r.visit_file(&file);
        *files += 1;
    }
}

/// One file as syn reads it, or `None` when it cannot be read or parsed.
fn parse(p: &std::path::Path) -> Option<syn::File> {
    std::fs::read_to_string(p).ok().and_then(|t| syn::parse_file(&t).ok())
}

/// Every `.rs` file under `dir` read on its own: `(path, rows)`. Each
/// function is told `within` its file and the file `within` its directory,
/// up to `dir`: the path is the condition hierarchy (docs/126), each
/// component a scope word (`crate::relate::scopes`).
pub fn read_files(dir: &std::path::Path, out: &mut Vec<(String, Vec<Row>)>) {
    for p in phistd::tree::under(dir, "rs") {
        one_file(dir, &p, out);
    }
}

/// One `.rs` file's rows, with its place in the tree told as `within`.
fn one_file(root: &std::path::Path, p: &std::path::Path, out: &mut Vec<(String, Vec<Row>)>) {
    let Some(file) = parse(p) else { return };
    let mut r = Reader::default();
    r.visit_file(&file);
    let rel = p.strip_prefix(root).unwrap_or(p).to_string_lossy().into_owned();
    let scopes = crate::relate::scopes(&rel, 0);
    // A function is within every scope on its path (the tree's cast reads a
    // scope as what its subtree says).
    for item in &file.items {
        if let syn::Item::Fn(f) = item {
            for s in &scopes {
                r.rows.push(row(&f.sig.ident.to_string(), "within", s));
            }
        }
    }
    for w in scopes.windows(2) {
        r.rows.push(row(&w[1], "within", &w[0]));
    }
    out.push((p.to_string_lossy().into_owned(), r.rows));
}
