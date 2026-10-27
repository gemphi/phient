//! One prediction and its correction: the gradient, by hand.

use super::scratch::{put, times_conj, Scratch, Table};
use super::Learner;

impl Learner {
    /// One prediction and its correction. Returns −ln p of the word that
    /// came, among the sampled ones.
    ///
    /// With `g_j = p_j − [j is the word that came]` on each sampled
    /// candidate: the candidate's answer row moves by `g_j · z` and its bias
    /// by `g_j`; the condition's gradient is `G = Σ_j g_j · answer_j`; and
    /// for any product `z ∋ a·x` the gradient of `x` is `G · conj(a)` — so
    /// each context word's `reach` row moves by `Σ_i conj(op_d[i][j]) ·
    /// G_i`, each operator entry by `G_i · conj(reach_j)`, and the pair's
    /// three factors each by `G` times the conjugate of the other two. All
    /// of it is read from the rows as they stood before any moves.
    pub(super) fn step(&mut self, ctx: &[u32], target: u32, s: &mut Scratch) -> f32 {
        let (d, b) = (self.config.channels, self.config.block);
        for v in [&mut s.zr, &mut s.zi, &mut s.gr, &mut s.gi, &mut s.tr, &mut s.ti, &mut s.mask] {
            v.clear();
            v.resize(d, 0.0);
        }
        let (mut zr, mut zi) = (std::mem::take(&mut s.zr), std::mem::take(&mut s.zi));
        self.condition(ctx, &mut zr, &mut zi);
        // Silence: a channel of the condition drops out, the rest are
        // scaled up to keep the reading's size.
        let keep = 1.0 - self.config.silence;
        for k in 0..d {
            s.mask[k] = match self.config.silence > 0.0 && self.chance() < self.config.silence {
                true => 0.0,
                false => 1.0 / keep,
            };
            zr[k] *= s.mask[k];
            zi[k] *= s.mask[k];
        }
        s.cand.clear();
        s.cand.push(target);
        for _ in 0..self.config.negatives {
            let r = self.next_random();
            let v = self.sampler[(r % self.sampler.len() as u64) as usize];
            if v != target {
                s.cand.push(v);
            }
        }
        let ln_k = (self.config.negatives as f32).ln();
        s.logit.clear();
        for &v in &s.cand {
            let lq = self.logq.get(v as usize).copied().unwrap_or(self.fresh_logq);
            s.logit.push(self.score(&zr, &zi, v) - (ln_k + lq));
        }
        let top = s.logit.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let sum: f32 = s.logit.iter().map(|l| (l - top).exp()).sum();
        let loss = -(s.logit[0] - top - sum.ln());
        // G, from the answer rows as they stood; the logits become the g_j.
        for j in 0..s.cand.len() {
            let g = (s.logit[j] - top).exp() / sum - if j == 0 { 1.0 } else { 0.0 };
            let (tr, ti) = self.answers().row(s.cand[j]);
            for k in 0..d {
                s.gr[k] += g * tr[k];
                s.gi[k] += g * ti[k];
            }
            s.logit[j] = g;
        }
        // Through the silence mask, back to the unmasked condition.
        for k in 0..d {
            s.gr[k] *= s.mask[k];
            s.gi[k] *= s.mask[k];
        }
        // The context rows and the operator, gathered from the rows as
        // they stood.
        s.grad.clear();
        s.op_r.clear();
        s.op_r.resize(ctx.len() * d * b, 0.0);
        s.op_i.clear();
        s.op_i.resize(ctx.len() * d * b, 0.0);
        let (gr, gi) = (std::mem::take(&mut s.gr), std::mem::take(&mut s.gi));
        for (dist, &w) in ctx.iter().enumerate() {
            let (ar, ai) = self.reach.row(w);
            s.tr.fill(0.0);
            s.ti.fill(0.0);
            for i in 0..d {
                let (mr, mi) = self.op.row((dist * d + i) as u32);
                let base = i - i % b;
                let at = (dist * d + i) * b;
                for j in 0..b {
                    // reach_j += conj(op_ij) · G_i ; op_ij += G_i · conj(reach_j)
                    s.tr[base + j] += mr[j] * gr[i] + mi[j] * gi[i];
                    s.ti[base + j] += mr[j] * gi[i] - mi[j] * gr[i];
                    s.op_r[at + j] = gr[i] * ar[base + j] + gi[i] * ai[base + j];
                    s.op_i[at + j] = gi[i] * ar[base + j] - gr[i] * ai[base + j];
                }
            }
            put(&mut s.grad, Table::Reach, w, &s.tr, &s.ti);
        }
        let paired = self.config.pair && ctx.len() >= 2;
        if paired {
            let ((ar, ai), (cr, ci), (ur, ui)) = (self.reach.row(ctx[0]), self.cond.row(ctx[1]), self.pair.row(0));
            let (mut xr, mut xi) = (vec![0.0f32; d], vec![0.0f32; d]);
            // The pair's three factors, each by G times the other two, conjugated.
            let factors = [(None, (ar, ai), (cr, ci)), (Some((Table::Reach, ctx[0])), (ur, ui), (cr, ci)), (Some((Table::Cond, ctx[1])), (ur, ui), (ar, ai))];
            for (to, x, y) in factors {
                for k in 0..d {
                    xr[k] = x.0[k] * y.0[k] - x.1[k] * y.1[k];
                    xi[k] = x.0[k] * y.1[k] + x.1[k] * y.0[k];
                }
                times_conj(&gr, &gi, &xr, &xi, &mut s.tr, &mut s.ti);
                match to {
                    Some((t, row)) => put(&mut s.grad, t, row, &s.tr, &s.ti),
                    None => {
                        s.pair_r.clone_from(&s.tr);
                        s.pair_i.clone_from(&s.ti);
                    }
                }
            }
        }
        // Now every move. The answers first: `g_j · z`, masked as read.
        let rate = self.config.rate;
        for j in 0..s.cand.len() {
            let (v, g) = (s.cand[j], s.logit[j]);
            for k in 0..d {
                s.tr[k] = g * zr[k];
                s.ti[k] = g * zi[k];
            }
            match self.config.tie {
                true => self.reach.learn(v, &s.tr, &s.ti, rate, true),
                false => self.reached.learn(v, &s.tr, &s.ti, rate, true),
            }
            let a = &mut self.bias_acc[v as usize];
            *a += g * g;
            self.bias[v as usize] -= rate * g / (*a + 1e-8).sqrt();
        }
        // The operator, a block's worth of channels at a time slower, so a
        // wide block does not move its output faster than a single turn.
        let op_rate = rate / b as f32;
        for row in 0..ctx.len() * d {
            let at = row * b;
            self.op.learn(row as u32, &s.op_r[at..at + b], &s.op_i[at..at + b], op_rate, false);
        }
        if paired {
            self.pair.learn(0, &s.pair_r, &s.pair_i, rate, false);
        }
        for (t, row, re, im) in std::mem::take(&mut s.grad) {
            match t {
                Table::Reach => self.reach.learn(row, &re, &im, rate, true),
                Table::Cond => self.cond.learn(row, &re, &im, rate, true),
            }
        }
        (s.gr, s.gi, s.zr, s.zi) = (gr, gi, zr, zi);
        loss
    }
}
