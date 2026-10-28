#[allow(unused_imports)]
use super::*;

impl DefinitionGrounder {

    /// Amplitude-weighted circular centroid of a definition's *content* words.
    ///
    /// Function words are excluded. They appear in nearly every definition, so
    /// at full weight they pull every word in the dictionary toward one shared
    /// point — grounding would then re-create the collapse it is meant to fix.
    pub(super) fn definition_centroid(facet: &Facet, definition: &str) -> Option<f64> {
        let tokens = Tokenizer::tokenize(definition);
        if tokens.is_empty() {
            return None;
        }

        let (mut sum_x, mut sum_y, mut count) = (0.0f64, 0.0f64, 0u32);
        for token in &tokens {
            if Tokenizer::is_function_word(token) {
                continue;
            }
            if let Some(phasor) = facet.lexicon.get(token) {
                let th = phasor.theta(0);
                sum_x += th.cos() * phasor.amplitude;
                sum_y += th.sin() * phasor.amplitude;
                count += 1;
            }
        }

        match count > 0 {
            true => Some(sum_y.atan2(sum_x).rem_euclid(TWO_PI)),
            false => None,
        }
    }
}
