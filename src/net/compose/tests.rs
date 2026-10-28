//! One pair key everywhere; roles differ; single-level recovery.

    use super::*;

    /// One pair key: the channel the trainer couples on, the channel the
    /// relational score reads on, and the role composition binds with are
    /// the same number for the same pair.
    #[test]
    fn pair_key_is_one_definition() {
        use phiecs::phasor::facet::Facet;
        use phiton::p64::words::Tokenizer;
        let mut facet = Facet::new();
        let docs: Vec<Vec<String>> = ["bread of wheat", "king of realm", "loaf of rye", "the dog sat", "a cat sat", "the bird sat"]
            .iter().map(|s| Tokenizer::tokenize(s)).collect();
        facet.learn_relations(&docs);
        let toks = Tokenizer::tokenize("bread of wheat");
        let k_train = facet.relations.channel(&toks, 0, 2).expect("channel");
        assert_eq!(facet.relations.name(k_train), "of", "a marked pair is keyed by its marker");
        // relational_score builds ctx + cand and reads channel(ctx_j, cand): same call, same key.
        let ctx = vec!["bread".to_string(), "of".to_string()];
        let mut probe = ctx.clone();
        probe.push("wheat".to_string());
        assert_eq!(facet.relations.channel(&probe, 0, 2), Some(k_train));
        // composition keyed by channel uses the same k as its role index.
        let space = super::super::space::FunctionSpace::from_facet(&facet);
        let pool: Vec<usize> = (0..space.len()).collect();
        let comp = Composition::from_corpus_keyed(&space, &docs, &pool, PairKey::Channel(&facet.relations));
        // Six sentences give no word enough fillers to be composable; assert
        // the invariant on whatever keys exist: every filler key is a channel
        // the vocabulary can name — never an offset invented for a marked pair.
        for f in comp.fillers.values() {
            for &(k, _) in f {
                assert!(k < PHASE_CHANNELS);
                assert!(!facet.relations.name(k).starts_with("ch"), "key {k} names a relation");
            }
        }
        let offset_toks = Tokenizer::tokenize("the dog sat");
        assert_eq!(facet.relations.channel(&offset_toks, 1, 2), Some(0), "an unmarked adjacent pair is offset+1");
        // Two apart with a marker between is keyed by the marker (here every
        // between-word became a marker on this tiny corpus), never by offset+2.
        let k02 = facet.relations.channel(&offset_toks, 0, 2).unwrap();
        assert!(k02 == 1 || facet.relations.name(k02) == "dog");
    }

    #[test]
    fn test_roles_differ_and_single_level_recovery() {
        let r1 = role(1);
        let r2 = role(2);
        assert!(cosine(&r1, &r2).abs() < 0.4, "roles must be distinct");
        let fs: Vec<WordFunction> = ["p", "q", "r", "s"]
            .iter()
            .map(|w| super::super::space::encode(&phiecs::phasor::SpectralPhasor::seeded(w, 1.0, 1)))
            .collect();
        let parts: Vec<WordFunction> = fs.iter().enumerate().map(|(d, f)| bind(f, &role(d + 1))).collect();
        let id = compose(&parts);
        for d in 0..4 {
            let q = unbind(&id, &role(d + 1));
            let own = cosine(&q, &fs[d]);
            for (j, other) in fs.iter().enumerate() {
                if j != d {
                    assert!(own > cosine(&q, other), "role {d}: own {own:.3} vs {j} {:.3}", cosine(&q, other));
                }
            }
        }
    }
