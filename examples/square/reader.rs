//! The Phient reader. A word is what it relates to under conditions, and
//! here the condition is *where the reader stands*: the offset from the
//! cell the word begins at. Each word holds two kinds of relation, both
//! counted from episodes:
//!
//! - `width`: how many squares the word moves the reader on (1 to 3 — the
//!   reader is told only that a word covers squares next to each other in
//!   a row, not how many);
//! - `colour|o`: the colour of the square at offset `o` from where it began.
//!
//! No episode says where a word begins, so the counts are expected counts
//! over every way the sentence can lie on the grid (a segmentation), each
//! weighted by how well the current relations fit it — counted again until
//! they settle (EM). A word that names two squares ends up holding posts
//! under `colour|0`, `colour|1` and `width = 2`; §47–48's "simple or
//! composite" is read off its relations, not decided beforehand.

use super::world::{Episode, CELLS, COLOURS};
use std::collections::HashMap;

/// The widest a word may be: a whole row.
pub const MAX_WIDTH: usize = 3;

#[derive(Default)]
pub struct Reader {
    /// word ⇒ counts of width 1, 2, 3
    width: HashMap<String, [f64; MAX_WIDTH]>,
    /// (word, offset) ⇒ colour counts in `COLOURS` order
    colour: HashMap<(String, usize), [f64; 4]>,
}

/// Every width assignment (1 to 3 a word) that covers the grid exactly, with
/// no word crossing a row's end.
pub fn segmentations(n: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    fn rec(w: &mut Vec<usize>, n: usize, cell: usize, out: &mut Vec<Vec<usize>>) {
        if w.len() == n {
            if cell == CELLS {
                out.push(w.clone());
            }
            return;
        }
        for width in 1..=MAX_WIDTH {
            if cell % 3 + width <= 3 {
                w.push(width);
                rec(w, n, cell + width, out);
                w.pop();
            }
        }
    }
    rec(&mut Vec::new(), n, 0, &mut out);
    out
}

fn ci(c: &str) -> usize {
    COLOURS.iter().position(|x| *x == c).unwrap_or(0)
}

impl Reader {
    fn p_width(&self, w: &str, k: usize) -> f64 {
        let c = self.width.get(w).copied().unwrap_or([0.0; MAX_WIDTH]);
        (c[k] + 1.0 / MAX_WIDTH as f64) / (c.iter().sum::<f64>() + 1.0)
    }

    fn p_colour(&self, w: &str, o: usize, colour: usize) -> f64 {
        let c = self.colour.get(&(w.to_string(), o)).copied().unwrap_or([0.0; 4]);
        (c[colour] + 0.25) / (c.iter().sum::<f64>() + 1.0)
    }

    /// How well one segmentation fits: the product of every word's width
    /// and colour relations along it.
    fn fit(&self, e: &Episode, seg: &[usize]) -> f64 {
        let mut cell = 0;
        let mut p = 1.0;
        for (w, &k) in e.words.iter().zip(seg) {
            p *= self.p_width(w, k - 1);
            for o in 0..k {
                p *= self.p_colour(w, o, ci(&e.grid[cell + o]));
            }
            cell += k;
        }
        p
    }

    pub fn learn(train: &[Episode], rounds: usize) -> Self {
        let mut me = Reader::default();
        for _ in 0..rounds {
            let mut next = Reader::default();
            for e in train {
                let segs = segmentations(e.words.len());
                let fits: Vec<f64> = segs.iter().map(|s| me.fit(e, s)).collect();
                let total: f64 = fits.iter().sum();
                for (seg, f) in segs.iter().zip(&fits) {
                    let share = if total > 0.0 { f / total } else { 1.0 / segs.len() as f64 };
                    let mut cell = 0;
                    for (w, &k) in e.words.iter().zip(seg) {
                        next.width.entry(w.clone()).or_default()[k - 1] += share;
                        for o in 0..k {
                            next.colour.entry((w.clone(), o)).or_default()[ci(&e.grid[cell + o])] += share;
                        }
                        cell += k;
                    }
                }
            }
            me = next;
        }
        me
    }

    /// The arrangement a sentence describes: the best-fitting segmentation
    /// by widths, then each square the colour its word relates to there.
    pub fn read(&self, words: &[String]) -> Vec<String> {
        let best_colour = |w: &str, o: usize| (0..4).max_by(|&a, &b| self.p_colour(w, o, a).total_cmp(&self.p_colour(w, o, b))).unwrap_or(0);
        let score = |seg: &[usize]| -> f64 {
            words.iter().zip(seg).map(|(w, &k)| self.p_width(w, k - 1).ln() + (0..k).map(|o| self.p_colour(w, o, best_colour(w, o)).ln()).sum::<f64>()).sum()
        };
        let segs = segmentations(words.len());
        let Some(seg) = segs.iter().max_by(|a, b| score(a).total_cmp(&score(b))) else { return vec!["?".into(); CELLS] };
        let mut grid = Vec::new();
        for (w, &k) in words.iter().zip(seg) {
            for o in 0..k {
                grid.push(COLOURS[best_colour(w, o)].to_string());
            }
        }
        grid
    }

    /// Each word's learned relations: width and colours by offset.
    pub fn posts(&self) -> String {
        let mut words: Vec<&String> = self.width.keys().collect();
        words.sort();
        let mut s = String::from("| word | P(width 1 / 2 / 3) | colour at offset 0 | colour at offset 1 |\n|---|---|---|---|\n");
        for w in words {
            let top = |o: usize| {
                let c = self.colour.get(&(w.clone(), o)).copied().unwrap_or([0.0; 4]);
                let t: f64 = c.iter().sum();
                let i = (0..4).max_by(|&a, &b| c[a].total_cmp(&c[b])).unwrap_or(0);
                if t < 1.0 { "—".to_string() } else { format!("{} ({:.2})", COLOURS[i], c[i] / t) }
            };
            s += &format!("| `{w}` | {:.2} / {:.2} / {:.2} | {} | {} |\n", self.p_width(w, 0), self.p_width(w, 1), self.p_width(w, 2), top(0), top(1));
        }
        s
    }
}
