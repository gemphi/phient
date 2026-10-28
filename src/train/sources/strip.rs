#[allow(unused_imports)]
use super::*;

/// Removes a sense's leading number and subject label.
///
/// `"1. (Zool.)  An animal..."` becomes `"An animal..."`. Without this the
/// sentence splitter stops at the period after the sense number and every gloss
/// comes back as the digit.
pub(super) fn strip_sense_head(sense: &str) -> &str {
    let s = sense.trim_start();
    // Leading sense number: digits then '.'
    let s = match s.find(|c: char| !c.is_ascii_digit()) {
        Some(n) if n > 0 && s[n..].starts_with('.') => s[n + 1..].trim_start(),
        _ => s,
    };
    // Leading lettered subsense: "(a)" / "(b)".
    let s = match s.starts_with('(') && s.len() > 3 && s.as_bytes()[2] == b')' {
        true => s[3..].trim_start(),
        false => s,
    };
    // Leading subject label: "(Zool.)", "(Naut.)", "(Astron.)".
    match s.starts_with('(') {
        true => match s.find(')') {
            Some(n) => s[n + 1..].trim_start(),
            None => s,
        },
        false => s,
    }
}

/// Truncates a sense at the first marker that introduces illustration rather
/// than definition.
///
/// These are Webster's own structural signals, not guesses: `Note:` opens an
/// editorial digression, `as,` opens a list of usage examples, `See` and `Cf.`
/// open cross-references, `Syn.` opens a synonym list.
pub(super) fn cut_at_illustration(sense: &str) -> &str {
    const MARKERS: [&str; 8] = ["Note:", "NOTE:", "; as,", ", as,", " See ", " Cf. ", "Syn.", "Usage:"];
    let mut end = sense.len();
    for m in MARKERS {
        if let Some(pos) = sense.find(m) {
            end = end.min(pos);
        }
    }
    &sense[..end]
}

/// The first sentence of a fragment.
///
/// Abbreviations ending in a period (`Felis domestica.`) are rare enough inside
/// a first gloss that a period followed by whitespace is a reliable boundary; a
/// single-letter token before the period is treated as an initial and skipped.
pub(super) fn first_sentence(text: &str) -> &str {
    let chars: Vec<char> = text.chars().collect();
    let mut idx = 0usize;
    for (i, c) in chars.iter().enumerate() {
        if (*c == '.' || *c == ';') && chars.get(i + 1).is_none_or(|n| n.is_whitespace()) {
            // "J. Milton" — a lone capital before the period is an initial.
            let initial = i >= 2
                && chars[i - 1].is_ascii_uppercase()
                && !chars[i - 2].is_alphanumeric();
            if !initial {
                idx = i;
                break;
            }
        }
    }
    match idx {
        0 => text,
        n => {
            let byte_end: usize = chars[..n].iter().map(|c| c.len_utf8()).sum();
            &text[..byte_end]
        }
    }
}

/// Dictionary metadata tokens — grammatical labels, editorial abbreviations and
/// citation attributions that are about the entry rather than part of it.
pub(super) fn is_apparatus(word: &str) -> bool {
    matches!(
        word,
        "n" | "v" | "adj" | "adv" | "prep" | "conj" | "interj" | "pron"
            | "noun" | "verb" | "adjective" | "adverb" | "participle"
            | "plural" | "singular" | "pl" | "sing" | "imp" | "pp"
            | "obs" | "obsolete" | "archaic" | "rare" | "colloq" | "dial"
            | "cf" | "viz" | "ie" | "eg" | "etym" | "etymology"
            | "syn" | "opp" | "abbr" | "var" | "cap" | "usu"
            | "webster" | "unabridged" | "shak" | "milton" | "dryden"
            | "spenser" | "tennyson" | "chaucer" | "pope" | "bacon"
            | "see" | "sometimes" | "formerly"
    )
}
