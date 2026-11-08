//! The Lisp's phient host (docs/165, docs/phiclm/lisp): the machine is
//! `phiclm::lisp`; phient plugs in its world — the personas (`held`,
//! `ask`, `tell`, `told`, `beings`) and the stories' scopes (`scopes`,
//! `next`, `says`). Nothing here infers; only `tell` changes a world.
//!
//! ```text
//! (meet (held bertrand-russell is-a) (held whitehead is-a))
//! (count (closure (next "scope:text/emma/ch01")))
//! (define mortal (compose (held russell is-a) (held whitehead is-a)))
//! (tell claude mortal is-a)
//! ```

#[cfg(test)]
mod tests;

pub use phiclm::lisp::{read, Env, Value};

use super::set::held;
use super::trace::Scope;
use phiecs::agent::Agent;
use phiecs::being::Being;
use phiecs::Row;
use phiclm::lisp::{arg, as_relation, as_text, Host};
use std::collections::BTreeMap;

/// The machine with phient's world plugged in.
pub type Machine = phiclm::lisp::Machine<Sys>;

/// Phient's world, as the machine's host: the personas by slug, and the
/// stories' scopes by path.
#[derive(Default)]
pub struct Sys {
    pub scopes: BTreeMap<String, Scope>,
    pub beings: BTreeMap<String, Being>,
}

const NAMES: [&str; 8] = ["next", "says", "scopes", "held", "ask", "tell", "told", "beings"];

/// Personas and scopes onto a machine, as the REPL and the tests build one.
pub trait Stage {
    fn with_beings(self, beings: Vec<Being>) -> Self;
    fn with_scopes(self, scopes: BTreeMap<String, Scope>) -> Self;
}

impl Stage for Machine {
    fn with_beings(mut self, beings: Vec<Being>) -> Self {
        for b in beings {
            self.host.beings.insert(b.persona.slug(), b);
        }
        self
    }

    fn with_scopes(mut self, scopes: BTreeMap<String, Scope>) -> Self {
        self.host.scopes = scopes;
        self
    }
}

impl Sys {
    fn being(&mut self, v: &Value, name: &str) -> Result<&mut Being, String> {
        let who = v.text().ok_or_else(|| format!("{name}: who?"))?.to_lowercase().replace(' ', "-");
        let key = self.beings.keys().find(|k| **k == who || k.split('-').any(|w| w == who)).cloned().ok_or_else(|| format!("{name}: no persona {who}"))?;
        Ok(self.beings.get_mut(&key).unwrap())
    }
}

impl Host for Sys {
    fn names(&self) -> &'static [&'static str] {
        &NAMES
    }

    fn call(&mut self, name: &'static str, a: Vec<Value>) -> Result<Value, String> {
        let p = |i: usize| as_relation(arg(&a, i, name)?);
        let t = |i: usize| as_text(&a, i, name);
        Ok(match name {
        "next" => {
            let path = t(0)?;
            Value::Relation(self.scopes.get(&path).map(|sc| sc.pairs()).ok_or_else(|| format!("no scope {path}"))?)
        }
        "says" => {
            let path = t(0)?;
            Value::Class(self.scopes.get(&path).map(|sc| sc.says.keys().cloned().collect()).ok_or_else(|| format!("no scope {path}"))?)
        }
        "scopes" => Value::List(self.scopes.keys().map(|k| Value::Str(k.clone())).collect()),
        "held" => {
            let kind = t(1)?;
            Value::Relation(held(self.being(arg(&a, 0, name)?, name)?, &kind))
        }
        "ask" => {
            let (word, kind) = (t(1)?, t(2)?);
            Value::List(self.being(arg(&a, 0, name)?, name)?.ask(&word, &kind).into_iter().map(|(o, w)| Value::List(vec![Value::Str(o), Value::Num(w)])).collect())
        }
        "tell" => {
            let (rel, kind) = (p(1)?, t(2)?);
            let b = self.being(arg(&a, 0, name)?, name)?;
            let rows: Vec<Row> = rel.iter().map(|(x, y)| Row::new(x, &kind, y)).collect();
            b.tell(&rows);
            Value::Num(rows.len() as f64)
        }
        "told" => Value::Num(self.being(arg(&a, 0, name)?, name)?.told() as f64),
        "beings" => Value::List(self.beings.keys().map(|k| Value::Sym(k.clone())).collect()),
            other => return Err(format!("unknown primitive {other}")),
        })
    }
}
