//! Classes from the ground up: a word is read through the frequent word it
//! behaves like.
//!
//! Every `every` lines, each word joins the anchor (one of the `anchors`
//! commonest words) whose followers look most like its own — the cosine of
//! their follower counts. Then a context is read by its class: what
//! followed the class of the word before, and what followed the word two
//! back's class and the word before. Where a word is rare and its own row is
//! thin, its class has seen what words like it are followed by.

use std::collections::HashMap;

pub struct Classes {
    anchors: usize,
    every: usize,
    told: Vec<Vec<String>>,
    follow: HashMap<String, HashMap<String, u32>>,
    class: HashMap<String, String>,
    one: HashMap<String, HashMap<String, u32>>,
    two: HashMap<String, HashMap<String, u32>>,
}

impl Classes {
    pub fn new(anchors: usize, every: usize) -> Self {
        Self { anchors, every, told: Vec::new(), follow: HashMap::new(), class: HashMap::new(), one: HashMap::new(), two: HashMap::new() }
    }

    fn of(&self, w: &str) -> String {
        self.class.get(w).cloned().unwrap_or_else(|| format!("?{w}"))
    }

    fn norm(m: &HashMap<String, u32>) -> f64 {
        m.values().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt()
    }

    /// The cosine of two follower rows, the second's norm given.
    fn cosine(a: &HashMap<String, u32>, na: f64, b: &HashMap<String, u32>, nb: f64) -> f64 {
        let dot: f64 = a.iter().filter_map(|(k, x)| b.get(k).map(|y| (*x as f64) * (*y as f64))).sum();
        match na * nb > 0.0 {
            true => dot / (na * nb),
            false => 0.0,
        }
    }

    /// Re-assigns every word to its nearest anchor, then recounts the class
    /// tables over everything told.
    fn regroup(&mut self) {
        let mut by_count: Vec<(&String, u32)> = self.follow.iter().map(|(w, m)| (w, m.values().sum())).collect();
        by_count.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        let anchors: Vec<(&String, f64)> = by_count.iter().take(self.anchors).map(|(w, _)| (*w, Self::norm(&self.follow[*w]))).collect();
        let class: HashMap<String, String> = self
            .follow
            .iter()
            .map(|(w, m)| {
                let na = Self::norm(m);
                let best = anchors.iter().map(|(a, nb)| (Self::cosine(m, na, &self.follow[*a], *nb), *a)).fold((0.0, w), |b, (c, a)| match c > b.0 {
                    true => (c, a),
                    false => b,
                });
                (w.clone(), best.1.clone())
            })
            .collect();
        self.class = class;
        self.one.clear();
        self.two.clear();
        let told = std::mem::take(&mut self.told);
        for words in &told {
            self.count(words);
        }
        self.told = told;
    }

    fn count(&mut self, words: &[String]) {
        for j in 1..words.len() {
            let c1 = self.of(&words[j - 1]);
            *self.one.entry(c1.clone()).or_default().entry(words[j].clone()).or_insert(0) += 1;
            if j >= 2 {
                let key = format!("{} {}", self.of(&words[j - 2]), words[j - 1]);
                *self.two.entry(key).or_default().entry(words[j].clone()).or_insert(0) += 1;
            }
        }
    }

    /// The class-two context where it was seen, else the class of the word
    /// before; as a distribution.
    pub fn dist(&self, said: &[String]) -> HashMap<String, f64> {
        let j = said.len();
        let two = (j >= 2).then(|| self.two.get(&format!("{} {}", self.of(&said[j - 2]), said[j - 1]))).flatten();
        let row = two.or_else(|| self.one.get(&self.of(&said[j - 1])));
        let Some(row) = row else { return HashMap::new() };
        let n: u32 = row.values().sum();
        row.iter().map(|(w, c)| (w.clone(), *c as f64 / n as f64)).collect()
    }

    pub fn tell(&mut self, words: &[String]) {
        for p in words.windows(2) {
            *self.follow.entry(p[0].clone()).or_default().entry(p[1].clone()).or_insert(0) += 1;
        }
        self.count(words);
        self.told.push(words.to_vec());
        if self.told.len() % self.every == 0 {
            self.regroup();
        }
    }
}
