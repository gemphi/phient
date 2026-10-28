//! Which readers each round puts in the arena.

use super::classes::Classes;
use super::hedge::Hedge;
use super::mix::{Mix, Report};
use super::parts::Part;
use super::readers::{bi, four, position, slot, suffix, tri, uni, Chain};
use super::scene::Scene;
use phient::lineup::learner::{Config, Learner};
use phient::lineup::Count;
use phient::study::Reader;

/// The experts of the hard trial and error (A5).
pub fn experts() -> Vec<Chain> {
    vec![
        Chain::new("the word before", vec![bi, uni]),
        Chain::new("two words before", vec![tri, bi, uni]),
        Chain::new("the open operator", vec![slot, bi, uni]),
        Chain::new("the place in the sentence", vec![position, bi, uni]),
    ]
}

const A6: &str = "A6 soft trial and error: mixing weights learned from the error";

/// A reader under a name of its own, so two of the same kind can be told apart.
struct Named<R: Reader>(String, R);

impl<R: Reader> Reader for Named<R> {
    fn name(&self) -> &str {
        &self.0
    }
    fn guess(&mut self, said: &[String]) -> Option<String> {
        self.1.guess(said)
    }
    fn tell(&mut self, words: &[String]) {
        self.1.tell(words)
    }
    fn ready(&mut self) {
        self.1.ready()
    }
}

/// The readers of `round`; `r` are the reports of its mixes, in order.
pub fn readers(round: u32, seam: u32, r: &[Report]) -> Vec<Box<dyn Reader>> {
    match round {
        8 | 10 => vec![
            Box::new(Count::new()),
            Box::new(Mix::new("A6 soft trial and error", 0).reporting(r[0].clone())),
            Box::new(Mix::new("A8 A6 with the scene (seam 3)", 0).with(Part::Scene(Scene::new(3)), r[1].clone())),
            Box::new(Mix::new("A9 A6 with classes", 0).with(Part::Classes(Classes::new(200, 500)), r[2].clone())),
            Box::new(Mix::new("A10 A6 with the open operator", 0).with(Part::Chain(Chain::new("slot", vec![slot])), r[3].clone())),
        ],
        12 => vec![
            Box::new(Count::new()),
            Box::new(Mix::new("A6 soft trial and error", 0).reporting(r[0].clone())),
            Box::new(Named("the phasor learner alone, online only".into(), Learner::new(Config { epochs: 0, ..Default::default() }))),
            Box::new(Named("the phasor learner alone, and 3 epochs on the shelf".into(), Learner::new(Config::default()))),
        ],
        11 => vec![
            Box::new(Count::new()),
            Box::new(Mix::new("A8 A6 with the scene (seam 3)", 0).with(Part::Scene(Scene::new(3)), r[0].clone())),
            Box::new(
                Mix::new("A11 A6 with the scene and the phasor learner", 0)
                    .with(Part::Scene(Scene::new(3)), r[1].clone())
                    .with(Part::Learner(Learner::new(Config::default())), r[1].clone()),
            ),
        ],
        7 => vec![
            Box::new(Count::new()),
            Box::new(Mix::new(A6, 0)),
            Box::new(Mix::new("A10 the same, with BCPL's open operator and its operand as a part", 0).with(Part::Chain(Chain::new("slot", vec![slot])), r[0].clone())),
        ],
        6 => vec![
            Box::new(Count::new()),
            Box::new(Mix::new(A6, 0)),
            Box::new(Mix::new("A9 the same, with classes: a word read through the anchor it behaves like", 0).with(Part::Classes(Classes::new(200, 500)), r[0].clone())),
        ],
        5 => vec![
            Box::new(Count::new()),
            Box::new(Mix::new(A6, 0)),
            Box::new(Mix::new("A8 the same, with the scene: what followed in the sentences this one joins", 0).with(Part::Scene(Scene::new(seam)), r[0].clone())),
        ],
        3 => vec![
            Box::new(Count::new()),
            Box::new(Chain::new("A1 two words before, backing off", vec![tri, bi, uni])),
            Box::new(Mix::new(A6, 0)),
            Box::new(Learner::new(Config::default())),
            Box::new(Mix::new("A7 the same, with the phasor learner as one more part", 0).with(Part::Learner(Learner::new(Config::default())), r[0].clone())),
        ],
        _ => vec![
            Box::new(Count::new()),
            Box::new(Chain::new("A0 the word before (backing off to the commonest)", vec![bi, uni])),
            Box::new(Chain::new("A1 two words before, backing off", vec![tri, bi, uni])),
            Box::new(Chain::new("A1b three words before, backing off", vec![four, tri, bi, uni])),
            Box::new(Chain::new("A2 bottom-up: a rare word read by its ending", vec![tri, bi, suffix, uni])),
            Box::new(Chain::new("A3 top-down: the place in the sentence", vec![position, bi, uni])),
            Box::new(Chain::new("A4 BCPL: the open operator and its operand", vec![slot, bi, uni])),
            Box::new(Chain::new("A4b two words, then the open operator", vec![tri, slot, bi, uni])),
            Box::new(Hedge::new("A5 trial and error: experts lose weight when wrong", experts(), 0.9, false)),
            Box::new(Hedge::new("A5b trial and error, weights per situation", experts(), 0.9, true)),
            Box::new(Mix::new(A6, 0)),
            Box::new(Mix::new("A6b the same, and the scope: the last 3 sentences' words recur", 3)),
        ],
    }
}
