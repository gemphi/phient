//! The phifno reader (docs/177): the Joint Fourier neural operator over a
//! table of functions — a word is a function, the kind `next` a row of the
//! same table, and the answer is read by correlating `F(a_w + a_next)`
//! against every word's function (docs/91–97).
//!
//! It cannot guess-then-learn position by position: an operator is trained
//! by epochs over examples, not incremented by a count. So it enters the
//! arena through [`crate::study::Arena::read_split`] — told the stream's
//! first half, trained once at the end of it, then scored on the second
//! half at the same positions as the online readers. Being trained on a
//! batch the counters only ever saw once is an advantage, and it is stated
//! in the table rather than hidden.

use phical::{Adam, FnoConfig};
use phiecs::Row;
use crate::operator::RelationModel;
use crate::study::Reader;

pub struct Operator {
    row: Vec<Row>,
    model: Option<RelationModel>,
    pool: Vec<usize>,
    epochs: usize,
    seed: u64,
}

impl Operator {
    pub fn new(epochs: usize, seed: u64) -> Self {
        Self { row: Vec::new(), model: None, pool: Vec::new(), epochs, seed }
    }

    /// Builds the table and trains the operator on everything told so far.
    /// Called once, when the arena's training half ends.
    pub fn train(&mut self) {
        if self.row.is_empty() {
            return;
        }
        let mut m = RelationModel::new(&self.row, None, FnoConfig::circle64(), self.seed);
        let example: Vec<_> = self
            .row
            .iter()
            .filter_map(|r| m.triple(r))
            .map(|t| phical::relate::example(&t))
            .collect();
        let kinds = m.kinds.clone();
        self.pool = (0..m.names.len())
            .filter(|&i| !kinds.iter().any(|k| m.names[i] == crate::operator::kind_token(k)))
            .collect();
        let lr = m.joint.fno.config.lr;
        let (mut op, mut tab) = (Adam::new(lr), Adam::new(lr * 0.5));
        for e in 0..self.epochs {
            m.joint.train_epoch(&example, &self.pool, 32, &mut op, &mut tab, self.seed ^ e as u64);
        }
        self.model = Some(m);
    }
}

impl Reader for Operator {
    fn name(&self) -> &str {
        "phifno (the Joint operator, batch-trained)"
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        let last = said.last()?.clone();
        let pool = std::mem::take(&mut self.pool);
        let m = self.model.as_mut()?;
        let out = m.answers(&last, "next", &pool, 1).first().map(|(w, _)| w.clone());
        self.pool = pool;
        out
    }

    fn tell(&mut self, words: &[String]) {
        self.row.extend(words.windows(2).map(|p| Row::new(&p[0], "next", &p[1])));
    }

    /// Trained once, here, on the told half — the arena's split lane.
    fn ready(&mut self) {
        self.train();
    }
}
