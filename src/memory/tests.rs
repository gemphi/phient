use super::*;

#[test]
fn test_empty_memo() {
    let m = Memo::new();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
}

#[test]
fn test_record_short_text() {
    let mut m = Memo::new();
    m.record((1.0, 0.5), "hi");
    assert!(!m.is_empty());
    assert_eq!(m.len(), 1);
    let layer = Memo::classify_layer("hi");
    assert!(layer < 4);
    assert_eq!(m.layer_count(layer), 1);
}

#[test]
fn test_record_long_text() {
    let mut m = Memo::new();
    let long = "this is a longer sentence with many words to classify it into a deeper band";
    m.record((0.0, 1.0), long);
    assert_eq!(m.len(), 1);
    let layer = Memo::classify_layer(long);
    assert!(layer >= 8);
}

#[test]
fn test_fnv1a_consistency() {
    let h1 = Memo::fnv1a_hash("test");
    let h2 = Memo::fnv1a_hash("test");
    assert_eq!(h1, h2);
}

#[test]
fn test_fnv1a_different_inputs() {
    let h1 = Memo::fnv1a_hash("hello");
    let h2 = Memo::fnv1a_hash("world");
    assert_ne!(h1, h2);
}
