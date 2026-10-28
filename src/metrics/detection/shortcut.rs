//! Shortcut detection: identifies when the model exploits surface features
//! instead of semantic phase relationships (Ch 14.3's shortcut rule).

use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutWarning {
    pub shortcut_type: String,
    pub description: String,
    pub severity: f64,
}

#[derive(Debug, Default)]
pub struct ShortcutDetector;

impl ShortcutDetector {
    /// Detects if the model is exploiting surface features.
    pub fn detect(facet: &Facet, prompt: &str, response: &str) -> Vec<ShortcutWarning> {
        let mut warnings = Vec::new();

        let prompt_tokens = Tokenizer::tokenize(prompt);
        let response_tokens = Tokenizer::tokenize(response);

        let avg_prompt_len: f64 = prompt_tokens.iter().map(|t| t.len() as f64).sum::<f64>()
            / prompt_tokens.len().max(1) as f64;
        let avg_response_len: f64 = response_tokens.iter().map(|t| t.len() as f64).sum::<f64>()
            / response_tokens.len().max(1) as f64;

        if (avg_response_len - avg_prompt_len).abs() < 0.5 && !response_tokens.is_empty() {
            warnings.push(ShortcutWarning {
                shortcut_type: "length_matching".to_string(),
                description: "Response words match prompt in length — may be exploiting surface form".to_string(),
                severity: 0.4,
            });
        }

        let prompt_freq: f64 = prompt_tokens.iter()
            .filter_map(|t| facet.lexicon.get(t))
            .map(|p| p.amplitude)
            .sum::<f64>() / prompt_tokens.len().max(1) as f64;
        let response_freq: f64 = response_tokens.iter()
            .filter_map(|t| facet.lexicon.get(t))
            .map(|p| p.amplitude)
            .sum::<f64>() / response_tokens.len().max(1) as f64;

        if (response_freq - prompt_freq).abs() < 0.1 && !response_tokens.is_empty() {
            warnings.push(ShortcutWarning {
                shortcut_type: "frequency_matching".to_string(),
                description: "Response words match prompt in frequency — may be exploiting familiarity".to_string(),
                severity: 0.3,
            });
        }

        let overlap: f64 = response_tokens.iter()
            .filter(|t| prompt_tokens.contains(t))
            .count() as f64 / response_tokens.len().max(1) as f64;

        if overlap > 0.7 {
            warnings.push(ShortcutWarning {
                shortcut_type: "high_overlap".to_string(),
                description: format!("Response has {:.0}% overlap with prompt — may be copying", overlap * 100.0),
                severity: overlap,
            });
        }

        warnings
    }
}
