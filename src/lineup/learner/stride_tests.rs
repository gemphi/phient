//! The stride that forgets by itself (`Config::stride`; phiano docs/231,
//! 232): the decay, a learner with it against one without, and the save.

use super::tests::{line, small};
use crate::study::Reader;
use super::*;

#[test]
fn an_accumulator_decays_by_the_lines_told_since_its_row_was_touched() {
    let decay = Config { stride: 10, ..small() }.decay();
    let mut last = 0;
    let acc = plane::decayed(1.0, &mut last, 10, decay);
    assert!((acc - 0.1).abs() < 1e-5, "a tenth after one stride: {acc}");
    assert_eq!(last, 10, "the row is touched now");
    let acc = plane::decayed(acc, &mut last, 30, decay);
    assert!((acc - 0.001).abs() < 1e-6, "a thousandth after two more: {acc}");
    assert_eq!(plane::decayed(1.0, &mut last, 40, Config::default().decay()), 1.0, "no stride, no decay");
    assert_eq!(plane::decayed(1.0, &mut last, 40, decay), 1.0, "touched at the same line, no decay");
}

#[test]
fn a_learner_with_a_stride_steps_further_than_one_without() {
    // The same lines, told to a learner that keeps its whole history and
    // to one whose accumulators fall to a tenth every line: the second
    // steps further on the last line told, as a forgotten stride does.
    let lines: Vec<Vec<String>> = ["the cat sat on the mat", "the dog sat on the log", "a cat sat on a log", "the cat sat on the log"].iter().map(|s| line(s)).collect();
    let mut sure = Learner::new(small());
    let mut kept = Learner::new(Config { stride: 1, ..small() });
    for s in &lines[..3] {
        sure.tell(s);
        kept.tell(s);
    }
    let before = |l: &Learner| l.scores(&line("the cat sat on the")).0;
    let (bs, bk) = (before(&sure), before(&kept));
    sure.tell(&lines[3]);
    kept.tell(&lines[3]);
    let moved = |l: &Learner, b: &[f32]| l.scores(&line("the cat sat on the")).0.iter().zip(b).map(|(x, y)| (x - y).abs()).sum::<f32>();
    assert!(moved(&kept, &bk) > moved(&sure, &bs), "a kept stride steps further: {} vs {}", moved(&kept, &bk), moved(&sure, &bs));
    let mut saved = std::env::temp_dir();
    saved.push("phient_stride_test.ckpt");
    kept.save(&saved).expect("saved");
    let back = Learner::load(&saved).expect("loaded");
    assert_eq!((back.config.stride, back.scores(&line("the cat sat on the")).0), (1, kept.scores(&line("the cat sat on the")).0), "the stride and the clocks come back");
}
