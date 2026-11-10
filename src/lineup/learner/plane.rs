//! A table of phasors and its Adagrad step.
//!
//! The accumulator is a sum of squared gradients, so the step shrinks for
//! ever as a row is told more. With a stride (`Config::stride`) the sum
//! decays by the lines told since the row was last touched, lazily, at the
//! touch: `acc *= exp(decay · (now − last))`, where `decay` is
//! `ln(0.1) / stride`. The same as scaling every accumulator by a tenth
//! once a stride, which is what `forget` does by hand (phiano docs/231).

/// One table of phasors: rows of `d` complex numbers, real and imaginary
/// parts apart (so the channel loops vectorise), with one Adagrad
/// accumulator per row.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Plane {
    pub(super) d: usize,
    pub(super) re: Vec<f32>,
    pub(super) im: Vec<f32>,
    pub(super) acc: Vec<f32>,
    /// Lines told when each row was last touched: the clock the stride
    /// decays by.
    pub(super) last: Vec<u32>,
}

impl Plane {
    pub(super) fn new(d: usize) -> Self {
        Self { d, re: Vec::new(), im: Vec::new(), acc: Vec::new(), last: Vec::new() }
    }

    pub(super) fn push(&mut self, angles: impl Iterator<Item = f64>, amplitude: f32) {
        for a in angles {
            self.re.push(amplitude * a.cos() as f32);
            self.im.push(amplitude * a.sin() as f32);
        }
        self.acc.push(0.0);
        self.last.push(0);
    }

    /// Every row counted as touched now, so nothing decays for lines it
    /// was never clocked over (a widened or a version-1 learner).
    pub(super) fn touch_all(&mut self, now: u32) {
        self.last.clear();
        self.last.resize(self.acc.len(), now);
    }

    pub(super) fn row(&self, i: u32) -> (&[f32], &[f32]) {
        let s = i as usize * self.d;
        (&self.re[s..s + self.d], &self.im[s..s + self.d])
    }

    /// One Adagrad step on row `i` against the gradient `(gr, gi)`, at
    /// `now` lines told with the stride's `decay` (`Config::decay`). With
    /// `circle`, every entry goes back on the unit circle after the step.
    pub(super) fn learn(&mut self, i: u32, gr: &[f32], gi: &[f32], rate: f32, circle: bool, now: u32, decay: f32) {
        let s = i as usize * self.d;
        let sq = gr.iter().zip(gi).map(|(a, b)| a * a + b * b).sum::<f32>() / self.d as f32;
        self.acc[i as usize] = decayed(self.acc[i as usize], &mut self.last[i as usize], now, decay) + sq;
        let step = rate / (self.acc[i as usize] + 1e-8).sqrt();
        for k in 0..self.d {
            let (mut r, mut m) = (self.re[s + k] - step * gr[k], self.im[s + k] - step * gi[k]);
            if circle {
                let n = (r * r + m * m).sqrt().max(1e-12);
                r /= n;
                m /= n;
            }
            self.re[s + k] = r;
            self.im[s + k] = m;
        }
    }
}

/// An accumulator decayed by the lines told since its row was last touched,
/// and the row touched now.
pub(super) fn decayed(acc: f32, last: &mut u32, now: u32, decay: f32) -> f32 {
    let gap = now.saturating_sub(*last);
    *last = now;
    match decay != 0.0 && gap > 0 {
        true => acc * (decay * gap as f32).exp(),
        false => acc,
    }
}
