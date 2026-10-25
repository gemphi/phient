//! `under` — the discovered DAG put back into the scope tree (docs/175),
//! so the cast can read a word UNDER its concept.
//!
//! [`Knowing::families`] finds which construction families a paragraph
//! belongs to; this re-tells each paragraph under EVERY one of them —
//! `found/<a>_<b>/eN` — which is the DAG as paths, not as a side table.
//! Reading then needs no labels and no new machinery: set `within` to the
//! family the words said so far show ([`Knowing::scope_said`]) and ask
//! [`Knowing::next`]. The question docs/170 fixed for Q2 is then askable
//! on held-out text: does the concept path beat the flat window?

use super::knowing::Knowing;

/// The path a family's paragraphs are told under.
pub fn family_path(pair: &(String, String)) -> String {
    format!("found/{}_{}", pair.0, pair.1)
}

impl Knowing {
    /// Re-tells everything known under its discovered families: one tree
    /// whose scopes ARE the concepts the student found. A paragraph in no
    /// family goes under `found/loose`, so nothing is dropped.
    pub fn refile(&self, at_least: usize) -> Knowing {
        let fams = self.families(at_least);
        let mut under: Vec<Vec<String>> = vec![Vec::new(); self.paragraphs.len()];
        for (pair, members) in &fams {
            for &i in members {
                under[i].push(family_path(pair));
            }
        }
        let mut out = Knowing::fresh();
        for (i, (_, words)) in self.paragraphs.iter().enumerate() {
            let text = words.join(" ");
            if under[i].is_empty() {
                let path = format!("found/loose/e{i}");
                out.tell(&path, &super::paragraph(&path, &[text.clone()]));
                continue;
            }
            for scope in &under[i] {
                let path = format!("{scope}/e{i}");
                out.tell(&path, &super::paragraph(&path, &[text.clone()]));
            }
        }
        out
    }

    /// The concept to read the next word under, from the words said so far:
    /// the most recent construction they show that this tree knows as a
    /// family. Causal — it looks only at what was said, never ahead.
    pub fn scope_said(&self, said: &[String], at_least: usize) -> Option<String> {
        let paths: std::collections::HashSet<String> =
            self.families(at_least).into_iter().map(|(p, _)| family_path(&p)).collect();
        self.scope_said_with(said, &self.known_words(at_least), &paths)
    }

    /// [`Self::scope_said`] with the known words and the family paths taken
    /// once — what a reader scoring many positions uses (docs/175).
    pub fn scope_said_with(
        &self,
        said: &[String],
        known: &std::collections::HashSet<String>,
        paths: &std::collections::HashSet<String>,
    ) -> Option<String> {
        Self::construction_with(said, known).iter().rev().map(family_path).find(|p| paths.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{paragraph, Knowing};

    #[test]
    fn the_dag_becomes_paths_and_the_prefix_finds_them() {
        let mut k = Knowing::fresh();
        for (i, s) in ["it does not rain a lot", "rice does not grow here", "we did a lot of work"].iter().enumerate() {
            k.tell(&format!("t/e{i}"), &paragraph("t", &[s.to_string()]));
        }
        let d = k.refile(2);
        // The first sentence shows both families, so it is told twice.
        assert!(d.paragraphs.len() > k.paragraphs.len(), "{} vs {}", d.paragraphs.len(), k.paragraphs.len());
        assert!(d.paragraphs.iter().any(|(p, _)| p.starts_with("found/does_not")));
        assert!(d.paragraphs.iter().any(|(p, _)| p.starts_with("found/a_lot")));
        let said: Vec<String> = ["it", "does", "not"].iter().map(|s| s.to_string()).collect();
        assert_eq!(d.scope_said(&said, 2).as_deref(), Some("found/does_not"));
    }
}
