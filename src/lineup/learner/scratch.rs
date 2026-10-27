//! The step's scratch space and the two small complex helpers.

/// The word tables a context gradient lands on.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Table {
    Reach,
    Cond,
}

/// Scratch for one training step, kept so a step allocates little.
#[derive(Default)]
pub(super) struct Scratch {
    pub(super) zr: Vec<f32>,
    pub(super) zi: Vec<f32>,
    /// The silence mask on the condition's channels: 0, or the keep scale.
    pub(super) mask: Vec<f32>,
    pub(super) gr: Vec<f32>,
    pub(super) gi: Vec<f32>,
    pub(super) tr: Vec<f32>,
    pub(super) ti: Vec<f32>,
    pub(super) cand: Vec<u32>,
    pub(super) logit: Vec<f32>,
    /// Gradients for the context word rows: (table, row, re, im).
    pub(super) grad: Vec<(Table, u32, Vec<f32>, Vec<f32>)>,
    /// Gradients for the operator rows, in row order.
    pub(super) op_r: Vec<f32>,
    pub(super) op_i: Vec<f32>,
    pub(super) pair_r: Vec<f32>,
    pub(super) pair_i: Vec<f32>,
}

/// Adds the gradient `(re, im)` to `t`'s row `row`, gathered so a row met
/// twice in one step (a word at two distances; `<s>` often) moves once,
/// by the sum.
pub(super) fn put(grad: &mut Vec<(Table, u32, Vec<f32>, Vec<f32>)>, t: Table, row: u32, re: &[f32], im: &[f32]) {
    let at = match grad.iter().position(|(gt, gw, ..)| *gt == t && *gw == row) {
        Some(at) => at,
        None => {
            grad.push((t, row, vec![0.0; re.len()], vec![0.0; re.len()]));
            grad.len() - 1
        }
    };
    let (_, _, gr, gi) = &mut grad[at];
    for k in 0..re.len() {
        gr[k] += re[k];
        gi[k] += im[k];
    }
}

/// `g · conj(x)`, channel by channel, into `(out_r, out_i)`.
pub(super) fn times_conj(gr: &[f32], gi: &[f32], xr: &[f32], xi: &[f32], out_r: &mut [f32], out_i: &mut [f32]) {
    for k in 0..gr.len() {
        out_r[k] = gr[k] * xr[k] + gi[k] * xi[k];
        out_i[k] = gi[k] * xr[k] - gr[k] * xi[k];
    }
}
