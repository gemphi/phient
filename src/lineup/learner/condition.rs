//! The condition `z` a context makes, and a candidate's score against it.

use super::Learner;

impl Learner {
    /// The condition `z` before a position, from its context.
    pub(super) fn condition(&self, ctx: &[u32], zr: &mut [f32], zi: &mut [f32]) {
        zr.fill(0.0);
        zi.fill(0.0);
        let (d, b) = (self.config.channels, self.config.block);
        for (dist, &w) in ctx.iter().enumerate() {
            let (ar, ai) = self.reach.row(w);
            for i in 0..d {
                let (mr, mi) = self.op.row((dist * d + i) as u32);
                let base = i - i % b;
                let (mut re, mut im) = (0.0f32, 0.0f32);
                for j in 0..b {
                    re += mr[j] * ar[base + j] - mi[j] * ai[base + j];
                    im += mr[j] * ai[base + j] + mi[j] * ar[base + j];
                }
                zr[i] += re;
                zi[i] += im;
            }
        }
        if self.config.pair && ctx.len() >= 2 {
            let ((ar, ai), (cr, ci), (ur, ui)) = (self.reach.row(ctx[0]), self.cond.row(ctx[1]), self.pair.row(0));
            for k in 0..d {
                let (pr, pi) = (ar[k] * cr[k] - ai[k] * ci[k], ar[k] * ci[k] + ai[k] * cr[k]);
                zr[k] += ur[k] * pr - ui[k] * pi;
                zi[k] += ur[k] * pi + ui[k] * pr;
            }
        }
    }

    pub(super) fn score(&self, zr: &[f32], zi: &[f32], v: u32) -> f32 {
        let (tr, ti) = self.answers().row(v);
        self.bias[v as usize] + zr.iter().zip(zi).zip(tr.iter().zip(ti)).map(|((a, b), (c, e))| a * c + b * e).sum::<f32>()
    }
}
