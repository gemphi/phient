//! Forgetting how far the learner has stepped, not what it has learned.
//!
//! Adagrad divides every step by the root of the squared gradients a row
//! has seen, so a learner resumed after a hundred thousand lines takes very
//! small steps: it is sure. Told something new, or asked to go on at a
//! higher rate, it may want its stride back. `forget` keeps every phasor
//! and scales every accumulator by a share — 0 is a fresh stride, 1 is no
//! change — so the next steps are as large as a learner that had seen only
//! that share of its history.

use super::Learner;

impl Learner {
    /// Scales every Adagrad accumulator by `share`, clamped to `0..=1`. The
    /// phasors, the biases and the lines told are untouched.
    pub fn forget(&mut self, share: f32) {
        let share = share.clamp(0.0, 1.0);
        for plane in [&mut self.reach, &mut self.reached, &mut self.cond, &mut self.op, &mut self.pair] {
            for a in &mut plane.acc {
                *a *= share;
            }
        }
        for a in &mut self.bias_acc {
            *a *= share;
        }
    }
}
