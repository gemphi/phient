//! Proposals from the guide and the relational reading, the operator factor, and sampling.

use super::guide::OPERATOR_PROPOSAL_COUNT;
use super::generator::RELATIONAL_WINDOW;
use super::guide::OPERATOR_PROPOSAL_FLOOR;
use super::Generator;
use phiecs::phasor::facet::Facet;

impl Generator {
    /// duplicates; n-gram counts win).
    pub(super) fn operator_propose(&self, cands: &mut Vec<(String, u32)>) {
        if let Some(op) = &self.operator {
            let k = op.borrow().proposals;
            if k == 0 {
                return;
            }
            let proposed = op.borrow_mut().propose(k, OPERATOR_PROPOSAL_FLOOR);
            for (w, c) in proposed {
                if !cands.iter().any(|(x, _)| *x == w) {
                    cands.push((w, c));
                }
            }
        }
    }

    /// Relational proposals: the `k` words whose target wheels agree most
    /// with the context's source wheels, channel by channel
    /// (`Facet::relational_score`), among words seen at least `floor` times
    /// and with at least two evidenced pairs. No operator, no prior, no
    /// context vector: the lexicon's own directed reading, pair by pair.
    pub(super) fn relational_propose(&self, facet: &Facet, k: usize, floor: u32, cands: &mut Vec<(String, u32)>) {
        if k == 0 {
            return;
        }
        let ctx = self.ctx.borrow();
        let start = ctx.len().saturating_sub(RELATIONAL_WINDOW);
        let window = &ctx[start..];
        if window.is_empty() {
            return;
        }
        let mut scored: Vec<(&String, f64, u32)> = facet
            .lexicon
            .iter()
            .filter(|(w, p)| p.count >= floor && Self::speakable(w))
            .filter_map(|(w, p)| {
                let (r, n) = facet.relational_score(window, w);
                (n >= 2).then_some((w, r, p.count))
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(b.2.cmp(&a.2)));
        for (w, _, _) in scored.into_iter().take(k) {
            if !cands.iter().any(|(x, _)| x == w) {
                cands.push((w.clone(), OPERATOR_PROPOSAL_COUNT));
            }
        }
    }

    /// The operator's multiplicative factor for a candidate, 1.0 without one.
    pub(super) fn operator_factor(&self, word: &str) -> f64 {
        match &self.operator {
            Some(op) => {
                let mut op = op.borrow_mut();
                let w = op.weight;
                match op.cosine(word) {
                    Some(c) => (1.0 - w) + w * (1.0 + c) / 2.0,
                    None => 1.0,
                }
            }
            None => 1.0,
        }
    }

    /// xorshift64* — a uniform draw in [0, 1).
    pub(super) fn next_uniform(&self) -> f64 {
        let (x, out) = phistd::random::xorshift64_star(self.rng.get());
        self.rng.set(x);
        phistd::random::unit_interval(out)
    }

    /// Samples an index from `scores` under a softmax at `temperature`.
    ///
    /// Temperature previously scaled a **fixed sinusoid of the step index**, so
    /// the same prompt always produced the same output and "temperature"
    /// controlled the amplitude of a deterministic wobble rather than the
    /// entropy of a distribution. This is real sampling; `deterministic()`
    /// remains available when reproducibility is what is wanted.
    pub(super) fn sample_index(&self, scores: &[f64]) -> Option<usize> {
        phistd::prob::sample_softmax(scores, self.temperature, || self.next_uniform())
    }

}
