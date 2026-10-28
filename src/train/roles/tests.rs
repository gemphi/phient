#[allow(unused_imports)]
use super::*;

pub(super) fn facet_of(words: &[&str]) -> Facet {
    let mut f = Facet::new();
    for w in words {
        f.lexicon
            .insert((*w).to_string(), SpectralPhasor::seeded(w, 1.0, 1));
    }
    f
}

/// Binding must be exactly invertible, or a composed word cannot be queried.
#[test]
fn test_bind_unbind_is_exact() {
    let p = SpectralPhasor::seeded("currency", 1.0, 1);
    for role in Role::ALL {
        let back = RoleTable::unbind(&RoleTable::bind(&p, role), role);
        for k in 0..PHASE_CHANNELS {
            let d = (back.theta(k) - p.theta(k)).abs();
            // Phases are byte-quantised, so equality is to one quantum.
            assert!(
                d < TWO_PI / 255.0 || (TWO_PI - d) < TWO_PI / 255.0,
                "role {} channel {}: {} vs {}",
                role.name(),
                k,
                back.theta(k),
                p.theta(k)
            );
        }
    }
}

/// RoleTable must be distinguishable, or every relation is the same relation.
#[test]
fn test_roles_are_near_orthogonal() {
    let p = SpectralPhasor::seeded("currency", 1.0, 1);
    for a in Role::ALL {
        for b in Role::ALL {
            let r = RoleTable::bind(&p, a).resonance(&RoleTable::bind(&p, b));
            match a == b {
                true => assert!(r > 0.99, "{} vs itself: {}", a.name(), r),
                false => assert!(
                    r.abs() < 0.35,
                    "{} and {} are not distinguishable: {}",
                    a.name(),
                    b.name(),
                    r
                ),
            }
        }
    }
}

/// **The claim.** A word composed from its relations must answer *what is
/// its genus?* by unbinding — and must give a different answer for a
/// different role.
#[test]
fn test_identity_answers_role_queries() {
    let f = facet_of(&[
        "money", "currency", "transaction", "paper", "cow", "animal", "milk", "hide",
        "tree", "plant", "shade", "wood",
    ]);
    let mut rules = RuleSet::new();
    rules.add("money", Role::Genus, "currency");
    rules.add("money", Role::Function, "transaction");
    rules.add("money", Role::Form, "paper");
    rules.add("cow", Role::Genus, "animal");
    rules.add("cow", Role::Function, "milk");
    rules.add("cow", Role::Form, "hide");
    rules.add("tree", Role::Genus, "plant");
    rules.add("tree", Role::Function, "shade");
    rules.add("tree", Role::Form, "wood");

    for (head, genus, function) in
        [("money", "currency", "transaction"), ("cow", "animal", "milk"), ("tree", "plant", "shade")]
    {
        let id = RoleTable::identity(&f, &rules, head).expect("composed");
        let g = RoleTable::query(&f, &id, Role::Genus, &[head], 3);
        let fun = RoleTable::query(&f, &id, Role::Function, &[head], 3);

        assert!(
            g.iter().any(|(w, _)| w == genus),
            "genus of {} should recover {}, got {:?}",
            head,
            genus,
            g
        );
        assert!(
            fun.iter().any(|(w, _)| w == function),
            "function of {} should recover {}, got {:?}",
            head,
            function,
            fun
        );
        assert_ne!(
            g[0].0, fun[0].0,
            "two roles of {} must not return the same filler",
            head
        );
    }
}

/// Hierarchy: *a particular cow, a cow, an animal*. The chain composes.
#[test]
fn test_genus_chain_ascends() {
    let mut rules = RuleSet::new();
    rules.add("bessie", Role::Instance, "cow");
    rules.add("bessie", Role::Genus, "cow");
    rules.add("cow", Role::Genus, "animal");
    rules.add("animal", Role::Genus, "organism");
    rules.add("organism", Role::Genus, "thing");

    let path = RoleTable::ascend(&rules, "bessie", Role::Genus, 5);
    assert_eq!(path, vec!["bessie", "cow", "animal", "organism", "thing"]);
}

/// A cycle must terminate rather than spin: dictionaries define circularly.
#[test]
fn test_cyclic_chain_terminates() {
    let mut rules = RuleSet::new();
    rules.add("tree", Role::Genus, "plant");
    rules.add("plant", Role::Genus, "tree");
    let path = RoleTable::ascend(&rules, "tree", Role::Genus, 10);
    assert_eq!(path, vec!["tree", "plant"]);
}

#[cfg(test)]
mod coherence;

#[cfg(test)]
mod discovery;

#[cfg(test)]
mod lexical;
