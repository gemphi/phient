//! Which tokens may be spoken, and how the output is formatted.

use super::generator::Generator;

impl Generator {
    /// Whether a token may be emitted.
    ///
    /// Numerals are allowed: the previous rule required every character to be
    /// alphabetic, so the model could not state a quantity, a version or a date.
    pub(super) fn speakable(word: &str) -> bool {
        let n = word.chars().count();
        if n == 0 || n > 16 || Self::boilerplate(word) {
            return false;
        }
        let all_digits = word.chars().all(|c| c.is_ascii_digit());
        if all_digits {
            return n <= 6;
        }
        n >= 2 && word.chars().all(|c| c.is_ascii_alphanumeric())
    }

    pub(super) fn boilerplate(word: &str) -> bool {
        matches!(
            word,
            "pertaining" | "genus" | "species" | "extant" | "parvorder"
                | "tokenizer" | "vocab" | "obsolete" | "archaic" | "namely"
                | "viz" | "ie" | "unabridged" | "webster" | "etymology"
                | "plural" | "singular" | "participle" | "adjective" | "adverb"
                | "noun" | "verb" | "hence" | "thereof" | "therein" | "whereby"
                | "whereof" | "aforesaid" | "called" | "written" | "also"
                | "syn" | "obs" | "shak" | "milton" | "cf" | "opp"
                | "idiom" | "dialect" | "phraseology" | "diction"
                | "edifieth" | "puffeth" | "bloodguiltiness"
        )
    }


    pub(super) fn format_output(tokens: &[String]) -> String {
        let mut formatted = String::new();
        for (i, t) in tokens.iter().enumerate() {
            match i {
                0 => {
                    let mut chars = t.chars();
                    if let Some(first) = chars.next() {
                        formatted.push_str(&first.to_uppercase().collect::<String>());
                        formatted.push_str(chars.as_str());
                    }
                }
                _ => {
                    formatted.push(' ');
                    formatted.push_str(t);
                }
            }
        }
        let needs_punct = !formatted.is_empty()
            && !formatted.ends_with('.')
            && !formatted.ends_with('!')
            && !formatted.ends_with('?');
        if needs_punct {
            formatted.push('.');
        }
        formatted
    }
}
