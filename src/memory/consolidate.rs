#[allow(unused_imports)]
use super::*;

impl Memo {

    /// Merges near-duplicate interactions, keeping a count.
    ///
    /// Real memory consolidates. Without this the log grows without bound and
    /// repeats the same experience as many separate ones, which distorts both
    /// recall and the novelty measure. Returns the number of entries merged.
    pub fn consolidate(&mut self, wave_eps: f64) -> usize {
        if self.entries.len() < 2 {
            return 0;
        }
        let mut kept: Vec<ContextWaveEntry> = Vec::with_capacity(self.entries.len());
        let mut merged = 0usize;

        for e in std::mem::take(&mut self.entries) {
            let dup = kept.iter_mut().find(|k| {
                k.text_hash == e.text_hash || {
                    let dx = k.superposition_wave.0 - e.superposition_wave.0;
                    let dy = k.superposition_wave.1 - e.superposition_wave.1;
                    dx.hypot(dy) < wave_eps
                }
            });
            match dup {
                Some(k) => {
                    k.count = k.count.saturating_add(e.count.max(1));
                    k.timestamp_ms = k.timestamp_ms.max(e.timestamp_ms);
                    merged += 1;
                }
                None => kept.push(e),
            }
        }

        self.entries = kept;
        self.reindex();
        merged
    }

    /// Classifies input text into one of 16 memory layers by surface shape.
    ///
    /// Retained for callers without a facet; prefer
    /// [`Memo::classify_layer_semantic`].
    pub(super) fn classify_layer(text: &str) -> usize {
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();

        let avg_len = words
            .iter()
            .map(|w| w.len())
            .sum::<usize>()
            .checked_div(word_count)
            .unwrap_or(0);

        let band = match word_count {
            0..=3 => MemoryBand::Surface,
            4..=8 => MemoryBand::Pattern,
            9..=16 => MemoryBand::Semantic,
            _ => MemoryBand::Deep,
        };

        let sub_layer = match avg_len {
            0..=4 => 0,
            5..=6 => 1,
            7..=8 => 2,
            _ => 3,
        };

        (band.base_layer() + sub_layer).min(15)
    }

    /// Returns the total number of recorded interactions.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the memory log contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of entries in a specific memory layer.
    pub fn layer_count(&self, layer: usize) -> usize {
        self.layers
            .get(layer)
            .map(|l| l.len())
            .unwrap_or(0)
    }

    /// Saves the memory log to a binary file, atomically (write then rename).
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let tmp = format!("{}.tmp", path);
        {
            let file = File::create(&tmp)?;
            let writer = BufWriter::new(file);
            bincode::serialize_into(writer, self)
                .map_err(Error::other)?;
        }
        std::fs::rename(&tmp, path)
    }

    /// Loads a memory log from a binary file using bincode deserialization.
    ///
    /// Older files were written before `count` was added to the entry (and
    /// before `layers` held indices). bincode is positional, so
    /// `#[serde(default)]` does not cover a missing field: reading such a file
    /// with the current layout misaligns after the first entry and asks for an
    /// exabyte of string. Allocation is bounded to the file size, and on
    /// failure the pre-`count` layout is tried.
    pub fn load_from_file(path: &str) -> Result<Self> {
        use bincode::Options;
        let bytes = std::fs::read(path)?;
        let opts = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_limit(bytes.len() as u64 * 4 + 1024);
        let mut memo: Self = match opts.deserialize::<Self>(&bytes) {
            Ok(m) => m,
            Err(_) => {
                #[derive(Deserialize)]
                struct EntryV0 {
                    timestamp_ms: u64,
                    superposition_wave: (f64, f64),
                    text_hash: u64,
                    text: String,
                    layer: usize,
                }
                #[derive(Deserialize)]
                struct MemoV0 {
                    entries: Vec<EntryV0>,
                }
                let v0: MemoV0 = opts
                    .deserialize(&bytes)
                    .map_err(Error::other)?;
                eprintln!("  [migrate] memory log written before `count`; {} entries", v0.entries.len());
                Self {
                    entries: v0
                        .entries
                        .into_iter()
                        .map(|e| ContextWaveEntry {
                            timestamp_ms: e.timestamp_ms,
                            superposition_wave: e.superposition_wave,
                            text_hash: e.text_hash,
                            text: e.text,
                            layer: e.layer,
                            count: 1,
                        })
                        .collect(),
                    layers: Default::default(),
                }
            }
        };
        if memo.layers.iter().all(|l| l.is_empty()) && !memo.entries.is_empty() {
            memo.reindex();
        }
        Ok(memo)
    }

    /// Computes an FNV-1a 64-bit hash of the input text.
    pub(super) fn fnv1a_hash(text: &str) -> u64 {
        phistd::fnv1a(text)
    }
}
