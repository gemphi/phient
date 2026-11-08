//! The phiecs reader: the words told as rows into a `World` and READ FROM
//! THE PHASORS — the difference under the condition (docs/187), never a
//! count. One angle per word per kind is the running mean of everything
//! the word was ever followed by, and crowds as more is fed; so the
//! CONDITION picks the channel: a pair is told under the construction
//! family the words so far show ("does not", "is a") when one is known,
//! and under plain `next` otherwise — the DAG of docs/174 on the circle.
//! Every family is a channel: past the base 64 a kind is an EXTENSION
//! channel a word pays for only where it touches it (docs/188), so the
//! conditions are not rationed — [`FAMILIES`] is a safety rail, not a
//! budget.

use super::Concept;
use crate::study::Reader;
use phiecs::agent::Agent;
use phiecs::being::Being;
use phiecs::persona::Persona;
use phiecs::Row;

/// Family channels opened at most — a rail far above any corpus here.
pub const FAMILIES: usize = 60_000;

pub struct Ecs {
    being: Being,
    /// Knows the construction families of what was said (the same reader
    /// the concept arm uses; here it only names the channel).
    family: Concept,
    opened: Vec<String>,
}

impl Ecs {
    pub fn new() -> Self {
        let p = Persona::from_md("---\nname: Arena\nseeds: thing\nkinds: next\n---\n").expect("persona");
        Self { being: Being::new(p), family: Concept::new(), opened: Vec::new() }
    }

    /// The channel for what follows `said`: its family's, if that family
    /// has a channel or one can still be opened; else `next`.
    fn channel(&mut self, said: &[String], open: bool) -> String {
        let Some((a, b)) = self.family.family(said) else { return "next".into() };
        let name = format!("{a} {b}");
        if self.opened.contains(&name) {
            return name;
        }
        if open && self.opened.len() < FAMILIES {
            self.opened.push(name.clone());
            return name;
        }
        "next".into()
    }

    /// The read of what follows `said`, ranked, from the deepest channel
    /// that answers at all. Deepest first (docs/170): the word BEFORE the
    /// last is the tightest condition — its own channel; then the family;
    /// then plain `next`. Each level answers only when the one below it is
    /// silent. (Summing the levels was measured in docs/189 and LOST —
    /// 20.7% against 24.3% — because the crowded shallow channel's many
    /// small votes swamp the deep channel's one confident answer.) The
    /// WHOLE read is returned, not only the winner, so a caller can see
    /// whether the difference under the condition is real.
    pub fn ranked(&mut self, said: &[String]) -> Vec<(String, f64)> {
        let Some(last) = said.last().cloned() else { return Vec::new() };
        if said.len() >= 2 {
            let deep = self.being.ask(&last, &format!("after {}", said[said.len() - 2]));
            if !deep.is_empty() {
                return deep;
            }
        }
        let kind = self.channel(said, false);
        let family = self.being.ask(&last, &kind);
        if !family.is_empty() {
            return family;
        }
        self.being.ask(&last, "next")
    }

    /// A word's angle under a kind, as the phasors hold it: `None` while the
    /// word or the kind is unknown, or the word has no evidence there. The
    /// angle is the word's TYPE as the world reads it (docs/200) — where it
    /// sits on the circle under that condition, before any name is given.
    pub fn angle(&self, word: &str, kind: &str) -> Option<f64> {
        use phiecs::agent::Agent;
        let w = self.being.world();
        let (id, k) = (w.lexicon.get(word)?, w.relations.get(kind)?);
        let p = w.read::<phiecs::phasor::Phasor>(id)?;
        (p.evidence(k) > 0).then(|| p.theta(k))
    }

    /// A word's PROFILE under a kind: its angle on each answer slot of the
    /// kind it has spoken on (`None` where it has not). One angle averages
    /// a mixture — `the` reaches nouns, `is` reaches adjectives, and under
    /// the plain channel both sit at the mean (docs/200); sixteen angles
    /// keep the mixture apart (docs/202).
    pub fn profile(&self, word: &str, kind: &str) -> Vec<Option<f64>> {
        use phiecs::agent::Agent;
        let w = self.being.world();
        let Some(id) = w.lexicon.get(word) else { return Vec::new() };
        let Some(p) = w.read::<phiecs::phasor::Phasor>(id) else { return Vec::new() };
        (0..phiecs::world::SLOTS)
            .map(|s| w.relations.get(&format!("{kind}#{s}")).filter(|r| p.spoken(*r) > 0).map(|r| p.theta(r)))
            .collect()
    }

    /// How far the first answer stands CLEAR of the second, as a share of
    /// the first: 1.0 when it answers alone, 0.0 when the two are tied.
    /// A crowded word — `the`, `to`, `a` — reads as a near-tie, because its
    /// phasor points at the mean of everything it was ever followed by; the
    /// condition then does not determine the next word, and a voice that
    /// speaks anyway drifts into that crowd (docs/198). The difference, not
    /// the winner, is what was learned.
    pub fn clearance(read: &[(String, f64)]) -> f64 {
        match (read.first(), read.get(1)) {
            (Some((_, a)), Some((_, b))) if *a > 0.0 => ((a - b) / a).clamp(0.0, 1.0),
            (Some(_), None) => 1.0,
            _ => 0.0,
        }
    }
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for Ecs {
    fn name(&self) -> &str {
        "phiecs (phasors: word before → family → next)"
    }

    fn guess(&mut self, said: &[String]) -> Option<String> {
        self.ranked(said).first().map(|(w, _)| w.clone())
    }

    fn tell(&mut self, words: &[String]) {
        let mut rows: Vec<Row> = Vec::with_capacity(words.len() * 3);
        for j in 1..words.len() {
            let kind = self.channel(&words[..j], true);
            rows.push(Row::new(&words[j - 1], "next", &words[j]));
            if kind != "next" {
                rows.push(Row::new(&words[j - 1], &kind, &words[j]));
            }
            if j >= 2 {
                rows.push(Row::new(&words[j - 1], &format!("after {}", words[j - 2]), &words[j]));
            }
        }
        self.being.tell(&rows);
        self.family.tell(words);
    }
}
