//! The parts of the mix beyond its own counts: each gives a distribution
//! over the next word, and is told every line.

use super::classes::Classes;
use super::readers::Chain;
use super::scene::Scene;
use phient::lineup::learner::Learner;
use phient::study::Reader;
use std::collections::HashMap;

pub enum Part {
    /// The phasor learner (docs/203).
    Learner(Learner),
    /// The sentence as a scene (round 5).
    Scene(Scene),
    /// Words read through the class they behave like (round 6).
    Classes(Classes),
    /// A chain of contexts read as a distribution (round 7: the open operator).
    Chain(Chain),
}

impl Part {
    pub fn dist(&self, said: &[String]) -> HashMap<String, f64> {
        match self {
            Part::Learner(l) => l.ranked(said, 64).into_iter().collect(),
            Part::Scene(s) => s.dist(said),
            Part::Classes(c) => c.dist(said),
            Part::Chain(c) => c.dist(said),
        }
    }

    pub fn tell(&mut self, words: &[String]) {
        match self {
            Part::Learner(l) => l.tell(words),
            Part::Scene(s) => s.tell(words),
            Part::Classes(c) => c.tell(words),
            Part::Chain(c) => c.tell(words),
        }
    }
}
