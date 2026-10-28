mod clean;
mod strip;
#[cfg(test)]
mod tests;
pub use clean::*;
#[allow(unused_imports)]
use strip::*;

pub mod api;
pub mod dialogue;
pub mod json;
pub mod local;
pub mod phi4;
pub mod wiktionary;

/// DictionarySource - a source of word definitions for bootstrapping the facet.
///
/// Implementations include local files, JSON dictionaries, API sources,
/// Wiktionary dumps, and Phi-4 references.
pub trait DictionarySource {
    /// Returns all (word, definition) pairs from this source.
    fn fetch_all(&self) -> Vec<(String, String)>;

    /// Returns all definitions for a single word from this source.
    fn fetch_definitions(&self, word: &str) -> Vec<String>;
}

/// Senses kept from an entry.
///
/// Webster's stacks every historical sense of a word into one entry — *cat* runs
/// from the animal through a type of sailing vessel to a constellation. Later
/// senses are real but rare, and composing all of them places the word at the
/// centre of mass of meanings nobody uses. Three keeps the polysemy that matters
/// and drops the tail.
const MAX_SENSES: usize = 3;

/// Truncation limit on the sense list read out of a dictionary entry.
///
/// This was documented as a *bound* — the claim that a word's uses are finite,
/// so a sense can be a type index rather than a string. It never did that work.
/// `defstats` measured 1.51 senses per word with a maximum of 45, so `.take(64)`
/// has never once fired: it is inert, and a cap that never binds demonstrates
/// nothing about whether the inventory is bounded. Worse, whatever senses do
/// come back are collapsed into a single phasor immediately afterwards, so the
/// representation cannot tell two of them apart in any case — which is where
/// `bin/frames` found Searle's third term going missing.
///
/// It stays as a guard against a malformed entry producing a runaway list. It is
/// not evidence for the architecture, and the doc no longer says it is.
pub const MAX_WORD_SENSES: usize = 64;
