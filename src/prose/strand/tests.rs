use super::*;

fn story() -> String {
    let a = "the robot learns by observation . the robot watches the world and learns a model of the world . ";
    let b = "the engineer states a constraint . the engineer says never the wall and the constraint holds . ";
    format!("{}{}{}{}", a, a, b, b)
}

fn prior() -> Prior {
    let mut p = Prior::default();
    // An English-like prior in miniature: `the` as common as the story uses it.
    for _ in 0..20 {
        p.add("the and a of the by the says the never . the and a of the the and by of a the .");
    }
    p
}

#[test]
fn a_story_is_sixty_four_levels_from_the_whole_to_the_word() {
    let s = Strand::new(&story(), prior());
    let n = s.tokens.len();
    assert_eq!(s.units[63], 1);
    assert_eq!(s.units[0], n);
    assert!(s.units.windows(2).all(|w| w[0] >= w[1]), "coarser as the codon rises");
    assert_eq!(s.level(63).len(), 1);
    assert_eq!(s.level(0).len(), n);
    // The first division falls where the robot gives way to the engineer.
    let two = s.cut.units_at(2);
    assert!(!s.text(two[0]).contains("engineer") && !s.text(two[1]).contains("robot"), "{} | {}", s.text(two[0]), s.text(two[1]));
    // A cut is cheaper to say after a mark the story wrote: the first one follows the full stop.
    assert!(s.text(two[0]).ends_with("world .") && s.text(two[1]).starts_with("the engineer"));
    assert!(s.cut.steps[0].gain > 0.0);
    // Each half says its own words beyond the whole; the whole says the story's beyond English.
    assert!(s.signature(two[0], 3).contains(&"robot".to_string()));
    assert!(s.signature(two[1], 3).contains(&"engineer".to_string()));
    let whole = s.signature(s.level(63)[0], 6);
    assert!(!whole.contains(&"the".to_string()), "a word the prior expects says nothing: {whole:?}");
}

#[test]
fn the_strand_is_a_tree_a_cast_can_be_drawn_into() {
    let s = Strand::new(&story(), prior());
    let rows = s.rows("talk");
    let mut counts = crate::relate::Tally::new(&["has-word", "within"]);
    let mut tree = crate::relate::ScopeTree::default();
    counts.learn(&rows);
    tree.add_rows(&rows);
    assert_eq!(tree.roots(), ["scope:talk"]);
    let found = tree.cast(&counts, &["engineer".into(), "constraint".into()], f64::MIN, 2);
    let leaf = found[0].path.last().unwrap();
    let span = leaf.rsplit(':').next().unwrap();
    let start: usize = span.split('-').next().unwrap().parse().unwrap();
    assert!(start >= s.tokens.len() / 2, "drawn into the engineer's half: {leaf}");
}

#[test]
fn a_unit_is_read_with_the_code_it_was_divided_by() {
    let s = Strand::new(&story(), prior());
    let two = s.cut.units_at(2);
    // The reading is the span's own share for every word it said.
    for (&id, &share) in &s.cut.spans[two[1]].share {
        assert!((s.read(Some(two[1]), &s.vocab[id as usize]) - share).abs() < 1e-12);
    }
    // Read where it was said, a word is likelier than in the other half, and than in the whole.
    let (robot, engineer) = (s.read(Some(two[0]), "engineer"), s.read(Some(two[1]), "engineer"));
    assert!(engineer > s.read(Some(0), "engineer") && s.read(Some(0), "engineer") > robot);
    // A word the story never said keeps a share, smaller than the prior's.
    let never = s.read(Some(two[0]), "zebra");
    assert!(never > 0.0 && never < s.prior.p("zebra"));
    assert_eq!(s.unit_of(62, s.tokens.len() - 1), *two.last().unwrap());
}
