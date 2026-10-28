//! The operator as a language-model scorer.
//!
//! `PhianoLM` ranks the vocabulary by mean phase coherence with a context
//! vector, softmaxes that, and mixes it with a unigram under a trigram affine.
//! [`OperatorScorer`] supplies the same kind of score from a different place:
//! the context's *function* on the relation circle, passed through a trained
//! [`Fno`], compared by cosine against every word's function. Plugged into
//! [`crate::metrics::harness::PhianoLM::gamma_sweep_scored`], it is measured
//! on exactly the footing the manifold's own coherence is — same β, same
//! affine, same γ grid, same held-out sentences.

use phical::Fno;
use crate::net::{FunctionSpace, WordFunction};
use phiton::units::PHASE_CHANNELS;

/// Scores every vocabulary item against `Fno(context)`.
pub struct OperatorScorer<'a> {
    pub fno: &'a mut Fno,
    pub space: &'a FunctionSpace,
    pub window: usize,
    /// Word functions flattened to `(Re, Im)` pairs, unit norm, for the dot.
    flat: Vec<f64>,
    /// Vocabulary index (LM order) → function-space index, or `None`.
    map: Vec<Option<usize>>,
}

impl<'a> OperatorScorer<'a> {
    /// `vocab` is the language model's vocabulary in its index order.
    pub fn new(fno: &'a mut Fno, space: &'a FunctionSpace, vocab: &[String], window: usize) -> Self {
        let map: Vec<Option<usize>> = vocab.iter().map(|w| space.index.get(w).copied()).collect();
        // The table words are ranked against: target sides for a directed
        // space, the functions themselves otherwise.
        let mut flat = Vec::with_capacity(space.len() * 2 * PHASE_CHANNELS);
        for f in space.table() {
            for z in f {
                flat.push(z.re);
                flat.push(z.im);
            }
        }
        Self { fno, space, window, flat, map }
    }

    /// Cosine of `y` against every vocabulary item, in LM index order.
    /// Words the space does not know score the mean, so they are neutral.
    fn cosines(&self, y: &WordFunction) -> Vec<f64> {
        let ny: f64 = y.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt().max(1e-12);
        let mut out = Vec::with_capacity(self.map.len());
        let mut sum = 0.0;
        let mut n = 0usize;
        for m in &self.map {
            match m {
                Some(i) => {
                    let base = i * 2 * PHASE_CHANNELS;
                    let mut dot = 0.0;
                    for (k, yk) in y.iter().enumerate().take(PHASE_CHANNELS) {
                        // Re(y · conj(f)) = y.re·f.re + y.im·f.im
                        dot += yk.re * self.flat[base + 2 * k] + yk.im * self.flat[base + 2 * k + 1];
                    }
                    let c = dot / ny;
                    sum += c;
                    n += 1;
                    out.push(c);
                }
                None => out.push(f64::NAN),
            }
        }
        let mean = match n {
            0 => 0.0,
            _ => sum / n as f64,
        };
        for v in out.iter_mut() {
            if v.is_nan() {
                *v = mean;
            }
        }
        out
    }

    /// Scores for the word at position `i` of `toks`, from the prefix.
    pub fn score(&mut self, toks: &[String], i: usize) -> Option<Vec<f64>> {
        let ids: Vec<usize> = toks[..i].iter().filter_map(|w| self.space.index.get(w).copied()).collect();
        if ids.is_empty() {
            return None;
        }
        let ctx = self.space.context(&ids, ids.len(), self.window)?;
        let y = self.fno.apply(&[ctx]).into_iter().next()?;
        Some(self.cosines(&y))
    }

    /// The identity baseline: the context function itself, no operator.
    pub fn score_identity(&self, toks: &[String], i: usize) -> Option<Vec<f64>> {
        let ids: Vec<usize> = toks[..i].iter().filter_map(|w| self.space.index.get(w).copied()).collect();
        if ids.is_empty() {
            return None;
        }
        let ctx = self.space.context(&ids, ids.len(), self.window)?;
        Some(self.cosines(&ctx))
    }
}
