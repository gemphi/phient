//! A trained Phinet operator as a guide to generation.

use phiecs::phasor::facet::Facet;

mod read;

/// A trained Phinet operator guiding generation.
///
/// The operator reads the context's *function* on the relation circle and
/// returns the function the next word should have; every candidate is then
/// weighted by its cosine against that function (`OPERATOR_WEIGHT`). Until
/// this existed generation scored candidates by manifold coherence only,
/// which reads 177 perplexity where the operator reads 134 (docs/65–68).
pub struct OperatorGuide {
    pub fno: phical::Fno,
    pub space: crate::net::space::FunctionSpace,
    pub window: usize,
    /// How much of a candidate's score the operator decides; defaults to
    /// [`OPERATOR_WEIGHT`].
    pub weight: f64,
    /// How many candidates the operator proposes per step; defaults to
    /// [`OPERATOR_PROPOSALS`].
    pub proposals: usize,
    /// How many n-gram candidates to keep when the operator is present
    /// (the unguided generator keeps 12–16). The operator's job is then to
    /// choose among *more* n-gram-legal continuations, not to invent them.
    pub widen: usize,
    /// Each word's cosine against the operator's output for the *average*
    /// context: what the word scores for fitting every context. A proposal
    /// is scored by how much it beats its own prior, so frequent target
    /// sides — which fit every context — stop winning by default.
    pub(super) prior: Vec<f64>,
    /// Tokens seen so far — the prompt, then everything emitted.
    pub(super) history: Vec<String>,
    /// The operator's output for the current history, cached per length.
    pub(super) cached: Option<(usize, crate::net::space::WordFunction)>,
    /// I Ching novelty: `(hexagram, every)` — every `every`-th emitted token
    /// the prediction is passed through the hexagram's spectral gate
    /// (`phistd::hexagram::apply`), so the candidates are read against a
    /// function with some of its octaves removed. `None` = no novelty.
    pub novelty: Option<(u8, usize)>,
    /// The prompt as a standing condition: its mean function is added to
    /// every step's context with this weight (relative to the unit window
    /// context) before the operator reads it. `0` = the window alone, which
    /// forgets the prompt after `window` tokens and lets one orbit own every
    /// prompt (docs/78). Defaults to [`OPERATOR_ANCHOR`].
    pub anchor: f64,
    /// The prompt's mean function, set at `reset`.
    pub(super) anchor_fn: Option<crate::net::space::WordFunction>,
}

/// Default weight of the prompt anchor in the operator's context.
pub const OPERATOR_ANCHOR: f64 = 0.0;

/// How much of a candidate's score the operator decides: the factor is
/// `(1 − w) + w · (1 + cos) / 2`.
pub const OPERATOR_WEIGHT: f64 = 0.5;
/// How many n-gram candidates the generator keeps per step when an operator
/// is attached (0 = the unguided 12/16). See `OperatorGuide::widen`.
pub const OPERATOR_WIDEN: usize = 0;
/// How many candidates the operator proposes alongside the n-gram's.
///
/// Off by default. Measured at 8 on the Rust Book (`bin/generate_eval`): the
/// proposals are dominated by frequent target sides ("the", "in", "we",
/// "parameter"), continuations halve in length and the Kneser-Ney judge goes
/// from 13.6 to 181. A trained operator proposes far better than an
/// untrained one (10,728), so the signal is real; the frequency bias of the
/// proposal step is the problem, and it is open (docs/71).
pub const OPERATOR_PROPOSALS: usize = 0;
/// Words a proposal may draw from must have been seen this often.
pub const OPERATOR_PROPOSAL_FLOOR: u32 = 10;
/// The pseudo-count a proposed candidate carries into scoring (the n-gram
/// count term is `ln(1 + min(count, 24))`, so 8 is a middling candidate).
pub const OPERATOR_PROPOSAL_COUNT: u32 = 8;

impl OperatorGuide {
    /// Loads a `.fno` file and encodes the facet with `sides`. Fails if the
    /// operator's space does not match the encoding's sample count.
    pub fn load(path: &str, facet: &Facet, sides: crate::net::space::Sides, window: usize) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let fno = phical::Fno::from_bytes(&bytes).ok_or_else(|| format!("{path}: not a phinet operator"))?;
        let space = crate::net::space::FunctionSpace::from_facet_sides(facet, sides);
        if !space.evidenced {
            return Err(format!(
                "{path}: the loaded lexicon carries no relation evidence (a v1–v3 file); an operator trained on an evidenced lexicon has nothing to read here — retrain the manifold first"
            ));
        }
        if fno.config.space != space.space() {
            return Err(format!(
                "{path}: operator space {} but the lexicon encodes to {} samples ({sides:?})",
                fno.config.space,
                space.space()
            ));
        }
        let mut guide = Self { fno, space, window, weight: OPERATOR_WEIGHT, proposals: OPERATOR_PROPOSALS, widen: OPERATOR_WIDEN, prior: Vec::new(), history: Vec::new(), cached: None, novelty: None, anchor: std::env::var("PHIANO_ANCHOR").ok().and_then(|v| v.parse().ok()).unwrap_or(OPERATOR_ANCHOR), anchor_fn: None };
        guide.compute_prior();
        Ok(guide)
    }

    /// The prior of every word: the operator applied to the mean source
    /// function, compared with each target side.
    pub(super) fn compute_prior(&mut self) {
        let n = self.space.len();
        if n == 0 {
            return;
        }
        let s = self.space.space();
        let mut mean = vec![phistd::c64::new(0.0, 0.0); s];
        for f in &self.space.funcs {
            for (m, z) in mean.iter_mut().zip(f) {
                *m += z;
            }
        }
        if !phistd::normalise(&mut mean) {
            self.prior = vec![0.0; n];
            return;
        }
        let y = self.fno.apply(&[mean]).into_iter().next().unwrap_or_default();
        let table = self.space.table();
        self.prior = (0..n).map(|i| phistd::cosine(&y, &table[i])).collect();
    }

    pub(super) fn reset(&mut self, prompt_tokens: &[String]) {
        self.history = prompt_tokens.to_vec();
        self.cached = None;
        let ids: Vec<usize> = prompt_tokens.iter().filter_map(|w| self.space.index.get(w).copied()).collect();
        self.anchor_fn = self.space.context(&ids, ids.len(), ids.len().max(1)).filter(|_| self.anchor > 0.0);
    }

    pub(super) fn note(&mut self, word: &str) {
        self.history.push(word.to_string());
    }

    /// Points the guide at an arbitrary token history (the tuner uses this).
    pub fn focus(&mut self, history: &[String]) {
        self.set_history(history, &[]);
    }

    /// Public prediction for measurement bins.
    pub fn predict_public(&mut self) -> Option<crate::net::space::WordFunction> {
        self.predict()
    }

    /// Public cosine for the tuner.
    pub fn cosine_of(&mut self, word: &str) -> Option<f64> {
        self.cosine(word)
    }

    /// Replaces the history wholesale — one hypothesis of a beam.
    pub(super) fn set_history(&mut self, prompt: &[String], emitted: &[String]) {
        self.history.clear();
        self.history.extend_from_slice(prompt);
        self.history.extend_from_slice(emitted);
        self.cached = None;
    }

    /// The operator's prediction for the next word's function.
    pub(super) fn predict(&mut self) -> Option<crate::net::space::WordFunction> {
        if let Some((n, f)) = &self.cached {
            if *n == self.history.len() {
                return Some(f.clone());
            }
        }
        let ids: Vec<usize> = self.history.iter().filter_map(|w| self.space.index.get(w).copied()).collect();
        if ids.is_empty() {
            return None;
        }
        let mut ctx = self.space.context(&ids, ids.len(), self.window)?;
        if let Some(a) = &self.anchor_fn {
            // The condition c the words move under: the prompt does not
            // scroll out of the window.
            for (z, p) in ctx.iter_mut().zip(a) {
                *z += p * self.anchor;
            }
            phistd::normalise(&mut ctx);
        }
        let mut y = self.fno.apply(&[ctx]).into_iter().next()?;
        if let Some((hex, every)) = self.novelty {
            if every > 0 && self.history.len().is_multiple_of(every) {
                y = phistd::hexagram::apply(&y, hex);
            }
        }
        self.cached = Some((self.history.len(), y.clone()));
        Some(y)
    }
}
