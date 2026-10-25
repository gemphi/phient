//! Going on measured (docs/135): paragraphs held out of the tree, the next
//! word read at every position from the words before it, scored against
//! the word that came — top-1, mean reciprocal rank, and perplexity — by
//! the flat reading (the last word and the last two) and by the tree (the
//! scopes a cast finds, then the words under them).

use super::Knowing;

#[derive(Default, Debug, Clone)]
pub struct Score {
    pub asked: usize,
    pub top1: usize,
    pub rr: f64,
    pub nll: f64,
}

impl Score {
    pub fn row(&self, name: &str) -> String {
        let n = self.asked.max(1) as f64;
        format!("| {name} | {} | {:.1}% | {:.3} | {:.1} |", self.asked, 100.0 * self.top1 as f64 / n, self.rr / n, (self.nll / n).exp())
    }
}

/// Every position of every held-out paragraph, from the third word on.
pub fn score(k: &Knowing) -> Score {
    let mut s = Score::default();
    for (_, words) in &k.held {
        for i in 2..words.len() {
            let (_, ranked, _) = k.next(&words[..i]);
            let truth = &words[i];
            // The scores are logs; normalised over the values for a probability.
            let top = ranked.first().map_or(0.0, |x| x.1);
            let z: f64 = ranked.iter().map(|x| (x.1 - top).exp()).sum();
            let (mut rank, mut p) = (ranked.len() + 1, 1e-6 / ranked.len().max(1) as f64);
            if let Some(i) = ranked.iter().position(|x| x.0 == *truth) {
                rank = i + 1;
                p = (ranked[i].1 - top).exp() / z;
            }
            s.asked += 1;
            s.top1 += (rank == 1) as usize;
            s.rr += 1.0 / rank as f64;
            s.nll -= p.max(1e-12).ln();
        }
    }
    s
}

/// Which paragraphs are held out: a `share` of them, by hash of the path.
pub fn held(share: f64, seed: u64) -> impl Fn(&str) -> bool {
    move |path: &str| {
        let h = phistd::mix64(phistd::fnv1a(path) ^ seed);
        ((h >> 11) as f64 / (1u64 << 53) as f64) < share
    }
}
