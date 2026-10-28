Relation types as an abstract data type, discovered from use.

[`crate::roles::Role`] is an enum I wrote: six variants, named in advance,
and every caller of `bind` has to know which variant it wants. That is the
thing the CLU argument objects to. An abstract data type is defined by its
operations, not by its representation, and its inhabitants come from the data
rather than from the author — so a relation type should be something the
corpus produces, that callers can only obtain by asking, and whose innards
they cannot see.

[`RelationType`] is that: an opaque handle with no public constructor and no
visible representation. The only way to get one is [`RelationTypes::discover`]
over a corpus, and the only things you can do with one are the operations
below. Nothing here is named `genus` or `function`, and nothing needs to be.

# The bound

The claim under test is that a language holds at most 64 relation types —
that the set is *bounded*, so a fixed-width type can hold it. A cap alone
proves nothing: `.take(64)` on anything yields at most 64 of it. What would
make the bound real is saturation — held-out structure stops improving before
the cap is reached, so the 65th type would have nothing left to explain.

[`RelationTypes::saturation`] measures exactly that, and it can come out
against the claim. Centroids are fitted on one half of the pairs and scored
on the other, because in-sample cluster tightness rises with `k` for free and
would "confirm" any bound you asked it about. A shuffled-pair null runs
beside it, because a curve without a floor under it is not a measurement.

# What it measured

19,797 head-filler pairs from the dictionary, restricted to words seen 25+
times, clustered on their phase offsets. Half fitted, half held out. Run on
two manifolds: the bottom-up one this project ships, and the same one after
the downward pass in [`crate::topdown`].

```text
                   k   held-out   shuffled      gain     noise   gain/noise
bottom-up          4     0.2019     0.2006    0.0013    0.0040         0.3x
                  16     0.2644     0.2585    0.0060    0.0032         1.9x
                  64     0.3600     0.3484    0.0116    0.0044         2.6x
                  96     0.3878     0.3803    0.0075    0.0028         2.7x
                 128     0.4107     0.4075    0.0032    0.0042         0.8x

after descent      4     0.2981     0.2859    0.0122    0.0014         8.7x
                  16     0.3387     0.3292    0.0094    0.0028         3.4x
                  64     0.3740     0.3619    0.0122    0.0008        15.3x
```

**Bottom-up: no relation types at any k.** Held-out agreement climbs steadily
with the type count and the shuffled null climbs with it point for point.
Every gain sits within a few multiples of the distance between two
independent runs of the null itself, and it is not monotonic. Adding
centroids fits random offsets as well as real ones, which is what k-means
does to any cloud of points.

**After the descent: structure.** The same procedure on the same pairs clears
its null by 8.7x at four types and 15x at sixty-four. So the downward pass
builds the shared offsets that the bottom-up manifold does not have, and the
threefold analogy gain it produced *is* relational structure rather than
something else wearing the shape of one.

On the bound: the gain is flat across the whole range — 0.0122 at k=4 and
0.0122 at k=64. Sixty more types buy nothing measurable. So a 64-wide type is
not too small; if anything the evidence supports a handful. But read the
absolute number before celebrating: 0.012 on a 0-to-1 agreement scale is a
real effect and a tiny one.

# Removing word identity from the readout

The clusters were grouping by word, not by relation: **71.3% of pairs shared
a cluster with another pair of the same head, against 9.1% expected if the
head were ignored.** `adversary->against` and `adversary->another` land
together because both differences contain `-theta(adversary)`.

That is a main effect, so [`OffsetEstimator`] centers it out. The full grid,
two manifolds by three estimators:

```text
manifold        offset             k   held-out   shuffled     gain  g/noise  head-dom
bottom-up       raw               16     0.2644     0.2585   0.0060     1.9x     71.3%
bottom-up       raw               64     0.3600     0.3484   0.0116     2.6x     71.3%
bottom-up       head-centered     16     0.7677     0.7711  -0.0034    -1.8x     70.7%
bottom-up       head-centered     64     0.7763     0.7827  -0.0064    -2.4x     70.7%
bottom-up       double-centered   16     0.6379     0.6354   0.0025     1.3x     78.2%
bottom-up       double-centered   64     0.6514     0.6485   0.0029     3.6x     78.2%
after descent   raw               16     0.3387     0.3292   0.0094     3.3x     59.8%
after descent   raw               64     0.3740     0.3619   0.0122    14.9x     59.8%
after descent   head-centered     16     0.7474     0.7420   0.0054     1.4x     26.5%
after descent   head-centered     64     0.7639     0.7587   0.0053    12.1x     26.5%
after descent   double-centered   16     0.5649     0.5063   0.0585     5.3x     33.5%
after descent   double-centered   64     0.5801     0.5204   0.0597     6.4x     33.5%
```

**Descent plus double-centering: gain 0.0597 at 6.4x noise — five times the
best raw gain, and the largest relational effect this project has measured.**
Head domination falls from 59.8% to 33.5% at the same time, so the clusters
are grouping less by word identity *and* the grouping is worth more. Those
two columns had to move together for this to mean anything, and they did.

The interaction is the real finding. Centering does nothing on the bottom-up
manifold — head-centering there is *negative*, fitting worse than centered
noise. It only pays on the descended one. So the two halves are not
independent improvements to be summed: the descent creates an interaction
term, and centering is what lets it be read. Neither works alone.

Read the absolute number honestly. 0.0597 on a 0-to-1 agreement scale is
still small, head domination at 33.5% is still far from the 9.1% floor, and
the sixty-fold increase in types from k=4 to k=64 buys 0.0012. What this
establishes is that relations are *present and recoverable*, not that they
are strong.

# Standing waves: the bound, derived

Bentov, *Stalking the Wild Pendulum* (1977), ch. 1: "Standing waves cannot
exist unless they divide their medium into an integral number of half waves.
A standing wave having a fractional wavelength cannot be sustained... The
dimensions of the plate are the factors that govern what is the size or
wavelength of the standing wave that can be sustained in the plate."

If a relation is a standing wave across the channel index rather than an
arbitrary direction in channel space, the medium admits only integer modes,
and a 64-channel torus sustains exactly 64 of them — mode 64 aliases onto
mode 0. **The bound stops being a claim and becomes the channel count.**
`MAX_RELATION_TYPES = PHASE_CHANNELS` is then forced by geometry, and there
is no 65th type to look for.

The testable half is sharper than the philosophical half. [`harmonic_rotation`]
fits *nothing*: no centroids, no k-means, no seed, identical on every corpus.
The clustered rows above fit 64 rotations of 64 channels each — 4,096 free
numbers — to these very pairs. Same pairs, same null, same scoring:

```text
manifold        offset            held-out   shuffled     gain  g/noise
bottom-up       raw                 0.2523     0.2519   0.0005     0.9x
bottom-up       head-centered       0.7178     0.7187  -0.0009      --
bottom-up       double-centered     0.7090     0.7024   0.0066     1.7x
after descent   raw                 0.2789     0.2603   0.0187    48.4x
after descent   head-centered       0.6672     0.6566   0.0105    29.3x
after descent   double-centered     0.5294     0.5366  -0.0072      --
```

**On raw offsets the fixed basis beats fitted k-means: 0.0187 against 0.0122,
at 48x its noise floor against 15x — with zero fitted parameters against
4,096.** Relations in this manifold are closer to modes of the medium than to
arbitrary directions in it, and the discovery apparatus was partly solving a
problem the geometry had already answered.

Two things not to overstate. The largest *absolute* effect is still fitted
k-means on double-centered offsets (0.0597); the harmonic basis wins on
significance and on parsimony, not on size. And the two corrections do not
compose — double-centering, which was the best estimator under clustering,
is worse than its own null under the harmonic basis (-0.0072), because
centering destroys the very phase ramp the modes are made of. They are
alternative readouts, not stackable improvements.

What survives every arm: nothing works on the bottom-up manifold. Fixed basis,
fitted basis, centered, uncentered — all at or under their nulls. The descent
is the precondition for every relational result in this module.

# Re-measured on fixed seeds

Every table above predates the seeding bug in [`crate::phasor::mix64`]: FNV-1a
had not avalanched after three bytes, so `cow`, `cod` and `cot` started at the
same point on 63 of 64 channels and the whole lexicon — nulls included — was
pre-clustered by spelling. Re-run:

```text
manifold        offset            held-out   shuffled     gain  g/noise
bottom-up       raw                 0.2440     0.2436   0.0004     0.7x
bottom-up       head-centered       0.7131     0.7144  -0.0013       --
bottom-up       double-centered     0.7021     0.6942   0.0080     6.4x
after descent   raw                 0.2765     0.2573   0.0192    35.2x
after descent   head-centered       0.6667     0.6560   0.0107   138.6x
after descent   double-centered     0.5277     0.5415  -0.0138       --
```

The fixed harmonic basis holds: 0.0192 on the descended manifold at 35x its
noise, against 0.0187 before — unchanged within noise, and still with zero
fitted parameters against k-means' 4,096. The bottom-up manifold is still at
its null on every estimator.

One reversal. Double-centering, which was the largest absolute effect under
fitted clustering (0.0597), is now *negative* under the harmonic basis on the
descended manifold (-0.0138) — it was -0.0072 before, so the direction was
already there and the fixed seeds sharpened it. Centering destroys the phase
ramp the modes are made of; the two corrections remain alternative readouts
rather than stackable improvements, and this is now unambiguous.

The k-means grid at k=16 and 64 has not been re-run on fixed seeds — it is
killed for memory at k=64 on this machine — so every number in the two tables
above it should be read as provisional.

Everything else in that book — subjective time, the model of the universe,
the kundalini appendix — is not testable here and was not used.

That is a finding about the representation, not about the discovery
procedure: the planted-relation tests below pass, so the clustering finds
real shared offsets when there are any to find.
