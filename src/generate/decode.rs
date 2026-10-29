//! Greedy decoding with the phase walk, and the public entry points.

use super::context::ContextWaveBuffer;
use super::generator::{Generator, FUNCTION_STREAK_LIMIT};
use crate::generate::constants::{PHI_CONJUGATE, SYNTACTIC_MOMENTUM_DEFAULT, TWO_PI};
use phiecs::phasor::facet::Facet;
use phiecs::phasor::flow::PhaseFlow;
use phiton::p64::words::Tokenizer;

impl Generator {
    pub fn generate(
        &self,
        facet: &Facet,
        context_buffer: &mut ContextWaveBuffer,
        prompt: &str,
    ) -> String {
        self.generate_with_flow(facet, context_buffer, prompt).0
    }

    /// Generates text and returns the live PhaseFlow trajectory alongside it.
    pub fn generate_with_flow(
        &self,
        facet: &Facet,
        context_buffer: &mut ContextWaveBuffer,
        prompt: &str,
    ) -> (String, PhaseFlow) {
        let (tokens, flow) = self.decode(facet, context_buffer, prompt);
        let formatted = Self::format_output(&tokens);
        context_buffer.push_turn(facet, &formatted);
        (formatted, flow)
    }

    /// Greedy decode: emits up to `max_tokens` tokens after `prompt` and
    /// returns them with the phase flow that steered the choice.
    pub fn decode(
        &self,
        facet: &Facet,
        context_buffer: &mut ContextWaveBuffer,
        prompt: &str,
    ) -> (Vec<String>, PhaseFlow) {
        context_buffer.push_turn(facet, prompt);

        // Under a collapsed manifold every candidate aligns with every target
        // phase, so the phase term stops discriminating and decoding degrades
        // smoothly into a plain n-gram sampler — with no error and no warning.
        // One O(V) check per generation buys the warning.
        let dispersion = facet.phase_dispersion();
        if dispersion < crate::generate::constants::DEGENERACY_WARN {
            eprintln!(
                "  [WARN] phase dispersion {:.4} — the manifold is not discriminating; \
                 generation is effectively n-gram only (docs/how/11_generation.md)",
                dispersion
            );
        }

        let mut generated_tokens: Vec<String> = Vec::new();
        let mut current_phase = context_buffer.context_phase();
        let mut phase_momentum: f64 = SYNTACTIC_MOMENTUM_DEFAULT;
        let mut flow = PhaseFlow::build(facet, prompt);
        flow.propagate(2);

        let prompt_tokens = Tokenizer::tokenize(prompt);
        self.operator_reset(&prompt_tokens);
        let mut last_word: Option<String> = prompt_tokens.last().cloned();
        let mut prev_word: Option<String> = match prompt_tokens.len() >= 2 {
            true => Some(prompt_tokens[prompt_tokens.len() - 2].clone()),
            false => None,
        };
        let mut recent_words: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut function_streak = 0usize;

        for step in 0..self.max_tokens {
            // Exploration now comes from sampling the candidate distribution
            // (see `sample_index`); this term only keeps the phase walk from
            // sitting exactly on a sector boundary.
            let jitter = (step as f64 * PHI_CONJUGATE).sin() * 0.01;
            let flow_bias = 0.45 * (flow.collective_phase - current_phase).sin();
            let target_phase = (current_phase + phase_momentum + jitter + flow_bias).rem_euclid(TWO_PI);

            // A run of function words used to end the composition outright
            // (a hard stop at four, which is why long compositions came back
            // three words long: "with me and" hits it). Now the run demands a
            // content word instead — the same wheel, a different gear — and
            // only when no content word can follow does the run end the text.
            let mut next_word = self.attractor_select(
                facet, &flow, &prev_word, &last_word, &recent_words, target_phase,
                function_streak >= FUNCTION_STREAK_LIMIT,
            );
            if next_word.is_none() && function_streak >= FUNCTION_STREAK_LIMIT {
                next_word = self.attractor_select(
                    facet, &flow, &prev_word, &last_word, &recent_words, target_phase, false,
                );
            }

            match next_word {
                Some(word) => {
                    if Tokenizer::is_function_word(&word) {
                        function_streak += 1;
                    } else {
                        function_streak = 0;
                    }
                    recent_words.insert(word.clone());
                    self.evict_old(&mut recent_words, &generated_tokens, 12);
                    self.apply_phase_kick(
                        facet, &last_word, &word, &mut current_phase, &mut phase_momentum,
                    );
                    let resonance = flow.resonance_with(facet, &word);
                    flow.record_step(step, Some(&word), resonance, flow.novelty());
                    flow.update_momentum(phase_momentum);
                    generated_tokens.push(word.clone());
                    self.operator_note(&word);
                    prev_word = last_word.clone();
                    last_word = Some(word);
                    if generated_tokens.len() >= self.max_tokens {
                        break;
                    }
                }
                None => break,
            }
        }

        (generated_tokens, flow)
    }

}
