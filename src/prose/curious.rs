//! `curious` — structure is meaning, and it can only be DISCOVERED
//! (docs/172). The student is not handed the concept tree: it follows what
//! interests it — the words it does not know yet — and places each new
//! paragraph where it belongs in what it already knows, so the tree grows
//! out of the reading. Interest must grow: every paragraph read changes
//! what is unknown, which changes what is interesting next.

use super::knowing::Knowing;
use super::unknown::KNOWN;

impl Knowing {
    /// Of the candidate paragraphs (their words), the index of the most
    /// interesting: the one with the most words this tree finds unknown —
    /// the next thing to read, because curiosity points at what is not yet
    /// known, and looking there is how knowledge stops being broken.
    pub fn interest(&self, candidates: &[Vec<String>]) -> Option<usize> {
        let said_in = |w: &str| self.paragraphs.iter().filter(|(_, ws)| ws.iter().any(|x| x == w)).count();
        candidates
            .iter()
            .enumerate()
            .map(|(i, ws)| (i, ws.iter().filter(|w| w.len() >= 2 && said_in(w) < KNOWN).count()))
            .max_by_key(|&(_, n)| n)
            .map(|(i, _)| i)
    }

    /// Where `words` belong in what is known: the scope (a paragraph path's
    /// parent) whose words they share the most, as `(scope, share)` with
    /// share the fraction of `words` already said there. `None` while
    /// nothing is known. Below a small share the student should open a new
    /// scope instead — that is how it builds the structure up itself.
    pub fn place(&self, words: &[String]) -> Option<(String, f64)> {
        use std::collections::{HashMap, HashSet};
        let cand: HashSet<&str> = words.iter().map(String::as_str).collect();
        let mut group: HashMap<&str, HashSet<&str>> = HashMap::new();
        for (path, ws) in &self.paragraphs {
            let parent = path.rsplit_once('/').map(|(p, _)| p).unwrap_or(path);
            group.entry(parent).or_default().extend(ws.iter().map(String::as_str));
        }
        group
            .into_iter()
            .map(|(g, ws)| (g.to_string(), cand.intersection(&ws).count() as f64 / cand.len().max(1) as f64))
            .max_by(|a, b| a.1.total_cmp(&b.1))
    }
}

impl Knowing {
    /// The structural skeleton of a sentence: its words said in at least
    /// [`KNOWN`] paragraphs of the whole tree, in order. The grammar lives
    /// in these (the closed stratum, docs/157) — is, not, does, has, to —
    /// while the open stratum (rain, nurse, cinema) is unknowable and
    /// structurally silent. Two sentences of one grammar share a skeleton
    /// even when they share no content word. `at_least`: how many
    /// paragraphs must have said a word for it to count ([`KNOWN`] is the
    /// usual bar; 2 hears rarer constructions).
    pub fn skeleton(&self, words: &[String], at_least: usize) -> Vec<String> {
        let known = self.known_words(at_least);
        words.iter().filter(|w| known.contains(*w)).cloned().collect()
    }

    /// Every word said in at least `at_least` paragraphs — the closed
    /// stratum, counted once so the readers below are O(1) per word
    /// instead of a scan of everything told (docs/175).
    pub fn known_words(&self, at_least: usize) -> std::collections::HashSet<String> {
        let mut seen: std::collections::HashMap<&str, usize> = Default::default();
        for (_, words) in &self.paragraphs {
            let mut once: Vec<&str> = words.iter().map(String::as_str).collect();
            once.sort_unstable();
            once.dedup();
            for w in once {
                *seen.entry(w).or_default() += 1;
            }
        }
        seen.into_iter().filter(|(_, n)| *n >= at_least.max(1)).map(|(w, _)| w.to_string()).collect()
    }

    /// The constructions of a sentence: adjacent pairs of known words —
    /// "does not", "has not", "is not" — the two-word units a grammar is
    /// made of. A pronoun or a preposition floats free between sentences
    /// of every tense; a construction is a pair, and the pair is the tense.
    pub fn construction(&self, words: &[String], at_least: usize) -> Vec<(String, String)> {
        Self::construction_with(words, &self.known_words(at_least))
    }

    /// [`Self::construction`] against a known-word set taken once.
    pub fn construction_with(words: &[String], known: &std::collections::HashSet<String>) -> Vec<(String, String)> {
        let known: Vec<bool> = words.iter().map(|w| known.contains(w)).collect();
        words
            .windows(2)
            .zip(known.windows(2))
            .filter(|(_, k)| k[0] && k[1])
            .map(|(w, _)| (w[0].clone(), w[1].clone()))
            .collect()
    }

    /// How well each known scope READS `words`: the share of next-word
    /// predictions the cast gets right (top-2) under that scope — placement
    /// by structure, not by shared vocabulary. Only the positions whose
    /// truth the tree already knows are scored (the closed stratum,
    /// docs/157): a content word never seen is unknowable under every
    /// scope, and scoring it would drown the structural signal the known
    /// words — is, not, does, has — carry.
    pub fn fit(&mut self, words: &[String]) -> Option<(String, f64)> {
        let known = self.known_words(super::unknown::KNOWN);
        let asked: Vec<usize> = (1..words.len()).filter(|&j| known.contains(&words[j])).collect();
        let mut parents: Vec<String> = self
            .paragraphs
            .iter()
            .map(|(p, _)| p.rsplit_once('/').map(|(a, _)| a.to_string()).unwrap_or_else(|| p.clone()))
            .collect();
        parents.sort();
        parents.dedup();
        let keep = std::mem::take(&mut self.within);
        let mut best: Option<(String, f64)> = None;
        for p in parents {
            self.within = crate::relate::scopes(&p, 0);
            let hit: usize = asked
                .iter()
                .filter(|&&j| self.next(&words[..j]).1.iter().take(2).any(|(w, _)| *w == words[j]))
                .count();
            let share = hit as f64 / asked.len().max(1) as f64;
            if best.as_ref().is_none_or(|(_, b)| share > *b) {
                best = Some((p, share));
            }
        }
        self.within = keep;
        best
    }
}

#[cfg(test)]
mod tests {
    use super::super::paragraph;
    use super::*;

    #[test]
    fn interest_points_at_the_unknown_and_place_at_the_known() {
        let mut k = Knowing::fresh();
        k.tell("t/a", &paragraph("t/a", &["the cat sat on the mat".into()]));
        k.tell("t/b", &paragraph("t/b", &["the cat ate the fish".into()]));
        let cands = vec![
            vec!["the".into(), "cat".into(), "sat".into()],
            vec!["quarks".into(), "bind".into(), "hadrons".into()],
        ];
        assert_eq!(k.interest(&cands), Some(1), "the unseen words are the interesting ones");
        let (scope, share) = k.place(&["the".into(), "cat".into(), "sat".into()]).unwrap();
        assert_eq!(scope, "t");
        assert!(share > 0.9, "all three words are known under t: {share}");
    }
}
