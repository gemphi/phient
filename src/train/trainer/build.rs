#[allow(unused_imports)]
use super::*;

impl Trainer {
    /// Creates a new trainer with the given learning rate.
    pub fn new(learning_rate: f64) -> Self {
        Self { learning_rate, neg_samples: NEG_SAMPLES, definitions: None, seed: 0, relations: true, rounding: Default::default() }
    }

    /// Sets how relation-channel nudges are rounded to their byte.
    pub fn with_rounding(mut self, rounding: phiecs::phasor::spectral::Rounding) -> Self {
        self.rounding = rounding;
        self
    }

    /// Disables relation-channel coupling — the control arm.
    pub fn without_relations(mut self) -> Self {
        self.relations = false;
        self
    }

    /// Couples each observed pair **on its relation's channel only** and
    /// deposits evidence for that relation on both words.
    ///
    /// This is the symmetry break `bin/dimensions` asked for. The centroid and
    /// ranking objectives move a rotating subset of channels on every
    /// observation, so nothing distinguishes channel 17 from channel 42 and no
    /// channel can come to mean anything. Here an observation is evidence about
    /// one relation — the offset between the pair, or the marker word standing
    /// between them — so it moves one channel and deposits its evidence there.
    /// After enough text, channel `r` holds an arrangement of the vocabulary
    /// specific to relation `r`, and the word's evidence profile says which
    /// relations it takes part in at all.
    ///
    /// Which channel a pair lands on is decided by [`Facet::relations`]. Until
    /// [`Facet::learn_relations`] has run only the offset channels exist, which
    /// is the one relation always observable; the corpus-level entry points
    /// learn the markers first.
    ///
    /// Returns the number of pair couplings applied.
    pub fn couple_relations(&self, facet: &mut Facet, tokens: &[String]) -> usize {
        if !self.relations || tokens.len() < 2 {
            return 0;
        }
        let rate = RELATION_RATE.min(self.learning_rate.max(RELATION_RATE));
        let negatives = self.neg_samples > 0;
        if negatives {
            facet.rebuild_sample_pool();
            facet.rebuild_relation_pool();
        }
        let step_seed = fnv1a(&tokens.join(" ")) ^ phistd::mix64(self.seed ^ 0x5EED_5EED);
        let mut applied = 0usize;
        for i in 0..tokens.len() {
            for j in (i + 1)..(i + 1 + RELATION_WINDOW).min(tokens.len()) {
                if tokens[i] == tokens[j] {
                    continue;
                }
                let k = match facet.relations.channel(tokens, i, j) {
                    Some(k) => k,
                    None => continue,
                };
                let (ta, tb) = match (facet.lexicon.get(&tokens[i]), facet.lexicon.get(&tokens[j])) {
                    (Some(a), Some(b)) => (a.theta(k), b.theta_target(k)),
                    _ => continue,
                };
                // Kuramoto on one channel, *directed*: the earlier word's
                // source side moves toward the later word's target side and
                // vice versa. Both gain evidence for this relation. Source and
                // target are different angles, so "a then b" and "b then c"
                // share no variable and cannot chain into one global angle.
                let d = (tb - ta).sin();
                if let Some(p) = facet.lexicon.get_mut(&tokens[i]) {
                    p.nudge_by(k, rate * d, self.rounding);
                    p.deposit(k, 1);
                    if k == 0 {
                        p.sync_phase();
                    }
                }
                if let Some(p) = facet.lexicon.get_mut(&tokens[j]) {
                    // A rare target side sees few updates; give each one
                    // more weight until the word has RARE_TARGET_FLOOR
                    // observations on this channel (docs/72).
                    let boost = match p.evidence_count(k) < RARE_TARGET_FLOOR {
                        true => RARE_TARGET_BOOST,
                        false => 1.0,
                    };
                    p.nudge_target_by(k, -rate * d * boost, self.rounding);
                    p.deposit(k, 1);
                }
                applied += 1;

                // Repulsion on the same channel. Attraction alone has one
                // attractor — every word at one angle — and a channel every
                // adjacent pair feeds reaches it fast: without this term the
                // offset channels measured at 0.3–0.5 dispersion after four
                // passes and agreed about everything. One sampled word that
                // was *not* observed under this relation with `tokens[i]` is
                // pushed away from it here, which is the same contrastive rule
                // the centroid objective uses, restricted to the channel.
                if negatives {
                    let r = phistd::mix64(step_seed ^ ((i as u64) << 32) ^ ((j as u64) << 8));
                    let neg = match facet.sample_relation_negative(r) {
                        Some(w) if !tokens.iter().any(|t| t == w) => w.clone(),
                        _ => continue,
                    };
                    let tn = match facet.lexicon.get(&neg) {
                        Some(p) => p.theta_target(k),
                        None => continue,
                    };
                    // The negative's target side moves away from this word's
                    // source side, and the source side away from it.
                    let away = -(ta - tn).sin();
                    if let Some(p) = facet.lexicon.get_mut(&neg) {
                        p.nudge_target_by(k, rate * NEG_RATE * away, self.rounding);
                    }
                    if let Some(p) = facet.lexicon.get_mut(&tokens[i]) {
                        p.nudge_by(k, -rate * NEG_RATE * away, self.rounding);
                        if k == 0 {
                            p.sync_phase();
                        }
                    }
                }
            }
        }
        applied
    }

    /// The rotating channel subset a centroid or ranking step moves.
    ///
    /// Channel ownership: when relation coupling is on, the step draws only
    /// from channels no relation has claimed, so a relation channel is shaped
    /// by its relation and nothing else. With every channel claimed (or
    /// relations off) it falls back to the whole torus, which is the original
    /// behaviour.
    pub(super) fn update_channels(&self, facet: &Facet, step_seed: u64) -> Vec<usize> {
        let pool: Vec<usize> = match self.relations {
            true => facet.relations.unclaimed(),
            false => Vec::new(),
        };
        let pool: Vec<usize> = match pool.is_empty() {
            true => (0..PHASE_CHANNELS).collect(),
            false => pool,
        };
        let n = CHANNELS_PER_UPDATE.min(pool.len());
        let off = (phistd::mix64(step_seed) as usize) % pool.len();
        (0..n).map(|i| pool[(off + i * 7) % pool.len()]).collect()
    }

    /// Learns the facet's relation inventory from `sentences` if it has none.
    pub(super) fn ensure_relations(&self, facet: &mut Facet, sentences: &[String]) {
        if !self.relations || !facet.relations.is_empty() {
            return;
        }
        let docs: Vec<Vec<String>> = sentences.iter().map(|s| Tokenizer::tokenize(s)).collect();
        facet.learn_relations(&docs);
    }

    /// Attaches a definition graph, enabling controlled negative sampling.
    ///
    /// Without this the filter in [`Trainer::apply_negatives`] is unreachable —
    /// the field stays `None` at every construction site — so calling it is
    /// what makes the mechanism live rather than merely present.
    /// A trainer at an explicit seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_definitions(
        mut self,
        graph: std::sync::Arc<crate::train::conception::DefinitionGraph>,
    ) -> Self {
        self.definitions = Some(graph);
        self
    }

    /// Creates a trainer with repulsion disabled — attraction only.
    /// Retained for ablation studies; not a good default.
    pub fn attraction_only(learning_rate: f64) -> Self {
        Self { learning_rate, neg_samples: 0, definitions: None, seed: 0, relations: true, rounding: Default::default() }
    }
}
