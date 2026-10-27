//! What the learner is built from and how it is trained.

/// How the learner is built and trained.
#[derive(Clone, Debug)]
pub struct Config {
    /// Phasors per word per role — "a word is 32–64 relations".
    pub channels: usize,
    /// Earlier words the condition reads, each through its own operator.
    pub context: usize,
    /// Channels that pass phase to each other at each distance: 1 is one
    /// rotation per channel, `channels` a full operator on the field.
    pub block: usize,
    /// The pair term: the word before the last rotates the last.
    pub pair: bool,
    /// One set of angles for a word reaching and reached.
    pub tie: bool,
    /// The chance a context word stands as `<unk>` while training, so no
    /// context can be learned by heart.
    pub dropout: f32,
    /// The chance a channel of the condition falls silent while training.
    pub silence: f32,
    /// Wrong answers sampled against the one that came, per prediction.
    pub negatives: usize,
    /// Adagrad's step.
    pub rate: f32,
    /// Passes over everything told, when the arena says training is over.
    pub epochs: usize,
    pub seed: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            channels: 64,
            context: 4,
            block: 1,
            pair: true,
            tie: false,
            dropout: 0.1,
            silence: 0.0,
            negatives: 128,
            rate: 0.5,
            epochs: 3,
            seed: 7,
        }
    }
}
