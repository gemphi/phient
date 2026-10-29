#[allow(unused_imports)]
use crate::relate::*;

use phiecs::Row;
use phical::relate::Triple;
use phistd::{c64, Function};
use phical::{Fno, FnoConfig, Joint};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A kind's row name in the table: `⟨is-a⟩`.
pub fn kind_token(kind: &str) -> String {
    format!("⟨{kind}⟩")
}

/// An application's own starting function for a word, when it has one.
pub type Seeding<'a> = &'a dyn Fn(&str) -> Option<Function>;

/// Words and kinds as rows of one function table, with the joint operator.
pub struct RelationModel {
    pub names: Vec<String>,
    pub index: HashMap<String, usize>,
    pub kinds: Vec<String>,
    pub joint: Joint,
}

#[derive(Serialize, Deserialize)]
struct Saved {
    names: Vec<String>,
    kinds: Vec<String>,
    table: Vec<Vec<(f64, f64)>>,
    fno: Vec<u8>,
}

impl RelationModel {
    /// Rows for every word and kind in `rows`. Each word's function is what
    /// `seeds` gives it when it gives one (an application's own angles for
    /// the word — phiano's facet — normalised), a seeded random unit function
    /// when not; each kind's is random.
    pub fn new(rows: &[Row], seeds: Option<Seeding>, config: FnoConfig, seed: u64) -> Self {
        let mut names: Vec<String> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        let mut kinds: Vec<String> = Vec::new();
        for r in rows {
            for w in [&r.about, &r.other] {
                if !index.contains_key(w) {
                    index.insert(w.clone(), names.len());
                    names.push(w.clone());
                }
            }
            if !kinds.contains(&r.kind) {
                kinds.push(r.kind.clone());
            }
        }
        for k in &kinds {
            index.insert(kind_token(k), names.len());
            names.push(kind_token(k));
        }
        let space = config.space;
        let table: Vec<Function> = names
            .iter()
            .enumerate()
            .map(|(i, w)| match seeds.and_then(|s| s(w)).filter(|f| f.len() == space) {
                Some(mut f) => {
                    phistd::normalise(&mut f);
                    f
                }
                None => phistd::unit_random(space, seed ^ (i as u64).wrapping_mul(0x9E37_79B9)),
            })
            .collect();
        let joint = Joint::new(Fno::new(config, seed), &table, 2);
        Self { names, index, kinds, joint }
    }

    /// Adds every word and kind in `rows` the model does not have yet, each
    /// with a seeded random function, keeping everything it learned: the
    /// table grows, the operator is the same. Returns how many rows of the
    /// table were added. This is how a model learns a new persona's words
    /// without starting again.
    pub fn extend(&mut self, rows: &[Row], seed: u64) -> usize {
        let space = self.joint.fno.config.space;
        let mut table = self.joint.functions();
        let before = table.len();
        let add = |name: String, names: &mut Vec<String>, index: &mut HashMap<String, usize>, table: &mut Vec<Function>| {
            if !index.contains_key(&name) {
                index.insert(name.clone(), names.len());
                names.push(name);
                table.push(phistd::unit_random(space, seed ^ (table.len() as u64).wrapping_mul(0x9E37_79B9)));
            }
        };
        for r in rows {
            add(r.about.clone(), &mut self.names, &mut self.index, &mut table);
            add(r.other.clone(), &mut self.names, &mut self.index, &mut table);
            if !self.kinds.contains(&r.kind) {
                self.kinds.push(r.kind.clone());
                add(kind_token(&r.kind), &mut self.names, &mut self.index, &mut table);
            }
        }
        let added = table.len() - before;
        if added > 0 {
            let fno = Fno::from_bytes(&self.joint.fno.to_bytes()).expect("an operator reads back its own bytes");
            self.joint = Joint::new(fno, &table, 2);
        }
        added
    }

    /// The row as indices; `None` if a name is unknown.
    pub fn triple(&self, r: &Row) -> Option<Triple> {
        Some(Triple { about: *self.index.get(&r.about)?, kind: *self.index.get(&kind_token(&r.kind))?, other: *self.index.get(&r.other)? })
    }

    /// The operator's answer to `word · kind → ?`: the function
    /// `F(a_word + a_kind)`, its input normalised. `None` for a word or kind
    /// the model never saw.
    pub fn output(&mut self, word: &str, kind: &str) -> Option<Function> {
        let (&a, &k) = (self.index.get(word)?, self.index.get(&kind_token(kind))?);
        let table = self.joint.functions();
        Some(self.output_at(&table, a, k))
    }

    /// As `output`, by row, against a copy of the table already taken
    /// (`joint.functions()` copies it, so a reader of many rows takes it once).
    pub fn output_at(&mut self, table: &[Function], word: usize, kind: usize) -> Function {
        let x = phical::relate::context(table, word, kind);
        self.joint.fno.apply(&[x]).remove(0)
    }

    /// The cosine of `word · kind → ?` with every one of `values`, in their
    /// order; a value the model never saw reads −1. The word itself is read
    /// too: under `next` a word can follow itself (docs/152).
    pub fn scores(&mut self, word: &str, kind: &str, values: &[String]) -> Option<Vec<f64>> {
        let y = self.output(word, kind)?;
        let table = self.joint.functions();
        Some(values.iter().map(|v| self.index.get(v).map_or(-1.0, |&i| phistd::cosine(&y, &table[i]))).collect())
    }

    /// Every word the operator reads as `word · kind · ?` among `pool`,
    /// strongest first, the word itself left out — the reading for relations
    /// such as `is-a`, where a word is not its own answer.
    pub fn answers(&mut self, word: &str, kind: &str, pool: &[usize], n: usize) -> Vec<(String, f64)> {
        let Some(&a) = self.index.get(word) else { return Vec::new() };
        let Some(y) = self.output(word, kind) else { return Vec::new() };
        let table = self.joint.functions();
        let mut out: Vec<(String, f64)> = pool.iter().filter(|&&w| w != a).map(|&w| (self.names[w].clone(), phistd::cosine(&y, &table[w]))).collect();
        out.sort_by(|p, q| q.1.partial_cmp(&p.1).unwrap_or(std::cmp::Ordering::Equal));
        out.truncate(n);
        out
    }

    /// Operator parameters plus table numbers (each complex sample counts two).
    pub fn size(&self) -> (usize, usize) {
        let t = self.joint.functions();
        (self.joint.fno.parameter_count(), 2 * t.iter().map(Vec::len).sum::<usize>())
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let table = self.joint.functions().iter().map(|f| f.iter().map(|z| (z.re, z.im)).collect()).collect();
        let s = Saved { names: self.names.clone(), kinds: self.kinds.clone(), table, fno: self.joint.fno.to_bytes() };
        let bytes = bincode::serialize(&s).map_err(std::io::Error::other)?;
        std::fs::write(path, bytes)
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let bad = |m: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{path}: {m}"));
        let s: Saved = bincode::deserialize(&bytes).map_err(|_| bad("not a .relate file"))?;
        let fno = Fno::from_bytes(&s.fno).ok_or_else(|| bad("bad operator"))?;
        let table: Vec<Function> = s.table.iter().map(|f| f.iter().map(|&(r, i)| c64::new(r, i)).collect()).collect();
        let index = s.names.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
        Ok(Self { names: s.names, index, kinds: s.kinds, joint: Joint::new(fno, &table, 2) })
    }
}
