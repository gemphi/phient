//! Dropping the words a learner is no longer told.
//!
//! A word is learned because it was told, and the reader decides what a
//! word is. When that rule changes — the guide's exercise numbers stopped
//! being words at phiano docs/237 — a resumed learner still holds every
//! word it ever learned, its angles, its bias and its count, and can still
//! say one back. The stride decays the accumulators, not the angles, so
//! nothing takes such a word out by itself (phiano docs/238).
//!
//! [`Learner::drop_words`] takes them out: the row of every per-word plane,
//! the bias with its clock, the count, the name, and the word's place in
//! every line told, with every other word's angles kept bit for bit. The
//! markers stay whatever the rule says of them. The index, the draw of
//! wrong answers and its `ln q` are derived, so they are thrown away and
//! built again rather than patched.

use super::{Learner, FIRST};

impl Learner {
    /// Drops every word `gone` no longer calls a word, and returns how
    /// many went. The lines told keep their order, each without the
    /// dropped words, so the epochs go on over what is left.
    pub fn drop_words(&mut self, gone: impl Fn(&str) -> bool) -> usize {
        let keep: Vec<bool> = self.word.iter().enumerate().map(|(i, w)| i < FIRST as usize || !gone(w)).collect();
        let dropped = keep.iter().filter(|k| !**k).count();
        if dropped == 0 {
            return 0;
        }
        // Where each word goes, and `None` for a word that goes nowhere.
        let mut to: Vec<Option<u32>> = vec![None; keep.len()];
        let mut next = 0;
        for (i, _) in keep.iter().enumerate().filter(|(_, k)| **k) {
            to[i] = Some(next);
            next += 1;
        }
        self.reach.keep_rows(&keep);
        self.reached.keep_rows(&keep);
        self.cond.keep_rows(&keep);
        keep_in(&mut self.bias, &keep);
        keep_in(&mut self.bias_acc, &keep);
        keep_in(&mut self.bias_last, &keep);
        keep_in(&mut self.seen, &keep);
        keep_in(&mut self.word, &keep);
        self.index = self.word.iter().enumerate().map(|(i, w)| (w.clone(), i as u32)).collect();
        for line in self.told.iter_mut() {
            *line = line.iter().filter_map(|v| to[*v as usize]).collect();
        }
        self.sampler.clear();
        self.logq.clear();
        self.sampled = 0;
        dropped
    }
}

/// The entries `keep` says stay, in order.
fn keep_in<T>(v: &mut Vec<T>, keep: &[bool]) {
    let mut i = 0;
    v.retain(|_| {
        i += 1;
        keep[i - 1]
    });
}
