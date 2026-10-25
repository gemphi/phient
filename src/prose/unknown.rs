//! What the tree does not know yet, and asking for it (docs/130). A word
//! said in fewer than `KNOWN` paragraphs of the whole tree cannot be told
//! from the paragraph it came in: its counts are that paragraph's. The
//! student lists those words of a scope — the story's — with the sentence
//! each was said in, and the quest asks the teacher to show each in use;
//! the answers are told as `uses/<word>.txt` and the loop runs again. This
//! is the code student's chain of requests (docs/124 §3) for prose.

use super::Knowing;

/// A word is known once this many paragraphs have said it.
pub const KNOWN: usize = 3;

impl Knowing {
    /// The words of the scopes under `within` said in fewer than `KNOWN`
    /// paragraphs of the whole tree, each with where it was said: the
    /// paragraph path and the text around it.
    pub fn unknown(&self) -> Vec<(String, String)> {
        let said_in = |w: &str| self.paragraphs.iter().filter(|(_, ws)| ws.iter().any(|x| x == w)).count();
        let mine: Vec<(&String, &Vec<String>)> = self.paragraphs.iter().map(|(p, ws)| (p, ws)).filter(|(p, _)| self.within.is_empty() || self.within.last().is_some_and(|s| format!("scope:{p}").starts_with(s.as_str()))).collect();
        let mut out: Vec<(String, String)> = Vec::new();
        for (_, words) in mine {
            for (i, w) in words.iter().enumerate() {
                if w.chars().all(|c| !c.is_alphanumeric()) || w.len() < 2 || out.iter().any(|x| x.0 == *w) || said_in(w) >= KNOWN {
                    continue;
                }
                let around: Vec<&str> = words[i.saturating_sub(4)..(i + 5).min(words.len())].iter().map(String::as_str).collect();
                out.push((w.clone(), around.join(" ")));
            }
        }
        out
    }
}

/// The question for a word the student does not know: the word shown in
/// use, as the builders' game shows a colour by pointing.
pub fn uses_prompt(word: &str, around: &str) -> String {
    format!(
        "Write three short sentences, each using the word \"{word}\" as it is used in \"{around}\", about everyday things. Reply with the three sentences only, one per line, no numbering."
    )
}
