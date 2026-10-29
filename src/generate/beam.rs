//! Beam search with a per-hypothesis operator history.

use super::guide::OPERATOR_PROPOSAL_FLOOR;
use crate::generate::constants::SYNTACTIC_MOMENTUM_DEFAULT;
use super::context::ContextWaveBuffer;
use super::generator::Generator;
use super::Hypothesis;
use crate::generate::constants::TWO_PI;
use phiecs::phasor::facet::Facet;
use phiecs::phasor::flow::PhaseFlow;
use phiton::p64::words::Tokenizer;

impl Generator {
    /// Beam-search decode.
    ///
    /// Greedy decoding commits to the highest-scoring token at every step and
    /// cannot recover from a choice that looked good locally and stranded the
    /// sequence. A beam keeps `width` hypotheses alive and lets a later token
    /// justify an earlier one. `src/synthesis/search.rs` already implemented
    /// beam search for programs; the decoder never used the idea.
    ///
    /// Returns the highest-scoring completed hypothesis.
    pub fn decode_beam(
        &self,
        facet: &Facet,
        context_buffer: &mut ContextWaveBuffer,
        prompt: &str,
        width: usize,
    ) -> Vec<String> {
        context_buffer.push_turn(facet, prompt);
        let flow = PhaseFlow::build(facet, prompt);

        let prompt_tokens = Tokenizer::tokenize(prompt);
        self.operator_reset(&prompt_tokens);
        let seed = Hypothesis {
            tokens: Vec::new(),
            prev: match prompt_tokens.len() >= 2 {
                true => Some(prompt_tokens[prompt_tokens.len() - 2].clone()),
                false => None,
            },
            last: prompt_tokens.last().cloned(),
            phase: context_buffer.context_phase(),
            score: 0.0,
            recent: std::collections::HashSet::new(),
        };

        let mut beam = vec![seed];
        let width = width.max(1);

        for _ in 0..self.max_tokens.min(20) {
            let mut next: Vec<Hypothesis> = Vec::new();

            for h in &beam {
                let target = (h.phase + SYNTACTIC_MOMENTUM_DEFAULT).rem_euclid(TWO_PI);

                let mut cands: Vec<(String, u32)> = Vec::new();
                if let (Some(a), Some(b)) = (&h.prev, &h.last) {
                    cands = facet.trigram_candidates(a, b);
                }
                if cands.is_empty() {
                    if let Some(b) = &h.last {
                        cands = facet.next_word_candidates(b);
                    }
                }
                if cands.is_empty() {
                    continue;
                }
                cands.sort_by_key(|x| std::cmp::Reverse(x.1));
                cands.truncate(self.keep(16));

                self.operator_focus(&prompt_tokens, &h.tokens);
                self.operator_propose(&mut cands);
                self.relational_propose(facet, self.relational_proposals, OPERATOR_PROPOSAL_FLOOR, &mut cands);
                let scored = self.score_candidates(facet, &flow, &cands, &h.recent, target);
                let mut ranked = scored;
                ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                for (word, sc) in ranked.into_iter().take(width) {
                    if sc <= 0.0 {
                        continue;
                    }
                    let mut t = h.tokens.clone();
                    t.push(word.clone());
                    let mut r = h.recent.clone();
                    r.insert(word.clone());
                    let phase = facet
                        .lexicon
                        .get(&word)
                        .map(|p| p.theta(0))
                        .unwrap_or(h.phase);
                    next.push(Hypothesis {
                        tokens: t,
                        prev: h.last.clone(),
                        last: Some(word),
                        phase,
                        // Length-normalised, so long hypotheses are not
                        // penalised purely for being long.
                        score: h.score + sc.ln_1p(),
                        recent: r,
                    });
                }
            }

            if next.is_empty() {
                break;
            }
            next.sort_by(|a, b| {
                let (an, bn) = (a.score / a.tokens.len() as f64, b.score / b.tokens.len() as f64);
                bn.partial_cmp(&an).unwrap_or(std::cmp::Ordering::Equal)
            });
            next.truncate(width);
            beam = next;
        }

        beam.into_iter().next().map(|h| h.tokens).unwrap_or_default()
    }

}
