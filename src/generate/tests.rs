//! Generation tests: the context buffer, flow trajectories, sampling, the guide.

mod buffer_tests {
    use super::super::*;
    use phiecs::phasor::facet::Facet;

    #[test]
    fn test_context_wave_buffer() {
        let mut facet = Facet::new();
        facet.get_or_init("rust");
        facet.get_or_init("code");

        let mut buffer = ContextWaveBuffer::new(CONTEXT_WINDOW_SIZE);
        buffer.push_turn(&facet, "rust code");

        assert!(buffer.context_amplitude() > 0.0);
        assert_eq!(CONTEXT_LAYERS_COUNT, 16);
    }

    #[test]
    fn test_generate_with_flow_records_trajectory() {
        let mut facet = Facet::new();
        let trainer = crate::train::trainer::Trainer::new(0.05);
        trainer.train_sentence(&mut facet, "the cat sat on the mat");
        trainer.train_sentence(&mut facet, "the dog sat on the rug");

        let gen = Generator::new(8, 0.1);
        let mut buffer = ContextWaveBuffer::new(64);
        let (text, flow) = gen.generate_with_flow(&facet, &mut buffer, "the cat");

        assert!(flow.order_parameter >= 0.0);
        assert!(flow.collective_phase >= 0.0);
        let _ = text;
    }
}

#[cfg(test)]
mod sampling_tests {
    use super::super::*;

    #[test]
    fn test_temperature_zero_is_argmax() {
        let g = Generator::deterministic(8, 0.0, 42);
        let scores = [0.1, 0.9, 0.3];
        for _ in 0..8 {
            assert_eq!(g.sample_index(&scores), Some(1));
        }
    }

    #[test]
    fn test_sampling_actually_varies() {
        // The old "temperature" scaled a fixed sinusoid of the step index, so
        // the same input always produced the same output.
        let g = Generator::deterministic(8, 1.0, 7);
        let scores = [1.0, 1.0, 1.0, 1.0];
        let picks: std::collections::HashSet<usize> =
            (0..64).filter_map(|_| g.sample_index(&scores)).collect();
        assert!(picks.len() > 1, "a positive temperature must produce variation");
    }

    #[test]
    fn test_sampling_respects_the_distribution() {
        let g = Generator::deterministic(8, 0.5, 11);
        let scores = [0.0, 5.0];
        let hits = (0..400).filter(|_| g.sample_index(&scores) == Some(1)).count();
        assert!(hits > 340, "the high-scoring option should dominate: {}/400", hits);
    }

    #[test]
    fn test_deterministic_seed_reproduces() {
        let scores = [1.0, 2.0, 3.0, 4.0];
        let a: Vec<_> = { let g = Generator::deterministic(8, 1.0, 99); (0..20).map(|_| g.sample_index(&scores)).collect() };
        let b: Vec<_> = { let g = Generator::deterministic(8, 1.0, 99); (0..20).map(|_| g.sample_index(&scores)).collect() };
        assert_eq!(a, b, "a fixed seed must reproduce");
    }
}

#[cfg(test)]
mod operator_guide_tests {
    use super::super::*;
    use phiecs::phasor::facet::Facet;
    use crate::train::trainer::Trainer;

    /// A guide built from a fresh (untrained) operator changes candidate
    /// scores by a bounded factor and never drops a candidate.
    #[test]
    fn operator_factor_is_bounded_and_history_tracks() {
        let mut facet = Facet::new();
        let t = Trainer::new(crate::generate::constants::LEARNING_RATE).with_seed(1);
        for s in ["the cat sat on the mat", "the dog sat on the rug", "a cat and a dog sat"] {
            t.train_sentence(&mut facet, s);
        }
        let space = crate::net::space::FunctionSpace::from_facet(&facet);
        let cfg = phical::FnoConfig { space: space.space(), width: 4, layers: 1, modes: 8, margin: 0.2, lr: 1e-2, tie: phical::Tie::Circle };
        let guide = OperatorGuide { fno: phical::Fno::new(cfg, 3), space, window: 4, weight: OPERATOR_WEIGHT, proposals: 0, widen: 0, prior: Vec::new(), history: Vec::new(), cached: None, novelty: None, anchor: 0.0, anchor_fn: None };
        let g = Generator::deterministic(8, 0.0, 7).with_operator(guide);
        g.operator_reset(&["the".into(), "cat".into()]);
        let f = g.operator_factor("sat");
        assert!((1.0 - OPERATOR_WEIGHT..=1.0).contains(&f), "factor {f}");
        assert_eq!(g.operator_factor("zebra"), 1.0, "unknown word is neutral");
        g.operator_note("sat");
        assert_eq!(g.operator.as_ref().unwrap().borrow().history.len(), 3);
        let f2 = g.operator_factor("on");
        assert!((1.0 - OPERATOR_WEIGHT..=1.0).contains(&f2));
    }
}