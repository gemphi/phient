//! The error-driven reader (docs/203): the phasors LEARN instead of count.
//!
//! Every reader before this one moves a word's angle toward the mean of
//! what it was seen with (`Phasor::couple`, a running average): it is told,
//! never corrected, and nothing pushes a word away from what does NOT
//! follow it — so the words reached from everywhere sit at the mean of
//! everything, one crowd (docs/200–202). Here a word is the same thing,
//! angles on closed circles, but every reading is a PREDICTION, the
//! prediction is scored against the word that came, and the error moves
//! the angles: toward the word that came, away from the words that were
//! expected and did not.
//!
//! A word holds `channels` unit phasors per role, born at
//! [`phistd::seed_angles`] so the same word starts at the same angles
//! everywhere: `reach` (the word as the thing reaching), `reached` (the
//! word as the thing reached — the domain/range split of docs/198, or the
//! same angles when `tie` is set) and `cond` (the word as the condition on
//! the word after it). The condition before a position is one complex
//! number per channel,
//!
//! ```text
//! z_i = Σ_d Σ_{j ∈ block(i)} op_d[i][j] · reach(w_{t−d})_j  +  pair_i · reach(w_{t−1})_i · cond(w_{t−2})_i
//! ```
//!
//! — each earlier word turned by its distance's own OPERATOR (a relation is
//! a rotation, docs/187; here it is learned, born as the old eighth-turn
//! on the diagonal, and with `block` > 1 the channels of a block pass phase
//! to each other), plus the PAIR: a product of phasors adds their angles,
//! so the word before the last rotates the last — the transition docs/202
//! asked for. A candidate `v` scores `bias_v + Σ_i Re(z_i · conj
//! reached(v)_i)`: agreement on each channel, weighed by the condition's
//! amplitude. The scores are a softmax over the vocabulary, the loss is
//! cross-entropy, and the gradient is taken by hand ([`Learner::step`]).
//!
//! Training contrasts the word that came with `negatives` words drawn by
//! how often they were told (to the ¾ power), the draw corrected away
//! (sampled softmax, Jean et al. 2015) so the scores stay full-softmax
//! scores. That frequency chooses only which wrong answers to contrast;
//! it is never the answer. Every word angle is put back on the unit circle
//! after each step: a word is its angles, the confidence lives in the
//! condition's amplitude.

mod condition;
mod config;
mod plane;
mod read;
mod reader;
mod scratch;
mod step;
mod train;
#[cfg(test)]
mod tests;

pub use config::Config;

use plane::Plane;
use std::collections::HashMap;

/// The three words every vocabulary starts with.
const START: u32 = 0;
const END: u32 = 1;
const UNKNOWN: u32 = 2;
/// The first real word: candidates are `FIRST..`, never the markers.
const FIRST: u32 = 3;
const SAMPLER: usize = 1 << 20;

#[derive(Clone)]
pub struct Learner {
    pub config: Config,
    word: Vec<String>,
    index: HashMap<String, u32>,
    /// Times each word was told: only for drawing wrong answers and for
    /// teaching `<unk>`, never read as an answer.
    seen: Vec<u64>,
    reach: Plane,
    /// Empty when `tie`: the reach angles answer for both roles.
    reached: Plane,
    cond: Plane,
    /// Row `dist · channels + i`: output channel `i`'s weights over the
    /// `block` input channels of its block, at that distance.
    op: Plane,
    pair: Plane,
    bias: Vec<f32>,
    bias_acc: Vec<f32>,
    told: Vec<Vec<u32>>,
    sampler: Vec<u32>,
    /// ln q(v) of the draw, per word, for the correction.
    logq: Vec<f32>,
    /// ln q of a word told once, for words interned since the last build.
    fresh_logq: f32,
    /// Vocabulary size when the sampler was last built.
    sampled: usize,
    rng: u64,
}

impl Default for Learner {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Learner {
    pub fn new(mut config: Config) -> Self {
        let d = config.channels;
        config.block = config.block.clamp(1, d);
        while d % config.block != 0 {
            config.block -= 1;
        }
        let b = config.block;
        let mut l = Self {
            word: Vec::new(),
            index: HashMap::new(),
            seen: Vec::new(),
            reach: Plane::new(d),
            reached: Plane::new(d),
            cond: Plane::new(d),
            op: Plane::new(b),
            pair: Plane::new(d),
            bias: Vec::new(),
            bias_acc: Vec::new(),
            told: Vec::new(),
            sampler: Vec::new(),
            logq: Vec::new(),
            fresh_logq: 0.0,
            sampled: 0,
            rng: phistd::mix64(config.seed),
            config,
        };
        for m in ["<s>", "</s>", "<unk>"] {
            l.intern(m);
        }
        // Each distance is born as the old fixed turn on the diagonal, an
        // eighth per step (docs/187), at an amplitude that fades with
        // distance and is small enough across `d` channels that the first
        // readings are unsure; the pair starts quiet. All of it is learned.
        let eighth = std::f64::consts::TAU / 8.0;
        let quiet = (4.0 / d as f64).sqrt();
        for dist in 0..l.config.context {
            let (c, s) = ((eighth * (dist + 1) as f64).cos(), (eighth * (dist + 1) as f64).sin());
            let amp = quiet / (dist + 1) as f64;
            for i in 0..d {
                l.op.push((0..b).map(|_| 0.0), 0.0);
                let at = (dist * d + i) * b + i % b;
                l.op.re[at] = (amp * c) as f32;
                l.op.im[at] = (amp * s) as f32;
            }
        }
        l.pair.push((0..d).map(|_| 0.0), 0.1);
        l
    }

    /// The words told so far, markers excluded.
    pub fn vocabulary(&self) -> usize {
        self.word.len() - FIRST as usize
    }

    fn intern(&mut self, w: &str) -> u32 {
        if let Some(&i) = self.index.get(w) {
            return i;
        }
        let i = self.word.len() as u32;
        let d = self.config.channels;
        let a = phistd::seed_angles(w, 3 * d);
        self.reach.push(a[..d].iter().copied(), 1.0);
        if !self.config.tie {
            self.reached.push(a[d..2 * d].iter().copied(), 1.0);
        }
        self.cond.push(a[2 * d..].iter().copied(), 1.0);
        self.bias.push(0.0);
        self.bias_acc.push(0.0);
        self.seen.push(0);
        self.word.push(w.to_string());
        self.index.insert(w.to_string(), i);
        i
    }

    fn id(&self, w: &str) -> u32 {
        self.index.get(w).copied().unwrap_or(UNKNOWN)
    }

    fn next_random(&mut self) -> u64 {
        self.rng = phistd::mix64(self.rng);
        self.rng
    }

    fn chance(&mut self) -> f32 {
        phistd::random::unit_interval_f32(self.next_random())
    }

    /// The table a candidate is read from.
    fn answers(&self) -> &Plane {
        match self.config.tie {
            true => &self.reach,
            false => &self.reached,
        }
    }

    /// The context before position `j` of `line`: the last word first,
    /// padded with `<s>`.
    fn context(&self, line: &[u32], j: usize) -> Vec<u32> {
        (1..=self.config.context).map(|d| if j >= d { line[j - d] } else { START }).collect()
    }

    fn context_of(&self, said: &[String]) -> Vec<u32> {
        let ids: Vec<u32> = said.iter().map(|w| self.id(w)).collect();
        self.context(&ids, ids.len())
    }
}
