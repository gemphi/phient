//! Text composition and bigram sequencing from river flow banks.
//!
//! Words collected along the river flow path across phase sectors are ordered
//! and woven into cohesive narrative prose. All operations are encapsulated
//! in [`Composer`], following the Diem convention that all public symbols belong
//! to named types.
//!
//! # Architecture
//!
//! ```text
//! River Flow Path & Sector Banks
//!   │
//!   ▼
//! Composer::reorder_with_bigrams() ──▶ Sequential Bigram Smoothing
//!   │
//!   ▼
//! Composer::compose()
//!   ├─▶ one generated segment per sector on the path, each pulled toward
//!   │   that sector's bank word (the generator's goal)
//!   └─▶ a closing segment pulled toward the most resonant word
//! ```

use crate::generate::constants as config;
use phiecs::phasor::facet::Facet;
use crate::generate::{ContextWaveBuffer, Decode, Generator};
use phiton::p64::words::Tokenizer;

/// Tokens generated per sector of the path.
pub const SEGMENT_TOKENS: usize = 8;
/// Context tokens carried from one segment to the next.
pub const SEGMENT_CONTEXT: usize = 6;
/// How hard each segment is pulled toward its sector's bank word.
pub const GOAL_PULL: f64 = 1.0;

/// Text composition engine for synthesizing prose from phase river flows.
pub struct Composer;

impl Composer {
    /// Composes text along the river flow: the prompt, then one generated
    /// segment per sector on the path, each pulled toward that sector's
    /// bank (`Generator::with_goal`) — the words move under the sector as
    /// the condition. Until this existed the segments were four fixed
    /// sentence frames with the bank words filled in.
    pub fn compose(facet: &Facet, prompt: &str, path: &[u16], banks: &[Vec<String>], resonant: &[String]) -> String {
        let mut text: Vec<String> = Vec::new();
        let mut context = Tokenizer::tokenize(prompt);
        let seed0 = phistd::mix64(path.first().copied().unwrap_or(0) as u64 ^ (prompt.len() as u64) << 8);
        let goals: Vec<Option<String>> = banks.iter().map(|b| b.iter().find(|w| w.len() > 2 && w.chars().all(|c| c.is_alphabetic())).cloned()).chain(std::iter::once(resonant.first().cloned())).collect();
        for (i, goal) in goals.iter().enumerate() {
            let g = Generator::deterministic(SEGMENT_TOKENS, 0.3, seed0.wrapping_add(i as u64 * 7919)).with_decode(Decode::Trigram);
            let g = match goal {
                Some(w) => g.with_goal(w, GOAL_PULL),
                None => g,
            };
            let start = context.len().saturating_sub(SEGMENT_CONTEXT);
            let seed_text = context[start..].join(" ");
            if seed_text.is_empty() {
                continue;
            }
            let mut buf = ContextWaveBuffer::new(config::DEFAULT_CONTEXT_WINDOW);
            let out = Tokenizer::tokenize(&g.generate(facet, &mut buf, &seed_text));
            if out.is_empty() {
                continue;
            }
            context.extend(out.iter().cloned());
            text.extend(out);
        }
        if text.is_empty() {
            return String::new();
        }
        let mut s = text.join(" ");
        if let Some(c) = s.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        s.push('.');
        s
    }

    /// Reorders words in a bank using bigram transition probabilities from the [`Facet`].
    pub fn reorder_with_bigrams(facet: &Facet, words: &[String]) -> Vec<String> {
        if words.len() <= 1 {
            return words.to_vec();
        }

        let mut remaining: Vec<String> = words.to_vec();
        let mut ordered: Vec<String> = Vec::new();

        ordered.push(remaining.remove(0));

        while !remaining.is_empty() {
            let current = ordered.last().unwrap();
            let mut best_idx = 0;
            let mut best_score = -1.0f64;

            for (i, candidate) in remaining.iter().enumerate() {
                let score = facet.bigram_probability(current, candidate);
                if score > best_score {
                    best_score = score;
                    best_idx = i;
                }
            }

            ordered.push(remaining.remove(best_idx));
        }

        ordered
    }
}
