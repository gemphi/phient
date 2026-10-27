//! The phical reader (docs/176): the relation `next` as an all-pass filter
//! on the circle — `q = H·a_w`, read by correlating that wave against every
//! word (docs/153). Unlike the counting readers it must be given its
//! vocabulary before it starts, because a word is a spectrum it owns a slot
//! for; that advantage is stated in the table rather than hidden, and it
//! still has to learn the filter from the same stream, trained on what it
//! has been told so far and never on what it is about to be asked.

use phiton::conjugate::{Moment, Told};
use crate::study::Reader;

pub struct Mirror {
    inner: phiton::conjugate::Mirror,
    moments: Moment,
    told: Vec<Told>,
    /// Lines told since the filter was last trained.
    since: usize,
    /// Train after this many new lines (the filter is closed-form cheap,
    /// but not free).
    every: usize,
    epochs: usize,
}

impl Mirror {
    /// `vocab`: every word of the stream, which this model needs up front.
    pub fn new(vocab: Vec<String>, modes: usize, seed: u64) -> Self {
        let inner = phiton::conjugate::Mirror::new(vocab, vec!["next".into()], modes, false, seed);
        let moments = Moment::new(&inner);
        Self { inner, moments, told: Vec::new(), since: 0, every: 64, epochs: 2 }
    }

    fn train(&mut self) {
        if self.told.is_empty() {
            return;
        }
        // A sampled pool, not the whole vocabulary: the contrastive step
        // costs pool × rows, and an online reader must stay affordable.
        let n = self.inner.names.len();
        let pool: Vec<usize> = (0..64.min(n)).map(|i| (i * 2654435761usize + self.told.len()) % n).collect();
        let mut mo = std::mem::replace(&mut self.moments, Moment::new(&self.inner));
        for e in 0..self.epochs {
            self.inner.train_epoch(&self.told, &pool, 0.2, 2e-2, &mut mo, 7 + e as u64);
        }
        self.moments = mo;
    }
}

impl Reader for Mirror {
    fn name(&self) -> &str {
        "phical conjugate (the relation as a filter)"
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        let scores = self.inner.read(said.last()?, "next", 1)?;
        let (i, _) = scores.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1))?;
        self.inner.names.get(i).cloned()
    }

    fn tell(&mut self, words: &[String]) {
        let (idx, kind) = (|w: &String| self.inner.index.get(w).copied(), self.inner.kind("next"));
        if let Some(kind) = kind {
            for p in words.windows(2) {
                if let (Some(a), Some(o)) = (idx(&p[0]), idx(&p[1])) {
                    self.told.push(Told { about: a, kind, other: o });
                }
            }
        }
        // Bounded memory: the filter is trained on the recent stream.
        if self.told.len() > 4000 {
            let keep = self.told.len() - 4000;
            self.told.drain(..keep);
        }
        self.since += 1;
        if self.since >= self.every {
            self.since = 0;
            self.train();
        }
    }
}
