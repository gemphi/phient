//! Teaching for hours (docs/132): the teacher goes on with the story, the
//! student's own going-on is held against it first, and the teacher's
//! paragraph is told. Every cycle leaves a paragraph in the tree and one
//! line in the log; nothing is retrained and a stopped run resumes from
//! the files.
//!
//! The measure is D20, agreement in judgments: the student goes on from
//! the last words of the story before the teacher does, and the two
//! continuations are compared — how many of the teacher's next words the
//! student said, and how far the two agree from the start.

use super::{tokens, Knowing};
use std::path::Path;

/// The question for the next paragraph: the story so far, nothing else.
pub fn next_prompt(so_far: &str) -> String {
    format!("Here is a story so far:\n\n{so_far}\n\nWrite the next paragraph of this story, three to five sentences, going on from exactly where it stops. Reply with the paragraph only.")
}

/// The chapter file the next paragraph goes to: `ch<N>.txt` under
/// `story/`, a new chapter every `per_chapter` paragraphs.
pub fn chapter_file(story_dir: &Path, per_chapter: usize) -> std::path::PathBuf {
    let mut chapters: Vec<_> = std::fs::read_dir(story_dir).map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "txt")).collect()).unwrap_or_default();
    chapters.sort();
    if let Some(last) = chapters.last() {
        let paragraphs = std::fs::read_to_string(last).map_or(0, |t| t.split("\n\n").filter(|p| !p.trim().is_empty()).count());
        if paragraphs < per_chapter {
            return last.clone();
        }
    }
    story_dir.join(format!("ch{}.txt", chapters.len() + 1))
}

/// The whole story so far, chapters in order, paragraphs joined.
pub fn so_far(story_dir: &Path) -> String {
    let mut chapters: Vec<_> = std::fs::read_dir(story_dir).map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "txt")).collect()).unwrap_or_default();
    chapters.sort();
    chapters.iter().filter_map(|p| std::fs::read_to_string(p).ok()).map(|t| t.trim().to_string()).collect::<Vec<_>>().join("\n\n")
}

/// Agreement between the student's going-on and the teacher's paragraph:
/// `(share of the teacher's distinct words the student said, words agreed
/// from the start)`.
pub fn agreement(student: &str, teacher: &str) -> (f64, usize) {
    let (s, t) = (tokens(student), tokens(teacher));
    let said: std::collections::HashSet<&String> = s.iter().collect();
    let distinct: std::collections::HashSet<&String> = t.iter().filter(|w| w.chars().any(char::is_alphanumeric)).collect();
    let overlap = distinct.iter().filter(|w| said.contains(*w)).count() as f64 / distinct.len().max(1) as f64;
    let from_start = s.iter().zip(&t).take_while(|(a, b)| a == b).count();
    (overlap, from_start)
}

/// One cycle: the student goes on from the story's last words, the
/// teacher writes the next paragraph, they are compared, the paragraph is
/// told (appended to the chapter file). Returns the log line.
pub fn cycle(k: &Knowing, story_dir: &Path, ask: &mut dyn FnMut(&str) -> String, per_chapter: usize) -> Option<String> {
    let story = so_far(story_dir);
    let tail: Vec<String> = tokens(&story).into_iter().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect();
    let from = tail.join(" ");
    let own = k.go(&from, 60, true);
    let own = own.lines().next().unwrap_or("").to_string();
    let reply = ask(&next_prompt(&story));
    let paragraph = reply.split("\n\n").map(str::trim).find(|p| p.split_whitespace().count() >= 8)?.to_string();
    let (overlap, from_start) = agreement(&own[from.len().min(own.len())..], &paragraph);
    let file = chapter_file(story_dir, per_chapter);
    let mut text = std::fs::read_to_string(&file).unwrap_or_default();
    if !text.trim().is_empty() {
        text = text.trim_end().to_string() + "\n\n";
    }
    text += &paragraph;
    text.push('\n');
    std::fs::write(&file, text).ok()?;
    Some(format!("{}\t{}\t{:.0}%\t{}\t{}", file.file_name().map_or(String::new(), |f| f.to_string_lossy().into_owned()), paragraph.split_whitespace().count(), 100.0 * overlap, from_start, own.replace('\t', " ")))
}
