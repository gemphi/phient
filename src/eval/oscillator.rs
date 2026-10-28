//! OscillatorEval and ComparisonResult - evaluation results.

use phiecs::phasor::oscillator::OscillatorField;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use std::fmt;

/// OscillatorEval - the result of evaluating text in oscillator mode.
pub struct OscillatorEval {
    pub text: String,
    pub coherence: f64,
    pub sync: f64,
    pub entropy: f64,
    pub dominant_colors: Vec<(String, f64)>,
    pub word_count: usize,
}

impl OscillatorEval {
    /// Evaluates text using the oscillator model.
    pub fn evaluate(field: &OscillatorField, text: &str) -> Self {
        let tokens = Tokenizer::tokenize(text);
        let coherence = field.sentence_coherence(&tokens);
        let sync = field.sentence_sync(&tokens);
        let entropy = field.spectral_entropy(0.0);
        let dominant = field.dominant_colors(0.0, 5);

        Self {
            text: text.to_string(),
            coherence, sync, entropy,
            dominant_colors: dominant,
            word_count: tokens.len(),
        }
    }
}

impl fmt::Display for OscillatorEval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  ── oscillator evaluation ──")?;
        writeln!(f, "  text: \"{}\"", self.text)?;
        writeln!(f, "  words: {}", self.word_count)?;
        writeln!(f)?;
        writeln!(f, "  coherence (order parameter): {:.4}", self.coherence)?;
        writeln!(f, "  sync (avg pairwise):          {:.4}", self.sync)?;
        writeln!(f, "  spectral entropy:             {:.4}", self.entropy)?;
        writeln!(f)?;
        writeln!(f, "  dominant colors:")?;
        for (color, amp) in &self.dominant_colors {
            writeln!(f, "    {} (amplitude {:.2})", color, amp)?;
        }
        Ok(())
    }
}

/// ComparisonResult - compares the transform model vs oscillator model.
pub struct ComparisonResult {
    pub text: String,
    pub transform_coherence: f64,
    pub transform_novelty: f64,
    pub transform_resonance: f64,
    pub transform_overall: f64,
    pub osc_coherence: f64,
    pub osc_sync: f64,
    pub osc_entropy: f64,
    pub agreement: f64,
}

impl ComparisonResult {
    /// Compares the two models on the same text.
    pub fn compare(facet: &Facet, text: &str) -> Self {
        let tokens = Tokenizer::tokenize(text);
        let eval = super::Evaluator::new().eval(facet, text);
        let field = OscillatorField::from_facet(facet);
        let osc_coherence = field.sentence_coherence(&tokens);
        let osc_sync = field.sentence_sync(&tokens);
        let osc_entropy = field.spectral_entropy(0.0);
        let agreement = 1.0 - (eval.coherence - osc_coherence).abs();

        Self {
            text: text.to_string(),
            transform_coherence: eval.coherence,
            transform_novelty: eval.novelty,
            transform_resonance: eval.resonance,
            transform_overall: eval.overall,
            osc_coherence, osc_sync, osc_entropy, agreement,
        }
    }
}

impl fmt::Display for ComparisonResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  ── model comparison: transform vs oscillator ──")?;
        writeln!(f, "  text: \"{}\"", self.text)?;
        writeln!(f)?;
        writeln!(f, "  ┌─────────────────────┬──────────┬──────────┐")?;
        writeln!(f, "  │ metric              │ transform│oscillator│")?;
        writeln!(f, "  ├─────────────────────┼──────────┼──────────┤")?;
        writeln!(f, "  │ coherence           │  {:.4}  │  {:.4}  │", self.transform_coherence, self.osc_coherence)?;
        writeln!(f, "  │ novelty             │  {:.4}  │    -     │", self.transform_novelty)?;
        writeln!(f, "  │ resonance           │  {:.4}  │    -     │", self.transform_resonance)?;
        writeln!(f, "  │ overall             │  {:.4}  │    -     │", self.transform_overall)?;
        writeln!(f, "  │ sync (pairwise)     │    -     │  {:.4}  │", self.osc_sync)?;
        writeln!(f, "  │ spectral entropy    │    -     │  {:.4}  │", self.osc_entropy)?;
        writeln!(f, "  └─────────────────────┴──────────┴──────────┘")?;
        writeln!(f)?;
        writeln!(f, "  agreement: {:.1}%", self.agreement * 100.0)?;
        if self.agreement > 0.8 {
            writeln!(f, "  → models agree - high confidence assessment")?;
        } else if self.agreement > 0.5 {
            writeln!(f, "  → models partially agree - moderate confidence")?;
        } else {
            writeln!(f, "  → models disagree - text sits at a model boundary")?;
        }
        Ok(())
    }
}
