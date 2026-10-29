//! The generator's and composition's own constants (they were phiano's
//! `config`; phiano's config re-exports them), and the shared ones the
//! engine reads, from the crates that own them.

pub use crate::metrics::kernel::{CONTEXT_LAMBDA, CONTEXT_OMEGA};
pub use crate::train::constants::LEARNING_RATE;
pub use phiecs::phasor::facet::constants::SYNTACTIC_MOMENTUM_DEFAULT;
pub use phiton::units::{ALPHA, PHASE_CHANNELS, PHI_CONJUGATE, TWO_PI};

/// Default context window ring buffer capacity.
pub const DEFAULT_CONTEXT_WINDOW: usize = 4096;

/// Candidate pool size for torus attractor decoding.
pub const TORUS_DECODE_POOL: usize = 48;

/// Phase dispersion below which generation warns that the manifold is degenerate.
pub const DEGENERACY_WARN: f64 = 0.2;

/// Learned generator weights (`learn::GenWeight`), fit by `bin/learn` from
/// held-out text. When present the generator scores candidates with them
/// instead of the hand-tuned constants.
pub const LEARNED_FILE: &str = "data/learned.json";

/// Convergence delta for composition - stop refining if improvement is below this.
pub const COMPOSE_CONVERGENCE_DELTA: f64 = 0.001;

/// Bonus weight for words that appeared in a winning sector last round
/// (Huberman's killer heuristic, 1968).
pub const KILLER_BONUS: f64 = 0.15;

/// Repulsion applied to losing compositions, relative to the learning rate.
pub const LOSER_REPULSION: f64 = 0.25;

/// Default pool size for ray cast in composition.
pub const RAY_CAST_POOL_SIZE: usize = 512;

/// Sector-score spread below which the variant population is degenerate.
pub const SPREAD_ALARM: f64 = 0.01;

/// Weight for prompt alignment in composition scoring.
pub const WEIGHT_ALIGNMENT: f64 = 0.30;

/// Weight for coherence in composition scoring.
pub const WEIGHT_COHERENCE: f64 = 0.25;

/// Weight for sector coverage in composition scoring.
pub const WEIGHT_COVERAGE: f64 = 0.05;

/// Weight for word diversity in composition scoring.
pub const WEIGHT_DIVERSITY: f64 = 0.10;

/// Weight for the composition length factor. Previously this term borrowed
/// WEIGHT_NOVELTY, which was therefore counted twice.
pub const WEIGHT_LENGTH: f64 = 0.15;

/// Weight for novelty in composition scoring.
pub const WEIGHT_NOVELTY: f64 = 0.15;

/// Weight for resonance in composition scoring.
pub const WEIGHT_RESONANCE: f64 = 0.15;
