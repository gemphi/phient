#[allow(unused_imports)]
use super::*;

/// Strips dictionary apparatus from a definition before it reaches the trainer.
///
/// Webster's entries carry part-of-speech markers, etymology brackets, sense
/// numbers and citation attributions. Trained as if they were content, those
/// tokens acquire positions in the manifold and then have to be blocked at
/// generation time by a hardcoded `boilerplate` list — which suppresses the
/// symptom while leaving the words wrongly *placed*. Cleaning at ingestion
/// fixes the cause.
pub fn clean_definition(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut depth_sq = 0i32;
    let mut depth_par = 0i32;

    for ch in raw.chars() {
        match ch {
            '[' => depth_sq += 1,
            ']' => depth_sq = (depth_sq - 1).max(0),
            '(' => depth_par += 1,
            ')' => depth_par = (depth_par - 1).max(0),
            _ if depth_sq == 0 && depth_par == 0 => out.push(ch),
            _ => {}
        }
    }

    // Drop leading sense numbers ("1.", "2.") and standalone apparatus tokens.
    let cleaned: Vec<String> = out
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| !w.is_empty())
        .filter(|w| !w.chars().all(|c| c.is_ascii_digit()))
        .filter(|w| !is_apparatus(w))
        .collect();

    cleaned.join(" ")
}

/// The definitional core of a dictionary entry.
///
/// [`clean_definition`] removes *apparatus* — brackets, parentheses, part-of-
/// speech markers. This removes *illustration*, which is a much larger share of
/// a Webster's entry and the reason two imported mechanisms failed:
///
/// ```text
/// cat: 1. (Zool.) An animal of various species of the genera Felis and Lynx.
///      The domestic cat is Felis domestica. The European wild cat ...
///      Note: The domestic cat includes many varieties ... as, the Angora cat;
///      the Maltese cat ...  2. (Naut.) A strong vessel with a narrow stern ...
/// ```
///
/// Only the first sentence of sense 1 defines *cat*. Everything after it is
/// elaboration, editorial note, usage example and cross-reference — words that
/// are *about* the entry rather than part of it. Kept, they inflate every
/// definer set, which is why the definition graph came out at 47.5∶1
/// weak∶strong against dict2vec's ~9∶1, and the grounding kernel at 49.6% of
/// entries against the literature's ~10%. Both numbers are measurable in
/// seconds (`cargo run --bin defstats`), and both are acceptance criteria.
///
/// The rule is structural, not a word list: take the first sentence of each of
/// the first [`MAX_SENSES`] senses, cutting each at the first illustrative
/// marker. A hardcoded list of poets — which is what the apparatus list had
/// grown — suppresses the symptom for the poets someone remembered.
pub fn definition_core(raw: &str) -> String {
    let mut kept: Vec<String> = Vec::new();

    for sense in split_senses(raw).into_iter().take(MAX_SENSES) {
        let body = strip_sense_head(&sense);
        let trimmed = cut_at_illustration(body);
        // The gloss is the first sentence. A quotation, when one follows, is a
        // *later* sentence — so taking the first sentence excludes attributions
        // without having to recognise the author.
        let gloss = first_sentence(trimmed);
        let cleaned = clean_definition(gloss);
        // Two content words is the floor for a usable gloss; below that the
        // sense was apparatus all the way down.
        if cleaned.split_whitespace().count() >= 2 {
            kept.push(cleaned);
        }
    }

    // A word whose every sense was too short still has to reach the manifold
    // somehow, so fall back to the old behaviour rather than dropping it.
    match kept.is_empty() {
        true => clean_definition(raw),
        false => kept.join(" "),
    }
}

/// The gloss of each numbered sense, separately.
///
/// [`definition_core`] concatenates the first [`MAX_SENSES`] senses into one
/// string, which is how every measurement in this project came to be taken on a
/// representation that cannot hold polysemy: *cat* has eight numbered senses in
/// Webster's — the animal, a strong sailing vessel, a double tripod, the
/// constellation — and they were merged into a single gloss, composed to a
/// single centroid, and stored at a single point.
///
/// A word is not one thing. This returns the senses so each can have its own
/// phasor, which is the bounded per-word type inventory the architecture was
/// always described as having and never had.
///
/// Capped at [`MAX_WORD_SENSES`]: the claim is that a word's uses are *bounded*,
/// not that they are unlimited, and a cap is what makes the sense index a type
/// rather than an ever-growing list.
pub fn definition_senses(raw: &str) -> Vec<String> {
    split_senses(raw)
        .into_iter()
        .take(MAX_WORD_SENSES)
        .filter_map(|sense| {
            let body = strip_sense_head(&sense);
            let gloss = first_sentence(cut_at_illustration(body));
            let cleaned = clean_definition(gloss);
            match cleaned.split_whitespace().count() >= 2 {
                true => Some(cleaned),
                false => None,
            }
        })
        .collect()
}

/// Splits an entry on its sense numbers (`1.`, `2.`, ...).
///
/// Only a digit run followed by a period and a space starts a sense, so decimal
/// numbers and abbreviations inside a gloss do not split it.
pub(super) fn split_senses(raw: &str) -> Vec<String> {
    let bytes: Vec<char> = raw.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut i = 0usize;

    while i < bytes.len() {
        let starts_sense = bytes[i].is_ascii_digit()
            && (i == 0 || bytes[i - 1] == ' ' || bytes[i - 1] == '\n')
            && {
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                j < bytes.len() && bytes[j] == '.' && bytes.get(j + 1) == Some(&' ')
            };

        if starts_sense && !cur.trim().is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(bytes[i]);
        i += 1;
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    match out.is_empty() {
        true => vec![raw.to_string()],
        false => out,
    }
}
