//! Reading candidates off the guide: proposals, the nearest pool words, a word's cosine.

use super::{OperatorGuide, OPERATOR_PROPOSAL_COUNT};

impl OperatorGuide {
    /// The `k` words whose table functions are closest to the prediction,
    /// among words seen at least `floor` times — the operator *proposing*
    /// candidates rather than only re-weighting the n-gram's. Returned with
    /// a pseudo-count so they can join an n-gram candidate list.
    pub(in crate::generate) fn propose(&mut self, k: usize, floor: u32) -> Vec<(String, u32)> {
        let y = match self.predict() {
            Some(y) => y,
            None => return Vec::new(),
        };
        let table = self.space.table();
        // Prior-corrected: how much better than usual this word fits here.
        // (Measured: over-corrects into the rare tail — docs/72. Kept for the
        // record; `Generator::relational_propose` is what ships.)
        let mut scored: Vec<(usize, f64)> = (0..self.space.len())
            .filter(|&i| self.space.counts[i] >= floor)
            .map(|i| (i, phistd::cosine(&y, &table[i]) - self.prior.get(i).copied().unwrap_or(0.0)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(k)
            .map(|(i, _)| {
                // A frequent word needs no help from a pseudo-count; a rare
                // one does. Decays from the full count to 1 over ~1,000 uses.
                let freq = self.space.counts[i] as f64;
                let c = (OPERATOR_PROPOSAL_COUNT as f64 / (1.0 + (freq / 100.0).ln_1p())).round().max(1.0) as u32;
                (self.space.words[i].clone(), c)
            })
            .collect()
    }

    /// The `k` pool words nearest the prediction by plain cosine — no prior
    /// correction, no pseudo-count. This is *decoding from the operator*: the
    /// candidate set comes from the function space, and the scorer (counts,
    /// relations, the learned weights) chooses among them.
    pub fn nearest(&mut self, k: usize, floor: u32) -> Vec<String> {
        let y = match self.predict() {
            Some(y) => y,
            None => return Vec::new(),
        };
        let table = self.space.table();
        let mut scored: Vec<(usize, f64)> = (0..self.space.len())
            .filter(|&i| self.space.counts[i] >= floor)
            .map(|i| (i, phistd::cosine(&y, &table[i])))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(i, _)| self.space.words[i].clone()).collect()
    }

    /// The `k` pool words chosen by Liskov's three operators rather than by
    /// fit alone (`phical::liskov`): from the `2k` nearest, each candidate is
    /// scored by its cosine now less `lambda` times its *refutation* — how far
    /// the best continuation the operator predicts after it falls below its
    /// own fit. A word that fits and leads nowhere loses to one that fits a
    /// little less and leads somewhere.
    pub fn nearest_forcing(&mut self, k: usize, floor: u32, lambda: f64) -> Vec<String> {
        let y = match self.predict() {
            Some(y) => y,
            None => return Vec::new(),
        };
        let pool: Vec<usize> = (0..self.space.len()).filter(|&i| self.space.counts[i] >= floor).collect();
        let mut scored: Vec<(usize, f64)> = pool.iter().map(|&i| (i, phistd::cosine(&y, &self.space.table()[i]))).collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(2 * k);
        let ids: Vec<usize> = self.history.iter().filter_map(|w| self.space.index.get(w).copied()).collect();
        let continuations: Vec<phistd::Function> = pool.iter().map(|&i| self.space.table()[i].clone()).collect();
        let candidates: Vec<(usize, phistd::Function)> = scored.iter().map(|(i, _)| (*i, self.space.table()[*i].clone())).collect();
        let mut ranked: Vec<(usize, f64)> = candidates
            .iter()
            .map(|(i, f)| {
                let mut ext = ids.clone();
                ext.push(*i);
                let after = self.space.context(&ext, ext.len(), self.window).and_then(|c| self.fno.apply(&[c]).into_iter().next());
                let refuted = after.map(|a| phical::liskov::worse(&y, f, &a, &continuations)).unwrap_or(0.0);
                (*i, phistd::cosine(&y, f) - lambda * refuted)
            })
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.into_iter().take(k).map(|(i, _)| self.space.words[i].clone()).collect()
    }

    /// Cosine of `word`'s table function against the prediction, or `None`
    /// when the space does not know the word.
    pub(in crate::generate) fn cosine(&mut self, word: &str) -> Option<f64> {
        let y = self.predict()?;
        let i = *self.space.index.get(word)?;
        Some(phistd::cosine(&y, &self.space.table()[i]))
    }
}
