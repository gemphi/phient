#[allow(unused_imports)]
use super::*;

impl DefinitionGraph {
    pub fn build(entries: &[(String, String)]) -> Self {
        let mut edges: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
        for (word, def) in entries {
            let set = edges.entry(word.clone()).or_default();
            for t in Tokenizer::tokenize(def) {
                if !is_structural(&t) && t != *word {
                    set.insert(t);
                }
            }
        }
        Self { edges }
    }

    /// Words appearing in `word`'s definition.
    pub fn definers(&self, word: &str) -> Option<&std::collections::HashSet<String>> {
        self.edges.get(word)
    }

    /// True when each word occurs in the other's definition.
    pub fn is_strong(&self, a: &str, b: &str) -> bool {
        self.contains(a, b) && self.contains(b, a)
    }

    /// True when the two are definitionally related in either direction — the
    /// test a negative sample has to fail.
    pub fn is_related(&self, a: &str, b: &str) -> bool {
        self.contains(a, b) || self.contains(b, a)
    }

    pub(super) fn contains(&self, of: &str, word: &str) -> bool {
        self.edges.get(of).is_some_and(|s| s.contains(word))
    }

    /// Promotes weak pairs to strong when the two words are among each other's
    /// nearest neighbours.
    ///
    /// Raw reciprocity is a weak signal in a single dictionary, and the
    /// measurement says how weak: once entries are reduced to their
    /// definitional core (~10 definers each), only 784 of 397,072 pairs are
    /// two-way — a ratio of 505∶1 against dict2vec's ~9∶1. Longer definitions
    /// buy reciprocity back but destroy the kernel structure (30.5% of entries
    /// instead of the literature's ~10%). Neither end of that trade gives the
    /// graph the split needs.
    ///
    /// Dict2vec §3.1 resolves it the same way: "some weak pairs can be promoted
    /// as strong pairs if the two words are among the K closest neighbours of
    /// each other", with K = 5. Their neighbours come from a pretrained
    /// embedding over the whole vocabulary. This uses the facet's own phase
    /// resonance, and restricts the candidate set to the words each already
    /// defines — a cheaper variant, stated as one, since scanning a 70k
    /// vocabulary per word is quadratic and the promotion can only ever apply to
    /// a pair that is already an edge.
    pub fn promote_neighbours(&mut self, facet: &Facet, k: usize) -> usize {
        // The candidate set has to be *undirected*, and getting this wrong makes
        // the whole method a no-op: ranking each word only among the words it
        // defines means `a` can be in `b`'s top-k only when `b` already defines
        // `a`, which is the definition of a strong pair. The first version did
        // exactly that and promoted 0 of 271,300 weak pairs — logically
        // incapable of promoting any. A word's neighbourhood is the words it
        // defines *and* the words that define it.
        let mut neighbourhood: HashMap<&str, std::collections::HashSet<&str>> = HashMap::new();
        for (word, defs) in &self.edges {
            let e = neighbourhood.entry(word.as_str()).or_default();
            for d in defs {
                e.insert(d.as_str());
            }
            for d in defs {
                neighbourhood.entry(d.as_str()).or_default().insert(word.as_str());
            }
        }

        // Top-k by phase resonance, against frozen phases and with ties broken
        // by word, so the cut at k does not depend on iteration order.
        let mut top: HashMap<&str, Vec<&str>> = HashMap::new();
        for (word, cands) in &neighbourhood {
            let mut scored: Vec<(f64, &str)> = cands
                .iter()
                .filter(|c| facet.lexicon.contains_key(**c))
                .map(|c| (facet.resonance(word, c), *c))
                .collect();
            scored.sort_by(|a, b| {
                b.0.partial_cmp(&a.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.1.cmp(b.1))
            });
            top.insert(word, scored.into_iter().take(k).map(|(_, w)| w).collect());
        }

        // Mutual nearness only. One-way nearness is what a weak pair already is.
        let mut pairs: Vec<(String, String)> = Vec::new();
        for (a, near) in &top {
            for b in near {
                if top.get(b).is_some_and(|n| n.contains(a)) && !self.is_strong(a, b) {
                    pairs.push(((*a).to_string(), (*b).to_string()));
                }
            }
        }

        // Promotion is recorded by adding the missing reverse edge, so
        // `is_strong` and `counts` see it without a second code path.
        let mut promoted = 0usize;
        for (a, b) in pairs {
            if self.edges.entry(b).or_default().insert(a) {
                promoted += 1;
            }
        }
        promoted
    }

    /// The **grounding kernel**: the words that survive recursively removing
    /// every word that is defined but defines nothing further.
    ///
    /// From Vincent-Lamarre et al., *The Latent Structure of Dictionaries*
    /// (2016). A dictionary's definitional graph is not flat. Peel away the
    /// words that appear in no other word's definition, then peel again — what
    /// is left (~10% of a full dictionary) is the set from which everything else
    /// is definable. Words nearer that core are more concrete, more frequent,
    /// and acquired earlier.
    ///
    /// The relevance here is scheduling. `compose_all` treats all 39,938 entries
    /// as peers and relaxes them simultaneously, which composes each word
    /// against neighbours that are themselves still moving. The structure says
    /// meaning propagates *outward from the kernel*, so the kernel is what
    /// should be held while the periphery is composed against it.
    ///
    /// It also marks this architecture's ceiling honestly: the kernel is defined
    /// circularly, so no amount of composition can originate its meaning — the
    /// symbol grounding problem, in the form the dictionary's own structure
    /// takes. Whatever grounds those words has to come from outside the
    /// definitions: corpus statistics here, sensorimotor experience in Harnad's
    /// original framing.
    ///
    /// **Compute this before promotion.** The kernel is a property of the
    /// definitional graph — which words the lexicographer used to define which
    /// others. Promotion adds edges inferred from phase similarity, which is a
    /// different relation, and peeling a graph that mixes the two measures
    /// neither.
    pub fn kernel(&self) -> std::collections::HashSet<String> {
        let mut alive: std::collections::HashSet<String> = self.edges.keys().cloned().collect();

        loop {
            // A word is *used* if it appears in the definition of some word
            // still in the graph.
            let mut used: std::collections::HashSet<&str> = std::collections::HashSet::new();
            for a in &alive {
                if let Some(defs) = self.edges.get(a) {
                    for b in defs {
                        if alive.contains(b) {
                            used.insert(b.as_str());
                        }
                    }
                }
            }

            let before = alive.len();
            alive.retain(|w| used.contains(w.as_str()));
            if alive.len() == before || alive.is_empty() {
                return alive;
            }
        }
    }

    /// Nodes in the graph.
    ///
    /// Not the same as the number of entries it was built from: promotion adds
    /// reverse edges, and a reverse edge whose target had no entry of its own
    /// introduces a new node. Reporting kernel size against the entry count
    /// after promotion produced a share of 103.5%, which is how that was found.
    pub fn nodes(&self) -> usize {
        self.edges.len()
    }
}
