use super::*;
use phical::circle;

/// One round's evidence: the corpus total and, per word, its own contribution,
/// its leave-one-out contexts, those contexts unit-normalised, and how many
/// sentences it appeared in.
pub(super) struct Votes<'a> {
    pub(super) total: Vec<c64>,
    pub(super) own_sum: HashMap<&'a str, Vec<c64>>,
    pub(super) loo_sum: HashMap<&'a str, Vec<c64>>,
    pub(super) unit_sum: HashMap<&'a str, Vec<c64>>,
    pub(super) counts: HashMap<&'a str, usize>,
}

impl Descent {
    /// UP: every sentence's state; DOWN: the leave-one-out votes it casts.
    pub(super) fn votes<'a>(candidate: &Facet, usable: &'a [Vec<(String, usize)>], up: Up) -> Votes<'a> {
        let states: Vec<Vec<c64>> = usable
            .iter()
            .map(|sent| {
                let mut h = vec![c64::new(0.0, 0.0); PHASE_CHANNELS];
                for (w, pos) in sent {
                    let p = &candidate.lexicon[w];
                    for (k, z) in h.iter_mut().enumerate() {
                        *z += Self::contribution(p, *pos, k, up);
                    }
                }
                h
            })
            .collect();

        // ---- DOWN ----
        //
        // Leave-one-out is per *type*, not per token: all occurrences of the
        // word are subtracted from its sentence's state. Subtracting only
        // the token being scored leaves the word's other occurrences in the
        // thing that moves it, which is the self-reinforcement this is
        // supposed to rule out — `the cat sat on the mat` would pull `the`
        // toward `the`.
        //
        // Everything below is accumulated linearly and length-normalised, so
        // that a word's own phase can be removed from the *corpus* average
        // exactly rather than approximately. That matters more than it
        // sounds: `the` sits inside the leave-one-out state of every other
        // word in its sentences, so any corpus average built without
        // removing it contains it, and subtracting such an average pushes
        // the word away from itself — a spreading force in semantic
        // costume. Two earlier formulations of the contrast failed exactly
        // there, each moving `the` further than a topical word instead of
        // less.
        let mut total = vec![c64::new(0.0, 0.0); PHASE_CHANNELS];
        // Per word: Σ its own contribution, Σ its leave-one-out contexts,
        // Σ those contexts unit-normalised (for the agreement test), and the
        // number of sentences it appeared in.
        let mut own_sum: HashMap<&str, Vec<c64>> = HashMap::new();
        let mut loo_sum: HashMap<&str, Vec<c64>> = HashMap::new();
        let mut unit_sum: HashMap<&str, Vec<c64>> = HashMap::new();
        let mut counts: HashMap<&str, usize> = HashMap::new();

        let zeros = || vec![c64::new(0.0, 0.0); PHASE_CHANNELS];

        for (sent, st) in usable.iter().zip(states.iter()) {
            let inv = 1.0 / sent.len() as f64;
            circle::add_scaled(&mut total, st, inv);

            let mut by_word: Vec<(&str, Vec<usize>)> = Vec::new();
            for (w, pos) in sent {
                match by_word.iter_mut().find(|(k, _)| *k == w.as_str()) {
                    Some((_, v)) => v.push(*pos),
                    None => by_word.push((w.as_str(), vec![*pos])),
                }
            }

            for (w, positions) in by_word {
                let p = &candidate.lexicon[w];
                let own: Vec<c64> = (0..PHASE_CHANNELS)
                    .map(|k| {
                        positions
                            .iter()
                            .map(|pos| Self::contribution(p, *pos, k, up))
                            .sum::<c64>()
                            * inv
                    })
                    .collect();

                let o = own_sum.entry(w).or_insert_with(zeros);
                for k in 0..PHASE_CHANNELS {
                    o[k] += own[k];
                }

                // A sentence made only of this word says nothing about it
                // once the word is removed — its own contribution still has
                // to come out of the corpus total above, but it casts no
                // vote below.
                let loo = circle::leave_one_out(st, &own, inv);
                if circle::is_silent(&loo) {
                    continue;
                }

                let l = loo_sum.entry(w).or_insert_with(zeros);
                for k in 0..PHASE_CHANNELS {
                    l[k] += loo[k];
                }
                let u = unit_sum.entry(w).or_insert_with(zeros);
                circle::add_directions(u, &loo);
                *counts.entry(w).or_insert(0) += 1;
            }
        }
        Votes { total, own_sum, loo_sum, unit_sum, counts }
    }

    /// Move each word toward what its contexts have that the average context
    /// does not; returns how many words moved.
    pub(super) fn settle(candidate: &mut Facet, v: &Votes, sentences: usize, step: f64, contrast: bool, concentration: f64) -> usize {
        let Votes { total, own_sum, loo_sum, unit_sum, counts } = v;
        let n_sent = sentences.max(1) as f64;

        // Applied in sorted order so the result does not depend on the
        // hash map's enumeration. The updates are independent, but the
        // reproducibility test is cheap to keep true.
        let mut keys: Vec<&str> = counts.keys().copied().collect();
        keys.sort_unstable();

        let mut words_moved = 0usize;
        let mut updates: Vec<(String, Vec<f64>)> = Vec::new();
        for w in keys {
            let n = counts[w] as f64;
            let loo = &loo_sum[w];
            let unit = &unit_sum[w];
            let own = &own_sum[w];
            let p = &candidate.lexicon[w];
            let mut moved = false;
            let mut thetas: Vec<f64> = (0..PHASE_CHANNELS).map(|k| p.theta(k)).collect();

            // A word with few contexts has a high resultant by accident:
            // n unit vectors drawn uniformly have expected resultant
            // sqrt(pi)/(2*sqrt(n)), which is 0.44 at n = 4. A fixed
            // threshold therefore lets rare words move on noise while
            // holding frequent ones still — exactly backwards. The floor is
            // the chance resultant for that word's own context count.
            let agree_floor = circle::rayleigh_floor(concentration, RAYLEIGH_C, n);

            for k in 0..PHASE_CHANNELS {
                // Do this word's contexts agree with each other at all?
                if circle::scattered(unit[k], n, agree_floor) {
                    continue;
                }

                // Where its contexts point, and where *every* context
                // points with this word taken out of them. Both are averages
                // of the same quantity — a length-normalised sentence state
                // minus this word — so the difference is exactly "what this
                // word's contexts have that the average context does not".
                // A word present in every sentence has the two averages
                // equal by construction and a residual of zero; under
                // contrast it has nothing left to move toward, and the
                // argument of a residual that small is a rounding error.
                let target = match circle::vote_target(loo[k], total[k], own[k], n, n_sent, contrast, CONTRAST_MIN) {
                    Some(t) => t,
                    None => continue,
                };

                thetas[k] = circle::step_toward(thetas[k], target, step);
                moved = true;
            }
            if moved {
                words_moved += 1;
                updates.push((w.to_string(), thetas));
            }
        }

        for (w, thetas) in updates {
            if let Some(p) = candidate.lexicon.get_mut(&w) {
                for (k, t) in thetas.iter().enumerate() {
                    p.set_theta(k, *t);
                }
                p.sync_phase();
            }
        }
        words_moved
    }
}
