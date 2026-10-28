//! `train` — learning the lexicon: the trainer that couples a facet's words
//! by what they were told with, the downward path (sentences constraining
//! their words), definitions as compositions, the grounding that lays a
//! facet's phases from them, the roles, the chunk store and the sources it
//! reads, and the learning around them — the relation kinds, the synthetic
//! data, the curriculum, the correction journal, unknown words, program
//! synthesis and the lifelong coordinator. One cycle, moved whole: it was phiano's `trainer`, `topdown`,
//! `chunker`, `conception`, `sources`, `roles` and `core::grounding`.

pub mod chunker;
pub mod correction;
pub mod curriculum;
pub mod envision;
pub mod kinds;
pub mod lifelong;
pub mod synthesis;
pub mod synthetic;
pub mod conception;
pub mod constants;
pub mod grounding;
pub mod roles;
pub mod sources;
pub mod topdown;
pub mod trainer;

pub use trainer::Trainer;
