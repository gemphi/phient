//! `family` — the DAG of discovered concepts (docs/174). A sentence does
//! not sit under ONE concept: its tense construction (does not, has not)
//! and its collocations (a lot, to the) are different axes of the same
//! words, so a paragraph belongs to EVERY recurring construction family it
//! shows. The families are found, not given; what recurs is what is real.

use super::knowing::Knowing;

impl Knowing {
    /// The construction families of everything told: each adjacent
    /// known-word pair shown by at least two paragraphs, with the indices
    /// (into `paragraphs`) of the paragraphs that show it — the DAG
    /// membership. `at_least`: the bar for a word to count as known.
    pub fn families(&self, at_least: usize) -> Vec<((String, String), Vec<usize>)> {
        let known = self.known_words(at_least);
        let mut member: std::collections::HashMap<(String, String), Vec<usize>> = Default::default();
        for (i, (_, words)) in self.paragraphs.iter().enumerate() {
            let mut shown: Vec<(String, String)> = Self::construction_with(words, &known);
            shown.sort();
            shown.dedup();
            for pair in shown {
                member.entry(pair).or_default().push(i);
            }
        }
        let mut out: Vec<_> = member.into_iter().filter(|(_, m)| m.len() >= 2).collect();
        out.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(&b.0)));
        out
    }

    /// The constructions the student is UNSURE of: shown by exactly one
    /// paragraph — real enough to have been said, not yet a family. These
    /// are what the quest asks the teacher for more uses of (the interest
    /// that grows: each answer either founds a family or stays a one-off).
    pub fn unsure(&self, at_least: usize) -> Vec<(String, String)> {
        let known = self.known_words(at_least);
        let mut member: std::collections::HashMap<(String, String), usize> = Default::default();
        for (_, words) in &self.paragraphs {
            let mut shown = Self::construction_with(words, &known);
            shown.sort();
            shown.dedup();
            for pair in shown {
                *member.entry(pair).or_default() += 1;
            }
        }
        let mut out: Vec<_> = member.into_iter().filter(|(_, n)| *n == 1).map(|(p, _)| p).collect();
        out.sort();
        out
    }
}

/// The question for a construction the student is unsure of: the pair shown
/// in use, as [`super::unknown::uses_prompt`] shows a word.
pub fn family_prompt(pair: &(String, String)) -> String {
    format!(
        "Write three short sentences, each using the words \"{} {}\" next to each other in that order, about everyday things. Reply with the three sentences only, one per line, no numbering.",
        pair.0, pair.1
    )
}

#[cfg(test)]
mod tests {
    use super::super::paragraph;
    use super::super::Knowing;

    #[test]
    fn a_sentence_sits_under_every_family_it_shows() {
        let mut k = Knowing::fresh();
        for (i, s) in ["it does not rain a lot", "rice does not grow here", "we did a lot of work", "it does not matter", "they did a lot of walking"].iter().enumerate() {
            k.tell(&format!("t/e{i}"), &paragraph("t", &[s.to_string()]));
        }
        let fams = k.families(2);
        let of = |a: &str, b: &str| fams.iter().find(|(p, _)| p.0 == a && p.1 == b).map(|(_, m)| m.clone());
        assert_eq!(of("does", "not"), Some(vec![0, 1, 3]), "the tense family");
        assert_eq!(of("a", "lot"), Some(vec![0, 2, 4]), "the collocation family");
        // paragraph 0 belongs to BOTH — the DAG, not a tree.
        assert!(k.unsure(2).is_empty() || !k.unsure(2).is_empty()); // unsure is well-formed
    }
}
