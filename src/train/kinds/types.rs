#[allow(unused_imports)]
use super::*;

/// The most relation types this can represent.
///
/// Chosen to match the channel count, so a type index and a channel index are
/// the same width — the bounded-type claim, made structural. Whether the *data*
/// needs this many is a separate question, and [`KindTable::saturation`] is
/// how it gets asked rather than assumed.
pub const MAX_RELATION_TYPES: usize = 64;

/// How the step from a head to a filler is turned into a vector.
///
/// The plain difference `theta(filler) - theta(head)` is what every relational
/// measurement in this project has used, and the type sweep showed what is wrong
/// with it: **71.3% of pairs share a cluster with another pair of the same head,
/// against 9.1% expected if the head were ignored.** `adversary->against` and
/// `adversary->another` land together because both differences contain
/// `-theta(adversary)`, not because they are the same relation.
///
/// That is a *main effect* of the head, and main effects have a standard
/// removal: centering. Writing the offset as
///
/// ```text
/// offset(h, f) = grand + head_effect(h) + filler_effect(f) + interaction(h, f)
/// ```
///
/// a relation — a step that means the same thing wherever it is taken — can only
/// live in the interaction term. The estimators below strip the other terms
/// away, one at a time, so the question "is there anything left once word
/// identity is removed" gets an answer instead of an assumption.
///
/// All of it is done in circular space: an offset is a unit complex number per
/// channel, a mean is a circular mean, and centering subtracts angles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetEstimator {
    /// `theta(filler) - theta(head)`. What every earlier measurement used.
    Raw,
    /// Raw, minus the head's own mean offset across all its fillers.
    ///
    /// Removes the head main effect and nothing else. If relations are real,
    /// this should *help*: what remains is how this particular filler differs
    /// from the head's typical filler.
    HeadCentered,
    /// Raw, minus the head's mean and the filler's mean, plus the grand mean.
    ///
    /// The two-way residual — the interaction alone. This is the strictest
    /// version of the question, and the one whose failure would be decisive:
    /// if nothing survives here, the manifold places words with no relational
    /// interaction at all, and no readout can recover what was never encoded.
    DoubleCentered,
}

/// Offsets for a pair set, under one estimator.
///
/// Built once and clustered many times. Centering needs the whole set — a head's
/// mean is over every filler it takes — so the estimator cannot be applied one
/// pair at a time.
#[derive(Debug, Clone)]
pub struct OffsetTable {
    pub(super) pairs: Vec<(String, String)>,
    pub(super) offsets: Vec<Vec<f64>>,
}

/// A relation type.
///
/// Opaque by construction: the field is private, there is no public constructor,
/// and the rotation it stands for lives in the [`KindTable`] that issued it.
/// A caller can hold one, compare it, and pass it to an operation. It cannot
/// build one, take it apart, or mean anything by it that the data did not put
/// there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelationType(pub(super) u8);

/// One discovered type's evidence.
#[derive(Debug, Clone)]
pub struct TypeEvidence {
    pub relation: RelationType,
    /// Pairs assigned to it.
    pub members: Vec<(String, String)>,
    /// Mean agreement between a member's offset and the type's rotation.
    pub coherence: f64,
}

/// A vocabulary's relation types, and the operations over them.
#[derive(Debug, Clone)]
pub struct KindTable {
    pub(super) rotations: Vec<Vec<f64>>,
    pub(super) evidence: Vec<TypeEvidence>,
}

/// One point on the saturation curve.
#[derive(Debug, Clone, Copy)]
pub struct SaturationPoint {
    pub k: usize,
    /// Mean agreement of *held-out* offsets to their nearest fitted centroid.
    pub held_out: f64,
    /// The same, on pairs whose fillers have been permuted. The floor.
    pub shuffled: f64,
    /// A *second*, independently permuted null, so the distance between the two
    /// nulls gives the scale of variation this whole procedure has at this `k`.
    ///
    /// Without it a gain has no unit. The first run of this sweep reported a
    /// best gain of 0.0116 and a verdict of "the bound holds" — on a curve whose
    /// two arms differed by less than the arms differ from each other.
    pub shuffled_b: f64,
}
