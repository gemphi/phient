#[allow(unused_imports)]
use super::*;

fn domain(prefix: &str, n: usize) -> Vec<String> {
    (0..n)
        .map(|i| format!("{} alpha{} beta{} gamma{}", prefix, i % 7, i % 5, i % 3))
        .collect()
}

/// The bug this benchmark shipped with: training all of A then all of B is
/// the *sequential* condition, so using it as the joint ceiling made the two
/// models identical and retention came out at exactly 100% by construction.
#[test]
fn test_joint_training_differs_from_sequential() {
    let t = Trainer::new(0.05);
    let a = domain("harbour vessel", 40);
    let b = domain("mineral basalt", 40);
    let joint = ForgettingBenchmark::train_joint(&t, &a, &b, false);
    let seq = ForgettingBenchmark::train_sequential(&t, &[&a, &b], false);

    let differs = joint.lexicon.iter().any(|(w, p)| {
        seq.lexicon.get(w).map(|q| (p.phase - q.phase).abs() > 1e-9).unwrap_or(true)
    });
    assert!(differs, "the ceiling must not be the sequential model in disguise");
}

#[test]
fn test_bounds_are_ordered() {
    let t = Trainer::new(0.05);
    let a = domain("harbour vessel", 60);
    let b = domain("mineral basalt", 60);
    let r = ForgettingBenchmark::run(&t, "A", &a[..50], &a[50..], "B", &b, false);

    assert!(r.ceiling_ppl.is_finite() && r.floor_ppl.is_finite());
    // A model that never saw A cannot beat one trained on it.
    assert!(
        r.floor_ppl >= r.ceiling_ppl,
        "floor {} should not beat ceiling {}",
        r.floor_ppl, r.ceiling_ppl
    );
}

#[test]
fn test_retention_is_bounded() {
    let t = Trainer::new(0.05);
    let a = domain("harbour vessel", 40);
    let b = domain("mineral basalt", 40);
    let r = ForgettingBenchmark::run(&t, "A", &a[..30], &a[30..], "B", &b, false);
    assert!(r.retention.is_nan() || (r.retention >= 0.0 && r.retention <= 1.5));
}
