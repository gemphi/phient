//! The files: every `.rs` under a directory read into one `Reader`, or
//! each on its own with its place in the tree told as `within` rows.

use super::extract::{row, Reader};
use phiecs::quest::Row;
use syn::visit::Visit;

/// Every `.rs` file under `dir` (target folders skipped), read into rows.
pub fn read_dir(dir: &std::path::Path, r: &mut Reader, files: &mut usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() && p.file_name().is_some_and(|n| n != "target" && n != ".git") {
            read_dir(&p, r, files);
        } else if p.extension().is_some_and(|x| x == "rs") {
            if let Some(file) = std::fs::read_to_string(&p).ok().and_then(|t| syn::parse_file(&t).ok()) {
                r.visit_file(&file);
                *files += 1;
            }
        }
    }
}

/// Every `.rs` file under `dir` read on its own: `(path, rows)`. Each
/// function is told `within` its file and the file `within` its directory,
/// up to `dir`: the path is the condition hierarchy (docs/126), each
/// component a scope word (`crate::relate::scopes`).
pub fn read_files(dir: &std::path::Path, out: &mut Vec<(String, Vec<Row>)>) {
    read_files_from(dir, dir, out)
}

fn read_files_from(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, Vec<Row>)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() && p.file_name().is_some_and(|n| n != "target" && n != ".git") {
            read_files_from(root, &p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            if let Some(file) = std::fs::read_to_string(&p).ok().and_then(|t| syn::parse_file(&t).ok()) {
                let mut r = Reader::default();
                r.visit_file(&file);
                let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().into_owned();
                let scopes = crate::relate::scopes(&rel, 0);
                // A function is within every scope on its path (the tree's
                // cast reads a scope as what its subtree says).
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
        }
    }
}
