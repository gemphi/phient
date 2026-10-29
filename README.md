# phient — the brain of Phient systems

In Phient's world, a word (w) is a seed: what it relates to, under what conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a word is 32–64 relations, each a complex phasor on a closed circle; meaning is a side effect of pairs of words moving under a condition, and they move by the error of what was expected — toward what came, away from what did not.

Phient is to Phient systems what ollama is to its models: the brain that knows what it has — a phiano in phinet, a phiora database — runs it, and measures it honestly. Formerly the crate phiplm; the name changed when the role became clear, the content is its lineage:

| module | what |
|---|---|
| `study` | the protocol: every next word predicted BEFORE it is told, sliced curves, `beats_floor`; and the **arena** — every model one table, the inventory the brain routes over |
| `prose` | text as a tree of knowing: `Knowing` (learn, `learn_new`, `next`), curiosity (interest points at the unknown), families discovered not given, the quest that asks a teacher |
| `relate` | the ledgers: `Tally`, levels, the `ScopeTree` cast, kin — counts as the contrast line and the bookkeeping, never the learning |
| `eval` | the Evaluator: a text's semantic quality as coherence, novelty and resonance, weighted (`Evaluator::overall`); it was phiano's `eval` |
| `memory` | the 16-layer interaction log (`Memo`): every interaction with its wave, by depth, consolidated by count; it was phiano's `memory` |
| `metrics` | the measures: the perplexity harness, benchmarks, baselines, generalisation, forgetting, detection, with the non-linear readout and the context kernel; it was phiano's `metrics` |
| `net` | the lexicon read as a function space: a facet's words as functions on the circle, joint training with write-back, nested role binding with the operator as cleanup, a word's sector against the lexicon mean (`space::sector`, from phiano's persona fingerprint) (it was phiano's `phinet`) |
| `train` | learning the lexicon (with, from phiano since, the relation kinds, synthetic data, curriculum, correction journal, unknown words, program synthesis and the lifelong coordinator): the trainer, the downward path, definitions as compositions, the grounding, the roles, the chunk store and the sources; it was phiano's `trainer`, `topdown`, `chunker`, `conception`, `sources`, `roles` and `core::grounding` |
| `generate` | the generation engine: the decoders, the beam, the context wave, the operator guide, the persona prior (a sector histogram), with `learn` (the generator's log-linear scorer and its tuner) and `compose` (better/worse composition, the river flow) inside it; it was phiano's `generate`, `learn` and `compose` |
| `operator` | the relation operator read as waves (FNO forward, mirror backward) |
| `code` | Rust and BCPL read by their own grammar into rows |
| `baseline` | interpolated Kneser-Ney: the number to beat |

The single-layer shape (docs/197 in [phiano](https://github.com/gemphi/phiano)): one phasor world as the base, a channel per feature, actions as told rows, and this crate as the orchestrator; an outside oracle (ollama) is its overflow, not its mind. The serving loop (`phient serve`) is the named next rung, not a shipped claim.

Part of the [phinet](https://github.com/gemphi/phinet) workspace. MIT.
