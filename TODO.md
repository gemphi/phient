# phient — what is left to do

Paths are under `src/`. Diagrams: [ARCHITECTURE.md](ARCHITECTURE.md).

## The lineup and the learner (docs/203)

- [ ] lineup: the learner does not beat the smoothed trigram on held-out sentences (21.3% top-1, perplexity 101.2; trigram 24.8%, 84.7).
- [ ] lineup: train it on more text than one book; try a wider block and more channels with more silence; full softmax for the last epochs.
- [ ] lineup: phiano's `talk` and workshop still generate from counts; they should read the learner.

## Correctness

- [ ] code: in `bcpl::learn` the `"$("` and `"$)"` arms can never match, because `prose::tokens` splits `$(` into `$` and `(`; depth moves only on `(` and `)` (code/bcpl/learn.rs:34, :76).
- [ ] operator: `MirrorWave::filter` caches `cols` from the first `values` and reuses them for later ones; `FnoWave::back` does the same with `backs` (operator/mirror.rs:22, operator/wave.rs:81).
- [ ] prose: the family test `assert!(k.unsure(2).is_empty() || !k.unsure(2).is_empty())` is always true, so `unsure` is untested (prose/family.rs:76).

## The match-not-else-if rule

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
