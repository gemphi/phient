//! Soft trial and error: every assumption gives a distribution over the
//! next word, and their mixing weights are learned from the error — after
//! each word, weight moves toward the assumptions that gave the true word
//! more probability (one online EM step). The weights are kept per bucket
//! of how often the two-word context was seen, because how far to trust a
//! context depends on how much stands behind it.

use super::parts::Part;
use super::table::{back, Table};
use phient::study::Reader;
use std::collections::{HashMap, VecDeque};

/// The weights each context bucket came to, shared so the lab can print
/// them after the arena has read.
pub type Report = std::rc::Rc<std::cell::RefCell<Tally>>;

/// What a mix came to: its weights per bucket, and its surprise at what
/// came — the bits of −log₂ of the probability it gave the true word.
#[derive(Clone, Default)]
pub struct Tally {
    pub weights: HashMap<u8, Vec<f64>>,
    pub bits: f64,
    pub words: usize,
}

pub struct Mix {
    name: String,
    tables: [Table; 3],
    /// The words of the last `scope` sentences, when the scope is on.
    recent: VecDeque<Vec<String>>,
    scope: usize,
    weights: HashMap<u8, Vec<f64>>,
    rate: f64,
    /// The parts beyond the counts, and their distributions at each
    /// position of the line being read.
    extra: Vec<Part>,
    cache: Vec<Vec<HashMap<String, f64>>>,
    report: Option<Report>,
}

impl Mix {
    pub fn new(name: &str, scope: usize) -> Self {
        Self {
            name: name.to_string(),
            tables: Default::default(),
            recent: VecDeque::new(),
            scope,
            weights: HashMap::new(),
            rate: 0.02,
            extra: Vec::new(),
            cache: Vec::new(),
            report: None,
        }
    }

    /// One more part, and where to report the weights.
    pub fn with(mut self, part: Part, report: Report) -> Self {
        self.extra.push(part);
        self.reporting(report)
    }

    /// Where to report the weights and the surprise.
    pub fn reporting(mut self, report: Report) -> Self {
        self.report = Some(report);
        self
    }

    fn parts(&self) -> usize {
        3 + usize::from(self.scope > 0) + self.extra.len()
    }

    fn bucket(&self, words: &[String], j: usize) -> u8 {
        match self.tables[2].seen(&back(words, j, 2)) {
            0 => 0,
            1 => 1,
            2..=4 => 2,
            5..=19 => 3,
            _ => 4,
        }
    }

    /// Each part's probability of `w` at position `j`: trigram, bigram,
    /// unigram, the scope's recent words, and the extra parts.
    fn probs(&self, words: &[String], j: usize, w: &str, recent: &HashMap<&str, usize>, all: usize, ex: &[HashMap<String, f64>]) -> Vec<f64> {
        let mut out: Vec<f64> = [2usize, 1, 0]
            .iter()
            .map(|&k| {
                let key = back(words, j, k);
                let n = self.tables[k].seen(&key);
                match n {
                    0 => 0.0,
                    _ => self.tables[k].count(&key, w) as f64 / n as f64,
                }
            })
            .collect();
        if self.scope > 0 {
            out.push(match all {
                0 => 0.0,
                _ => recent.get(w).copied().unwrap_or(0) as f64 / all as f64,
            });
        }
        out.extend(ex.iter().map(|m| m.get(w).copied().unwrap_or(0.0)));
        out
    }

    /// The scope's words with their counts, and how many words it holds.
    fn recent(&self) -> (HashMap<&str, usize>, usize) {
        let mut m: HashMap<&str, usize> = HashMap::new();
        for w in self.recent.iter().flatten() {
            *m.entry(w.as_str()).or_insert(0) += 1;
        }
        let all = m.values().sum();
        (m, all)
    }

    fn weights_at(&self, b: u8) -> Vec<f64> {
        let n = self.parts();
        self.weights.get(&b).cloned().unwrap_or_else(|| vec![1.0 / n as f64; n])
    }
}

impl Reader for Mix {
    fn name(&self) -> &str {
        &self.name
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        let j = said.len();
        let lam = self.weights_at(self.bucket(said, j));
        let (recent, all) = self.recent();
        let ex: Vec<HashMap<String, f64>> = self.extra.iter().map(|p| p.dist(said)).collect();
        // Candidates: what followed the two words and the word before, and the
        // commonest words; the scope's words when it is on.
        let mut cand: Vec<&str> = Vec::new();
        for k in [2usize, 1] {
            cand.extend(self.tables[k].followers(&back(said, j, k)));
        }
        cand.extend(self.tables[0].top(""));
        cand.extend(recent.keys().copied());
        cand.extend(ex.iter().flat_map(|m| m.keys().map(String::as_str)));
        cand.sort_unstable();
        cand.dedup();
        let out = cand
            .into_iter()
            .map(|w| (w, self.probs(said, j, w, &recent, all, &ex).iter().zip(&lam).map(|(p, l)| p * l).sum::<f64>()))
            .fold(None::<(&str, f64)>, |b, (w, p)| match b {
                Some((_, bp)) if bp >= p => b,
                _ => Some((w, p)),
            })
            .map(|(w, _)| w.to_string());
        if self.cache.len() <= j {
            self.cache.resize(j + 1, Vec::new());
        }
        self.cache[j] = ex;
        out
    }

    fn tell(&mut self, words: &[String]) {
        let (recent, all) = self.recent();
        let recent: HashMap<String, usize> = recent.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        let recent: HashMap<&str, usize> = recent.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        for j in 1..words.len() {
            let b = self.bucket(words, j);
            let lam = self.weights_at(b);
            let ex = self.cache.get(j).cloned().unwrap_or_default();
            let p = self.probs(words, j, &words[j], &recent, all, &ex);
            let total: f64 = p.iter().zip(&lam).map(|(p, l)| p * l).sum();
            if let Some(r) = &self.report {
                let mut r = r.borrow_mut();
                r.bits -= total.max(1e-7).log2();
                r.words += 1;
            }
            if total > 0.0 {
                let post: Vec<f64> = p.iter().zip(&lam).map(|(p, l)| p * l / total).collect();
                let w = self.weights.entry(b).or_insert(lam);
                w.iter_mut().zip(&post).for_each(|(x, q)| *x += self.rate * (q - *x));
            }
            for k in 0..3 {
                self.tables[k].add(&back(words, j, k), &words[j]);
            }
        }
        self.cache.clear();
        for p in &mut self.extra {
            p.tell(words);
        }
        if let Some(r) = &self.report {
            r.borrow_mut().weights = self.weights.clone();
        }
        if self.scope > 0 {
            self.recent.push_back(words.to_vec());
            if self.recent.len() > self.scope {
                self.recent.pop_front();
            }
        }
    }
}
