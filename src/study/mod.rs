//! `study` — step-by-step learning, measured without fudging (docs/172).
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and
//! composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon.
//!
//! One module for every method — prose under concepts, code by depth, a
//! person's lines — the same honest protocol: **predict first, learn
//! after** (prequential), so every position is unseen when it is scored,
//! and the number cannot be fudged by testing on what was trained. The
//! stream is sliced in reading order into a curve; learning shows as the
//! curve rising, and the condition of satisfaction (docs/170) is fixed in
//! advance: beat twice the floor, and rise.

pub mod arena;
pub use arena::{Arena, Reader};

/// A study: predictions ticked in reading order, sliced into a curve.
pub struct Study {
    /// What is being learned, for the report line.
    pub subject: String,
    /// Chance: what a guesser scores knowing nothing (percent).
    pub floor: f64,
    pub positions: usize,
    pub hits: usize,
    slice: Vec<(usize, usize)>,
}

impl Study {
    pub fn new(subject: &str, slices: usize, floor: f64) -> Self {
        Self { subject: subject.into(), floor, positions: 0, hits: 0, slice: vec![(0, 0); slices.max(1)] }
    }

    /// One prediction, made BEFORE its item was learned, at progress `i` of
    /// `total` items through the stream.
    pub fn tick(&mut self, i: usize, total: usize, hit: bool) {
        let s = (i * self.slice.len() / total.max(1)).min(self.slice.len() - 1);
        self.slice[s].0 += 1;
        self.slice[s].1 += hit as usize;
        self.positions += 1;
        self.hits += hit as usize;
    }

    pub fn accuracy(&self) -> f64 {
        100.0 * self.hits as f64 / self.positions.max(1) as f64
    }

    /// Accuracy in each equal slice of the stream, in reading order.
    pub fn curve(&self) -> Vec<f64> {
        self.slice.iter().map(|(n, h)| 100.0 * *h as f64 / (*n).max(1) as f64).collect()
    }

    /// Last slice minus first: how much reading more helped.
    pub fn lift(&self) -> f64 {
        match self.curve().as_slice() {
            [first, .., last] => last - first,
            _ => 0.0,
        }
    }

    /// The condition of satisfaction, fixed in advance (docs/170): the
    /// whole stream beats twice the floor, and the curve rises.
    pub fn beats_floor(&self) -> bool {
        self.accuracy() > self.floor * 2.0
    }
    pub fn rises(&self) -> bool {
        self.lift() > 1.0
    }

    /// The report, straight: curve, overall, floor, and the verdict.
    pub fn report(&self) -> String {
        let curve = self.curve();
        let mut s = format!("## {} — {} positions, floor {:.1}%\n\n| slice | top-1 |\n|---|---|\n", self.subject, self.positions, self.floor);
        for (i, c) in curve.iter().enumerate() {
            s.push_str(&format!("| {:>2}/{} | {c:.1}% |\n", i + 1, curve.len()));
        }
        s.push_str(&format!("\noverall {:.1}% (floor {:.1}%), first {:.1}% → last {:.1}% ({:+.1} pts)\n{}\n",
            self.accuracy(), self.floor,
            curve.first().copied().unwrap_or(0.0), curve.last().copied().unwrap_or(0.0), self.lift(),
            self.verdict()));
        s
    }

    pub fn verdict(&self) -> &'static str {
        match (self.beats_floor(), self.rises()) {
            (true, true) => "MET: learns from examples and improves as it reads.",
            (true, false) => "PARTIAL: beats the floor but has saturated — feed more, or deepen.",
            (false, _) => "NOT MET — reported straight.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_is_honest() {
        let mut s = Study::new("toy", 4, 25.0);
        for i in 0..100 {
            s.tick(i, 100, i >= 50); // learns halfway through
        }
        assert_eq!(s.positions, 100);
        assert_eq!(s.accuracy(), 50.0);
        assert_eq!(s.curve(), vec![0.0, 0.0, 100.0, 100.0]);
        assert!(s.rises() && !s.beats_floor() || s.beats_floor());
        assert!(s.lift() == 100.0);
    }
}
