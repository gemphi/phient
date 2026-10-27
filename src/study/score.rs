//! `Score` — one held-out reading, the same numbers for every model. At each
//! position only two things are known: where the word that came was ranked,
//! and what probability it was given. Top-1, top-5, mean reciprocal rank and
//! perplexity all come from those two, so they are counted once, here
//! (docs/135, docs/203), whatever the model.

/// What a reading of held-out positions scored.
#[derive(Clone, Copy, Debug, Default)]
pub struct Score {
    /// Positions asked.
    pub asked: usize,
    pub top1: usize,
    pub top5: usize,
    /// Summed reciprocal rank; a word not ranked at all adds nothing.
    pub rr: f64,
    /// Positions that carried a probability.
    pub known: usize,
    /// Summed −ln p over the known positions.
    pub nll: f64,
}

impl Score {
    /// One position asked: the rank of the word that came (1 is first), or
    /// `None` when the model did not rank it at all.
    pub fn rank(&mut self, rank: Option<usize>) {
        self.asked += 1;
        self.top1 += (rank == Some(1)) as usize;
        self.top5 += matches!(rank, Some(r) if r <= 5) as usize;
        self.rr += rank.map_or(0.0, |r| 1.0 / r as f64);
    }

    /// The probability that position gave the word that came, as −ln p.
    pub fn prob(&mut self, nll: f64) {
        self.known += 1;
        self.nll += nll;
    }

    /// Two readings of different positions, as one.
    pub fn merge(self, o: Self) -> Self {
        Self {
            asked: self.asked + o.asked,
            top1: self.top1 + o.top1,
            top5: self.top5 + o.top5,
            rr: self.rr + o.rr,
            known: self.known + o.known,
            nll: self.nll + o.nll,
        }
    }

    /// Top-1, percent of the positions asked.
    pub fn top1(&self) -> f64 {
        100.0 * self.top1 as f64 / self.asked.max(1) as f64
    }

    /// Top-5, percent of the positions asked.
    pub fn top5(&self) -> f64 {
        100.0 * self.top5 as f64 / self.asked.max(1) as f64
    }

    pub fn mrr(&self) -> f64 {
        self.rr / self.asked.max(1) as f64
    }

    /// Perplexity over the positions that carried a probability.
    pub fn perplexity(&self) -> f64 {
        (self.nll / self.known.max(1) as f64).exp()
    }

    /// `| name | asked | top-1 | MRR | perplexity |`, the held-out table row.
    pub fn row(&self, name: &str) -> String {
        format!("| {name} | {} | {:.1}% | {:.3} | {:.1} |", self.asked, self.top1(), self.mrr(), self.perplexity())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rank_and_a_probability_are_all_it_needs() {
        let mut s = Score::default();
        s.rank(Some(1));
        s.prob(0.5f64.ln().abs());
        s.rank(Some(4));
        s.prob(0.25f64.ln().abs());
        s.rank(None);
        assert_eq!((s.asked, s.top1, s.top5, s.known), (3, 1, 2, 2));
        assert!((s.mrr() - (1.0 + 0.25) / 3.0).abs() < 1e-12);
        assert!((s.perplexity() - (1.0f64 / (0.5 * 0.25)).sqrt()).abs() < 1e-9);
        let both = s.merge(s);
        assert_eq!((both.asked, both.known), (6, 4));
        assert!((both.perplexity() - s.perplexity()).abs() < 1e-9);
    }
}
