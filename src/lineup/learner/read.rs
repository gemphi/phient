//! Reading: every word's score, the ranked answer, held-out measurement, speech.

use super::{Learner, END, FIRST};
use crate::study::Score;
use rayon::prelude::*;

impl Learner {
    /// Every real word's score after `said`, and the end's.
    pub(super) fn scores(&self, said: &[String]) -> (Vec<f32>, f32) {
        let ctx = self.context_of(said);
        let d = self.config.channels;
        let (mut zr, mut zi) = (vec![0.0; d], vec![0.0; d]);
        self.condition(&ctx, &mut zr, &mut zi);
        let s: Vec<f32> = (FIRST..self.word.len() as u32).map(|v| self.score(&zr, &zi, v)).collect();
        let end = self.score(&zr, &zi, END);
        (s, end)
    }

    /// The `k` likeliest next words after `said`, with their probabilities
    /// (among the real words — the end of the line is not a word).
    pub fn ranked(&self, said: &[String], k: usize) -> Vec<(String, f64)> {
        let (s, _) = self.scores(said);
        let top = s.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let z: f64 = s.iter().map(|x| ((x - top) as f64).exp()).sum();
        let mut i: Vec<usize> = (0..s.len()).collect();
        i.sort_by(|a, b| s[*b].total_cmp(&s[*a]));
        i.into_iter()
            .take(k)
            .map(|j| (self.word[j + FIRST as usize].clone(), ((s[j] - top) as f64).exp() / z))
            .collect()
    }

    /// Scores held-out lines at the arena's positions, in parallel.
    pub fn measure(&self, lines: &[Vec<String>]) -> Score {
        lines
            .par_iter()
            .map(|line| {
                let mut r = Score::default();
                for j in 1..line.len() {
                    let (s, _) = self.scores(&line[..j]);
                    let target = self.id(&line[j]);
                    if target < FIRST {
                        r.rank(None);
                        continue;
                    }
                    let mine = s[(target - FIRST) as usize];
                    r.rank(Some(1 + s.iter().filter(|x| **x > mine).count()));
                    let top = s.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                    let z: f64 = s.iter().map(|x| ((x - top) as f64).exp()).sum();
                    r.prob(z.ln() - (mine - top) as f64);
                }
                r
            })
            .reduce(Score::default, Score::merge)
    }

    /// Goes on from `prompt` for at most `words` words: the likeliest word
    /// each time that does not repeat a pair already said, stopping where
    /// the line is likelier to end.
    pub fn speak(&self, prompt: &[String], words: usize) -> Vec<String> {
        let mut said = prompt.to_vec();
        let mut out = Vec::new();
        while out.len() < words {
            let (s, end) = self.scores(&said);
            let mut i: Vec<usize> = (0..s.len()).collect();
            i.sort_by(|a, b| s[*b].total_cmp(&s[*a]));
            let last = said.last().cloned().unwrap_or_default();
            let fresh = i.into_iter().find(|j| {
                let w = &self.word[j + FIRST as usize];
                !said.windows(2).any(|p| p[0] == last && p[1] == *w)
            });
            let Some(j) = fresh else { break };
            if end > s[j] {
                break;
            }
            let w = self.word[j + FIRST as usize].clone();
            said.push(w.clone());
            out.push(w);
        }
        out
    }

    /// The angle of `word`'s answer phasor on channel `k`, and its modulus.
    pub fn reached(&self, word: &str, k: usize) -> Option<(f64, f64)> {
        let (re, im) = self.answers().row(*self.index.get(word)?);
        let (r, i) = (re[k] as f64, im[k] as f64);
        Some((i.atan2(r), (r * r + i * i).sqrt()))
    }
}
