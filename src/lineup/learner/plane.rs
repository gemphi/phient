//! A table of phasors and its Adagrad step.

/// One table of phasors: rows of `d` complex numbers, real and imaginary
/// parts apart (so the channel loops vectorise), with one Adagrad
/// accumulator per row.
#[derive(Clone)]
pub(super) struct Plane {
    pub(super) d: usize,
    pub(super) re: Vec<f32>,
    pub(super) im: Vec<f32>,
    pub(super) acc: Vec<f32>,
}

impl Plane {
    pub(super) fn new(d: usize) -> Self {
        Self { d, re: Vec::new(), im: Vec::new(), acc: Vec::new() }
    }

    pub(super) fn push(&mut self, angles: impl Iterator<Item = f64>, amplitude: f32) {
        for a in angles {
            self.re.push(amplitude * a.cos() as f32);
            self.im.push(amplitude * a.sin() as f32);
        }
        self.acc.push(0.0);
    }

    pub(super) fn row(&self, i: u32) -> (&[f32], &[f32]) {
        let s = i as usize * self.d;
        (&self.re[s..s + self.d], &self.im[s..s + self.d])
    }

    /// One Adagrad step on row `i` against the gradient `(gr, gi)`. With
    /// `circle`, every entry goes back on the unit circle after the step.
    pub(super) fn learn(&mut self, i: u32, gr: &[f32], gi: &[f32], rate: f32, circle: bool) {
        let s = i as usize * self.d;
        let sq = gr.iter().zip(gi).map(|(a, b)| a * a + b * b).sum::<f32>() / self.d as f32;
        self.acc[i as usize] += sq;
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
