# phient — what is left to do

Paths are under `src/`. Diagrams: [ARCHITECTURE.md](ARCHITECTURE.md).

## The lineup and the learner (docs/203)

- [ ] lineup: the learner does not beat the smoothed trigram on held-out sentences (21.3% top-1, perplexity 101.2; trigram 24.8%, 84.7).
- [ ] lineup: train it on more text than one book; try a wider block and more channels with more silence; full softmax for the last epochs.
- [x] lineup: phiano's `talk` and workshop read the learner now (2026-10-28).

## Correctness

- [ ] code: in `bcpl::learn` the `"$("` and `"$)"` arms can never match, because `prose::tokens` splits `$(` into `$` and `(`; depth moves only on `(` and `)` (code/bcpl/learn.rs:34, :76).
- [ ] operator: `MirrorWave::filter` caches `cols` from the first `values` and reuses them for later ones; `FnoWave::back` does the same with `backs` (operator/mirror.rs:22, operator/wave.rs:81).
- [ ] prose: the family test `assert!(k.unsure(2).is_empty() || !k.unsure(2).is_empty())` is always true, so `unsure` is untested (prose/family.rs:76).

## The match-not-else-if rule

- [ ] scribe: `mold::of` is an if / else-if chain (scribe/mold.rs:30-38).
- [ ] code: `role::of` is an if / else-if chain (code/bcpl/role.rs:14-24).
- [ ] prose: the cost in `best_division` is an if / else-if chain (prose/strand/code.rs:63).
- [ ] relate: `Table::add` is an if-let / else-if chain (relate/table.rs:33-37).
- [ ] study: the verdict in `Arena::table` is an if / else-if chain with its own strings, beside `Study::verdict` (study/arena.rs:94).

## Tests and duplication

- [ ] prose: teach.rs, telling.rs, going.rs and unknown.rs have no tests; `chapter_file` and `so_far` repeat the chapter listing (prose/teach.rs:23, :36).
- [ ] code: `read_dir` and `read_files_from` repeat the same directory walk (code/files.rs:9, :33).
- [ ] operator: a dead glob import `#[allow(unused_imports)] use crate::relate::*;` (operator/model.rs:1).

## Stale docs

- [ ] relate: the doc about the `scope:` prefix sits on `impl Tally` (relate/levels.rs:24).
- [ ] relate: the module doc says phiano names `RelationModel`, which now lives in `phient::operator` (relate/mod.rs:12).
- [ ] lib.rs: says phiano "re-exports these three modules" while its table lists six (lib.rs:28).
- [ ] study: the arena's lineup (`lineup`) now includes the error-driven learner (docs/203); the README's model list predates it.

## Moved in (2026-10-28)

- [x] train: phiano's training cluster (trainer, topdown, chunker, conception, sources, roles, core::grounding) is `train`, its 18 constants in `train::constants`; metrics (with `nonlinear`, `data` and the context kernel) is `metrics`; eval, memory, and the rest of `net` (reader, lm) came with them. phiano reads all of it from here.
- [x] train: `kinds` came here too (docs/phiano/distribution), so `RoleDiscovery::offset`, `agreement` and `centroid` are `pub(crate)` again.
- [x] train: `kinds`, `synthetic`, `curriculum`, `correction`, `envision`, `synthesis` and `lifelong` moved in from phiano; `eval::oscillator` too.
- [x] generate, reasoning: phiano's engine (phase B, docs/phiano/engine): `generate` with `learn`, `compose` and `constants` inside it, and `reasoning` with its `Chain`.
- [x] reasoning left again, as its own crate: `philog`, the logic crate (2026-10-28).
- [ ] Calculations belong in phical (the ruling of 2026-10-28): phient orchestrates and calls them. Surveyed: about 150 calculations, nine batches; the plan is [docs/phient/calculations](../../docs/phient/calculations/plan.md). Batch 1 done (2026-10-28).
- [x] eval: `verdict.rs` capped the novelty angle at π instead of folding it; folded (`phical::circle::distance`), at the user's "Yes, fold it". 41 of 1,993 guide sentences on the Austen lexicon moved (docs/206).
- [ ] generate: phiano's `lib.rs` keeps `pub use phient::generate::compose;` only for the read-only example `flow`.
- [ ] generate: the only caller of phiecs's `Attention::next_words` is the dead `Generator::attention_pick` (generate/steer.rs:13-25); phiecs's `attention/cross.rs` is never declared.
- [ ] train: phiano's `lib.rs` keeps `pub use phient::train::trainer;` only for the read-only example `meaning` (`phiano::trainer::Trainer`).
- [ ] train: the sources fetch definitions over HTTP (`train/sources/api`), so phient now depends on reqwest; a fetching source belongs with the apps if phient is to stay offline.
