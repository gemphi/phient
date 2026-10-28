#[allow(unused_imports)]
use super::*;

impl Conception {

    /// The full rule: composition, graded reinforcement, and a retrofitting
    /// anchor back toward the trained phase.
    ///
    /// `anchor` weighs "stay where training put you" against "move to your
    /// definers", exactly as in retrofitting. The anchor is taken against the
    /// phases as they were *before* the first round, not against the previous
    /// round, so iterating cannot walk the manifold away one step at a time.
    ///
    /// `hold` names words to move at a reduced rate — the grounding kernel. A
    /// dictionary's definitional graph has a small core that defines everything
    /// else (Vincent-Lamarre et al., 2016: the Kernel is ~10% of entries,
    /// MinSets ~1%), and meaning propagates outward from it. Letting the core
    /// drift as freely as the periphery composes each against a moving target.
    #[allow(clippy::too_many_arguments)]
    pub fn compose_anchored(
        facet: &mut Facet,
        entries: &[(String, String)],
        rounds: usize,
        head_step: f64,
        strong: f64,
        weak: f64,
        bind: bool,
        graph: Option<&DefinitionGraph>,
        anchor: f64,
        hold: Option<&std::collections::HashSet<String>>,
    ) -> CompositionReport {
        let mut report = CompositionReport { rounds, ..Default::default() };
        if entries.is_empty() {
            return report;
        }

        // The anchor is against the pre-composition phases, captured once.
        // Anchoring to the previous round instead would only slow the drift,
        // not bound it.
        let origin: HashMap<String, Vec<f64>> = match anchor > 0.0 {
            false => HashMap::new(),
            true => facet
                .lexicon
                .iter()
                .map(|(w, p)| {
                    (w.clone(), (0..PHASE_CHANNELS).map(|k| p.theta(k)).collect())
                })
                .collect(),
        };

        for _ in 0..rounds {
            // ---- READ: every target computed against frozen phases ----
            let mut head_targets: Vec<(String, Vec<f64>)> = Vec::with_capacity(entries.len());
            // Definer pulls accumulate as unit vectors per channel, so a word
            // appearing in many definitions ends up at their circular mean
            // rather than at whichever entry happened to be written last.
            let mut definer_pull: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
            let mut definer_hits: HashMap<String, u32> = HashMap::new();

            for (word, def) in entries {
                if !facet.lexicon.contains_key(word) {
                    continue;
                }
                let target = match Self::compose_with(facet, def, bind) {
                    Some(t) => t,
                    None => continue,
                };

                if strong > 0.0 || weak > 0.0 {
                    if let Some(head) = facet.lexicon.get(word) {
                        for t in Tokenizer::tokenize(def) {
                            if is_structural(&t) || t == *word {
                                continue;
                            }
                            if !facet.lexicon.contains_key(&t) {
                                continue;
                            }
                            // Pull weight is the pair's strength, and it enters
                            // as the vector's magnitude so a strong pair counts
                            // for more in the circular mean as well as moving
                            // further.
                            let w = match graph {
                                Some(g) if !g.is_strong(word, &t) => weak,
                                Some(_) => strong,
                                None => strong,
                            };
                            if w <= 0.0 {
                                continue;
                            }
                            *definer_hits.entry(t.clone()).or_insert(0) += 1;
                            let e = definer_pull
                                .entry(t)
                                .or_insert_with(|| vec![(0.0, 0.0); PHASE_CHANNELS]);
                            for (k, slot) in e.iter_mut().enumerate() {
                                let a = head.theta(k);
                                slot.0 += w * a.cos();
                                slot.1 += w * a.sin();
                            }
                        }
                    }
                }

                head_targets.push((word.clone(), target));
            }
            report.entries_used = head_targets.len();

            // ---- WRITE: heads first, then the reinforcement pull ----
            let mut moved = 0usize;
            let mut shift_sum = 0.0f64;
            let mut shift_n = 0usize;

            for (word, target) in &head_targets {
                // A kernel word is composed too, but at a fraction of the rate:
                // it is what the periphery is being composed *against*.
                let rate = match hold {
                    Some(h) if h.contains(word) => head_step * KERNEL_STEP,
                    _ => head_step,
                };
                let anchor_at = origin.get(word);
                if let Some(p) = facet.lexicon.get_mut(word) {
                    for k in 0..PHASE_CHANNELS {
                        if !target[k].is_finite() {
                            continue;
                        }
                        let cur = p.theta(k);
                        let mut step = rate * Self::delta(cur, target[k]);
                        // Retrofitting's first term: a pull back toward where
                        // training put this word, competing with the pull toward
                        // its definers.
                        if let Some(o) = anchor_at {
                            step += rate * anchor * Self::delta(cur, o[k]);
                        }
                        p.set_theta(k, cur + step);
                        shift_sum += step.abs();
                        shift_n += 1;
                    }
                    p.sync_phase();
                    moved += 1;
                }
            }
            report.heads_moved = moved;
            report.final_mean_shift = match shift_n {
                0 => 0.0,
                n => shift_sum / n as f64,
            };

            let mut reinforced = 0usize;
            for (word, pulls) in &definer_pull {
                let count = definer_hits.get(word).copied().unwrap_or(1) as f64;
                if let Some(p) = facet.lexicon.get_mut(word) {
                    for (k, &(x, y)) in pulls.iter().enumerate() {
                        if x.hypot(y) < 1e-12 {
                            continue;
                        }
                        let cur = p.theta(k);
                        // The accumulated magnitude already carries the pair
                        // weights; the step is scaled by the mean weight so a
                        // word pulled only by weak pairs moves less than one
                        // pulled by strong ones.
                        let scale = (x.hypot(y) / count.max(1.0)).min(1.0);
                        let step = head_step * scale * Self::delta(cur, y.atan2(x));
                        p.set_theta(k, cur + step);
                    }
                    p.sync_phase();
                    reinforced += 1;
                }
            }
            report.definers_reinforced = reinforced;
        }

        report
    }
}
