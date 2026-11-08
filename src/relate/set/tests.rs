use super::*;
use phiclm::lisp::set::converse;
use phiecs::being::Being;
use phiecs::persona::Persona;
use phiecs::Row;

fn being(name: &str, rows: &[(&str, &str, &str)]) -> Being {
    let mut b = Being::new(Persona::from_md(&format!("---\nname: {name}\nseeds: x\nkinds: is-a, part-of\n---\n")).unwrap());
    b.tell(&rows.iter().map(|(a, k, o)| Row::new(a, k, o)).collect::<Vec<_>>());
    b
}

#[test]
fn relations_are_sets_of_pairs() {
    let russell = being("Russell", &[("socrates", "is-a", "man"), ("man", "is-a", "mortal")]);
    let whitehead = being("Whitehead", &[("man", "is-a", "mortal"), ("process", "part-of", "reality")]);
    let (r, w) = (held(&russell, "is-a"), held(&whitehead, "is-a"));
    assert_eq!(union(&r, &w).len(), 2);
    assert_eq!(meet(&r, &w).len(), 1, "they agree that man is mortal");
    let syllogism = compose(&r, &w);
    assert!(syllogism.contains(&("socrates".into(), "mortal".into())), "socrates is-a man ∘ man is-a mortal");
    assert_eq!(converse(&r).iter().next().unwrap(), &("man".to_string(), "socrates".to_string()));
    let t = think(&russell, &whitehead);
    assert_eq!(t.iter().map(|x| x.kind.as_str()).collect::<Vec<_>>(), ["is-a", "part-of"]);
    assert!(render("R", "W", &t).contains("socrates→mortal"));
}

