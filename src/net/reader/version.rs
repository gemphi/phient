#[allow(unused_imports)]
use super::*;

/// Reads the facet header and returns its version only if the whole header is
/// plausible: a small version, a vocabulary that fits in memory, and a
/// fine-structure alpha in (0, 1). A `Memo` file starts with a `Vec` length
/// and fails the alpha test.
pub(super) fn header_version(path: &str) -> Option<u32> {
    let mut f = BufReader::new(File::open(path).ok()?);
    let mut b = [0u8; 20];
    f.read_exact(&mut b).ok()?;
    let version = u32::from_le_bytes(b[0..4].try_into().ok()?);
    let vocab = u64::from_le_bytes(b[4..12].try_into().ok()?);
    let alpha = f64::from_le_bytes(b[12..20].try_into().ok()?);
    match version <= 16 && vocab < 1_000_000_000 && alpha.is_finite() && alpha > 0.0 && alpha < 1.0 {
        true => Some(version),
        false => None,
    }
}

/// Opens a model file and describes it. A facet is returned alongside so the
/// caller can encode it without loading twice.
pub fn inspect(path: &str) -> (ModelReport, Option<Facet>) {
    let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let mut report = ModelReport {
        path: path.to_string(),
        kind: ModelKind::Unknown,
        bytes,
        format_version: 0,
        words: 0,
        evidenced_words: 0,
        bigram_contexts: 0,
        trigram_contexts: 0,
        phase_lags: 0,
        marker_channels: 0,
        dispersion_ch0: 0.0,
        dispersion_ch0_frequent: 0.0,
        dispersion_other: 0.0,
        amplitude_mean: 0.0,
        amplitude_max: 0.0,
        band_n_max: 0,
        memo_entries: 0,
        notes: Vec::new(),
    };

    // A facet header is (u32 version, u64 vocab, f64 alpha); a plausible
    // version is small. A Memo starts with a Vec length, which is not.
    if let Some(version) = header_version(path) {
        match Storage::load(path) {
            Ok(mut facet) => {
                report.kind = ModelKind::Facet;
                report.format_version = version;
                let migrated = facet.migrate_channels();
                if migrated > 0 {
                    report.notes.push(format!(
                        "{migrated} words had no channels; seeded from identity, learned angle kept on channel 0"
                    ));
                }
                report.words = facet.vocabulary_size();
                report.evidenced_words = facet.evidenced_words();
                report.bigram_contexts = facet.bigrams.len();
                report.trigram_contexts = facet.trigrams.len();
                report.phase_lags = facet.phase_lags.len();
                report.marker_channels = facet.relations.markers();
                report.dispersion_ch0 = facet.dispersion_above(0);
                let has_counts = facet.lexicon.values().any(|p| p.count > 0);
                report.dispersion_ch0_frequent = match has_counts {
                    true => facet.dispersion_above(25),
                    false => f64::NAN,
                };
                if !has_counts {
                    report.notes.push("no observation counts stored (pre-`count` layout); frequency floors cannot be applied".into());
                }
                report.dispersion_other = (1..PHASE_CHANNELS)
                    .map(|k| dispersion_all(&facet, k))
                    .sum::<f64>()
                    / (PHASE_CHANNELS - 1) as f64;
                let n = facet.lexicon.len().max(1) as f64;
                report.amplitude_mean = facet.lexicon.values().map(|p| p.amplitude).sum::<f64>() / n;
                report.amplitude_max = facet.lexicon.values().map(|p| p.amplitude).fold(0.0, f64::max);
                report.band_n_max = facet.lexicon.values().map(|p| p.band_n).max().unwrap_or(0);
                if report.evidenced_words == 0 {
                    report.notes.push(
                        "no relation evidence: every word is a flat function (equal amplitude on all channels)".into(),
                    );
                }
                if report.band_n_max > 10_000 {
                    report.notes.push(format!(
                        "band_n reaches {}: it was incremented on every converged step and carries no information",
                        report.band_n_max
                    ));
                }
                return (report, Some(facet));
            }
            Err(e) => report.notes.push(format!("failed to load as a facet: {e}")),
        }
    }
    match Memo::load_from_file(path) {
        Ok(memo) => {
            report.kind = ModelKind::Memo;
            report.memo_entries = memo.entries.len();
            report.notes.push("a context-wave memory log, not a lexicon".into());
        }
        Err(e) => report.notes.push(format!("not a memo either: {e}")),
    }
    (report, None)
}

/// Dispersion of channel `k` over every word, evidence or not.
pub(super) fn dispersion_all(facet: &Facet, k: usize) -> f64 {
    let (mut x, mut y, mut n) = (0.0f64, 0.0f64, 0usize);
    for p in facet.lexicon.values() {
        let t = p.theta(k);
        x += t.cos();
        y += t.sin();
        n += 1;
    }
    match n {
        0 => 1.0,
        _ => 1.0 - x.hypot(y) / n as f64,
    }
}

impl ModelReport {
    pub fn render(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("{}  ({:.1} MB)\n", self.path, self.bytes as f64 / 1e6));
        match self.kind {
            ModelKind::Facet => {
                s.push_str(&format!(
                    "  facet, format v{}: {} words, {} with relation evidence, {} marker channels\n",
                    self.format_version, self.words, self.evidenced_words, self.marker_channels
                ));
                s.push_str(&format!(
                    "  n-grams: {} bigram contexts, {} trigram contexts, {} phase lags\n",
                    self.bigram_contexts, self.trigram_contexts, self.phase_lags
                ));
                s.push_str(&format!(
                    "  channel-0 dispersion {:.3} (all) / {:.3} (25+ occurrences); other channels {:.3}\n",
                    self.dispersion_ch0, self.dispersion_ch0_frequent, self.dispersion_other
                ));
                s.push_str(&format!(
                    "  amplitude mean {:.3} max {:.3}; band_n max {}\n",
                    self.amplitude_mean, self.amplitude_max, self.band_n_max
                ));
            }
            ModelKind::Memo => {
                s.push_str(&format!("  memory log: {} entries\n", self.memo_entries));
            }
            ModelKind::Unknown => s.push_str("  unrecognised\n"),
        }
        for n in &self.notes {
            s.push_str(&format!("  note: {n}\n"));
        }
        s
    }
}
