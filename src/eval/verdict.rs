#[allow(unused_imports)]
use super::*;

impl Quality {
    /// Determines a verdict from the three eval scores.
    ///
    /// `novelty` may be NaN when the sentence wave is too small for its
    /// direction to mean anything; every comparison below is written so that an
    /// undefined novelty falls through to a coherence-and-resonance judgement
    /// rather than silently taking a branch.
    pub fn from_scores(coherence: f64, novelty: f64, resonance: f64) -> Self {
        if resonance < 0.3 {
            return Self::Noise;
        }
        if coherence < 0.2 {
            return if novelty > 0.7 { Self::DissonantNovel } else { Self::Incoherent };
        }
        if coherence > 0.7 && novelty < 0.3 {
            return Self::CoherentGrounded;
        }
        if coherence > 0.7 && novelty > 0.6 {
            return Self::CoherentNovel;
        }
        if coherence > 0.5 {
            return if novelty > 0.5 { Self::ModerateNovel } else { Self::CoherentFamiliar };
        }
        Self::WeaklyCoherent
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty input"),
            Self::Noise => write!(f, "Noise-like, mostly unknown vocabulary"),
            Self::DissonantNovel => write!(f, "Dissonant but novel - may be creative or nonsensical"),
            Self::Incoherent => write!(f, "Incoherent - words do not resonate together"),
            Self::CoherentGrounded => write!(f, "Coherent and well-grounded"),
            Self::CoherentNovel => write!(f, "Coherent and novel - insightful"),
            Self::ModerateNovel => write!(f, "Moderately coherent with novel elements"),
            Self::CoherentFamiliar => write!(f, "Coherent and familiar"),
            Self::WeaklyCoherent => write!(f, "Weakly coherent - marginal meaning"),
        }
    }
}

impl fmt::Display for Eval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let novelty = match self.novelty.is_nan() {
            true => "  n/a".to_string(),
            false => format!("{:.2}", self.novelty),
        };
        write!(
            f,
            "  Coherence: {:.2}  Novelty: {}  Resonance: {:.2}  Overall: {:.2}\n  Verdict: {}",
            self.coherence, novelty, self.resonance, self.overall, self.verdict,
        )
    }
}

impl Evaluator {
    /// Creates a new evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates a text string against the facet.
    ///
    /// - **Resonance**: fraction of input tokens the facet knows
    /// - **Coherence**: how well the known words' wave superposition aligns
    /// - **Novelty**: distance of the input wave from the facet centroid
    /// - **Overall**: weighted combination (coherence 45%, resonance 40%, novelty 15%)
    pub fn eval(&self, facet: &Facet, text: &str) -> Eval {
        let tokens = Tokenizer::tokenize(text);
        if tokens.is_empty() {
            return Eval {
                coherence: 0.0,
                novelty: 0.0,
                resonance: 0.0,
                overall: 0.0,
                verdict: Quality::Empty,
            };
        }

        let known = tokens.iter().filter(|t| facet.contains_word(t)).count();
        let resonance = known as f64 / tokens.len() as f64;

        let wave = Wave::sentence(facet, &tokens);
        let centroid = facet.centroid();

        let coherence = if known == 0 {
            0.0
        } else if known == 1 {
            // Single word: measure alignment with facet centroid direction
            let alignment = (centroid.arg() - wave.arg()).cos().max(0.0);
            alignment * 0.5 + 0.25
        } else {
            // Multiple words: Kuramoto order parameter
            (wave.norm() / known as f64).clamp(0.0, 1.0)
        };

        // Novelty is the *direction* of the sentence wave relative to the
        // lexicon centroid — but the direction of a near-zero vector is
        // floating-point noise. Exactly when coherence is lowest, `arg()` is
        // least meaningful, so novelty is left undefined rather than folded
        // into `overall` as though it were a measurement.
        let degenerate = known == 0
            || wave.norm() < 0.1 * known as f64
            || facet.vocabulary_size() == 0
            || centroid.norm() < 1e-10;

        let novelty = match degenerate {
            true => f64::NAN,
            false => {
                let angular_dist = ((centroid.arg() - wave.arg()).abs()).min(PI);
                let normalized = angular_dist / PI;
                (1.0 - (-(normalized * NOVELTY_SCALE * 5.0).min(20.0)).exp())
                    .clamp(0.0, 1.0)
            }
        };

        let overall = match novelty.is_nan() {
            // Renormalise across the terms that are actually defined.
            true => {
                let w = EVAL_WEIGHT_COHERENCE + EVAL_WEIGHT_RESONANCE;
                (coherence * EVAL_WEIGHT_COHERENCE
                    + resonance * EVAL_WEIGHT_RESONANCE)
                    / w
            }
            false => Evaluator::overall(coherence, novelty, resonance),
        };

        let verdict = Quality::from_scores(coherence, novelty, resonance);

        Eval {
            coherence,
            novelty,
            resonance,
            overall,
            verdict,
        }
    }

}

impl Evaluator {
    /// Evaluates a text, measuring novelty against **experience** rather than
    /// geometry.
    ///
    /// Centroid-distance novelty degrades as the lexicon grows: the centroid
    /// becomes a stable average that barely moves, and under phase collapse it
    /// converges on the point every word is converging on, so novelty tends to
    /// zero for everything. Distance to the nearest thing ever processed does
    /// not have that failure mode, and the memory log has been recording it
    /// since the beginning.
    pub fn eval_with_memory(&self, facet: &Facet, memo: &crate::memory::Memo, text: &str) -> Eval {
        let mut base = self.eval(facet, text);
        if memo.is_empty() {
            return base;
        }
        let wave = Wave::text_bound(facet, text);
        base.novelty = memo.novelty((wave.re, wave.im));
        base.overall =
            Evaluator::overall(base.coherence, base.novelty, base.resonance);
        base.verdict = Quality::from_scores(base.coherence, base.novelty, base.resonance);
        base
    }
}

impl Evaluator {
    /// The overall score: `coherence × EVAL_WEIGHT_COHERENCE + novelty ×
    /// EVAL_WEIGHT_NOVELTY + resonance × EVAL_WEIGHT_RESONANCE`.
    pub fn overall(coherence: f64, novelty: f64, resonance: f64) -> f64 {
        coherence * EVAL_WEIGHT_COHERENCE + novelty * EVAL_WEIGHT_NOVELTY + resonance * EVAL_WEIGHT_RESONANCE
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
