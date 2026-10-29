//! The tuner recovers planted weights.

    use super::*;

    /// A planted weight vector is recovered from data it generated.
    #[test]
    fn tuner_recovers_planted_weights() {
        let planted = [1.0, 0.8, -0.5, 0.3, -1.0, 0.0, 0.6, 0.0];
        let mut seed = 7u64;
        let mut rnd = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let mut make = |n: usize| -> Vec<Position> {
            (0..n)
                .map(|_| {
                    let feats: Vec<CandidateFeature> = (0..8)
                        .map(|_| CandidateFeature([rnd() * 3.0, rnd(), rnd() * 2.0 - 1.0, rnd().round(), rnd().round(), 0.0, rnd() * 2.0 - 1.0, 0.0]))
                        .collect();
                    let scores: Vec<f64> = feats.iter().map(|x| planted.iter().zip(&x.0).map(|(a, b)| a * b).sum::<f64>()).collect();
                    let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let ex: Vec<f64> = scores.iter().map(|s| (s - max).exp()).collect();
                    let z: f64 = ex.iter().sum();
                    let mut r = rnd() * z;
                    let mut truth = ex.len() - 1;
                    for (i, e) in ex.iter().enumerate() {
                        r -= e;
                        if r <= 0.0 {
                            truth = i;
                            break;
                        }
                    }
                    Position { feats, truth }
                })
                .collect()
        };
        let train = make(4000);
        let held = make(1000);
        let (w, trace) = Tuner { lr: 0.05, epochs: 400, l2: 0.0 }.fit(GenWeight::counts_only().w, &train, &held);
        assert!(trace.last().unwrap() < &trace[0], "held-out NLL falls");
        for f in [0usize, 1, 2, 3, 4, 6] {
            assert!((w.w[f] - planted[f]).abs() < 0.25, "weight {f}: {} vs planted {}", w.w[f], planted[f]);
        }
        assert!(w.heldout_nll < w.baseline_nll);
    }
