//! Going on from what is known: `Knowing::go`, the next word read and said,
//! until what was told ends — or past it, into the language.

use super::knowing::Knowing;
use super::{self as prose, END};

impl Knowing {
    /// Goes on from `from` for up to `words` words; the text, then a log of
    /// where the words were drawn and which conditions posted.
    /// `beyond`: where what was told ends, go on with the language rather
    /// than stop.
    pub fn go(&self, from: &str, words: usize, beyond: bool) -> String {
        let mut said = prose::tokens(from);
        let mut out = String::new();
        let mut log = String::new();
        let mut last_path = Vec::new();
        for _ in 0..words {
            let (path, ranked, posts) = self.next(&said);
            if path != last_path {
                log += &format!("· after \"{}\": drawn into {} (posts: {})\n", said.iter().rev().take(4).rev().cloned().collect::<Vec<_>>().join(" "), path.last().map_or("nothing", String::as_str), posts.join(", "));
                last_path = path;
            }
            let n = said.len();
            let repeats = |w: &str| n >= 2 && said.windows(3).any(|t| t[0] == said[n - 2] && t[1] == said[n - 1] && t[2] == w);
            let top = ranked.iter().find(|(w, _)| !repeats(w)).map(|x| x.0.as_str());
            if top == Some(END) && !beyond {
                break;
            }
            let Some((w, _)) = ranked.iter().find(|(w, _)| !repeats(w) && w != END) else { break };
            said.push(w.clone());
        }
        for (i, t) in said.iter().enumerate() {
            let glue = t.chars().all(|c| !c.is_alphanumeric()) || i == 0;
            if !glue {
                out.push(' ');
            }
            out.push_str(t);
        }
        format!("{out}\n\n{log}")
    }
}
