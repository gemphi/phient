//! The learner as an arena reader.

use super::scratch::Scratch;
use super::{Learner, END};
use crate::study::Reader;

impl Reader for Learner {
    fn name(&self) -> &str {
        "phient learner (phasors corrected by their error)"
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        if self.vocabulary() == 0 {
            return None;
        }
        self.ranked(said, 1).into_iter().next().map(|(w, _)| w)
    }

    /// Told a line, the learner corrects itself on it at once — one step per
    /// position, on the fly — and keeps it for the epochs of [`Reader::ready`].
    fn tell(&mut self, words: &[String]) {
        let ids: Vec<u32> = words.iter().map(|w| self.intern(w)).collect();
        for &i in &ids {
            self.seen[i as usize] += 1;
        }
        self.seen[END as usize] += 1;
        self.told.push(ids);
        self.fresh_sampler();
        let line = self.told.len() - 1;
        let mut s = Scratch::default();
        for (ctx, target) in self.positions(line) {
            self.step(&ctx, target, &mut s);
        }
    }

    /// The arena's training half is over: the epochs, on everything told.
    fn ready(&mut self) {
        for _ in 0..self.config.epochs {
            self.epoch();
        }
    }
}
