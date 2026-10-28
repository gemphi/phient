//! Trial and error over the assumptions themselves.
//!
//! Every assumption is an expert that guesses. The guess is the word the
//! most weight stands behind; when the line is told, every expert that was
//! wrong at a position loses a share `1 − β` of its weight there. Situated,
//! the weights are kept per situation — after an operator, or after a
//! content word — so the structure decides which assumption to trust where.

use super::readers::Chain;
use phient::study::Reader;
use std::collections::HashMap;

pub struct Hedge {
    name: String,
    experts: Vec<Chain>,
    weights: HashMap<&'static str, Vec<f64>>,
    beta: f64,
    situated: bool,
    /// Each expert's guess at each position of the line being read.
    guessed: Vec<Vec<Option<String>>>,
}

impl Hedge {
    pub fn new(name: &str, experts: Vec<Chain>, beta: f64, situated: bool) -> Self {
        Self { name: name.to_string(), experts, weights: HashMap::new(), beta, situated, guessed: Vec::new() }
    }

    fn situation(&self, words: &[String], j: usize) -> &'static str {
        match (self.situated, self.experts[0].is_op(&words[j - 1])) {
            (false, _) => "all",
            (true, true) => "after an operator",
            (true, false) => "after a content word",
        }
    }

    /// The weights of each situation, after the stream: what was trusted where.
    pub fn trust(&self) -> Vec<(&'static str, Vec<(String, f64)>)> {
        let mut out: Vec<_> = self
            .weights
            .iter()
            .map(|(s, w)| (*s, self.experts.iter().zip(w).map(|(e, x)| (e.name().to_string(), *x)).collect()))
            .collect();
        out.sort_by_key(|(s, _)| *s);
        out
    }
}

impl Reader for Hedge {
    fn name(&self) -> &str {
        &self.name
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        let j = said.len();
        let guesses: Vec<Option<String>> = self.experts.iter().map(|e| e.answer(said, j).map(|(w, _)| w)).collect();
        let n = self.experts.len();
        let s = self.situation(said, j);
        let w = self.weights.entry(s).or_insert_with(|| vec![1.0 / n as f64; n]).clone();
        let mut score: Vec<(&str, f64)> = Vec::new();
        for (g, x) in guesses.iter().zip(&w) {
            if let Some(g) = g {
                match score.iter_mut().find(|(k, _)| *k == g.as_str()) {
                    Some(e) => e.1 += x,
                    None => score.push((g.as_str(), *x)),
                }
            }
        }
        let best = score.iter().fold(None::<(&str, f64)>, |b, &(k, v)| match b {
            Some((_, bv)) if bv >= v => b,
            _ => Some((k, v)),
        });
        let out = best.map(|(k, _)| k.to_string());
        if self.guessed.len() <= j {
            self.guessed.resize(j + 1, Vec::new());
        }
        self.guessed[j] = guesses;
        out
    }

    fn tell(&mut self, words: &[String]) {
        for j in 1..words.len().min(self.guessed.len()) {
            let s = self.situation(words, j);
            let n = self.experts.len();
            let guesses = std::mem::take(&mut self.guessed[j]);
            let w = self.weights.entry(s).or_insert_with(|| vec![1.0 / n as f64; n]);
            for (x, g) in w.iter_mut().zip(&guesses) {
                if g.as_deref() != Some(words[j].as_str()) {
                    *x *= self.beta;
                }
            }
            let sum: f64 = w.iter().sum();
            w.iter_mut().for_each(|x| *x /= sum);
        }
        self.guessed.clear();
        for e in &mut self.experts {
            e.tell(words);
        }
    }
}
