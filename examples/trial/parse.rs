//! BCPL's expression parser read on English, its priorities learned by
//! trial and error.
//!
//! BCPL's `rexp(n)` (phiclm `parse/expression`): an operator closes every
//! open operator of equal or higher priority, then waits for its operand.
//! Here the operators are the commonest words, and the top of the open
//! stack is the construction a sentence is still inside. Equal priorities
//! give the flat rule (the last operator is the open one, assumption A4);
//! learned priorities let a wide construction (`that`, `and`) stay open
//! across a narrow one (`the`). Each priority is an assumption: changed,
//! it is kept only if the surprise on the learning part of the book drops.

use std::collections::HashMap;

pub type Prio = HashMap<String, u8>;

/// The open operator before position `j`: the stack after `words[..j-1]`,
/// each operator closing those of equal or higher priority.
fn open<'a>(words: &'a [String], j: usize, prio: &Prio) -> Option<&'a str> {
    let mut stack: Vec<(&str, u8)> = Vec::new();
    for w in &words[..j.saturating_sub(1)] {
        if let Some(&p) = prio.get(w) {
            while stack.last().is_some_and(|&(_, q)| q >= p) {
                stack.pop();
            }
            stack.push((w.as_str(), p));
        }
    }
    stack.last().map(|&(w, _)| w)
}

type Rows = HashMap<String, HashMap<String, u32>>;

fn p(rows: &Rows, key: &str, w: &str) -> Option<f64> {
    let row = rows.get(key)?;
    let n: u32 = row.values().sum();
    Some(row.get(w).copied().unwrap_or(0) as f64 / n as f64)
}

fn add(rows: &mut Rows, key: String, w: &str) {
    *rows.entry(key).or_default().entry(w.to_string()).or_insert(0) += 1;
}

/// Bits per word of surprise over `lines[from..to]`, every line before
/// told first, online. `prio = None` reads the word before alone.
pub fn surprise(lines: &[Vec<String>], from: usize, to: usize, prio: Option<&Prio>) -> f64 {
    let (mut uni, mut bi, mut op): (Rows, Rows, Rows) = Default::default();
    let (mut bits, mut n) = (0.0, 0usize);
    for (i, words) in lines[..to].iter().enumerate() {
        for j in 1..words.len() {
            let w = &words[j];
            let key = prio.and_then(|pr| open(words, j, pr)).map(|o| format!("{o} … {}", words[j - 1]));
            if i >= from {
                let pu = p(&uni, "", w).unwrap_or(0.0);
                let pb = p(&bi, &words[j - 1], w);
                let po = key.as_deref().and_then(|k| p(&op, k, w));
                let q = match (po, pb) {
                    (Some(o), Some(b)) => 0.5 * o + 0.35 * b + 0.15 * pu,
                    (None, Some(b)) => 0.7 * b + 0.3 * pu,
                    (Some(o), None) => 0.7 * o + 0.3 * pu,
                    (None, None) => pu,
                };
                bits -= q.max(1e-7).log2();
                n += 1;
            }
            add(&mut uni, String::new(), w);
            add(&mut bi, words[j - 1].clone(), w);
            if let Some(k) = key {
                add(&mut op, k, w);
            }
        }
    }
    bits / n.max(1) as f64
}

/// The `k` commonest words, the operators.
pub fn operators(lines: &[Vec<String>], k: usize) -> Vec<String> {
    let mut f: HashMap<&String, u32> = HashMap::new();
    lines.iter().flatten().for_each(|w| *f.entry(w).or_insert(0) += 1);
    let mut v: Vec<_> = f.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    v.into_iter().take(k).map(|(w, _)| w.clone()).collect()
}

/// Learns the priorities on `lines[..learn]`: start flat at 3, try each
/// operator at every other level in turn, keep a change only if the
/// surprise drops. Returns the priorities and every kept change.
pub fn learn(lines: &[Vec<String>], learn: usize, ops: &[String], sweeps: usize) -> (Prio, Vec<(String, u8, f64)>) {
    let mut prio: Prio = ops.iter().map(|o| (o.clone(), 3)).collect();
    let from = learn / 2;
    let mut best = surprise(lines, from, learn, Some(&prio));
    let mut kept = Vec::new();
    for _ in 0..sweeps {
        for o in ops {
            for level in 1..=5u8 {
                let was = prio[o];
                if level == was {
                    continue;
                }
                prio.insert(o.clone(), level);
                let s = surprise(lines, from, learn, Some(&prio));
                match s < best {
                    true => {
                        best = s;
                        kept.push((o.clone(), level, s));
                    }
                    false => {
                        prio.insert(o.clone(), was);
                    }
                }
            }
        }
    }
    (prio, kept)
}
