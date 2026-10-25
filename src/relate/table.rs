//! A count table for one condition under one kind (docs/133).

use phiecs::word::Word;
use std::collections::HashMap;

/// A count table: a short list while a condition has said few things (most
/// have said one or two), a map once it has said many.
#[derive(Clone, Debug)]
pub enum Table {
    Few(Vec<(Word, f64)>),
    Many(HashMap<Word, f64>),
}

const FEW: usize = 12;

impl Default for Table {
    fn default() -> Self {
        Table::Few(Vec::new())
    }
}

impl Table {
    pub fn get(&self, w: &Word) -> Option<&f64> {
        match self {
            Table::Few(v) => v.iter().find(|(x, _)| x == w).map(|(_, n)| n),
            Table::Many(m) => m.get(w),
        }
    }

    pub fn add(&mut self, w: Word, by: f64) {
        match self {
            Table::Few(v) => {
                if let Some(e) = v.iter_mut().find(|(x, _)| *x == w) {
                    e.1 += by;
                } else if v.len() < FEW {
                    v.push((w, by));
                } else {
                    let mut m: HashMap<Word, f64> = v.drain(..).collect();
                    m.insert(w, by);
                    *self = Table::Many(m);
                }
            }
            Table::Many(m) => *m.entry(w).or_default() += by,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Table::Few(v) => v.len(),
            Table::Many(m) => m.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn sum(&self) -> f64 {
        match self {
            Table::Few(v) => v.iter().map(|x| x.1).sum(),
            Table::Many(m) => m.values().sum(),
        }
    }

    pub fn keys(&self) -> Vec<Word> {
        match self {
            Table::Few(v) => v.iter().map(|x| x.0).collect(),
            Table::Many(m) => m.keys().copied().collect(),
        }
    }
}

