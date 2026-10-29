//! The operator's candidates for a step: nearest by fit, or by Liskov's
//! forcing choice (`Decode::Forcing`).

use super::generator::{Decode, Generator};
use super::guide::OPERATOR_PROPOSAL_FLOOR;
use phiecs::phasor::facet::Facet;

impl Generator {
    /// The operator's nearest `k` pool words, each with the n-gram count the
    /// tables have for it after the current context (0 when they have none —
    /// the scorer's `ln_count` feature then says so instead of the gate).
    pub(super) fn operator_candidates(&self, facet: &Facet, prev: &Option<String>, last: &Option<String>, k: usize) -> Vec<(String, u32)> {
        let op = match &self.operator {
            Some(op) => op,
            None => return Vec::new(),
        };
        let words = match self.decode {
            Decode::Forcing { lambda, .. } => op.borrow_mut().nearest_forcing(k, OPERATOR_PROPOSAL_FLOOR, lambda),
            _ => op.borrow_mut().nearest(k, OPERATOR_PROPOSAL_FLOOR),
        };
        let tri: Vec<(String, u32)> = match (prev, last) {
            (Some(a), Some(b)) => facet.trigram_candidates(a, b),
            _ => Vec::new(),
        };
        let bi: Vec<(String, u32)> = match last {
            Some(b) => facet.next_word_candidates(b),
            None => Vec::new(),
        };
        words
            .into_iter()
            .map(|w| {
                let c = tri.iter().find(|(x, _)| *x == w).or_else(|| bi.iter().find(|(x, _)| *x == w)).map(|(_, c)| *c).unwrap_or(0);
                (w, c)
            })
            .collect()
    }

    /// Puts the goal among the candidates when the corpus has seen it follow
    /// `last`. The goal's pull (`goal_bonus`) only reaches words already in the
    /// candidate list, and a dialogue's goal is a rare content word that the
    /// top n-gram continuations almost never include — measured, the shipped
    /// dialogue reached its goal in 0 of 54 replies. Proposing it where a
    /// bigram licenses it gives the pull something to act on without
    /// inventing a transition the corpus never made. `PHIANO_GOAL_PROPOSE=0`
    /// turns it off.
    pub(super) fn goal_propose(&self, facet: &Facet, last: Option<&String>, cands: &mut Vec<(String, u32)>) {
        let Some((goal, w)) = &self.goal else { return };
        if *w == 0.0 || std::env::var("PHIANO_GOAL_PROPOSE").as_deref() == Ok("0") {
            return;
        }
        if cands.iter().any(|(x, _)| x == goal) || self.ctx.borrow().iter().any(|t| t == goal) {
            return;
        }
        let Some(last) = last else { return };
        if let Some((_, c)) = facet.next_word_candidates(last).into_iter().find(|(x, _)| x == goal) {
            cands.push((goal.clone(), c));
        }
    }
}
