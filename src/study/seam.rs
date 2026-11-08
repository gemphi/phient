//! The seam (born as the phient crate, moved here from phiano when phient became
//! the brain): the phasor student coupled with the compiler it thinks with.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and
//! composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical
//! stores no point: a word is 32–64 relations, each a complex phasor on a
//! closed circle; meaning is a side effect of pairs of words moving under a
//! condition, and they move by the error of what was expected — toward what
//! came, away from what did not.
//!
//! Phient is the model those crates add up to: a phasor student
//! ([`crate::lineup::Ecs`]) that learns a voice ON THE FLY from what it is
//! told, coupled with the compiler it thinks with ([`phiclm`]) for the
//! things no model should memorize — arithmetic, loops, exactness. A person
//! does the same: they speak from what they have heard, and reach for a pen
//! when the answer must be computed rather than recalled.
//!
//! What is here is only the seam, honest about its size: `tell` streams a
//! sentence in, `guess` continues a prompt from the phasors, `compute`
//! judges a program. The measured state of each half is in
//! `phiano/docs/186–194`; neither half is overclaimed by being named here.

use super::arena::Reader;

/// The Phient model: the phasor student and the compiler, one seam.
/// (phiano's `Model` is a different thing: the app's whole model.)
pub struct Seam {
    student: crate::lineup::Ecs,
    /// Sentences told, words told — so a demo can say how much it has heard.
    pub heard: (usize, usize),
}

impl Seam {
    pub fn fresh() -> Self {
        Self { student: crate::lineup::Ecs::new(), heard: (0, 0) }
    }

    /// One sentence told to the student, on the fly — no retraining, ever.
    pub fn tell(&mut self, sentence: &str) {
        let words = words(sentence);
        if words.is_empty() {
            return;
        }
        self.heard = (self.heard.0 + 1, self.heard.1 + words.len());
        self.student.tell(&words);
    }

    /// The continuation of `prompt` from the phasors, up to `more` words:
    /// the student's next-word guesses, each fed back in, with an immediate-
    /// repeat guard. This is continuation in the learned voice — it is NOT
    /// comprehension, and callers should not present it as an answer.
    pub fn continue_from(&mut self, prompt: &str, more: usize) -> Vec<String> {
        let mut said = words(prompt);
        let mut out = Vec::new();
        for _ in 0..more {
            let Some(next) = self.student.guess(&said) else { break };
            if said.last() == Some(&next) {
                break;
            }
            said.push(next.clone());
            out.push(next);
        }
        out
    }

    /// The compiler half: `source` (phiclm's BCPL) run under `limit` steps —
    /// a broken source is a [`phiclm::Verdict`], never a panic. This is the
    /// part of an answer that must be computed, not recalled.
    pub fn compute(source: &str, limit: usize) -> phiclm::Verdict {
        phiclm::judge(source, limit)
    }
}

impl Default for Seam {
    fn default() -> Self {
        Self::fresh()
    }
}

/// Lower-cased alphanumeric words of a sentence (the student's diet).
pub fn words(sentence: &str) -> Vec<String> {
    sentence
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_learns_on_the_fly_and_computes_what_it_should_not_memorize() {
        let mut m = Seam::fresh();
        for _ in 0..8 {
            m.tell("the cat sat on the mat");
        }
        assert_eq!(m.heard.0, 8);
        let got = m.continue_from("the cat", 2);
        assert!(!got.is_empty(), "the student continues what it was told: {got:?}");
        let v = Seam::compute("LET start() = VALOF $( RESULTIS 6*7 $)", 10_000);
        assert!(v.compiled, "{:?}", v.error);
        assert_eq!(v.result, 42);
    }
}
