#[allow(unused_imports)]
use super::*;

impl LexicalRule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.triples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }

    pub fn add(&mut self, head: &str, role: &str, filler: &str) {
        if head == filler || role.is_empty() {
            return;
        }
        let i = self.triples.len();
        self.triples
            .push((head.to_string(), role.to_string(), filler.to_string()));
        self.by_role.entry(role.to_string()).or_default().push(i);
        self.by_head.entry(head.to_string()).or_default().push(i);
    }

    /// Pairs standing in one named relation — the group coherence is measured on.
    pub fn pairs_for_role(&self, role: &str) -> Vec<(String, String)> {
        self.by_role
            .get(role)
            .map(|ix| {
                ix.iter()
                    .map(|i| (self.triples[*i].0.clone(), self.triples[*i].2.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn roles(&self) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .by_role
            .iter()
            .map(|(r, ix)| (r.clone(), ix.len()))
            .collect();
        v.sort_by_key(|a| std::cmp::Reverse(a.1));
        v
    }

    /// Register per head — the number that was 1.06 with the enum extractor.
    pub fn mean_per_head(&self) -> f64 {
        match self.by_head.is_empty() {
            true => 0.0,
            false => self.triples.len() as f64 / self.by_head.len() as f64,
        }
    }

    pub fn heads_with_at_least(&self, n: usize) -> usize {
        self.by_head.values().filter(|v| v.len() >= n).count()
    }

    /// Extracts `preposition(head, noun)` triples from a gloss.
    ///
    /// For each relational preposition, the next content word is the filler.
    /// No semantics are assigned to the preposition: it is a label, and whether
    /// two labels name one relation is left to [`RoleDiscovery`] rather than
    /// decided here.
    pub fn extract(&mut self, head: &str, gloss: &str, is_content: impl Fn(&str) -> bool) {
        let toks: Vec<String> = phiton::p64::words::Tokenizer::tokenize(gloss);
        for (i, w) in toks.iter().enumerate() {
            if !RELATIONAL_PREPOSITIONS.contains(&w.as_str()) {
                continue;
            }
            // The filler is the first content word after the preposition, within
            // a short window — beyond that the phrase has usually ended.
            if let Some(f) = toks.iter().skip(i + 1).take(3).find(|x| is_content(x)) {
                self.add(head, w, f);
            }
        }
    }
}
