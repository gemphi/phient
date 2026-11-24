//! Learned heuristics: uses facet phases to guide program synthesis search.
//! Deep learning guides the discrete program search (Ch 14.5).

use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;

#[derive(Debug, Default)]
pub struct SynthesisHeuristic;

impl SynthesisHeuristic {
    /// Generates phase-based heuristics from examples to guide search.
    pub fn phase(facet: &Facet, examples: &[(String, String)]) -> Vec<f64> {
        let mut heuristics = Vec::new();

        for (input, output) in examples {
            let input_tokens = Tokenizer::tokenize(input);
            let output_tokens = Tokenizer::tokenize(output);

            let input_phase = Self::mean_phase(facet, &input_tokens);
            let output_phase = Self::mean_phase(facet, &output_tokens);

            let delta = phical::circle::distance(output_phase, input_phase);

            heuristics.push(delta);
        }

        heuristics
    }

    /// Suggests likely program structures based on phase patterns.
    pub fn suggest_structure(_facet: &Facet, examples: &[(String, String)]) -> Vec<&'static str> {
        let mut suggestions = Vec::new();

        for (input, output) in examples {
            let input_tokens = Tokenizer::tokenize(input);
            let output_tokens = Tokenizer::tokenize(output);

            suggestions.push(match input_tokens.len().cmp(&output_tokens.len()) {
                std::cmp::Ordering::Equal => {
                    let reversed: Vec<String> = input_tokens.iter().rev().cloned().collect();
                    let sorted: Vec<String> = {
                        let mut t = input_tokens.clone();
                        t.sort();
                        t
                    };
                    match (reversed == output_tokens, sorted == output_tokens) {
                        (true, _) => "reverse",
                        (_, true) => "sort",
                        _ => "map",
                    }
                }
                // Fewer words out than in, or more.
                std::cmp::Ordering::Greater => "filter",
                std::cmp::Ordering::Less => "compose",
            });
        }

        suggestions
    }

    fn mean_phase(facet: &Facet, tokens: &[String]) -> f64 {
        let mut r = phical::circle::Resultant::new();
        let mut count = 0;
        for token in tokens {
            if let Some(p) = facet.lexicon.get(token) {
                r.add(p.amplitude, p.phase);
                count += 1;
            }
        }
        match count {
            0 => 0.0,
            _ => r.direction_lifted(),
        }
    }
}
