//! A persona's quest folders on disk: `<dir>/<seed>/relations.tsv` and
//! `sentences.tsv`, as `examples/persona_quest` writes them.

use phiecs::quest::Row;
use std::path::Path;

/// Every relation row and every sentence under one persona's folder.
pub fn read_quests(dir: &Path) -> (Vec<Row>, Vec<String>) {
    let (mut rows, mut sentences) = (Vec::new(), Vec::new());
    let Ok(seeds) = std::fs::read_dir(dir) else { return (rows, sentences) };
    let mut seeds: Vec<_> = seeds.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_dir()).collect();
    seeds.sort();
    for seed in seeds {
        if let Ok(t) = std::fs::read_to_string(seed.join("relations.tsv")) {
            rows.extend(phiecs::quest::parse_tsv(&t));
        }
        if let Ok(t) = std::fs::read_to_string(seed.join("sentences.tsv")) {
            sentences.extend(t.lines().filter_map(|l| l.split_once('\t')).map(|(_, s)| s.trim().to_string()));
        }
    }
    (rows, sentences)
}

/// True for the fifth of rows held out, by a hash of the row — the same
/// split every phiano relation reading uses.
pub fn held_out(r: &Row) -> bool {
    let h = phistd::mix64(phistd::fnv1a(&format!("{}\t{}\t{}", r.about, r.kind, r.other)));
    ((h >> 11) as f64 / (1u64 << 53) as f64) < 0.2
}
