#[allow(unused_imports)]
use super::*;

pub(super) use phiecs::phasor::spectral::SpectralPhasor;

pub(super) fn facet_with(words: &[&str]) -> Facet {
    let mut f = Facet::new();
    for w in words {
        f.lexicon
            .insert((*w).to_string(), SpectralPhasor::seeded(w, 1.0, 1));
    }
    f
}

/// The whole point of positional rotation: a definition is not a bag.
#[test]
fn test_composition_is_order_sensitive() {
    let f = facet_with(&["board", "directors", "member", "group"]);
    let a = Conception::compose(&f, "member board directors group").unwrap();
    let b = Conception::compose(&f, "group directors board member").unwrap();
    let differing = (0..PHASE_CHANNELS)
        .filter(|k| (a[*k] - b[*k]).abs() > 1e-6)
        .count();
    assert!(
        differing > PHASE_CHANNELS / 2,
        "reordering a definition must move most channels, moved {}",
        differing
    );
}

/// Every channel must carry the definition, not just channel 0.
#[test]
fn test_composition_writes_every_channel() {
    let mut f = facet_with(&["grandmother", "mother", "parent", "female"]);
    let before: Vec<f64> = (0..PHASE_CHANNELS)
        .map(|k| f.lexicon["grandmother"].theta(k))
        .collect();

    Conception::compose_all(
        &mut f,
        &[("grandmother".into(), "mother of a parent female".into())],
        1,
    );

    let after: Vec<f64> = (0..PHASE_CHANNELS)
        .map(|k| f.lexicon["grandmother"].theta(k))
        .collect();
    let moved = (0..PHASE_CHANNELS)
        .filter(|k| (before[*k] - after[*k]).abs() > 1e-6)
        .count();
    assert!(
        moved > PHASE_CHANNELS / 2,
        "composition touched only {} of {} channels — this is the \
         single-channel limitation it exists to remove",
        moved,
        PHASE_CHANNELS
    );
}

/// Reinforcement must move the definers, and must be the only thing that
/// does — otherwise the control that isolates it is not a control.
#[test]
fn test_reinforcement_moves_definers_and_is_switchable() {
    let entries = vec![("grandmother".to_string(), "mother of a parent".to_string())];

    let mut off = facet_with(&["grandmother", "mother", "parent"]);
    let before = off.lexicon["mother"].theta(3);
    Conception::compose_all_with(&mut off, &entries, 1, HEAD_STEP, 0.0);
    assert!(
        (off.lexicon["mother"].theta(3) - before).abs() < 1e-9,
        "with reinforcement off, a definer must not move"
    );

    let mut on = facet_with(&["grandmother", "mother", "parent"]);
    Conception::compose_all_with(&mut on, &entries, 1, HEAD_STEP, REINFORCE);
    assert!(
        (on.lexicon["mother"].theta(3) - before).abs() > 1e-9,
        "with reinforcement on, a definer must move toward the headword"
    );
}

/// Strong and weak pairs must be told apart by reciprocity, not by
/// co-occurrence, and both must be excluded from negative sampling.
#[test]
fn test_definition_graph_strong_weak_and_relatedness() {
    let entries = vec![
        ("car".to_string(), "road vehicle with an engine".to_string()),
        ("vehicle".to_string(), "a car or truck".to_string()),
        ("road".to_string(), "a wide way between places".to_string()),
    ];
    let g = DefinitionGraph::build(&entries);

    // car defines vehicle and vehicle defines car: reciprocal.
    assert!(g.is_strong("car", "vehicle"));
    // car defines road, road does not define car: one-way.
    assert!(!g.is_strong("car", "road"));
    // but still related, so still barred from being a negative sample.
    assert!(g.is_related("car", "road"));
    assert!(!g.is_related("car", "places"));
}

/// Grading by pair strength must actually move strong and weak definers by
/// different amounts — otherwise it is the flat rule under another name.
#[test]
fn test_strong_pairs_move_further_than_weak() {
    let entries = vec![
        ("car".to_string(), "road vehicle".to_string()),
        ("vehicle".to_string(), "a car".to_string()),
        ("road".to_string(), "a wide way".to_string()),
    ];
    let g = DefinitionGraph::build(&entries);
    assert!(g.is_strong("car", "vehicle") && !g.is_strong("car", "road"));

    let start = facet_with(&["car", "vehicle", "road", "wide", "way"]);

    let mut graded = start.clone();
    Conception::compose_graded(
        &mut graded,
        &entries,
        1,
        HEAD_STEP,
        BETA_STRONG,
        BETA_WEAK,
        false,
        Some(&g),
    );

    let shift = |a: &Facet, b: &Facet, w: &str| {
        (0..PHASE_CHANNELS)
            .map(|k| Conception::delta(a.lexicon[w].theta(k), b.lexicon[w].theta(k)).abs())
            .sum::<f64>()
    };

    assert!(
        shift(&start, &graded, "vehicle") > shift(&start, &graded, "road"),
        "the strong definer must be pulled further than the weak one"
    );
}

#[cfg(test)]
mod bounds;
