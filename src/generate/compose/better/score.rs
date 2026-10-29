#[allow(unused_imports)]
use super::*;

impl SectorScore {
    /// Huberman's `better(p, q)`: advance a stage, or improve the measure within
    /// the current one.
    ///
    /// `better(p,q) = st(q) > st(p) ∨ [st(q) = st(p) ∧ m(q) < m(p)]`  (thesis 3.6)
    pub fn better_than(&self, other: &Self) -> bool {
        self.stage > other.stage || (self.stage == other.stage && self.score > other.score)
    }

    /// Huberman's `worse(p, q)` — deliberately *not* the negation of `better`.
    ///
    /// Stage zero is unconditionally worse (thesis 3.8: `st(q) = 0 ∨ …`). In the
    /// original this is a search guard: the tree is never expanded through a
    /// worse position. Here it is a survival guard: a stage-0 variant is never a
    /// survivor, whatever its fine score.
    pub fn is_worse(&self) -> bool {
        self.stage == 0
    }
}

impl KillerWord {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the content words of a winning composition.
    pub fn record_winner(&mut self, text: &str) {
        for w in Tokenizer::content_words(text) {
            *self.hits.entry(w).or_insert(0) += 1;
        }
    }

    /// Multiplicative bonus for a composition containing previously-winning words.
    pub fn bonus(&self, text: &str) -> f64 {
        if self.hits.is_empty() {
            return 1.0;
        }
        let words = Tokenizer::content_words(text);
        if words.is_empty() {
            return 1.0;
        }
        let hit: u32 = words.iter().filter_map(|w| self.hits.get(w)).sum();
        1.0 + config::KILLER_BONUS * (hit as f64).ln_1p() / (words.len() as f64).sqrt()
    }
}
