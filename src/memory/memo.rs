#[allow(unused_imports)]
use super::*;

impl Memo {
    /// Creates an empty memory log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a new interaction in the memory log.
    pub fn record(&mut self, wave: (f64, f64), text: &str) {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let text_hash = Self::fnv1a_hash(text);
        let layer = Self::classify_layer(text);

        let entry = ContextWaveEntry {
            timestamp_ms,
            superposition_wave: wave,
            text_hash,
            text: text.to_string(),
            layer,
            count: 1,
        };

        self.layers[layer].push(self.entries.len());
        self.entries.push(entry);
    }

    /// The `k` past interactions whose waves are closest to `query`.
    ///
    /// The memory log has always recorded every interaction with its wave, its
    /// timestamp and its text — and nothing ever read it back during inference.
    /// This is the retrieval half.
    pub fn recall(&self, query: (f64, f64), k: usize) -> Vec<&ContextWaveEntry> {
        let mut scored: Vec<(&ContextWaveEntry, f64)> = self
            .entries
            .iter()
            .map(|e| {
                let dx = query.0 - e.superposition_wave.0;
                let dy = query.1 - e.superposition_wave.1;
                (e, dx.hypot(dy))
            })
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(e, _)| e).collect()
    }

    /// Recency-weighted recall: distance is divided by an exponential decay in
    /// age, so an older memory must be substantially closer to outrank a recent
    /// one.
    pub fn recall_weighted(
        &self,
        query: (f64, f64),
        k: usize,
        half_life_ms: f64,
    ) -> Vec<&ContextWaveEntry> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);

        let mut scored: Vec<(&ContextWaveEntry, f64)> = self
            .entries
            .iter()
            .map(|e| {
                let dx = query.0 - e.superposition_wave.0;
                let dy = query.1 - e.superposition_wave.1;
                let age = (now - e.timestamp_ms as f64).max(0.0);
                let recency = 0.5f64.powf(age / half_life_ms.max(1.0));
                (e, dx.hypot(dy) / recency.max(1e-6))
            })
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(e, _)| e).collect()
    }

    /// Novelty of a wave measured against *experience* rather than geometry.
    ///
    /// Distance to the nearest thing ever processed, squashed to [0, 1]. This
    /// does not degrade as the lexicon grows, and it cannot be driven to a
    /// constant by phase collapse the way centroid-distance novelty can.
    pub fn novelty(&self, query: (f64, f64)) -> f64 {
        if self.entries.is_empty() {
            return 1.0;
        }
        let nearest = self
            .entries
            .iter()
            .map(|e| {
                let dx = query.0 - e.superposition_wave.0;
                let dy = query.1 - e.superposition_wave.1;
                dx.hypot(dy)
            })
            .fold(f64::MAX, f64::min);
        1.0 - (-nearest).exp()
    }

    /// Rebuilds the layer index from `entries`. Used after loading a file
    /// written before layers held indices.
    pub fn reindex(&mut self) {
        for l in self.layers.iter_mut() {
            l.clear();
        }
        for (i, e) in self.entries.iter().enumerate() {
            let layer = e.layer.min(MEMORY_LAYERS - 1);
            self.layers[layer].push(i);
        }
    }

    /// Records an interaction, classifying it by *meaning* rather than shape.
    ///
    /// See [`Memo::classify_layer_semantic`].
    pub fn record_grounded(&mut self, facet: &phiecs::phasor::facet::Facet, wave: (f64, f64), text: &str) {
        let layer = Self::classify_layer_semantic(facet, text);
        self.record_at(wave, text, layer);
    }

    /// Records at an explicit layer.
    pub fn record_at(&mut self, wave: (f64, f64), text: &str, layer: usize) {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let layer = layer.min(MEMORY_LAYERS - 1);
        let entry = ContextWaveEntry {
            timestamp_ms,
            superposition_wave: wave,
            text_hash: Self::fnv1a_hash(text),
            text: text.to_string(),
            layer,
            count: 1,
        };
        self.layers[layer].push(self.entries.len());
        self.entries.push(entry);
    }

    /// Classifies text by conceptual breadth and coherence.
    ///
    /// The band names — Surface, Pattern, Semantic, Deep — promised meaning
    /// that word count and average word length could not deliver: "Deep" meant
    /// *long sentence with long words*, so a profound three-word statement
    /// filed as Surface while a rambling list of polysyllables filed as Deep.
    ///
    /// Band is now how many distinct phase sectors the content words span
    /// (conceptual breadth) and sub-layer is how tightly they cohere.
    pub fn classify_layer_semantic(facet: &phiecs::phasor::facet::Facet, text: &str) -> usize {
        let toks = phiton::p64::words::Tokenizer::content_words(text);
        if toks.is_empty() {
            return 0;
        }

        let mut sectors = std::collections::HashSet::new();
        for t in &toks {
            if let Some(s) = phiecs::phasor::wave::Wave::word_sector(facet, t) {
                sectors.insert(s);
            }
        }

        let known = toks.iter().filter(|t| facet.contains_word(t)).count();
        let z = phiecs::phasor::wave::Wave::sentence(facet, &toks);
        let coherence = match known {
            0 => 0.0,
            n => (z.norm() / n as f64).clamp(0.0, 1.0),
        };

        let band = match sectors.len() {
            0..=1 => 0,  // single-topic
            2..=3 => 4,  // linked topics
            4..=6 => 8,  // multi-domain
            _ => 12,     // broadly integrative
        };
        let sub = ((coherence * 3.99) as usize).min(3);
        (band + sub).min(MEMORY_LAYERS - 1)
    }
}
