//! A persona as a condition on generation: the writer's sector histogram
//! as a prior over every candidate.
//!
//! In Phient's world a persona is which conditions a writer puts words
//! under, aggregated into a histogram over the 64 sectors of the relation
//! circle (phiano's `persona::Fingerprint`), handed in as that histogram. Until this existed the only way to
//! compose "as" a writer was a separate lexicon per writer (`bin/austen`);
//! this conditions one shared lexicon instead: a candidate whose centred
//! sector the persona favours gains `weight · ln(p_persona(sector) / p_uniform)`,
//! clipped to ±[`PERSONA_CLIP`]. `bin/persona_eval` measures whether an
//! attribution judge then reads the text as that persona's.

use super::generator::Generator;
use phiecs::phasor::facet::Facet;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Largest log-ratio a sector may contribute, either way.
pub const PERSONA_CLIP: f64 = 2.0;

#[derive(Clone)]
pub struct PersonaPrior {
    /// The persona's sector histogram, normalised to sum to one.
    pub hist: Vec<f64>,
    /// How much of the score the persona decides (0 = off).
    pub weight: f64,
    /// The lexicon mean the fingerprint's sectors were centred on.
    mean: Vec<phistd::c64>,
    /// A word's sector, memoised: the same word is scored thousands of times.
    sectors: RefCell<HashMap<String, Option<usize>>>,
    /// The persona's own bigram counts, from its example sentences: the
    /// conditions *this* writer put pairs of words under. `ngram_weight ·
    /// ln(1 + count(prev, word))` joins the bonus when set.
    pub bigrams: HashMap<(String, String), u32>,
    pub ngram_weight: f64,
}

impl PersonaPrior {
    /// `hist`: a persona's sector histogram (its fingerprint's).
    pub fn new(facet: &Facet, hist: &[f64], weight: f64) -> Self {
        let total: f64 = hist.iter().sum::<f64>().max(1e-12);
        let hist = hist.iter().map(|h| h / total).collect();
        Self { hist, weight, mean: crate::net::space::lexicon_mean(facet), sectors: RefCell::new(HashMap::new()), bigrams: HashMap::new(), ngram_weight: 0.0 }
    }

    /// Adds the persona's own bigram table from its example sentences.
    pub fn with_examples(mut self, examples: &[String], ngram_weight: f64) -> Self {
        for e in examples {
            let toks = phiton::p64::words::Tokenizer::tokenize(e);
            for w in toks.windows(2) {
                *self.bigrams.entry((w[0].clone(), w[1].clone())).or_insert(0) += 1;
            }
        }
        self.ngram_weight = ngram_weight;
        self
    }

    /// The bigram part of the bonus, given the previous word.
    pub fn ngram_bonus(&self, prev: Option<&str>, word: &str) -> f64 {
        if self.ngram_weight == 0.0 {
            return 0.0;
        }
        let Some(p) = prev else { return 0.0 };
        let c = self.bigrams.get(&(p.to_string(), word.to_string())).copied().unwrap_or(0);
        self.ngram_weight * (c as f64).ln_1p()
    }

    /// `weight · ln(p(sector) / uniform)`, clipped; zero for an unknown word.
    pub fn bonus(&self, facet: &Facet, word: &str) -> f64 {
        if self.weight == 0.0 || self.hist.is_empty() {
            return 0.0;
        }
        let sector = *self.sectors.borrow_mut().entry(word.to_string()).or_insert_with(|| crate::net::space::relational_sector(facet, word, &self.mean).map(|s| s as usize));
        let Some(s) = sector else { return 0.0 };
        let n = self.hist.len() as f64;
        let p = self.hist.get(s).copied().unwrap_or(0.0).max(1e-4);
        self.weight * (p * n).ln().clamp(-PERSONA_CLIP, PERSONA_CLIP)
    }
}

impl Generator {
    /// ConditionSet generation on a persona (see [`PersonaPrior`]).
    // The prior is shared (`Rc`) with `persona::dialog`, never sent across
    // threads: its sector memo is a `RefCell`.
    pub fn with_persona(self, facet: &Facet, hist: &[f64], weight: f64) -> Self {
        self.with_persona_prior(Rc::new(PersonaPrior::new(facet, hist, weight)))
    }

    /// ConditionSet generation on a prior already built — one persona's
    /// bigram table is large, and a dialogue reuses it across turns and
    /// candidates rather than rebuilding it.
    pub fn with_persona_prior(mut self, prior: Rc<PersonaPrior>) -> Self {
        self.persona = Some(prior);
        self
    }

    /// The persona's additive bonus for a candidate (zero without a persona):
    /// the sector prior plus the persona's own bigram evidence for the pair
    /// (previous word on the current path, candidate).
    pub(super) fn persona_bonus(&self, facet: &Facet, word: &str) -> f64 {
        let Some(p) = &self.persona else { return 0.0 };
        let ctx = self.ctx.borrow();
        p.bonus(facet, word) + p.ngram_bonus(ctx.last().map(|s| s.as_str()), word)
    }

    /// ConditionSet generation on a persona and its own bigram table.
    pub fn with_persona_examples(self, facet: &Facet, hist: &[f64], examples: &[String], weight: f64, ngram_weight: f64) -> Self {
        self.with_persona_prior(Rc::new(PersonaPrior::new(facet, hist, weight).with_examples(examples, ngram_weight)))
    }
}
