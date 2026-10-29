use super::*;
use crate::lineup::Count;
use crate::study::Reader;

fn line(s: &str) -> Vec<String> {
    s.split(' ').map(String::from).collect()
}

fn small() -> Config {
    Config { channels: 16, context: 3, negatives: 8, rate: 0.1, epochs: 30, ..Config::default() }
}

#[test]
fn it_learns_what_counting_the_word_before_cannot() {
    // After `x` comes `c` or `d`, decided by the word BEFORE `x`. The
    // count of what follows `x` is a tie it breaks by spelling, so it
    // gets one of the two wrong; the learner reads both words back.
    for config in [small(), Config { block: 4, tie: true, silence: 0.2, ..small() }] {
        let mut l = Learner::new(config);
        let mut count = Count::new();
        for _ in 0..40 {
            for s in ["a x c", "b x d"] {
                l.tell(&line(s));
                count.tell(&line(s));
            }
        }
        l.ready();
        assert_eq!(l.guess(&line("a x")).as_deref(), Some("c"), "{:?}", l.config);
        assert_eq!(l.guess(&line("b x")).as_deref(), Some("d"), "{:?}", l.config);
        let wrong = [("a x", "c"), ("b x", "d")].iter().filter(|(p, w)| count.guess(&line(p)).as_deref() != Some(*w)).count();
        assert_eq!(wrong, 1, "the count of the word before is right only half the time here");
    }
}

#[test]
fn the_error_falls_as_it_learns() {
    let mut l = Learner::new(small());
    for s in ["the cat sat on the mat", "the dog sat on the rug", "a cat ran to the door", "a dog ran to the gate"] {
        l.tell(&line(s));
    }
    let first = l.epoch();
    let mut last = first;
    for _ in 0..40 {
        last = l.epoch();
    }
    assert!(last < first * 0.6, "the error fell from {first:.3} to {last:.3}");
    let p: f64 = l.ranked(&line("the cat"), usize::MAX).iter().map(|(_, p)| p).sum();
    assert!((p - 1.0).abs() < 1e-4, "a distribution: {p}");
}

#[test]
fn a_word_is_its_angles() {
    for tie in [false, true] {
        let mut l = Learner::new(Config { tie, ..small() });
        for _ in 0..5 {
            l.tell(&line("the cat sat on the mat"));
        }
        l.ready();
        for k in 0..16 {
            let (_, m) = l.reached("cat", k).expect("told");
            assert!((m - 1.0).abs() < 1e-4, "channel {k} left the circle: {m}");
        }
    }
}

#[test]
fn a_block_of_one_is_the_turn_and_a_wide_block_is_born_the_same() {
    // Born, the operator is the diagonal turn whatever the block: the
    // first readings of two learners differing only in block agree.
    let mut a = Learner::new(Config { block: 1, ..small() });
    let mut b = Learner::new(Config { block: 8, ..small() });
    for l in [&mut a, &mut b] {
        l.intern("x");
        l.intern("y");
    }
    let (sa, _) = a.scores(&line("x"));
    let (sb, _) = b.scores(&line("x"));
    for (p, q) in sa.iter().zip(&sb) {
        assert!((p - q).abs() < 1e-5, "{p} vs {q}");
    }
}

#[test]
fn an_unknown_word_is_read_not_refused() {
    let mut l = Learner::new(small());
    for _ in 0..5 {
        l.tell(&line("i have been waiting for the bus"));
    }
    l.ready();
    assert!(l.guess(&line("i have never zygomorphic")).is_some());
    let r = l.measure(&[line("i have been waiting for the zygote")]);
    assert_eq!(r.asked, 6);
    assert_eq!(r.known, 5, "the unknown word carries no probability");
}

#[test]
fn the_same_seed_learns_the_same_thing() {
    let run = || {
        let mut l = Learner::new(small());
        for s in ["the cat sat on the mat", "a dog ran to the gate"] {
            l.tell(&line(s));
        }
        l.ready();
        l.ranked(&line("the"), 3)
    };
    assert_eq!(run(), run());
}

/// The English guide's sentences, cut as `talk` and the learner example cut
/// them; `None` when the guide is not beside the workspace.
fn guide(n: usize) -> Option<Vec<Vec<String>>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../phiano/data/person/murphy.txt");
    let text = std::fs::read_to_string(path).ok()?;
    let cut = |w: Vec<String>| w.into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>();
    Some(text.split(['.', '!', '?', '\n']).map(crate::prose::tokens).map(cut).filter(|w| w.len() >= 4).take(n).collect())
}

/// The learner's numbers on the guide, pinned before its math moved to phical
/// (batch 4 of docs/phient/calculations): told the guide's first 600
/// sentences and trained two epochs, the bits of each epoch's mean loss and a
/// fingerprint of every score it gives on the next 40 sentences' prefixes must
/// not change. (`measure` reduces in parallel, so its sums are not pinned.)
#[test]
fn the_numbers_on_the_guide_do_not_move() {
    let Some(lines) = guide(640) else { return };
    let mut l = Learner::new(Config { channels: 16, context: 4, block: 4, negatives: 16, rate: 0.5, epochs: 2, dropout: 0.1, silence: 0.2, ..Config::default() });
    for s in &lines[..600] {
        l.tell(s);
    }
    let losses: Vec<u64> = (0..2).map(|_| l.epoch().to_bits()).collect();
    let mut print = 0u64;
    for s in &lines[600..] {
        for j in 1..s.len() {
            let (scores, end) = l.scores(&s[..j]);
            for x in scores.iter().chain([&end]) {
                print = phistd::mix64(print ^ x.to_bits() as u64);
            }
        }
    }
    // Pinned 2026-10-29 from the code before batch 4.
    assert_eq!((losses, print), (vec![4612860791556017263u64, 4611911870419914518u64], 0x12fcdbd401d3768cu64));
}
