//! A story written into language sets (docs/164): every scope on its path
//! — story, chapter, paragraph — gets `says.tsv`, `next.tsv` and
//! `trace.txt` under `<out>/<path>/`, parents the union of their children.
//!
//! ```text
//! cargo run --release -p phient --example sets -- <text root> <out dir> [--show scope:text/emma]
//! ```

use phient::relate::trace;
use std::path::PathBuf;

fn main() {
    let mut a: Vec<String> = std::env::args().skip(1).collect();
    let show = phient::study::lab::flag(&mut a, "--show");
    let (Some(root), Some(out)) = (a.first().map(PathBuf::from), a.get(1).map(PathBuf::from)) else {
        eprintln!("usage: sets <text root> <out dir> [--show scope:path]");
        std::process::exit(2);
    };
    let tree = phient::prose::read_tree(&root);
    let scopes = trace::scopes_of(&tree);
    let n = trace::write(&out, &scopes).unwrap_or_else(|e| panic!("cannot write {}: {e}", out.display()));
    let top = scopes.keys().filter(|k| k.matches('/').count() == 1).cloned().collect::<Vec<_>>();
    println!("{} paragraphs → {n} scopes → {}\n", tree.len(), out.display());
    for t in &top {
        print!("{}\n", trace::summary(&scopes, t));
    }
    if let Some(s) = show {
        print!("{}", trace::summary(&scopes, &s));
    }
}
