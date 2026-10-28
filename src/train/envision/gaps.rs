#[allow(unused_imports)]
use super::*;

impl Envision {
    pub fn new() -> Self {
        Self::default()
    }

    /// Detects knowledge gaps, resolving what it can before asking.
    ///
    /// Escalation order: the local dictionary first (instant, offline), and the
    /// user only when that fails. Suggestions blend spelling with **context**:
    /// an unknown word has no phase of its own, but the known words beside it
    /// do, and ray casting turns that wave into semantic neighbours. Comparing
    /// spellings alone offered `photograph` for `photosynthesis` while the
    /// manifold that could have offered `plant` went unread.
    pub fn detect_gaps(
        &mut self,
        facet: &mut Facet,
        chunk_store: Option<&ChunkStore>,
        text: &str,
    ) -> Option<Vision> {
        let tokens = Tokenizer::tokenize(text);
        let unknown: Vec<String> = tokens
            .iter()
            .filter(|t| !facet.contains_word(t))
            .cloned()
            .collect();

        if unknown.is_empty() {
            return None;
        }

        // Context wave from the *known* words of the same input.
        let context = Wave::sentence(facet, &tokens);

        let mut auto_learned = Vec::new();
        let mut still_unknown = Vec::new();

        for word in &unknown {
            self.ledger.encounter(word);

            // Escalate to the dictionary before spending the user's attention.
            let resolved = chunk_store
                .and_then(|cs| cs.load_definition(word))
                .map(|def| {
                    facet.get_or_init(word);
                    (word.clone(), def)
                });

            match resolved {
                Some((w, _def)) => {
                    self.ledger.mark_resolved(&w);
                    auto_learned.push(w);
                }
                None if self.ledger.should_ask(word) => {
                    self.ledger.mark_asked(word);
                    still_unknown.push(word.clone());
                }
                None => {}
            }
        }

        if auto_learned.is_empty() && still_unknown.is_empty() {
            return None;
        }

        let mut parts = Vec::new();
        if !auto_learned.is_empty() {
            parts.push(format!("Looked up {} from the dictionary.", auto_learned.join(", ")));
        }

        if !still_unknown.is_empty() {
            let list = still_unknown
                .iter()
                .map(|w| format!("'{}'", w))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("I don't know {}. Can you define them?", list));

            for word in &still_unknown {
                let suggestions = Self::suggest(facet, context, word);
                if !suggestions.is_empty() {
                    let rendered = suggestions
                        .iter()
                        .map(|(w, s, kind)| format!("{} ({:.2}, {})", w, s, kind))
                        .collect::<Vec<_>>()
                        .join(", ");
                    parts.push(format!("  Is '{}' related to {}?", word, rendered));
                }
            }
        }

        Some(Vision { text: parts.join("\n"), auto_learned })
    }

    /// Blends spelling similarity with context resonance.
    pub(super) fn suggest(
        facet: &Facet,
        context: phistd::c64,
        word: &str,
    ) -> Vec<(String, f64, &'static str)> {
        let mut scored: HashMap<String, (f64, &'static str)> = HashMap::new();

        // --- orthographic: borrow, prefilter by length, clone only survivors ---
        let wlen = word.chars().count() as i64;
        for kw in facet.lexicon.keys() {
            // A word more than three characters different in length shares too
            // little to clear the threshold; skipping it avoids the bigram work.
            if (kw.chars().count() as i64 - wlen).abs() > 3 {
                continue;
            }
            let s = Self::string_similarity(word, kw);
            if s > SUGGEST_THRESHOLD {
                scored.insert(kw.clone(), (ORTHOGRAPHIC_WEIGHT * s, "spelling"));
            }
        }

        // --- semantic: what the surrounding known words point at ---
        if context.norm() > 1e-9 {
            for (w, delta) in Wave::ray_cast(facet, context, SUGGEST_K * 2) {
                if Tokenizer::is_function_word(&w) || w.len() < 2 {
                    continue;
                }
                let sim = (1.0 - delta).clamp(0.0, 1.0);
                let entry = scored.entry(w).or_insert((0.0, "context"));
                entry.0 += SEMANTIC_WEIGHT * sim;
                if entry.1 == "spelling" {
                    entry.1 = "both";
                }
            }
        }

        let mut out: Vec<(String, f64, &'static str)> =
            scored.into_iter().map(|(w, (s, k))| (w, s, k)).collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        out.truncate(SUGGEST_K);
        out
    }

    /// Spelling similarity: 40% shared prefix, 60% character-bigram Jaccard.
    pub(super) fn string_similarity(a: &str, b: &str) -> f64 {
        if a == b {
            return 1.0;
        }
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let ac: Vec<char> = a.chars().collect();
        let bc: Vec<char> = b.chars().collect();

        let prefix = ac.iter().zip(bc.iter()).take_while(|(x, y)| x == y).count();
        let shorter = ac.len().min(bc.len());
        let prefix_score = match shorter {
            0 => 0.0,
            n => prefix as f64 / n as f64,
        };

        let a_bigrams: HashSet<(char, char)> = ac.windows(2).map(|w| (w[0], w[1])).collect();
        let b_bigrams: HashSet<(char, char)> = bc.windows(2).map(|w| (w[0], w[1])).collect();
        if a_bigrams.is_empty() || b_bigrams.is_empty() {
            return prefix_score;
        }

        let intersection = a_bigrams.intersection(&b_bigrams).count();
        let union = a_bigrams.union(&b_bigrams).count();
        let bigram_score = match union {
            0 => 0.0,
            u => intersection as f64 / u as f64,
        };

        prefix_score * 0.4 + bigram_score * 0.6
    }

}
