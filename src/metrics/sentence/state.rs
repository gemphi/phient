#[allow(unused_imports)]
use super::*;

impl SentenceBenchmark {
    /// Phase state of a sentence: the per-channel superposition of its words.
    pub(super) fn sentence_state(facet: &Facet, words: &[String]) -> Vec<c64> {
        Self::sentence_state_as(facet, words, SentenceEncoding::Bag)
    }

    /// Cosine between two per-channel states, averaged over channels.
    pub(super) fn state_similarity(a: &[c64], b: &[c64]) -> f64 {
        let mut sum = 0.0;
        let mut n = 0usize;
        for (x, y) in a.iter().zip(b.iter()) {
            let (nx, ny) = (x.norm(), y.norm());
            if nx > 1e-12 && ny > 1e-12 {
                sum += (x.re * y.re + x.im * y.im) / (nx * ny);
                n += 1;
            }
        }
        match n {
            0 => 0.0,
            _ => sum / n as f64,
        }
    }

    /// Rank of the truth under a scoring function, 1-based.
    pub(super) fn rank_by<F: Fn(&Item, usize) -> f64>(item: &Item, score: F) -> usize {
        let truth_score = score(item, item.truth);
        let beating = (0..item.candidates.len())
            .filter(|i| *i != item.truth && score(item, *i) > truth_score)
            .count();
        beating + 1
    }

    pub(super) fn summarise(name: &str, ranks: &[usize]) -> ScorerResult {
        let n = ranks.len().max(1) as f64;
        ScorerResult {
            name: name.to_string(),
            top1: ranks.iter().filter(|r| **r == 1).count() as f64 / n,
            top5: ranks.iter().filter(|r| **r <= 5).count() as f64 / n,
            mrr: ranks.iter().map(|r| 1.0 / *r as f64).sum::<f64>() / n,
            items: ranks.len(),
        }
    }

    /// Runs the benchmark over a corpus split into documents.
    pub fn evaluate(facet: &Facet, docs: &[Vec<String>], pool: &[Vec<String>], seed: u64) -> SentenceReport {
        let items = Self::build_items(docs, pool, seed);

        let per_item: Vec<(Vec<usize>, usize)> = items
            .par_iter()
            .map(|item| {
                // ---- phase, one rank per encoding ----
                // The context is one accumulated state, not three separate
                // ones: the compression claim is that a prefix collapses into a
                // single state of fixed size however long it is.
                let phase_ranks: Vec<usize> = SentenceEncoding::ALL
                    .iter()
                    .map(|enc| {
                        let mut ctx = vec![c64::new(0.0, 0.0); phiton::units::PHASE_CHANNELS];
                        for sent in &item.context {
                            let s = Self::sentence_state_as(facet, sent, *enc);
                            for (a, b) in ctx.iter_mut().zip(s.iter()) {
                                *a += *b;
                            }
                        }
                        Self::rank_by(item, |it, i| {
                            Self::state_similarity(
                                &ctx,
                                &Self::sentence_state_as(facet, &it.candidates[i], *enc),
                            )
                        })
                    })
                    .collect();

                // ---- lexical ----
                let ctx_words: HashSet<&String> = item.context.iter().flatten().collect();
                let lex_rank = Self::rank_by(item, |it, i| {
                    let cand: HashSet<&String> = it.candidates[i].iter().collect();
                    let overlap = ctx_words.intersection(&cand).count() as f64;
                    // Length-normalised, or long candidates win by having more
                    // chances to overlap rather than by being the continuation.
                    overlap / (it.candidates[i].len() as f64).sqrt().max(1.0)
                });

                (phase_ranks, lex_rank)
            })
            .collect();

        let lexical: Vec<usize> = per_item.iter().map(|(_, l)| *l).collect();

        let chance_mrr = phical::relate::chance(CANDIDATES);

        SentenceReport {
            items: items.len(),
            candidates: CANDIDATES,
            chance_top1: 1.0 / CANDIDATES as f64,
            chance_mrr,
            scorers: SentenceEncoding::ALL
                .iter()
                .enumerate()
                .map(|(e, enc)| {
                    let ranks: Vec<usize> = per_item.iter().map(|(p, _)| p[e]).collect();
                    Self::summarise(enc.label(), &ranks)
                })
                .chain(std::iter::once(Self::summarise("lexical overlap", &lexical)))
                .collect(),
        }
    }
}
