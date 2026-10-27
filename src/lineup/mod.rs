//! `lineup` — the readers the arena scores (docs/176): one per model, so one
//! run says which of Phient's models is learning better. Every reader
//! answers the same question at the same position before any of them is
//! told it ([`crate::study::Arena`]); the table at the end is the
//! leaderboard. A reader that needs an application's world (phiano's
//! `arena::Ecs`, which reads a persona) lives with that application.
//!
//! | reader | the model it stands for |
//! |---|---|
//! | [`count::Count`] | phient's counts: the word before, nothing else (the line to beat) |
//! | [`concept::Concept`] | phient's concepts: the construction family the words said show, backing off onto the counts (docs/174–175) |
//! | [`mirror::Mirror`] | phical's conjugate: the relation as an all-pass filter on the circle, read by correlation |
//! | [`operator::Operator`] | phifno: the Joint operator over a table of functions, batch-trained on the split lane (docs/177) |
//! | [`learner::Learner`] | the phasors corrected by their error: predict, compare, move — not counted (docs/203) |

pub mod concept;
pub mod count;
pub mod learner;
pub mod mirror;
pub mod operator;

pub use concept::Concept;
pub use count::Count;
pub use learner::Learner;
pub use mirror::Mirror;
pub use operator::Operator;
