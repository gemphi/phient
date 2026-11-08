//! The body as a tree of words (phiecs::kit, docs/140–141): the face's shapes,
//! the pose's and the skeleton's joints, the hand's, tracking's anchors and
//! sessions, the device's registry — every type and case a thing whose
//! words are the path it lies on. Nothing is read from a file: the kit is
//! code, named in our words, and the tree is what the code says of itself.
//!
//! `cargo run --release -p phient --example anatomy -- [--ask "left elbow"] [--rows out.tsv] [--show N]`

use phiecs::kit::Kit;
use phient::relate::{kit::said_by, Casting};
use phient::relate::{Tally, ScopeTree};

use phient::study::lab::flag;

fn print_tree(tree: &ScopeTree, counts: &Tally, scope: &str, depth: usize, max_depth: usize, width: usize) {
    let children = tree.children(scope);
    let words = said_by(counts, scope);
    let name = scope.rsplit('/').next().unwrap_or(scope);
    println!("{}{name}  ({} below, {} words)", "  ".repeat(depth), children.len(), words.len());
    if depth < max_depth {
        for c in children.iter().take(width) {
            print_tree(tree, counts, c, depth + 1, max_depth, width);
        }
        if children.len() > width {
            println!("{}… {} more", "  ".repeat(depth + 1), children.len() - width);
        }
    }
}

fn main() {
    let mut a: Vec<String> = std::env::args().skip(1).collect();
    let ask = flag(&mut a, "--ask");
    let rows_out = flag(&mut a, "--rows");
    let show: usize = flag(&mut a, "--show").and_then(|s| s.parse().ok()).unwrap_or(6);
    let kit = Kit::new();
    println!("the kit: {} rows", kit.rows.len());
    if let Some(out) = rows_out {
        let tsv: String = kit.rows.iter().map(|r| format!("{}\t{}\t{}\t{:.0}\n", r.about, r.kind, r.other, r.weight)).collect();
        std::fs::write(&out, tsv).expect("rows");
        println!("rows written to {out}");
    }
    let (counts, tree) = kit.knowing();
    println!("\nwhat the kit names most:");
    for (k, n) in kit.kinds().iter().take(16) {
        let members = kit.of_kind(k);
        let sample: Vec<String> = members.iter().take(4).map(|m| kit.words(m).unwrap_or_else(|| m.clone())).collect();
        println!("  {k}: {n} — {}{}", sample.join(", "), if members.len() > 4 { ", …" } else { "" });
    }
    println!("\nthe tree (domain → module → kind → thing):");
    for r in tree.roots() {
        print_tree(&tree, &counts, r, 0, 2, show);
    }
    if let Some(q) = ask {
        let conds: Vec<String> = phient::prose::tokens(&q);
        let found = tree.cast(&counts, &conds, f64::MIN, 3);
        println!("\n\"{q}\" is drawn into:");
        for c in found.iter().take(8) {
            let leaf = c.path.last().map(String::as_str).unwrap_or("");
            println!("  {leaf}  (said {}, own {}, attraction {:.2})  {:?}", c.said, c.own, c.attraction, kit.under(leaf));
        }
    }
}
