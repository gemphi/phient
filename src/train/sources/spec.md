# Sources Module Specification

## Overview

The sources module provides dictionary data for bootstrapping the facet.
All sources implement the `DictionarySource` trait.

## Trait

```rust
pub trait DictionarySource {
    fn fetch_all(&self) -> Vec<(String, String)>;
    fn fetch_definitions(&self, word: &str) -> Vec<String>;
}
```

## Implementations

| Source       | File           | Description |
|--------------|----------------|-------------|
| ApiSource    | api.rs         | Fetches definitions from a dictionary API |
| JsonDictionarySource | json.rs | Loads from a JSON `{word: definition}` file |
| LocalSource  | local.rs       | Reads from a local `word: definition` text file |
| WiktionarySource | wiktionary.rs | Parses Wiktionary JSON/JSONL dumps |

## Ingester

The `Ingester` struct provides a static `ingest` method that trains the
facet from any `DictionarySource` for a specified number of epochs.

## Data Flow

```
DictionarySource
    │
    ├─► fetch_all() ──► Vec<(word, definition)>
    │                        │
    │                        ▼
    │                   Ingester::ingest()
    │                        │
    │                        ├─► Trainer::train_definition() × epochs
    │                        │
    │                        ▼
    │                   Facet (updated lexicon)
    │
    └─► fetch_definitions(word) ──► Used by Define command
```
