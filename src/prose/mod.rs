//! Prose as a tree of knowing (docs/127). A story on disk is a tree —
//! `story/ch1/p1/s1.txt` — and every word in it is a seed told under its
//! path: what word comes next, which words it was said with, and which
//! scopes it is within. Nothing is parsed but the tree and the order of
//! words; the rows are the quest's rows and the reading is `relate`'s.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it.

pub mod heldout;
mod curious;
mod family;
mod going;
mod knowing;
mod telling;
mod under;
pub mod strand;
pub mod teach;
pub mod unknown;
pub use knowing::{Base, Knowing};
pub use family::family_prompt;
pub use under::family_path;
pub use unknown::{uses_prompt, KNOWN};

use phiecs::quest::Row;

// `tokens`, `paragraph` and `END` moved down to `phiecs::speak` (the kit's
// face speaks frames with the same mouth a story speaks sentences); the
// address here still answers.
pub use phiecs::speak::{paragraph, tokens, END};

/// Every `.txt` under `root`: a directory is a scope, a file is a scope
/// (its stem: a chapter), and each blank-line paragraph in it is a scope
/// `p1`, `p2`, … holding its words. Returns `(paragraph path relative to
/// root's parent, rows)`. A directory of one-sentence files (docs/127's
/// first form) reads the same way, each file one paragraph.
pub fn read_tree(root: &std::path::Path) -> Vec<(String, Vec<Row>)> {
    let mut out = Vec::new();
    for_each_paragraph(root, &mut |path, rows| out.push((path, rows)));
    out
}

/// As `read_tree`, one paragraph at a time, so a tree of any size is read
/// without all its rows in memory at once.
pub fn for_each_paragraph(root: &std::path::Path, f: &mut dyn FnMut(String, Vec<Row>)) {
    let base = root.parent().unwrap_or(root);
    walk(base, root, f);
}

fn walk(base: &std::path::Path, dir: &std::path::Path, out: &mut dyn FnMut(String, Vec<Row>)) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for p in &paths {
        if p.is_dir() {
            walk(base, p, out);
        } else if p.extension().is_some_and(|x| x == "txt") {
            let Ok(text) = std::fs::read_to_string(p) else { continue };
            let file = p.with_extension("");
            let rel = file.strip_prefix(base).unwrap_or(&file).to_string_lossy().replace('\\', "/");
            let paragraphs: Vec<String> = text.split("\n\n").map(|x| x.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|x| !x.is_empty()).collect();
            for (i, para) in paragraphs.iter().enumerate() {
                let path = format!("{rel}/p{}", i + 1);
                out(path.clone(), paragraph(&path, std::slice::from_ref(para)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_new_is_told_without_reading_the_told_again() {
        let dir = std::env::temp_dir().join(format!("phiano_learn_new_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("story")).unwrap();
        std::fs::write(dir.join("story/ch1.txt"), "He remained presumptuous even now.\n").unwrap();
        let (mut k, told) = Knowing::learn(&dir);
        assert_eq!(told.len(), 1);
        let rows = k.rows;
        assert_eq!(k.learn_new(&dir), 0, "nothing new, nothing read");
        assert_eq!(k.rows, rows);
        std::fs::write(dir.join("story/ch1.txt"), "He remained presumptuous even now.\n\nShe came to know it.\n").unwrap();
        assert_eq!(k.learn_new(&dir), 1);
        assert!(k.rows > rows);
        assert_eq!(k.paragraphs.len(), 2);
        assert!(k.paragraphs.iter().any(|p| p.0.ends_with("/p2")));
        assert!(k.counts.count(None, "next", "came") >= 1.0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
