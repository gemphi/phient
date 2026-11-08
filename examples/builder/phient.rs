//! The Phient learner: a token is a seed — what it relates to, under
//! what conditions. Every `(token, aspect|game) ⇒ value` the episodes tell
//! is one relation, kept once with its count (relations, not data); phinet's
//! one operator learns them all as functions on the circle. To act, each
//! token's reading under the aspect is weighted by how firmly it relates
//! (its count) and the readings are summed — the sentence is the pairs of
//! words moving under a condition.

use super::learners::{Action, Compose, GameCounts, Learner};
use super::world::{Episode, ASPECTS};
use phient::relate::{kind_token, RelationModel};
use phiecs::quest::Row;
use phical::relate::{self, Triple};
use phistd::{Adam, Function};
use phical::{FnoConfig, Tie};
use std::collections::HashMap;

pub struct Phient {
    counts: GameCounts,
    m: RelationModel,
    /// `(token, kind)` ⇒ the operator's reading of `unit(token + kind)`.
    read: HashMap<(String, String), Function>,
    table: Vec<Function>,
    values: Vec<Vec<(String, usize)>>,
    pub relations: usize,
    compose: Compose,
    /// `(token, kind)` whose function names the same top answer as its count.
    agreed: std::collections::HashSet<(String, String)>,
}

fn value_name(a: usize, v: &str) -> String {
    format!("{}={v}", ASPECTS[a])
}

impl Phient {
    pub fn new(train: &[Episode], relations_per_word: usize, epochs: usize, seed: u64, compose: Compose) -> Self {
        let counts = GameCounts::new(train);
        let aspect_of = |k: &str| ASPECTS.iter().position(|a| k.split('|').next() == Some(*a)).unwrap_or(0);
        let mut rows: Vec<Row> = Vec::new();
        for (t, k) in counts.pairs() {
            for (v, c) in counts.table(&t, &k).unwrap_or_default() {
                rows.push(Row { about: t.clone(), kind: k.clone(), other: value_name(aspect_of(&k), &v), weight: c });
            }
        }
        rows.sort_by(|x, y| (&x.about, &x.kind, &x.other).cmp(&(&y.about, &y.kind, &y.other)));
        let r = relations_per_word;
        let config = FnoConfig { space: r, width: 16, layers: 2, modes: (r / 4).min(16), margin: 0.3, lr: 5e-3, tie: Tie::Circle };
        let mut m = RelationModel::new(&rows, None, config, seed);
        let told: Vec<Triple> = rows.iter().filter_map(|x| m.triple(x)).collect();
        // Each relation is learned against the other answers of its own
        // aspect (the decoy is another colour, not a numeral): the operator
        // must tell answers apart, not only answer types (docs/92).
        let aspect_rows: Vec<usize> = rows.iter().map(|x| aspect_of(&x.kind)).collect();
        // A relation's count enters training as log₂ repetitions: the kept
        // relation is seen more when it was told more, not once per telling.
        let groups: Vec<(Vec<usize>, Vec<_>)> = (0..ASPECTS.len())
            .map(|a| {
                let mine: Vec<(&Triple, &Row)> = told.iter().zip(&rows).zip(&aspect_rows).filter(|(_, x)| **x == a).map(|(p, _)| p).collect();
                let ts: Vec<Triple> = mine.iter().map(|(t, _)| **t).collect();
                let ex = mine.iter().flat_map(|(t, x)| std::iter::repeat_n(relate::example(t), 1 + x.weight.log2().max(0.0) as usize)).collect();
                (relate::pools(&ts).0, ex)
            })
            .collect();
        let (mut a, mut b) = (Adam::new(config.lr), Adam::new(config.lr * 0.5));
        for e in 0..epochs {
            for (g, (pool, examples)) in groups.iter().enumerate() {
                m.joint.train_epoch(examples, pool, 32, &mut a, &mut b, seed ^ e as u64 ^ (g as u64) << 32);
            }
        }
        let table = m.joint.functions();
        let keys: Vec<(String, String)> = counts.pairs();
        let inputs: Vec<Function> = keys
            .iter()
            .map(|(t, k)| {
                let (i, j) = (m.index[t], m.index[&kind_token(k)]);
                let mut f: Function = table[i].iter().zip(&table[j]).map(|(x, y)| x + y).collect();
                phistd::normalise(&mut f);
                f
            })
            .collect();
        let read = keys.into_iter().zip(m.joint.fno.apply(&inputs)).collect();
        let values = (0..ASPECTS.len()).map(|a| counts.values(a).into_iter().filter_map(|v| Some((v.clone(), *m.index.get(&value_name(a, &v))?))).collect()).collect();
        let mut me = Self { counts, m, read, table, values, relations: rows.len(), compose, agreed: Default::default() };
        me.agreed = me.agreement();
        me
    }

    /// The relations whose function already says what their count says:
    /// the same top answer among the aspect's values.
    fn agreement(&self) -> std::collections::HashSet<(String, String)> {
        let aspect = |k: &str| ASPECTS.iter().position(|a| k.split('|').next() == Some(*a)).unwrap_or(0);
        let top = |xs: Vec<(String, f64)>| xs.into_iter().max_by(|x, y| x.1.total_cmp(&y.1).then(y.0.cmp(&x.0))).map(|x| x.0);
        self.read
            .iter()
            .filter(|((t, k), y)| {
                let a = aspect(k);
                let f = top(self.values[a].iter().map(|(v, i)| (v.clone(), phistd::cosine(y, &self.table[*i]))).collect());
                let c = top(self.counts.table(t, k).unwrap_or_default().into_iter().collect());
                f.is_some() && f == c
            })
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Share of relations the functions alone could hold (see `agreement`).
    pub fn agreed_share(&self) -> f64 {
        self.agreed.len() as f64 / self.read.len().max(1) as f64
    }

    /// Numbers the model holds: operator plus table.
    pub fn size(&self) -> usize {
        let (op, tab) = self.m.size();
        op + tab
    }
}

impl Learner for Phient {
    fn name(&self) -> String {
        let how = if self.compose == Compose::Sum { "summed" } else { "movement" };
        format!("Phient relations, {how} (phinet, R={})", self.m.joint.fno.config.space)
    }
    fn act(&mut self, e: &Episode) -> Action {
        if self.compose == Compose::Movement {
            return self.movement(e, 0.0);
        }
        let table = &self.table;
        let mut out: Action = Default::default();
        for (a, slot) in out.iter_mut().enumerate() {
            let mut reads: Vec<(f64, &Function)> = Vec::new();
            for t in GameCounts::tokens(e) {
                let Some(k) = self.counts.read_kind(&t, a, &e.game) else { continue };
                let w = self.counts.firmness(&t, &k);
                reads.push((w, &self.read[&(t, k)]));
            }
            let score = |i: usize| reads.iter().map(|(w, y)| w * phistd::cosine(y, &table[i])).sum::<f64>();
            let best = self.values[a].iter().map(|(v, i)| (v, score(*i))).max_by(|x, y| x.1.total_cmp(&y.1));
            *slot = best.map(|x| x.0.clone()).unwrap_or_default();
        }
        out
    }
}

/// Temperature turning the operator's cosines into a distribution; fixed
/// before any test was read.
const TEMP: f64 = 0.1;

impl Phient {
    /// Trial 2: the operator's reading of each word, as a distribution over
    /// the aspect's values, put together by movement (`compose`). A relation
    /// told fewer than `consolidate` times is read from its count instead
    /// (two speeds; 0 reads every relation from the functions).
    pub fn movement(&self, e: &Episode, consolidate: f64) -> Action {
        let mut out: Action = Default::default();
        for (a, slot) in out.iter_mut().enumerate() {
            let names: Vec<String> = self.values[a].iter().map(|x| x.0.clone()).collect();
            let dist = |t: &str, k: &str, base: &[f64]| {
                let key = (t.to_string(), k.to_string());
                let young = match consolidate < 0.0 {
                    true => !self.agreed.contains(&key),
                    false => self.counts.told(t, k) < consolidate,
                };
                if young {
                    return self.counts.dist(t, k, &names, base);
                }
                let y = self.read.get(&(t.to_string(), k.to_string()))?;
                let z: Vec<f64> = self.values[a].iter().map(|(_, i)| (phistd::cosine(y, &self.table[*i]) / TEMP).exp()).collect();
                let s: f64 = z.iter().sum();
                Some(z.iter().zip(base).map(|(x, b)| 0.9 * x / s + 0.1 * b).collect())
            };
            *slot = super::compose::movement(&self.counts, e, a, &names, dist);
        }
        out
    }
}

/// Two-speed learning (docs/103): a new relation acts from its count at
/// once — one sighting is enough to move an answer — and is read from the
/// operator's functions once it has been told `consolidate` times. The
/// operator trains on every relation all along; only the reading switches.
pub struct TwoSpeed {
    pub p: std::rc::Rc<Phient>,
    pub consolidate: f64,
}

impl Learner for TwoSpeed {
    fn name(&self) -> String {
        match self.consolidate {
            0.0 => format!("Phient relations, movement (phinet, R={})", self.p.m.joint.fno.config.space),
            n if n < 0.0 => format!("Phient two-speed: counts until the function agrees (phinet, R={})", self.p.m.joint.fno.config.space),
            n => format!("Phient two-speed: counts until {n} sightings (phinet, R={})", self.p.m.joint.fno.config.space),
        }
    }
    fn act(&mut self, e: &Episode) -> Action {
        self.p.movement(e, self.consolidate)
    }
}
