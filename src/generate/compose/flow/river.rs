#[allow(unused_imports)]
use super::*;

impl RiverFlow {
    /// Traces a river flow from a prompt, starting at a specific sector.
    /// Computes its own ray cast - use generate_variations for batch efficiency.
    pub fn trace(
        facet: &Facet,
        prompt: &str,
        forced_sector: Option<u16>,
        depth: usize,
    ) -> Self {
        let tokens = Tokenizer::tokenize(prompt);
        let wave = Wave::sentence(facet, &tokens);
        let resonant_pool = Wave::ray_cast(facet, wave, config::RAY_CAST_POOL_SIZE);
        let sector_map: std::collections::HashMap<String, u16> = resonant_pool
            .iter()
            .filter_map(|(w, _)| Wave::word_sector(facet, w).map(|s| (w.clone(), s)))
            .collect();
        let fallback_map = Self::build_fallback_map(facet);
        Self::trace_with_pool(facet, prompt, forced_sector, depth, &resonant_pool, &sector_map, &fallback_map)
    }

    /// Traces a river flow from a prompt, starting at a specific sector.
    /// Uses a pre-computed resonant pool, sector map, and fallback map for efficiency.
    pub(super) fn trace_with_pool(
        facet: &Facet,
        prompt: &str,
        forced_sector: Option<u16>,
        depth: usize,
        resonant_pool: &[(String, f64)],
        sector_map: &std::collections::HashMap<String, u16>,
        fallback_map: &std::collections::HashMap<u16, Vec<String>>,
    ) -> Self {
        let tokens = Tokenizer::tokenize(prompt);
        let wave = Wave::sentence(facet, &tokens);

        let source_sector = forced_sector.unwrap_or_else(|| Wave::wave_sector(wave));
        let tension_sector = Wave::opposite_sector(source_sector);

        let path = Self::build_path(source_sector, depth);

        let mut banks = Vec::new();
        for &sector in &path {
            let mut words: Vec<String> = Vec::new();

            let mut in_sector: Vec<(String, f64)> = resonant_pool
                .iter()
                .filter(|(w, _)| sector_map.get(w) == Some(&sector))
                .map(|(w, delta)| (w.clone(), *delta))
                .collect();
            in_sector.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            words.extend(in_sector.into_iter().take(4).map(|(w, _)| w));

            let n = phiecs::phasor::wave::Wave::sector_count();
            let prev_s = if sector == 0 { n - 1 } else { sector - 1 };
            let next_s = (sector + 1) % n;
            let mut adj: Vec<(String, f64)> = resonant_pool
                .iter()
                .filter(|(w, _)| {
                    sector_map.get(w).map(|s| *s == prev_s || *s == next_s).unwrap_or(false)
                })
                .map(|(w, delta)| (w.clone(), *delta))
                .collect();
            adj.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            words.extend(adj.into_iter().take(2).map(|(w, _)| w));

            if words.is_empty() {
                if let Some(fb) = fallback_map.get(&sector) {
                    words.extend(fb.iter().take(4).cloned());
                }
            }

            if sector == tension_sector {
                let back: Vec<String> = resonant_pool
                    .iter()
                    .filter(|(w, _)| sector_map.get(w) == Some(&source_sector))
                    .take(3)
                    .map(|(w, _)| w.clone())
                    .collect();
                words.extend(back);
            }

            words.dedup();
            banks.push(words);
        }

        let resonant_words: Vec<String> = resonant_pool
            .iter()
            .take(5)
            .map(|(w, _)| w.clone())
            .collect();

        // Reorder each bank using bigram transition probabilities
        let reordered_banks: Vec<Vec<String>> = banks
            .iter()
            .map(|bank| compose::Composer::reorder_with_bigrams(facet, bank))
            .collect();

        let text = compose::Composer::compose(facet, prompt, &path, &reordered_banks, &resonant_words);

        RiverFlow {
            prompt: prompt.to_string(),
            source_sector,
            text,
        }
    }

    /// Generates all sector variations of a prompt.
    ///
    /// Computes the ray cast ONCE and reuses it for all sector variations,
    /// instead of re-doing the expensive 155K-word scan 64 times.
    pub fn generate_variations(
        facet: &Facet,
        prompt: &str,
        depth: usize,
    ) -> Vec<RiverFlow> {
        let n = Wave::sector_count();

        // Compute ray cast once - same prompt wave for all sectors
        let tokens = Tokenizer::tokenize(prompt);
        let wave = Wave::sentence(facet, &tokens);
        let resonant_pool = Wave::ray_cast(facet, wave, config::RAY_CAST_POOL_SIZE);

        // Pre-compute sector for each word in the resonant pool
        let sector_map: std::collections::HashMap<String, u16> = resonant_pool
            .iter()
            .filter_map(|(w, _)| Wave::word_sector(facet, w).map(|s| (w.clone(), s)))
            .collect();

        // Pre-compute fallback words for each sector (top 4 by amplitude)
        let fallback_map = Self::build_fallback_map(facet);

        let mut flows = Vec::with_capacity(n as usize);
        for sector in 0..n {
            let flow = Self::trace_with_pool(
                facet, prompt, Some(sector), depth,
                &resonant_pool, &sector_map, &fallback_map,
            );
            flows.push(flow);
        }

        flows
    }

    /// Pre-computes fallback words (top 4 by amplitude) for each sector.
    /// Done once instead of per-empty-sector during composition.
    pub(super) fn build_fallback_map(facet: &Facet) -> std::collections::HashMap<u16, Vec<String>> {
        let n = Wave::sector_count();
        let mut map: std::collections::HashMap<u16, Vec<(&String, f64)>> =
            std::collections::HashMap::new();

        for (word, phasor) in &facet.lexicon {
            let effective = phasor.phase + (phasor.band_n as f64 * crate::generate::constants::ALPHA);
            let sector = Wave::sector_of(effective);
            map.entry(sector).or_default().push((word, phasor.amplitude));
        }

        let mut result = std::collections::HashMap::new();
        for sector in 0..n {
            if let Some(words) = map.get_mut(&sector) {
                words.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                result.insert(sector, words.iter().take(4).map(|(w, _)| (*w).clone()).collect());
            }
        }
        result
    }
}
