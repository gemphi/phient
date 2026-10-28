//! `scribe` — the writing student (docs/181): shown a use, taught by the
//! judge, remembering at two levels. The token [`Memory`] wanders and
//! discovers; the [`MoldSet`] reuses whole proven buildings and varies
//! their leaves. Neither is a model; both are counts moved in place.

mod memory;
mod mold;

pub use memory::{coin, draw, Memory, END};
pub use mold::{MoldSet, Slot};

/// The two memories together, kept in step.
#[derive(Default)]
pub struct Scribe {
    pub memory: Memory,
    pub mold: MoldSet,
}

impl Scribe {
    pub fn new() -> Self {
        Self::default()
    }

    /// A program shown or kept: told to both memories. Returns true when
    /// its skeleton was new — structural novelty, the stricter measure.
    pub fn keep(&mut self, tok: &[String]) -> bool {
        self.memory.tell(tok);
        self.mold.keep(tok)
    }
}
