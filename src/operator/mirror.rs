//! The phase-conjugate mirror (`phical::conjugate::Mirror`) as a [`Wave`]
//! (docs/153): `n` steps on under `next{n}` when the mirror was trained on
//! that kind and `powers` is off, else under `Hⁿ`; backwards under `H̄`.

use super::wave::{inner, offset_kind, Wave};
use phical::c64;
use phical::conjugate::Mirror;

pub struct MirrorWave {
    pub m: Mirror,
    pub powers: bool,
    cols: Option<Vec<Option<usize>>>,
}

impl MirrorWave {
    pub fn new(m: Mirror, powers: bool) -> Self {
        MirrorWave { m, powers, cols: None }
    }

    fn filter(&mut self, q: Vec<c64>, values: &[String]) -> Vec<c64> {
        let m = &self.m;
        let cols = self.cols.get_or_insert_with(|| values.iter().map(|v| m.index.get(v).copied()).collect());
        cols.iter().map(|i| i.map_or(c64::new(-1.0, 0.0), |i| inner(&q, &m.spectra[i]))).collect()
    }

    /// The kind and power a word `steps` back is read under.
    fn step(&self, steps: usize) -> Option<(usize, i32)> {
        match (self.powers, self.m.kind(&offset_kind(steps))) {
            (false, Some(k)) => Some((k, 1)),
            _ => Some((self.m.kind("next")?, steps as i32)),
        }
    }
}

impl Wave for MirrorWave {
    fn ahead(&mut self, word: &str, steps: usize, values: &[String]) -> Option<Vec<c64>> {
        let ((k, p), w) = (self.step(steps)?, *self.m.index.get(word)?);
        let q = self.m.wave(w, k, p);
        Some(self.filter(q, values))
    }

    fn back(&mut self, word: &str, values: &[String]) -> Option<Vec<c64>> {
        let (k, w) = (self.m.kind("next")?, *self.m.index.get(word)?);
        let q = self.m.wave(w, k, -1);
        Some(self.filter(q, values))
    }

    fn under(&self, steps: usize) -> String {
        match self.step(steps) {
            Some((_, p)) if p > 1 => format!("H{}", ["", "", "²", "³", "⁴"].get(p as usize).unwrap_or(&"ⁿ")),
            _ => offset_kind(steps),
        }
    }
}
