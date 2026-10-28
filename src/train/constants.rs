//! The trainer's constants: its rates and margins, the negative sampling,
//! the grounding rounds, the channel primes. They were phiano's `config`
//! constants; phiano's `config` re-exports them from here.

/// Channels updated per training token. Updating a random subset each step is
/// dropout-like regularisation and keeps the cost of a token update bounded.
pub const CHANNELS_PER_UPDATE: usize = 16;

/// The first 64 primes, one per phase channel.
///
/// The golden angle solves one problem: a single irrational rotation never
/// repeats, so positions do not collide. It does nothing about a second one.
/// Every channel currently derives its frequency from the same linear ramp
/// (`CONTEXT_OMEGA * (1 + 4·k/K)`), and linearly spaced frequencies are
/// **commensurate** — channel k and channel 2k share periods, alias into each
/// other, and carry overlapping information. Sixty-four channels that alias are
/// fewer than sixty-four channels.
///
/// Coprime moduli fix that, and this is the oldest result in the area: the
/// Chinese remainder theorem says residues modulo pairwise-coprime numbers are
/// independent and jointly determine the value. Give channel *k* a frequency
/// proportional to the *k*-th prime and no two channels share a period until
/// their product, so the composite state's period is the product of all 64
/// primes rather than the smallest common multiple of a ramp.
///
/// It is a different fix from the golden angle rather than a replacement for
/// it, and like everything else here it is a switch with a measurement behind
/// it, not an assertion.
pub const CHANNEL_PRIMES: [u32; 64] = [
    2, 3, 5, 7, 11, 13, 17, 19,
    23, 29, 31, 37, 41, 43, 47, 53,
    59, 61, 67, 71, 73, 79, 83, 89,
    97, 101, 103, 107, 109, 113, 127, 131,
    137, 139, 149, 151, 157, 163, 167, 173,
    179, 181, 191, 193, 197, 199, 211, 223,
    227, 229, 233, 239, 241, 251, 257, 263,
    269, 271, 277, 281, 283, 293, 307, 311,
];

/// Weight of the pull back toward each word's trained phase during startup
/// composition.
///
/// This is a product decision, not a tuning one, because the sweep has no
/// dominant setting: alpha = 0.25 gives the best analogy MRR and alpha = 1.00
/// the best pair/random and a healthier manifold, monotonically, in opposite
/// directions. 0.5 sits between them, with the dispersion floor in
/// `core::grounding` as the binding constraint rather than this constant.
pub const COMPOSITION_ANCHOR: f64 = 0.5;

/// Phase convergence threshold - when |sin(φ_target - φ_old)| < this,
/// band_n is incremented (prevents phase collapse).
pub const CONVERGENCE_THRESHOLD: f64 = 0.0005;

/// Amplitude floor after a correction.
///
/// The floor was AMPLITUDE_INITIAL (1.0), so a repeatedly-corrected word could
/// never become *less* familiar than a word never seen — leaving no way to
/// represent "I have actively learned this is wrong" as distinct from "I have
/// no idea". Those are different epistemic states.
pub const CORRECTION_FLOOR: f64 = 0.3;

/// Weight applied to function words when computing a sentence centroid.
/// Closed-class words appear in nearly every sentence; at full weight they
/// transitively couple the entire vocabulary and accelerate collapse.
pub const FUNCTION_WORD_WEIGHT: f64 = 0.1;

/// Number of relaxation rounds for definition grounding.
pub const GROUNDING_ROUNDS: usize = 5;

/// Grounding pass version. A facet whose stored version matches is not
/// re-grounded at startup.
pub const GROUNDING_VERSION: u32 = 2;

/// Ground startup phases by multi-channel composition rather than the
/// single-channel centroid.
///
/// The centroid grounder writes one channel of 64 and was measured to move no
/// relation metric. Composition across all 64 moves analogy MRR from
/// 0.0002 ± 0.0001 to 0.0270 ± 0.0031 over five seeds on 296 pairs. Both paths
/// remain reachable for one release so the switch can be reverted without a
/// rebuild; the old one is then retired.
pub const GROUND_BY_COMPOSITION: bool = true;

/// Margin for the hinge loss on next-word retrieval.
pub const HINGE_MARGIN: f64 = 0.05;

/// Kuramoto learning rate for phase relaxation.
/// Recommended range: [0.01, 0.15]. Default: 0.05 (balanced).
pub use phiecs::phasor::facet::constants::LEARNING_RATE;

/// Repulsion rate for negative samples, relative to the learning rate.
pub const NEG_RATE: f64 = 0.5;

/// Negative samples drawn per token per training step.
///
/// Kuramoto coupling is attraction-only, and attraction-only dynamics have a
/// single stable attractor: total synchronisation. The negative term is what
/// makes the fixed point informative instead of degenerate.
pub const NEG_SAMPLES: usize = 5;

/// Anti-phase pulse applied to a misunderstood concept (π radians).
pub use phiecs::phasor::facet::constants::PHASE_REPULSION;

/// Modulus the prime frequencies are taken against.
///
/// `omega_k = 2*pi * p_k / PRIME_MODULUS`, and the size of this constant is the
/// whole design, not a detail.
///
/// The first value tried was 313 — the next prime above the largest channel
/// prime — on the reasoning that every channel should stay under one turn per
/// step. That satisfies coprimality and destroys the thing the recurrence
/// actually needs. At 313 the frequencies span 0.04 to 6.24 rad/step, so the
/// fastest channels rotate almost a full turn per step and are
/// indistinguishable from very slow ones running backwards. Measured, it cost
/// 3% of phase-alone perplexity against the linear ramp it replaced
/// (175.10 → 180.92).
///
/// A recurrence needs a *range of timescales*: slow channels that remember far
/// back, fast ones that track the last few tokens, and an ordering between
/// them. A large modulus keeps every frequency slow and well-separated while
/// leaving them pairwise incommensurate, which is what the primes were for.
/// 4099 is prime, so no channel's period divides another's.
pub const PRIME_MODULUS: f64 = 4099.0;

/// See `RARE_TARGET_FLOOR`. 1.0 disables.
///
/// Measured at 3.0 on the Rust Book harness, directed reading (docs/72):
/// identity 158.5 → 158.5, hinge 153.4 → 157.2, mixture 142.8 → 145.8 —
/// the operators alone get worse — while the best mix with the n-gram
/// tables improves 125.52 → 125.25, the largest phase gain over the tables
/// yet, and still a third of a point. Not a win; disabled, kept reproducible.
pub const RARE_TARGET_BOOST: f64 = 1.0;

/// Below this many observations on a channel, a word's target-side step is
/// multiplied by `RARE_TARGET_BOOST`. A target side moves only when the
/// word is the later word of a pair, so rare words' target sides are
/// under-trained and full-vocabulary perplexity pays (docs/71 §1).
pub const RARE_TARGET_FLOOR: u32 = 16;

/// Kuramoto rate for coupling a pair on the channel of the relation it was
/// observed under. Matches `phiecs::phasor::relational::RATE`, where it was measured.
pub const RELATION_RATE: f64 = 0.06;
