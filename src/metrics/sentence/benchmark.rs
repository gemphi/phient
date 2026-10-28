#[allow(unused_imports)]
use super::*;

impl SentenceBenchmark {
    /// Builds items from a document — a run of consecutive sentences.
    ///
    /// Distractors are drawn from `pool`, which must come from *other*
    /// positions in the corpus. Drawing them from the same document would make
    /// them plausible continuations and the task unanswerable rather than hard.
    pub(super) fn build_items(docs: &[Vec<String>], pool: &[Vec<String>], seed: u64) -> Vec<Item> {
        let mut items = Vec::new();
        if pool.len() < CANDIDATES {
            return items;
        }

        for (d, doc) in docs.iter().enumerate() {
            let toks: Vec<Vec<String>> = doc.iter().map(|s| Tokenizer::tokenize(s)).collect();
            for i in CONTEXT_LEN..toks.len() {
                let context: Vec<Vec<String>> = toks[i - CONTEXT_LEN..i].to_vec();
                let truth_toks = toks[i].clone();
                if truth_toks.len() < 4 {
                    continue;
                }

                // Deterministic distractor draw: the item set must be identical
                // between scorers and between runs, or the comparison is
                // between different tasks.
                let mut chosen: Vec<Vec<String>> = Vec::with_capacity(CANDIDATES);
                let mut r = fnv1a(&format!("{}:{}", d, i)) ^ seed;
                let mut seen: HashSet<usize> = HashSet::new();
                while chosen.len() + 1 < CANDIDATES {
                    r ^= r << 13;
                    r ^= r >> 7;
                    r ^= r << 17;
                    let idx = (r % pool.len() as u64) as usize;
                    if !seen.insert(idx) {
                        continue;
                    }
                    if pool[idx] == truth_toks {
                        continue;
                    }
                    chosen.push(pool[idx].clone());
                }

                // The truth goes at a position that varies with the item, so a
                // scorer cannot win by always guessing the same slot.
                let truth = (r % CANDIDATES as u64) as usize;
                chosen.insert(truth.min(chosen.len()), truth_toks);

                items.push(Item { context, candidates: chosen, truth });
            }
        }
        items
    }
}

impl SentenceEncoding {
    pub fn label(self) -> &'static str {
        match self {
            SentenceEncoding::Bag => "phase (bag)",
            SentenceEncoding::Bound => "phase (bound)",
            SentenceEncoding::Recurrent => "phase (recurrent)",
        }
    }

    pub const ALL: [SentenceEncoding; 3] = [
        SentenceEncoding::Bag,
        SentenceEncoding::Bound,
        SentenceEncoding::Recurrent,
    ];
}

impl SentenceBenchmark {
    /// Per-channel recurrence kernel, matching the language model's.
    #[inline]
    pub(super) fn kernel(k: usize) -> c64 {
        crate::metrics::kernel::channel_kernel(k, phiton::units::PHASE_CHANNELS)
    }

    /// Phase state of a sentence under a given encoding.
    pub(super) fn sentence_state_as(facet: &Facet, words: &[String], enc: SentenceEncoding) -> Vec<c64> {
        match enc {
            SentenceEncoding::Bag => Wave::sentence_channels(facet, words, false),
            SentenceEncoding::Bound => Wave::sentence_channels(facet, words, true),
            SentenceEncoding::Recurrent => {
                let n = phiton::units::PHASE_CHANNELS;
                let mut h = vec![c64::new(0.0, 0.0); n];
                for w in words {
                    for (k, z) in h.iter_mut().enumerate() {
                        *z *= Self::kernel(k);
                    }
                    if let Some(p) = facet.lexicon.get(w) {
                        for (k, z) in h.iter_mut().enumerate().take(n) {
                            *z += c64::from_polar(p.amplitude, p.theta(k));
                        }
                    }
                }
                h
            }
        }
    }
}
