//! The scribe's token memory: the counts at depth two backing off to depth
//! one (docs/170's lesson — depth, not flat leaves — arriving in writing),
//! held in two plain [`Count`]s. `deep` is told the PAIRED stream
//! (`"a b" "b c" …`), so its choices after "a b" are the pairs that
//! followed — the same counting, one level up.

use crate::lineup::Count;
use crate::study::Reader;

/// The end-of-program word the scribe learns and stops on.
pub const END: &str = "⟨end⟩";

#[derive(Default)]
pub struct Memory {
    one: Count,
    deep: Count,
}

/// A deterministic coin.
pub fn coin(state: &mut u64) -> f64 {
    phistd::random::unit_interval(phistd::random::lcg(state))
}

/// A weighted draw.
pub fn draw(choice: &[(&String, u32)], rng: &mut u64) -> String {
    let weights: Vec<u32> = choice.iter().map(|(_, n)| *n).collect();
    choice[phistd::random::draw_by_weight(&weights, coin(rng))].0.clone()
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    /// One program's tokens told, at both depths, ending in [`END`].
    pub fn tell(&mut self, tok: &[String]) {
        let mut t: Vec<String> = tok.to_vec();
        t.push(END.into());
        self.one.tell(&t);
        let paired: Vec<String> = t.windows(2).map(|p| format!("{} {}", p[0], p[1])).collect();
        self.deep.tell(&paired);
    }

    /// One program written: the last two tokens consulted first, backing
    /// off to the last one, exploring `vocab` at rate `explore`.
    pub fn write(&self, vocab: &[String], rng: &mut u64, explore: f64, cap: usize) -> Vec<String> {
        let mut tok: Vec<String> = vec!["let".into()];
        while tok.len() < cap {
            let next = if coin(rng) < explore {
                vocab[(coin(rng) * vocab.len() as f64) as usize % vocab.len()].clone()
            } else {
                let two = match tok.len() {
                    0 | 1 => Vec::new(),
                    n => self.deep.choices(&format!("{} {}", tok[n - 2], tok[n - 1])),
                };
                if !two.is_empty() {
                    draw(&two, rng).split(' ').nth(1).unwrap_or(END).to_string()
                } else {
                    let one = self.one.choices(tok.last().expect("seeded"));
                    if one.is_empty() {
                        vocab[(coin(rng) * vocab.len() as f64) as usize % vocab.len()].clone()
                    } else {
                        draw(&one, rng)
                    }
                }
            };
            if next == END {
                break;
            }
            tok.push(next);
        }
        tok
    }
}
