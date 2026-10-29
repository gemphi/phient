//! Turning a corpus into scored positions: every candidate's features at every step.

use super::features::CandidateFeature;

/// Candidates kept per position when building training positions.
pub const CANDIDATES: usize = 64;
/// Context words the features look back over.
pub const WINDOW: usize = 6;

/// Builds tuner positions from sentences: at every position `i ≥ 2`, the
/// n-gram's top-`CANDIDATES` continuations with their features, if the true
/// word is among them. Returns `(positions, positions seen, positions covered)`.
pub fn corpus_positions(
    facet: &phiecs::phasor::facet::Facet,
    sentences: &[String],
    cap: usize,
    guide: &mut Option<crate::generate::OperatorGuide>,
) -> (Vec<Position>, usize, usize) {
    let mut out = Vec::new();
    let (mut seen, mut covered) = (0usize, 0usize);
    'outer: for s in sentences {
        let toks = phiton::p64::words::Tokenizer::tokenize(s);
        for i in 2..toks.len() {
            seen += 1;
            let mut cands = facet.trigram_candidates(&toks[i - 2], &toks[i - 1]);
            if cands.is_empty() {
                cands = facet.next_word_candidates(&toks[i - 1]);
            }
            cands.sort_by_key(|c| std::cmp::Reverse(c.1));
            cands.truncate(CANDIDATES);
            let truth = match cands.iter().position(|(w, _)| *w == toks[i]) {
                Some(t) => t,
                None => continue,
            };
            covered += 1;
            let start = i.saturating_sub(WINDOW);
            let ctx = &toks[start..i];
            let target_phase = phiecs::phasor::wave::Wave::sentence(facet, ctx).arg().rem_euclid(std::f64::consts::TAU);
            let mut flow = phiecs::phasor::flow::PhaseFlow::build(facet, &ctx.join(" "));
            flow.propagate(2);
            let recent: std::collections::HashSet<&String> = toks[i.saturating_sub(12)..i].iter().collect();
            if let Some(g) = guide.as_mut() {
                g.focus(&toks[..i]);
            }
            let feats = cands
                .iter()
                .map(|(w, c)| {
                    let op = guide.as_mut().and_then(|g| g.cosine_of(w)).unwrap_or(0.0);
                    CandidateFeature::build(facet, w, *c, target_phase, flow.collective_phase, recent.contains(w), op, ctx)
                })
                .collect();
            out.push(Position { feats, truth });
            if out.len() >= cap {
                break 'outer;
            }
        }
    }
    (out, seen, covered)
}


/// One training position: the candidates' features and which one is true.
pub struct Position {
    pub feats: Vec<CandidateFeature>,
    pub truth: usize,
}
