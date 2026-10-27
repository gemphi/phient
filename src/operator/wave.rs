//! A reading as a wave (docs/153). Phient's learned readers answer `w ⇒ ?`
//! by a matched filter: an output function `q` correlated with every word's
//! function, `⟨q, a_v⟩ / (|q| |a_v|)`. The real part is the cosine they are
//! trained on, phase and amplitude together; the modulus is the amplitude
//! alone, what an envelope detector keeps.
//!
//! - [`FnoWave`], the Joint FNO (`RelationModel`): `q = F(a_w + a_kind)`.
//!   Read backwards — which word `w` came after — it has no inverse, so the
//!   filter is run from the other end: `F(a_u + a_next)` for every word `u`,
//!   one forward pass each, cached.
//! - [`MirrorWave`](super::MirrorWave), the mirror (`phiton::conjugate`):
//!   `q = Hⁿ a_w`, the kind an all-pass filter, read backwards by `H̄`.

use super::model::{kind_token, RelationModel};
use phical::{c64, normalise, Function};

/// A reader whose answers are complex matched filters over `values`.
pub trait Wave {
    /// The matched filter for the word `steps` places back, read `steps` on.
    fn ahead(&mut self, word: &str, steps: usize, values: &[String]) -> Option<Vec<c64>>;
    /// The relation read backwards: which of `values` the word `word` came after.
    fn back(&mut self, word: &str, values: &[String]) -> Option<Vec<c64>>;
    /// What a word `steps` back is read under, for tables.
    fn under(&self, steps: usize) -> String;
}

/// `⟨q, a⟩ / (|q| |a|)`.
pub fn inner(q: &[c64], a: &[c64]) -> c64 {
    let (n, m) = (q.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt(), a.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt());
    q.iter().zip(a).map(|(x, y)| x * y.conj()).sum::<c64>() / (n * m).max(1e-12)
}

/// The kind a word `steps` back is read under: `next`, `next2`, …
pub fn offset_kind(steps: usize) -> String {
    if steps == 1 { "next".into() } else { format!("next{steps}") }
}

/// The Joint FNO as a wave, with its table taken once and the backward
/// filter cached.
pub struct FnoWave<'a> {
    pub m: &'a mut RelationModel,
    table: Vec<Function>,
    backs: Option<Vec<Option<Function>>>,
}

impl<'a> FnoWave<'a> {
    pub fn new(m: &'a mut RelationModel) -> Self {
        let table = m.joint.functions();
        FnoWave { m, table, backs: None }
    }

    /// `phical::Fno::defects` over `n` of the model's words read under `next`.
    pub fn defects(&mut self, n: usize) -> Option<(f64, f64)> {
        let k = *self.m.index.get(&kind_token("next"))?;
        let inputs: Vec<Function> = (0..self.table.len())
            .filter(|&i| !self.m.names[i].starts_with('⟨'))
            .take(n)
            .map(|w| {
                let mut x: Function = self.table[w].iter().zip(&self.table[k]).map(|(p, q)| p + q).collect();
                normalise(&mut x);
                x
            })
            .collect();
        Some(self.m.joint.fno.defects(&inputs, 1.0))
    }

    fn at(&self, values: &[String]) -> Vec<Option<usize>> {
        values.iter().map(|v| self.m.index.get(v).copied()).collect()
    }
}

impl Wave for FnoWave<'_> {
    fn ahead(&mut self, word: &str, steps: usize, values: &[String]) -> Option<Vec<c64>> {
        let (w, k) = (*self.m.index.get(word)?, *self.m.index.get(&kind_token(&offset_kind(steps)))?);
        let y = self.m.output_at(&self.table, w, k);
        Some(self.at(values).iter().map(|i| i.map_or(c64::new(-1.0, 0.0), |i| inner(&y, &self.table[i]))).collect())
    }

    fn back(&mut self, word: &str, values: &[String]) -> Option<Vec<c64>> {
        let (v, k) = (*self.m.index.get(word)?, *self.m.index.get(&kind_token("next"))?);
        if self.backs.is_none() {
            let at = self.at(values);
            self.backs = Some(at.iter().map(|i| i.map(|u| self.m.output_at(&self.table, u, k))).collect());
        }
        let backs = self.backs.as_ref()?;
        Some(backs.iter().map(|y| y.as_ref().map_or(c64::new(-1.0, 0.0), |y| inner(y, &self.table[v]))).collect())
    }

    fn under(&self, steps: usize) -> String {
        offset_kind(steps)
    }
}
