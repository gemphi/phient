mod source;
mod fetch;
#[allow(unused_imports)]
use source::*;
#[allow(unused_imports)]
use fetch::*;

use crate::train::sources::local::LocalSource;
use crate::train::sources::DictionarySource;
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

/// Free Dictionary API & Multi-Source Auto-Feed Engine.
///
/// Fetches rich word definitions on-demand from:
/// 1. Free Dictionary API (api.dictionaryapi.dev)
/// 2. Morphological Lemmatization (past tense -ed, -ing, plurals -s, adverbs -ly)
/// 3. Wikipedia REST API Summary for proper nouns and encyclopedia terms
/// 4. Auto-trains the facet in real-time.
pub struct ApiSource {
    pub cache_path: String,
    pub timeout_secs: u64,
}

#[derive(Deserialize)]
struct ApiEntry {
    pub(super) word: String,
    pub(super) phonetic: Option<String>,
    pub(super) meanings: Vec<ApiMeaning>,
}

#[derive(Deserialize)]
struct ApiMeaning {
    #[serde(rename = "partOfSpeech")]
    pub(super) part_of_speech: String,
    pub(super) definitions: Vec<ApiDefinition>,
}

#[derive(Deserialize)]
struct ApiDefinition {
    pub(super) definition: String,
    pub(super) example: Option<String>,
}
