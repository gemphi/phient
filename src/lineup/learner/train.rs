//! The epochs: the draw of wrong answers, the positions of a line, a pass.

use super::scratch::Scratch;
use super::{Learner, END, FIRST, SAMPLER, START, UNKNOWN};

impl Learner {
    /// The draw of wrong answers: every real word and `</s>`, by count^¾.
    pub(super) fn build_sampler(&mut self) {
        let w: Vec<f64> = (0..self.word.len())
            .map(|i| match i as u32 {
                START | UNKNOWN => 0.0,
                _ => (self.seen[i] as f64).powf(0.75),
            })
            .collect();
        let total: f64 = w.iter().sum();
        self.sampler.clear();
        self.logq = w.iter().map(|x| ((x / total.max(1e-12)).max(1e-12)).ln() as f32).collect();
        self.fresh_logq = (1.0 / total.max(1.0)).ln() as f32;
        if total <= 0.0 {
            return;
        }
        let mut acc = 0.0;
        let mut i = 0usize;
        for s in 0..SAMPLER {
            let at = (s as f64 + 0.5) / SAMPLER as f64 * total;
            while acc + w[i] < at && i + 1 < w.len() {
                acc += w[i];
                i += 1;
            }
            self.sampler.push(i as u32);
        }
        self.sampled = self.word.len();
    }

    /// Every position of a told line, as (context, the word that came):
    /// the first word after `<s>`, each next word, and `</s>` at the end.
    /// A word told only once stands as `<unk>` in a context half the time,
    /// so the unknown word has angles of its own when a reading meets one;
    /// any other word, with the chance `dropout`.
    pub(super) fn positions(&mut self, line: usize) -> Vec<(Vec<u32>, u32)> {
        let ids = self.told[line].clone();
        (0..=ids.len())
            .map(|j| {
                let mut ctx = self.context(&ids, j);
                for c in ctx.iter_mut() {
                    let chance = match self.seen.get(*c as usize) {
                        _ if *c < FIRST => 0.0,
                        Some(1) => 0.5f32.max(self.config.dropout),
                        _ => self.config.dropout,
                    };
                    if self.chance() < chance {
                        *c = UNKNOWN;
                    }
                }
                (ctx, ids.get(j).copied().unwrap_or(END))
            })
            .collect()
    }

    pub(super) fn fresh_sampler(&mut self) {
        if self.sampler.is_empty() || self.word.len() > self.sampled + self.sampled / 20 {
            self.build_sampler();
        }
    }

    /// One pass over everything told, in a shuffled order. Returns the mean
    /// loss — the error the pass corrected.
    pub fn epoch(&mut self) -> f64 {
        self.build_sampler();
        let mut order: Vec<usize> = (0..self.told.len()).collect();
        for i in (1..order.len()).rev() {
            let j = (self.next_random() % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        let (mut total, mut n) = (0.0f64, 0usize);
        let mut s = Scratch::default();
        for line in order {
            for (ctx, target) in self.positions(line) {
                total += self.step(&ctx, target, &mut s) as f64;
                n += 1;
            }
        }
        total / n.max(1) as f64
    }
}
