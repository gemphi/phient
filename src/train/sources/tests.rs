#[allow(unused_imports)]
use super::*;

#[test]
fn test_brackets_and_apparatus_are_removed() {
    let raw = "1. (Zool.) A small [OE. catte] furry animal; n. -- Shak.";
    let c = clean_definition(raw);
    assert!(!c.contains("zool"), "parenthetical apparatus removed: {}", c);
    assert!(!c.contains("catte"), "etymology bracket removed: {}", c);
    assert!(!c.contains("shak"), "citation removed: {}", c);
    assert!(!c.split_whitespace().any(|w| w == "n"), "pos marker removed: {}", c);
    assert!(c.contains("furry") && c.contains("animal"), "content survives: {}", c);
}

/// A1's golden file: the definitional core of a real Webster's entry must
/// be the gloss, and must not contain the note, the usage examples, the
/// cross-references or the quoted lines.
#[test]
fn test_definition_core_keeps_the_gloss_and_drops_illustration() {
    let cat = "1. (Zoöl.)  An animal of various species of the genera Felis \
               and Lynx. The domestic cat is Felis domestica. The European wild \
               cat (Felis catus) is much larger than the domestic cat. In the \
               United States the name wild cat is commonly applied to the bay \
               lynx (Lynx rufus) See Wild cat, and Tiger cat. Note: The domestic \
               cat includes many varieties named from their place of origin or \
               from some peculiarity; as, the Angora cat; the Maltese cat. \
               2. (Naut.) A strong vessel with a narrow stern.";
    let core = definition_core(cat);

    assert!(core.contains("animal") && core.contains("species"), "gloss kept: {}", core);
    assert!(!core.contains("angora"), "usage examples dropped: {}", core);
    assert!(!core.contains("maltese"), "usage examples dropped: {}", core);
    assert!(!core.contains("varieties"), "editorial note dropped: {}", core);
    assert!(!core.contains("domestica"), "elaboration sentences dropped: {}", core);
    assert!(core.contains("vessel"), "sense 2 gloss kept: {}", core);
}

/// A quotation and its attribution are later sentences, so taking the first
/// sentence excludes them without needing to know who the poet was. The
/// apparatus list had grown a hardcoded roll of poets doing this job badly.
#[test]
fn test_quotations_and_attributions_are_dropped_structurally() {
    let car = "3. A chariot of war or of triumph; a vehicle of splendor. \
               The gilded car of day. Milton. The towering car, the sable \
               steeds. Tennyson.";
    let core = definition_core(car);
    assert!(core.contains("chariot"), "gloss kept: {}", core);
    assert!(!core.contains("gilded"), "quotation dropped: {}", core);
    assert!(!core.contains("sable"), "quotation dropped: {}", core);

    // Structural, not a name list: an invented poet must be dropped too.
    let made_up = "1. A small furry animal. The cat sat upon the mat. Quillfeather.";
    let c2 = definition_core(made_up);
    assert!(c2.contains("furry"), "gloss kept: {}", c2);
    assert!(!c2.contains("quillfeather"), "unknown attribution dropped: {}", c2);
}

/// Only the first senses are composed, or a word ends up at the centre of
/// mass of meanings nobody uses.
#[test]
fn test_sense_count_is_capped() {
    let words = [
        "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel",
    ];
    let many: String = words
        .iter()
        .enumerate()
        .map(|(i, w)| format!("{}. A sense meaning {} entirely. ", i + 1, w))
        .collect();
    let core = definition_core(&many);

    for kept in &words[..MAX_SENSES] {
        assert!(core.contains(kept), "sense {} must be kept: {}", kept, core);
    }
    for dropped in &words[MAX_SENSES..] {
        assert!(!core.contains(dropped), "sense {} must be dropped: {}", dropped, core);
    }
}

#[test]
fn test_plain_text_is_untouched_apart_from_case() {
    assert_eq!(clean_definition("An adult female person"), "an adult female person");
}
