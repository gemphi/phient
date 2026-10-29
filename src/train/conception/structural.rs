#[allow(unused_imports)]
use super::*;
use phical::circle::Resultant;

/// Words too common to carry the meaning of an entry.
///
/// Not a stopword list for retrieval: these are the words that appear in a
/// large fraction of *all* definitions, so binding them into a composition adds
/// the same vector to every concept and washes the manifold out.
pub(super) fn is_structural(w: &str) -> bool {
    matches!(
        w,
        "a" | "an" | "the" | "of" | "or" | "and" | "to" | "in" | "is" | "are" | "as" | "that"
            | "which" | "with" | "for" | "by" | "on" | "at" | "from" | "it" | "its" | "any"
            | "one" | "used" | "being" | "having" | "esp" | "especially" | "also"
    )
}

impl Conception {
    /// Order-bound composition of a definition, per channel.
    ///
    /// Returns one target angle per channel, or `None` when the definition has
    /// no content words the facet knows.
    ///
    /// Position `i` rotates that word by `i · φ` (the golden angle) before it
    /// joins the sum. Rotation by an irrational multiple of a turn never
    /// repeats, so no two positions collide and the sum is invertible in
    /// principle rather than a bag of words. This is the same binding
    /// [`phiecs::phasor::wave::Wave::sentence_channels`] applies to sentences, used here
    /// on the definition that *constitutes* the word.
    pub fn compose(facet: &Facet, definition: &str) -> Option<Vec<f64>> {
        Self::compose_with(facet, definition, true)
    }

    /// [`Conception::compose`], with positional binding switchable.
    ///
    /// `bind = false` sums the definers with no rotation: a bag of words, where
    /// the same set of definers gives the same target however the entry is
    /// phrased. `bind = true` rotates each definer by its position. Which is
    /// better is an empirical question about the *source*: position is signal
    /// in a sentence and largely phrasing in a dictionary entry, and only the
    /// measurement can say which dominates.
    pub fn compose_with(facet: &Facet, definition: &str, bind: bool) -> Option<Vec<f64>> {
        let tokens: Vec<String> = Tokenizer::tokenize(definition)
            .into_iter()
            .filter(|w| !is_structural(w))
            .collect();

        let mut acc = vec![Resultant::new(); PHASE_CHANNELS];
        let mut used = 0usize;

        for (i, w) in tokens.iter().enumerate() {
            let p = match facet.lexicon.get(w) {
                Some(p) => p,
                None => continue,
            };
            used += 1;
            let roll = match bind {
                true => i as f64 * GOLDEN_ANGLE,
                false => 0.0,
            };
            // Amplitude is log-frequency, so a rare, specific definer already
            // counts for less than a common one. Inverting it here would let
            // hapax typos dominate an entry.
            let m = p.amplitude.max(1e-6);
            for (k, slot) in acc.iter_mut().enumerate() {
                slot.add(m, p.theta(k) + roll);
            }
        }

        if used == 0 {
            return None;
        }

        // A channel whose definers cancelled exactly carries no information;
        // leaving it at 0 would be a claim, so it is flagged with NaN and
        // skipped by the writer.
        Some(acc.iter().map(|r| r.direction_or_none().unwrap_or(f64::NAN)).collect())
    }

    /// Composes every entry into the manifold, `rounds` times.
    ///
    /// Definitions are recursive — *grandmother* is defined through *mother*,
    /// which is defined through *parent* — so one pass propagates meaning
    /// exactly one hop and the process is iterated. The 0.5 step halves the
    /// remaining error each round, so this converges rather than oscillating.
    pub fn compose_all(
        facet: &mut Facet,
        entries: &[(String, String)],
        rounds: usize,
    ) -> CompositionReport {
        Self::compose_all_with(facet, entries, rounds, HEAD_STEP, REINFORCE)
    }

    /// [`Conception::compose_all`] with explicit step sizes.
    ///
    /// `reinforce = 0.0` turns the mutual pull off, which is the control that
    /// isolates what reinforcement contributes from what multi-channel
    /// order-bound composition contributes.
    pub fn compose_all_with(
        facet: &mut Facet,
        entries: &[(String, String)],
        rounds: usize,
        head_step: f64,
        reinforce: f64,
    ) -> CompositionReport {
        Self::compose_all_bound(facet, entries, rounds, head_step, reinforce, true)
    }

    /// [`Conception::compose_all_with`], with positional binding switchable.
    pub fn compose_all_bound(
        facet: &mut Facet,
        entries: &[(String, String)],
        rounds: usize,
        head_step: f64,
        reinforce: f64,
        bind: bool,
    ) -> CompositionReport {
        Self::compose_graded(facet, entries, rounds, head_step, reinforce, reinforce, bind, None)
    }

    /// [`Conception::compose_all_bound`] with the reinforcement pull graded by
    /// pair strength.
    ///
    /// `graph = Some(..)` splits each definer into a strong pair (each word in
    /// the other's definition) or a weak one (one-way), and pulls them at
    /// `strong` and `weak` respectively. `graph = None` applies `strong` to
    /// everything, which is the flat behaviour and the control the grading has
    /// to beat.
    #[allow(clippy::too_many_arguments)]
    pub fn compose_graded(
        facet: &mut Facet,
        entries: &[(String, String)],
        rounds: usize,
        head_step: f64,
        strong: f64,
        weak: f64,
        bind: bool,
        graph: Option<&DefinitionGraph>,
    ) -> CompositionReport {
        Self::compose_anchored(
            facet, entries, rounds, head_step, strong, weak, bind, graph, 0.0, None,
        )
    }
}
