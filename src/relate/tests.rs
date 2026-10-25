use super::*;
use phiecs::quest::parse_tsv;
use phistd::Adam;
use phical::{FnoConfig, Tie};

fn cfg() -> FnoConfig {
    FnoConfig { space: 64, width: 4, layers: 1, modes: 4, margin: 0.3, lr: 1e-2, tie: Tie::Circle }
}

#[test]
fn a_quest_table_trains_saves_and_reads_back_by_name() {
    let rows = parse_tsv("cat\tis-a\tanimal\t0.9\ndog\tis-a\tanimal\t0.9\ncat\thas\tfur\t0.8\ndog\thas\tfur\t0.8\n");
    let mut m = RelationModel::new(&rows, None, cfg(), 3);
    assert_eq!(m.kinds, vec!["is-a", "has"]);
    let told: Vec<_> = rows.iter().filter_map(|r| m.triple(r)).collect();
    let ex: Vec<_> = told.iter().map(phical::relate::example).collect();
    let (pool, _) = phical::relate::pools(&told);
    let (mut a, mut b) = (Adam::new(1e-2), Adam::new(5e-3));
    for e in 0..60 {
        m.joint.train_epoch(&ex, &pool, 4, &mut a, &mut b, e);
    }
    assert_eq!(m.answers("cat", "is-a", &pool, 1)[0].0, "animal");
    assert_eq!(m.answers("dog", "has", &pool, 1)[0].0, "fur");
    let path = std::env::temp_dir().join(format!("phiano-relate-{}.relate", std::process::id()));
    let path = path.to_str().unwrap();
    m.save(path).unwrap();
    let mut back = RelationModel::load(path).unwrap();
    let _ = std::fs::remove_file(path);
    assert_eq!(back.answers("cat", "is-a", &pool, 1)[0].0, "animal");
}

#[test]
fn a_seeded_word_starts_from_its_own_angles() {
    let rows = parse_tsv("cat\tis-a\tanimal\t0.9\n");
    let angles: Vec<f64> = (0..64).map(|k| k as f64 * 0.1).collect();
    let seeds = |w: &str| (w == "cat").then(|| angles.iter().map(|t| phistd::c64::from_polar(1.0, *t)).collect::<Vec<_>>());
    let m = RelationModel::new(&rows, Some(&seeds), cfg(), 3);
    let f = &m.joint.functions()[m.index["cat"]];
    let d = (f[5].im.atan2(f[5].re) - angles[5]).rem_euclid(std::f64::consts::TAU);
    assert!(d < 1e-9 || (std::f64::consts::TAU - d) < 1e-9);
}

#[test]
fn a_model_grows_without_forgetting_its_table() {
    let rows = parse_tsv("cat\tis-a\tanimal\t0.9\n");
    let mut m = RelationModel::new(&rows, None, cfg(), 3);
    let cat = m.joint.functions()[m.index["cat"]].clone();
    let more = parse_tsv("borrow\tis-a\treference\t0.9\nborrow\trequires\towner\t0.8\n");
    assert_eq!(m.extend(&more, 5), 4, "borrow, reference, owner and ⟨requires⟩");
    assert_eq!(m.joint.functions()[m.index["cat"]], cat, "old rows unchanged");
    assert!(m.kinds.contains(&"requires".to_string()));
    assert!(m.triple(&more[1]).is_some());
}

fn row(a: &str, k: &str, o: &str) -> phiecs::Row {
    phiecs::Row::new(a, k, o)
}

#[test]
fn a_word_is_drawn_to_the_scope_whose_functions_said_it() {
    let mut rows = vec![row("scope:sum", "within", "scope:data"), row("scope:count", "within", "scope:data")];
    for (f, scope, w) in [("sum_even", "scope:sum", "even"), ("sum_abs", "scope:sum", "abs"), ("count_even", "scope:count", "even"), ("count_below", "scope:count", "below"), ("count_odd", "scope:count", "odd")] {
        rows.push(row(f, "within", scope));
        rows.push(row(f, "within", "scope:data"));
        rows.push(row(f, SAYS, w));
        rows.push(row(f, SAYS, f.split('_').next().unwrap()));
    }
    let mut c = Tally::new(&["within"]);
    c.learn(&rows);
    let t = ScopeTree::from_rows(&rows);
    assert_eq!(t.roots(), ["scope:data"]);
    let found = t.cast(&c, &["count".into(), "odd".into()], f64::MIN, 2);
    assert_eq!(found[0].path, ["scope:data", "scope:count"], "{found:?}");
    let found = t.cast(&c, &["sum".into()], f64::MIN, 1);
    assert_eq!(found[0].path, ["scope:data", "scope:sum"], "{found:?}");
    assert_eq!(found.len(), 2);
}

