#[allow(unused_imports)]
use super::*;

/// A relation type. The rotation is derived from the name, so it is stable
/// across processes, corpora and runs without being stored anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    /// *X counts as Y*: the kind X belongs to. `genus(money, currency)`.
    Genus,
    /// What X is for. `function(money, transaction)`.
    Function,
    /// What X is made of or realised as. `form(money, paper)`.
    Form,
    /// X is a particular of Y. `instance(bessie, cow)`.
    Instance,
    /// Where or when X holds — Searle's *context C*.
    Context,
    /// A defining property of X. `property(gold, heavy)`.
    Property,
}

/// One constitutive rule: `head` counts as `filler` under `role`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub head: String,
    pub role: Role,
    pub filler: String,
}

/// The relations a vocabulary holds, and the composition they induce.
#[derive(Debug, Clone, Default)]
pub struct RuleSet {
    pub(super) rules: Vec<Rule>,
    pub(super) by_head: HashMap<String, Vec<usize>>,
}

/// Binding and unbinding by role.
pub struct RoleTable;

/// A relation type **discovered from use**, not declared.
///
/// [`Role`] is six variants someone chose, and the extractor that fills them is
/// a list of regexes hardcoding the phrasings someone thought of. That is the
/// thing CLU's abstract data types were an argument against: a type is defined
/// by the operations its instances share, not by a label supplied from outside
/// and a hand-written list of the cases you happened to anticipate.
///
/// The manifold already has the invariant. If `genus` is a real relation, then
/// the per-channel offset `θ(filler) − θ(head)` is *approximately the same
/// vector* for `(dog, animal)` and for `(oak, tree)` — that shared offset is
/// what makes it one relation rather than two coincidences. So the types can be
/// recovered by clustering offsets, and nothing needs to be named in advance.
///
/// The test is not whether the clusters look tidy. It is whether they line up
/// with relation labels the clustering never saw.
#[derive(Debug, Clone)]
pub struct DiscoveredRole {
    /// The shared offset, one angle per channel — the role's rotation.
    pub rotation: Vec<f64>,
    /// Pairs assigned to this cluster, as `(head, filler)`.
    pub members: Vec<(String, String)>,
    /// Mean agreement between a member's offset and the centroid. Near 1.0 is a
    /// tight relation; near 0 is a bag of unrelated pairs sharing a bin.
    pub coherence: f64,
}

/// Recovers relation types by clustering phase offsets.
pub struct RoleDiscovery;

/// Training the manifold to hold relations consistently.
///
/// Coherence — how well pairs in one relation agree on a shared offset — sits at
/// **0.27** on the trained manifold, against >0.9 on relations planted by
/// construction. Nothing in the training objective optimises it or anything like
/// it: ranking optimises next-word retrieval and gets 0.27 as a by-product.
///
/// That is the same fact the analogy score reports indirectly. `a:b::c:d`
/// assumes `b − a ≈ d − c`, so analogy accuracy *is* offset consistency, seen
/// through a probe set. Optimising coherence directly needs no probe set, no
/// labels, and no held-out families.
pub struct Coherence;

/// A relation type named by the language rather than by an enum.
///
/// [`Role`] is six variants someone chose, and the measurements showed what that
/// costs: 94% of extracted relations landed in one `genus` bucket, and forcing
/// 6,833 heterogeneous pairs to share an offset trained coherence 23× on its own
/// metric while moving nothing held out. One over-broad bucket is not a
/// relation.
///
/// English already marks its relation types, and marks them explicitly:
///
/// ```text
/// an instrument for driving nails   →  for(hammer, nails)
/// a vessel used in cooking          →  in(pot, cooking)
/// covered with hair                 →  with(mammal, hair)
/// a disease of the lungs            →  of(pneumonia, lungs)
/// a blow struck by the hand         →  by(slap, hand)
/// ```
///
/// The preposition *is* the role. Nothing is named in advance, nothing is
/// enumerated, and the inventory is whatever the corpus uses — which is the CLU
/// position applied to relations: the type comes from how the instances are
/// used, not from a declaration.
///
/// It also reaches the one role that was never extracted once. *Used **in**
/// cooking* is Searle's context C, and no regex for `function` was ever going to
/// find it.
#[derive(Debug, Clone, Default)]
pub struct LexicalRule {
    pub(super) triples: Vec<(String, String, String)>,
    pub(super) by_role: HashMap<String, Vec<usize>>,
    pub(super) by_head: HashMap<String, Vec<usize>>,
}

/// Prepositions that mark a relation between a head and a following noun.
///
/// Deliberately only the relational ones. `the`, `a` and `and` mark no
/// relation; `of`, `for`, `with` and the rest each mark a different one, and
/// which is which is not decided here — the clustering decides whether two
/// prepositions turn out to name the same relation.
pub const RELATIONAL_PREPOSITIONS: [&str; 12] = [
    "of", "for", "in", "with", "by", "to", "from", "on", "at", "into", "upon", "against",
];
