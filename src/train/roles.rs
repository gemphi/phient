//! Constitutive rules: *X counts as Y in context C*.
//!
//! Searle's constitutive rule is a **typed triple**, and money is his own
//! canonical example. *Money is a form of currency that enables transactions;
//! usually a paper or coin representation* is not a bag of eleven words. It is:
//!
//! ```text
//! genus(money, currency)        money counts as currency
//! function(money, transaction)  ... in the context of exchange
//! form(money, paper)            ... realised as paper
//! ```
//!
//! # Why this is the fix, and not just a nicer story
//!
//! The measurements found the same failure three times: rotating a word by its
//! **position** times the golden angle hurt inside definitions (−44 pp
//! pair/random), in the two-word language-model context (+11 perplexity) and in
//! sentence composition (0.171 → 0.101 MRR). Three tasks, three scales, one
//! direction.
//!
//! The diagnosis is not "order is bad". Positional rotation is a **nonce** role:
//! *currency* lands at angle 1·φ in one entry and 7·φ in another, so the same
//! concept is scattered to a different angle in every context it appears in.
//! Superposing scattered copies cancels rather than accumulates, which is
//! precisely what a shared representation must not do.
//!
//! A **role** rotation is the opposite. Every `genus` relation binds at the same
//! angle, everywhere, forever. *currency* as the genus of *money* and *animal*
//! as the genus of *cow* land at the same offset from their heads, so the
//! genus relation itself becomes a direction in the manifold — and directions
//! superpose constructively.
//!
//! That is the whole difference, and it is testable rather than rhetorical:
//! unbinding a composed word by a role should recover that role's filler, and it
//! cannot if the binding was positional.
//!
//! # Hierarchy
//!
//! The rule composes, which is what makes *a cow as a particular cow, a cow as
//! an animal, an animal as a species* one structure rather than three facts.
//! `genus` chains: `bessie →genus cow →genus animal →genus organism`. Following
//! the chain is repeated unbinding, so a hierarchy is a path through the
//! manifold rather than a table beside it.

mod types;
mod role;
mod bind;
mod discovery;
mod purity;
mod lexical;
#[cfg(test)]
mod tests;
pub use types::*;
#[allow(unused_imports)]
use role::*;
#[allow(unused_imports)]
use bind::*;
#[allow(unused_imports)]
use discovery::*;
#[allow(unused_imports)]
use purity::*;
#[allow(unused_imports)]
use lexical::*;

use phiton::units::{PHASE_CHANNELS, TWO_PI};
use phiecs::phasor::facet::Facet;
use phiecs::phasor::spectral::{fnv1a, SpectralPhasor};
use std::collections::HashMap;
