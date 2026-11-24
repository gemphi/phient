//! Dropping a word the reader no longer tells (`drop`; phiano docs/238):
//! what goes, what is kept bit for bit, and that the learner goes on.

use super::tests::{line, small};
use super::*;
use crate::study::Reader;

fn a_number(w: &str) -> bool {
    w.chars().any(|c| c.is_ascii_digit())
}

fn told() -> Learner {
    let mut l = Learner::new(small());
    for s in ["the cat sat on 103 the mat", "a dog ran to 41c the gate", "the cat ran to the door"] {
        l.tell(&line(s));
    }
    l
}

fn angles(l: &Learner, w: &str) -> (Vec<f32>, Vec<f32>, f32, u64) {
    let i = l.index[w];
    let (re, im) = l.reach.row(i);
    (re.to_vec(), im.to_vec(), l.bias[i as usize], l.seen[i as usize])
}

#[test]
fn a_dropped_word_leaves_the_vocabulary_the_lines_and_the_answers() {
    let mut l = told();
    let (before, kept) = (l.vocabulary(), angles(&l, "cat"));
    let dropped = l.drop_words(a_number);
    assert_eq!(dropped, 2, "the two numbers told");
    assert_eq!(l.vocabulary(), before - 2);
    assert_eq!(l.word.len(), l.bias.len(), "a name for every bias");
    assert!(!l.index.contains_key("103") && !l.index.contains_key("41c"), "{:?}", l.word);
    assert!(l.told.iter().flatten().all(|v| !a_number(&l.word[*v as usize])), "no number is still told");
    assert!(l.ranked(&line("the cat sat on"), l.vocabulary()).iter().all(|(w, _)| !a_number(w)), "no number is an answer");
    assert_eq!(angles(&l, "cat"), kept, "every other word's angles, bias and count are kept bit for bit");
    assert_eq!(l.drop_words(a_number), 0, "nothing left to drop");
    for m in ["<s>", "</s>", "<unk>"] {
        assert!(l.index.contains_key(m), "the markers stay: {m}");
    }
}

#[test]
fn a_learner_with_a_word_dropped_goes_on_and_saves() {
    let mut l = told();
    assert_eq!(l.drop_words(a_number), 2);
    let loss = l.epoch();
    assert!(loss.is_finite() && loss > 0.0, "it steps after the drop: {loss}");
    l.tell(&line("the cat sat on the mat"));
    let dir = std::env::temp_dir().join("phient_drop_words");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("dropped.ckpt");
    l.save(&path).unwrap();
    let back = Learner::load(&path).unwrap();
    assert_eq!(back.vocabulary(), l.vocabulary());
    assert!(!back.index.contains_key("103"));
    assert_eq!(angles(&back, "cat"), angles(&l, "cat"));
}
