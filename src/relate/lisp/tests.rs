use super::*;
use phiecs::being::Being;
use phiecs::persona::Persona;
use phiecs::Row;
use phiecs::agent::Agent;

fn machine() -> Machine {
    let mut r = Being::new(Persona::from_md("---\nname: Bertrand Russell\nseeds: set\nkinds: is-a\n---\n").unwrap());
    r.tell(&[Row::new("socrates", "is-a", "man"), Row::new("man", "is-a", "mortal")]);
    let w = Being::new(Persona::from_md("---\nname: Alfred North Whitehead\nseeds: process\nkinds: is-a\n---\n").unwrap());
    let tree = vec![("text/tale/ch1/p1".to_string(), crate::prose::paragraph("text/tale/ch1/p1", &["the cat sat".to_string()]))];
    Machine::new().with_beings(vec![r, w]).with_scopes(crate::relate::trace::scopes_of(&tree))
}

#[test]
fn russell_on_the_machine() {
    let mut m = machine();
    assert_eq!(m.run("(count (held russell is-a))").unwrap(), Value::Num(2.0));
    assert_eq!(m.run("(count (closure (held russell is-a)))").unwrap(), Value::Num(3.0));
    assert_eq!(m.run("(members (relata (closure (held russell is-a)) socrates))").unwrap(), Value::List(vec![Value::Str("man".into()), Value::Str("mortal".into())]));
    assert_eq!(m.run("(count (pairs (a b) (b c)))").unwrap(), Value::Num(2.0));
    assert!(m.run("(transitive? (ancestral (pairs (a b) (b c))))").unwrap().truthy());
    // Whitehead is told the syllogism, and only then holds it.
    assert_eq!(m.run("(count (held whitehead is-a))").unwrap(), Value::Num(0.0));
    assert_eq!(m.run("(tell whitehead (difference (closure (held russell is-a)) (held russell is-a)) is-a)").unwrap(), Value::Num(1.0));
    // Read from the phasors (docs/187): the answer is the word one turn
    // past socrates on is-a; its score is agreement × amplitude, not a count.
    match m.run("(ask whitehead socrates is-a)").unwrap() {
        Value::List(answers) => match answers.first() {
            Some(Value::List(pair)) => assert_eq!(pair[0], Value::Str("mortal".into())),
            other => panic!("no answer: {other:?}"),
        },
        other => panic!("not a list: {other}"),
    }
}

#[test]
fn stories_on_the_machine() {
    let mut m = machine();
    assert_eq!(m.run("(count (next \"scope:text/tale/ch1\"))").unwrap(), Value::Num(2.0));
    assert_eq!(m.run("(count (says \"scope:text/tale\"))").unwrap(), Value::Num(3.0));
    assert_eq!(m.run("(members (image (next \"scope:text/tale\") (class the)))").unwrap(), Value::List(vec![Value::Str("cat".into())]));
    assert!(m.run("(next \"scope:nowhere\")").is_err());
    assert!(m.run("(frobnicate 1)").is_err());
}
