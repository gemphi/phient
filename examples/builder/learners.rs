//! The learner trait, and Phical counts — what each token relates to under
//! each condition, with a count. The counts are phient's (`relate::Tally`);
//! what is the game's own — which game a word may be read in, its posts,
//! the teacher — is here. (Lookup is in `lookup.rs`, the naming table in
//! `naming.rs`.)

use super::world::{Episode, ASPECTS};
use phiecs::quest::Row;
use std::collections::{BTreeSet, HashMap};

/// The kind a told triple's token is given under, in phient's tally.
const SAYS: &str = "⟨says⟩";

pub type Action = [String; 5];

pub trait Learner {
    fn name(&self) -> String;
    fn act(&mut self, e: &Episode) -> Action;
}

/// The condition token of an episode: the game is part of what relates.
pub fn game_token(e: &Episode) -> String {
    format!("⟨game:{}⟩", e.game)
}

/// What an episode tells, cross-situationally: every token relates to every
/// aspect of what followed. A grammar word (`colour`, §30) stations the next
/// word at its post: that word is told under the colour aspect only.
pub fn told(e: &Episode) -> Vec<(String, usize, String)> {
    let post = e.tokens.first().map(String::as_str) == Some("colour");
    let mut out = Vec::new();
    let mut toks: Vec<String> = e.tokens.clone();
    toks.push(game_token(e));
    for (i, t) in toks.iter().enumerate() {
        match (post, i) {
            (true, 0) => {}
            (true, 1) => out.push((t.clone(), 2, e.action[2].clone())),
            _ => out.extend((0..ASPECTS.len()).map(|a| (t.clone(), a, e.action[a].clone()))),
        }
    }
    out
}

/// How the tokens' readings are put together. `Sum`: each token votes its
/// reading, weighted by firmness (trial 1). `Movement`: each token counts
/// only as far as it moves the answer away from what the condition alone
/// says — log P(v | t, k) − log P(v | k), smoothed toward the condition
/// (trial 2; naive Bayes in the counts, the same difference in phinet).
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Compose {
    #[default]
    Sum,
    Movement,
}

/// The game's counts of `(token, kind) ⇒ value`, kinds `aspect|game` and
/// `aspect`, held by phient's `relate::Tally`: each told triple is one row
/// there, its token given under `SAYS`, so a word said twice counts twice.
#[derive(Default)]
pub struct GameCounts {
    pub tally: ::phient::relate::Tally,
    /// Every token told, sorted (game tokens included).
    pub words: Vec<String>,
    pub compose: Compose,
    /// Nats a word must move an aspect to hold a post there (`compose::POST`
    /// unless swept, docs/105).
    pub post: f64,
    /// Per aspect, when set: a teacher of tendencies (docs/111).
    pub post_by: Vec<f64>,
    /// Per aspect: may a word be read across games (see `compose`).
    pub open: Vec<bool>,
    /// Token ⇒ the games it was told in.
    pub games: HashMap<String, Vec<String>>,
}

pub fn kind(a: usize, game: Option<&str>) -> String {
    match game {
        Some(g) => format!("{}|{g}", ASPECTS[a]),
        None => ASPECTS[a].to_string(),
    }
}

impl GameCounts {
    pub fn new(train: &[Episode]) -> Self {
        let mut c = GameCounts { tally: ::phient::relate::Tally::new(&[SAYS]), post: super::compose::POST, ..GameCounts::default() };
        let (mut rows, mut words) = (Vec::new(), BTreeSet::new());
        for (i, e) in train.iter().enumerate() {
            for (j, (t, a, v)) in told(e).into_iter().enumerate() {
                let about = format!("⟨{i}·{j}⟩");
                rows.push(Row::new(&about, SAYS, &t));
                for k in [kind(a, Some(&e.game)), kind(a, None)] {
                    rows.push(Row::new(&about, &k, &v));
                }
                let gs = c.games.entry(t.clone()).or_default();
                if !gs.iter().any(|x| *x == e.game) {
                    gs.push(e.game.clone());
                }
                words.insert(t);
            }
        }
        c.tally.learn(&rows);
        c.words = words.into_iter().collect();
        let mut names: Vec<String> = train.iter().map(|e| e.game.clone()).collect();
        names.sort();
        names.dedup();
        c.open = (0..ASPECTS.len())
            .map(|a| names.iter().any(|g1| names.iter().any(|g2| g1 < g2 && c.agree_between(a, g1, g2) == Some(true))))
            .collect();
        c
    }

    /// The condition a token is read under for aspect `a`: its game's, or
    /// any game's when it was never told in this one.
    pub fn read_kind(&self, t: &str, a: usize, game: &str) -> Option<String> {
        [kind(a, Some(game)), kind(a, None)].into_iter().find(|k| self.has(t, k))
    }

    /// Was `t` told under `k` at all?
    pub fn has(&self, t: &str, k: &str) -> bool {
        self.tally.told(t, k) > 0.0
    }

    /// What `t` was told under `k`, value ⇒ count; `None` when never.
    pub fn table(&self, t: &str, k: &str) -> Option<HashMap<String, f64>> {
        let m: HashMap<String, f64> = self.tally.said(Some(t), k).into_iter().collect();
        (!m.is_empty()).then_some(m)
    }

    /// Every `(token, kind)` told, in the words' order.
    pub fn pairs(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for t in &self.words {
            for a in 0..ASPECTS.len() {
                let games = self.games.get(t).map(Vec::as_slice).unwrap_or(&[]);
                let ks = games.iter().map(|g| kind(a, Some(g))).chain([kind(a, None)]);
                out.extend(ks.filter(|k| self.has(t, k)).map(|k| (t.clone(), k)));
            }
        }
        out
    }

    /// How firmly a token relates under a kind: its top count over its
    /// total plus one — one sighting is never certainty.
    pub fn firmness(&self, t: &str, k: &str) -> f64 {
        self.table(t, k).map_or(0.0, |m| {
            let total: f64 = m.values().sum();
            m.values().cloned().fold(0.0, f64::max) / (total + 1.0)
        })
    }

    /// How many times `(t, k)` was told.
    pub fn told(&self, t: &str, k: &str) -> f64 {
        self.tally.told(t, k)
    }

    /// Every value told under aspect `a`.
    pub fn values(&self, a: usize) -> Vec<String> {
        self.tally.values(ASPECTS[a])
    }

    pub fn tokens(e: &Episode) -> Vec<String> {
        let mut t = e.tokens.clone();
        t.push(game_token(e));
        t
    }
}

impl Learner for GameCounts {
    fn name(&self) -> String {
        match self.compose {
            Compose::Sum => "Phical counts, summed (no operator)".into(),
            Compose::Movement => "Phical counts, movement (no operator)".into(),
        }
    }
    fn act(&mut self, e: &Episode) -> Action {
        if self.compose == Compose::Movement {
            return self.movement(e);
        }
        let mut out: Action = Default::default();
        for (a, slot) in out.iter_mut().enumerate() {
            let mut score: HashMap<String, f64> = HashMap::new();
            for t in GameCounts::tokens(e) {
                let Some(k) = self.read_kind(&t, a, &e.game) else { continue };
                let w = self.firmness(&t, &k);
                let Some(m) = self.table(&t, &k) else { continue };
                let total: f64 = m.values().sum();
                for (v, c) in &m {
                    *score.entry(v.clone()).or_default() += w * c / total;
                }
            }
            *slot = score.into_iter().max_by(|x, y| x.1.total_cmp(&y.1).then(y.0.cmp(&x.0))).map(|x| x.0).unwrap_or_default();
        }
        out
    }
}

impl GameCounts {
    /// The count distribution of `(t, k)` over `values`, smoothed toward
    /// `base` with weight one: one sighting is never certainty. (phient's
    /// `Tally::dist` is the same sum, added in another order; this one keeps
    /// the game's recorded numbers to the last bit.)
    pub fn dist(&self, t: &str, k: &str, values: &[String], base: &[f64]) -> Option<Vec<f64>> {
        let m = self.table(t, k)?;
        let total: f64 = m.values().sum();
        Some(values.iter().zip(base).map(|(v, b)| (m.get(v).copied().unwrap_or(0.0) + b) / (total + 1.0)).collect())
    }

    fn movement(&self, e: &Episode) -> Action {
        let mut out: Action = Default::default();
        for (a, slot) in out.iter_mut().enumerate() {
            let values = self.values(a);
            *slot = super::compose::movement(self, e, a, &values, |t, k, base| self.dist(t, k, &values, base));
        }
        out
    }
}
