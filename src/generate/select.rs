//! Where a step's candidates come from, and how they are scored.

use super::generator::{Decode, Generator, RELATIONAL_WINDOW};
use super::guide::OPERATOR_PROPOSAL_FLOOR;
use phiecs::phasor::facet::Facet;
use phiecs::phasor::flow::PhaseFlow;
use phiton::p64::words::Tokenizer;

impl Generator {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn attractor_select(
        &self,
        facet: &Facet,
        flow: &PhaseFlow,
        prev_word: &Option<String>,
        last_word: &Option<String>,
        recent: &std::collections::HashSet<String>,
        target_phase: f64,
        content_only: bool,
    ) -> Option<String> {
        let keep_content = |c: &mut Vec<(String, u32)>| {
            if content_only {
                c.retain(|(w, _)| !Tokenizer::is_function_word(w));
            }
        };
        // Decoding from the operator: the candidate set is the function
        // space's, optionally joined with the n-gram's.
        match self.decode {
            Decode::Trigram => {}
            Decode::Operator { k } | Decode::Union { k } | Decode::Forcing { k, .. } => {
                let mut cands = self.operator_candidates(facet, prev_word, last_word, k);
                if let (Decode::Union { .. }, Some(a), Some(b)) = (self.decode, prev_word, last_word) {
                    let mut tri = facet.trigram_candidates(a, b);
                    tri.sort_by_key(|x| std::cmp::Reverse(x.1));
                    tri.truncate(self.keep(12));
                    for (w, c) in tri {
                        if !cands.iter().any(|(x, _)| *x == w) {
                            cands.push((w, c));
                        }
                    }
                }
                keep_content(&mut cands);
                if let Some(word) = self.pick_ngram(facet, flow, &cands, recent, target_phase) {
                    return Some(word);
                }
            }
        }
        if let (Some(a), Some(b)) = (prev_word, last_word) {
            let mut tri = facet.trigram_candidates(a, b);
            tri.sort_by_key(|x| std::cmp::Reverse(x.1));
            tri.truncate(self.keep(12));
            self.operator_propose(&mut tri);
            self.relational_propose(facet, self.relational_proposals, OPERATOR_PROPOSAL_FLOOR, &mut tri);
            self.goal_propose(facet, last_word.as_ref(), &mut tri);
            keep_content(&mut tri);
            if let Some(word) = self.pick_ngram(facet, flow, &tri, recent, target_phase) {
                return Some(word);
            }
        }
        if let Some(prev) = last_word {
            let mut bigram = facet.next_word_candidates(prev);
            bigram.sort_by_key(|a| std::cmp::Reverse(a.1));
            bigram.truncate(self.keep(16));
            self.operator_propose(&mut bigram);
            self.relational_propose(facet, self.relational_proposals, OPERATOR_PROPOSAL_FLOOR, &mut bigram);
            self.goal_propose(facet, last_word.as_ref(), &mut bigram);
            keep_content(&mut bigram);
            if let Some(word) = self.pick_ngram(facet, flow, &bigram, recent, target_phase) {
                return Some(word);
            }
        }
        if content_only {
            return None;
        }
        self.torus_ray_cast(facet, flow, target_phase, recent)
    }

    pub(super) fn pick_ngram(
        &self,
        facet: &Facet,
        flow: &PhaseFlow,
        candidates: &[(String, u32)],
        recent: &std::collections::HashSet<String>,
        target_phase: f64,
    ) -> Option<String> {
        let scored = self.score_candidates(facet, flow, candidates, recent, target_phase);
        let values: Vec<f64> = scored.iter().map(|(_, s)| *s).collect();
        self.sample_index(&values).map(|i| scored[i].0.clone())
    }

    /// Scores every admissible candidate without choosing one.
    pub(super) fn score_candidates(
        &self,
        facet: &Facet,
        flow: &PhaseFlow,
        candidates: &[(String, u32)],
        recent: &std::collections::HashSet<String>,
        target_phase: f64,
    ) -> Vec<(String, f64)> {
        let mut scored: Vec<(String, f64)> = Vec::with_capacity(candidates.len());
        if let Some(w) = &self.weights {
            // Learned log-linear scorer: one feature definition shared with the
            // tuner (`learn::features`), so what was fit is what is used.
            let ctx = self.ctx.borrow();
            let start = ctx.len().saturating_sub(RELATIONAL_WINDOW);
            let window = &ctx[start..];
            for (word, count) in candidates {
                if !Self::speakable(word) {
                    continue;
                }
                let x = crate::generate::learn::CandidateFeature::build(
                    facet,
                    word,
                    *count,
                    target_phase,
                    flow.collective_phase,
                    recent.contains(word),
                    self.operator_cos(word),
                    window,
                );
                scored.push((word.clone(), w.score(&x) + self.goal_bonus(facet, word) + self.persona_bonus(facet, word)));
            }
            return scored;
        }
        for (word, count) in candidates {
            if !Self::speakable(word) {
                continue;
            }
            // Soft penalty rather than a hard block: an outright ban on any word
            // seen in the last 12 tokens makes "the cat sat on the mat"
            // unwriteable.
            let repeat_penalty = if recent.contains(word) { 0.15 } else { 1.0 };
            let phase_align = facet
                .lexicon
                .get(word)
                .map(|p| (p.phase - target_phase).cos().max(0.0))
                .unwrap_or(0.0);
            let resonance = flow.resonance_with(facet, word);
            let capped = (*count as f64).min(24.0).ln_1p();
            let content = if Tokenizer::is_function_word(word) { 0.55 } else { 1.35 };
            let score = capped
                * (0.35 + 0.25 * phase_align + 0.40 * resonance)
                * content
                * repeat_penalty
                * self.operator_factor(word);
            scored.push((word.clone(), score * (1.0 + self.goal_bonus(facet, word)) * self.persona_bonus(facet, word).exp()));
        }
        scored
    }

}
