//! Three other ways to learn the squares game from the same episodes:
//! the whole sentence remembered, the Augustinian naming table (each word
//! names one colour, a sentence lists them), and the standard positional
//! classifier (a softmax per square over `word@position` features).

use super::world::{Episode, CELLS, COLOURS};
use std::collections::HashMap;

pub trait Learner {
    fn name(&self) -> String;
    fn read(&self, words: &[String]) -> Vec<String>;
}

pub struct Lookup(HashMap<String, Vec<String>>);

impl Lookup {
    pub fn new(train: &[Episode]) -> Self {
        Self(train.iter().map(|e| (e.words.join(" "), e.grid.clone())).collect())
    }
}

impl Learner for Lookup {
    fn name(&self) -> String {
        "lookup (whole sentence)".into()
    }
    fn read(&self, words: &[String]) -> Vec<String> {
        self.0.get(&words.join(" ")).cloned().unwrap_or_else(|| vec!["R".into(); CELLS])
    }
}

/// §1: a word names the colour it most often goes with at its own place
/// in the sentence; the sentence is the list of what its words name.
pub struct Naming(HashMap<String, String>);

impl Naming {
    pub fn new(train: &[Episode]) -> Self {
        let mut n: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for e in train {
            for (i, w) in e.words.iter().enumerate() {
                if let Some(c) = e.grid.get(i) {
                    *n.entry(w.clone()).or_default().entry(c.clone()).or_default() += 1;
                }
            }
        }
        Self(n.into_iter().map(|(w, m)| (w, m.into_iter().max_by_key(|x| x.1).map(|x| x.0).unwrap_or_default())).collect())
    }
}

impl Learner for Naming {
    fn name(&self) -> String {
        "naming table (Augustine, §1)".into()
    }
    fn read(&self, words: &[String]) -> Vec<String> {
        let mut g: Vec<String> = words.iter().map(|w| self.0.get(w).cloned().unwrap_or_else(|| "R".into())).collect();
        g.resize(CELLS, "R".into());
        g
    }
}

/// A softmax per square over `word@position` features and a bias, SGD.
pub struct Positional {
    w: Vec<HashMap<String, [f64; 4]>>,
}

fn feats(words: &[String]) -> Vec<String> {
    let mut f: Vec<String> = words.iter().enumerate().map(|(i, w)| format!("{w}@{i}")).collect();
    f.push(format!("⟨len {}⟩", words.len()));
    f.push("⟨bias⟩".into());
    f
}

fn ci(c: &str) -> usize {
    COLOURS.iter().position(|x| *x == c).unwrap_or(0)
}

impl Positional {
    pub fn new(train: &[Episode], epochs: usize) -> Self {
        let mut w: Vec<HashMap<String, [f64; 4]>> = vec![HashMap::new(); CELLS];
        for epoch in 0..epochs {
            let lr = 0.2 / (1.0 + epoch as f64 * 0.1);
            for e in train {
                let f = feats(&e.words);
                for (cell, wc) in w.iter_mut().enumerate() {
                    let p = softmax(&f, wc);
                    let y = ci(&e.grid[cell]);
                    for x in &f {
                        let row = wc.entry(x.clone()).or_insert([0.0; 4]);
                        for k in 0..4 {
                            row[k] -= lr * (p[k] - (k == y) as usize as f64 + 1e-4 * row[k]);
                        }
                    }
                }
            }
        }
        Self { w }
    }
}

fn softmax(f: &[String], w: &HashMap<String, [f64; 4]>) -> [f64; 4] {
    let mut z = [0.0; 4];
    for x in f {
        if let Some(r) = w.get(x) {
            for k in 0..4 {
                z[k] += r[k];
            }
        }
    }
    let m = z.iter().cloned().fold(f64::MIN, f64::max);
    let e: Vec<f64> = z.iter().map(|v| (v - m).exp()).collect();
    let s: f64 = e.iter().sum();
    [e[0] / s, e[1] / s, e[2] / s, e[3] / s]
}

impl Learner for Positional {
    fn name(&self) -> String {
        "positional softmax (word@position)".into()
    }
    fn read(&self, words: &[String]) -> Vec<String> {
        let f = feats(words);
        self.w.iter().map(|wc| {
            let p = softmax(&f, wc);
            COLOURS[(0..4).max_by(|&a, &b| p[a].total_cmp(&p[b])).unwrap_or(0)].to_string()
        }).collect()
    }
}
