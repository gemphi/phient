#[allow(unused_imports)]
use super::*;

impl RoleTable {
    /// Binds a filler into a role: rotate the filler's phases by the role's.
    ///
    /// Phase addition is circular convolution in this representation, so
    /// binding is invertible by subtraction — which is what makes the composed
    /// word queryable rather than merely a blend.
    pub fn bind(filler: &SpectralPhasor, role: Role) -> SpectralPhasor {
        let rot = role.rotation();
        let mut out = *filler;
        for (k, r) in rot.iter().enumerate() {
            out.set_theta(k, filler.theta(k) + r);
        }
        out.sync_phase();
        out
    }

    /// Removes a role's rotation, recovering the filler's phases.
    pub fn unbind(bound: &SpectralPhasor, role: Role) -> SpectralPhasor {
        let rot = role.rotation();
        let mut out = *bound;
        for (k, r) in rot.iter().enumerate() {
            out.set_theta(k, bound.theta(k) - r);
        }
        out.sync_phase();
        out
    }

    /// The relational identity of a word: the superposition of its bound
    /// relations.
    ///
    /// This is the operational form of "a word is defined by the relations it
    /// has". The head's own phases are *not* included — the identity is the
    /// relations, and mixing the head in would let a word be recovered from
    /// itself and make the recovery test vacuous.
    ///
    /// Returns `None` when the facet knows none of the fillers.
    pub fn identity(facet: &Facet, rules: &RuleSet, head: &str) -> Option<SpectralPhasor> {
        let mine = rules.rules_for(head);
        if mine.is_empty() {
            return None;
        }

        let mut acc = vec![phical::circle::Resultant::new(); PHASE_CHANNELS];
        let mut used = 0usize;
        for r in mine {
            let p = match facet.lexicon.get(&r.filler) {
                Some(p) => p,
                None => continue,
            };
            used += 1;
            let bound = Self::bind(p, r.role);
            for (k, a) in acc.iter_mut().enumerate() {
                a.add_unit(bound.theta(k));
            }
        }
        if used == 0 {
            return None;
        }

        let mut out = SpectralPhasor::seeded(head, 1.0, 1);
        for (k, a) in acc.iter().enumerate() {
            if let Some(direction) = a.direction_or_none() {
                out.set_theta(k, direction);
            }
        }
        out.sync_phase();
        Some(out)
    }

    /// Asks a composed identity what fills a role, returning the nearest words.
    ///
    /// `identity ⊖ role` should land on the filler that was bound there. This is
    /// the query that positional binding cannot answer: with a nonce rotation
    /// there is no role to subtract.
    pub fn query(
        facet: &Facet,
        identity: &SpectralPhasor,
        role: Role,
        exclude: &[&str],
        k: usize,
    ) -> Vec<(String, f64)> {
        let probe = Self::unbind(identity, role);
        let mut scored: Vec<(&str, f64)> = facet
            .lexicon
            .iter()
            .filter(|(w, _)| !exclude.contains(&w.as_str()))
            .map(|(w, p)| (w.as_str(), probe.resonance(p)))
            .collect();
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(b.0))
        });
        scored
            .into_iter()
            .take(k)
            .map(|(w, s)| (w.to_string(), s))
            .collect()
    }

    /// Walks a role chain: *bessie → cow → animal → organism*.
    ///
    /// A hierarchy is repeated unbinding, so it is a path through the manifold
    /// rather than a table beside it. Stops at `depth`, at a cycle, or when the
    /// chain runs out.
    pub fn ascend(rules: &RuleSet, start: &str, role: Role, depth: usize) -> Vec<String> {
        let mut path = vec![start.to_string()];
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        seen.insert(start.to_string());

        let mut cur = start.to_string();
        for _ in 0..depth {
            let next = rules
                .rules_for(&cur)
                .into_iter()
                .find(|r| r.role == role)
                .map(|r| r.filler.clone());
            match next {
                Some(n) if seen.insert(n.clone()) => {
                    path.push(n.clone());
                    cur = n;
                }
                _ => break,
            }
        }
        path
    }
}
