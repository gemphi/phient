//! The generator: its knobs (weights, operator, decode mode, goal) and constructors.

use super::guide::OperatorGuide;
use phiecs::phasor::facet::Facet;


/// Generator - phase-guided sequence sampler.
pub struct Generator {
    pub max_tokens: usize,
    pub temperature: f64,
    /// Sampling state. `Cell` because decoding takes `&self`.
    pub(super) rng: std::cell::Cell<u64>,
    /// Optional Phinet operator. `RefCell` because decoding takes `&self`
    /// and the operator's tape is mutable.
    pub operator: Option<std::cell::RefCell<OperatorGuide>>,
    /// Learned log-linear candidate weights (`learn::GenWeight`). When set,
    /// candidate scores are logits `w·x` and sampling is a softmax over them;
    /// when not, the hand-tuned multiplicative scorer is used.
    pub weights: Option<crate::generate::learn::GenWeight>,
    /// The endgame: a word the composition is to reach, and how hard to pull
    /// toward it. Every candidate's score gains `weight · calls_for(cand, goal)`
    /// until the goal has been emitted. `None` composes freely.
    pub goal: Option<(String, f64)>,
    /// The prompt plus everything emitted on the current path — the wheels
    /// the relational features read against. Kept whether or not an
    /// operator is attached.
    pub(super) ctx: std::cell::RefCell<Vec<String>>,
    /// How many relational proposals join the n-gram's candidates per step
    /// (`RELATIONAL_PROPOSALS` by default; 0 = none).
    pub relational_proposals: usize,
    /// Where the candidate set comes from each step (see [`Decode`]).
    pub decode: Decode,
    /// The persona the composition is conditioned on, if any
    /// (`with_persona`; see `generate::persona`).
    pub persona: Option<std::rc::Rc<super::persona::PersonaPrior>>,
    /// Which channels the goal's pull reads (`PHIANO_READING`).
    pub reading: phiecs::phasor::facet::Reading,
}

/// Where a step's candidates come from.
///
/// The trigram sampler was the ceiling of every composition number
/// (docs/74–75): a candidate had to be an n-gram continuation, so the
/// operator and the relational reading could only re-weight what the tables
/// already allowed. `Operator` decodes from the function space instead — the
/// `k` words nearest the operator's prediction — and lets the scorer choose
/// among them with the n-gram count as one feature rather than the gate;
/// `Union` offers both sets. `bin/decode_eval` measures the three.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Decode {
    /// n-gram continuations only (the original sampler).
    Trigram,
    /// the `k` nearest words to the operator's prediction, then bigram fallback.
    Operator { k: usize },
    /// n-gram continuations plus the `k` nearest words.
    Union { k: usize },
    /// Liskov's forcing choice: the `k` best of the `2k` nearest by fit less
    /// `lambda` × refutation (`phical::liskov`), then bigram fallback.
    Forcing { k: usize, lambda: f64 },
}

/// Default number of relational proposals per step.
/// Consecutive function words after which the next pick must be a content
/// word (the composition no longer ends here).
pub(super) const FUNCTION_STREAK_LIMIT: usize = 4;
pub const RELATIONAL_PROPOSALS: usize = 0;

/// Context words the relational features look back over.
pub const RELATIONAL_WINDOW: usize = 6;

impl Generator {
    /// Creates a generator seeded from the clock, so repeated calls differ.
    pub fn new(max_tokens: usize, temperature: f64) -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15);
        Self { max_tokens, temperature, rng: std::cell::Cell::new(seed | 1), operator: None, weights: crate::generate::learn::GenWeight::load_default().map(|w| w.for_decoding()), ctx: std::cell::RefCell::new(Vec::new()), relational_proposals: RELATIONAL_PROPOSALS, goal: None, decode: Decode::Trigram, persona: None, reading: phiecs::phasor::facet::Reading::from_env() }
    }

    /// Creates a generator with a fixed seed, for reproducible output. Does
    /// not load learned weights; attach them with [`Self::with_weights`].
    pub fn deterministic(max_tokens: usize, temperature: f64, seed: u64) -> Self {
        Self { max_tokens, temperature, rng: std::cell::Cell::new(seed | 1), operator: None, weights: None, ctx: std::cell::RefCell::new(Vec::new()), relational_proposals: RELATIONAL_PROPOSALS, goal: None, decode: Decode::Trigram, persona: None, reading: phiecs::phasor::facet::Reading::from_env() }
    }

    /// Scores candidates with learned weights (or `None` for the hand scorer).
    pub fn with_weights(mut self, weights: Option<crate::generate::learn::GenWeight>) -> Self {
        self.weights = weights.map(|w| w.for_decoding());
        self
    }

    /// The operator's cosine for a word, or 0 without an operator.
    pub(super) fn operator_cos(&self, word: &str) -> f64 {
        match &self.operator {
            Some(op) => op.borrow_mut().cosine(word).unwrap_or(0.0),
            None => 0.0,
        }
    }

    /// Attaches a trained operator; candidates are then weighted by it.
    /// Chooses where candidates come from (see [`Decode`]).
    pub fn with_decode(mut self, decode: Decode) -> Self {
        self.decode = decode;
        self
    }

    /// Sets the endgame (see `goal`).
    pub fn with_goal(mut self, goal: &str, weight: f64) -> Self {
        self.goal = Some((goal.to_string(), weight));
        self
    }

    /// The goal bonus for a candidate, zero once the goal has been emitted.
    pub(super) fn goal_bonus(&self, facet: &Facet, word: &str) -> f64 {
        match &self.goal {
            Some((g, w)) if *w != 0.0 => {
                if self.ctx.borrow().iter().any(|t| t == g) {
                    0.0
                } else if word == g {
                    *w
                } else {
                    w * facet.calls_for_with(word, g, self.reading)
                }
            }
            _ => 0.0,
        }
    }

    pub fn with_operator(mut self, guide: OperatorGuide) -> Self {
        self.operator = Some(std::cell::RefCell::new(guide));
        self
    }

    pub(super) fn operator_reset(&self, prompt_tokens: &[String]) {
        *self.ctx.borrow_mut() = prompt_tokens.to_vec();
        if let Some(op) = &self.operator {
            op.borrow_mut().reset(prompt_tokens);
        }
    }

    pub(super) fn operator_note(&self, word: &str) {
        self.ctx.borrow_mut().push(word.to_string());
        if let Some(op) = &self.operator {
            op.borrow_mut().note(word);
        }
    }

    /// Points the guide at one beam hypothesis: the prompt plus what that
    /// hypothesis has emitted. One operator application per hypothesis per
    /// step; the cache makes the per-candidate cosines free after that.
    pub(super) fn operator_focus(&self, prompt: &[String], emitted: &[String]) {
        {
            let mut c = self.ctx.borrow_mut();
            c.clear();
            c.extend_from_slice(prompt);
            c.extend_from_slice(emitted);
        }
        if let Some(op) = &self.operator {
            op.borrow_mut().set_history(prompt, emitted);
        }
    }

    /// The candidate-list length to keep: the guide's `widen` if set, else
    /// the unguided default.
    pub(super) fn keep(&self, default: usize) -> usize {
        match &self.operator {
            Some(op) if op.borrow().widen > 0 => op.borrow().widen,
            _ => default,
        }
    }

}
