//! The lab bench (docs/163): what every experiment under `examples/`
//! needed and each had written for itself — the one `setup()` of
//! docs/159 step 5. Flags, the Austen sentences, the three novels.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it. An experiment is a condition put on the
//! same corpus; the bench holds the corpus so the experiment holds only
//! the condition.

use phiton::p64::words::Tokenizer;

/// `--name value` taken out of `a`, if present.
pub fn flag(a: &mut Vec<String>, name: &str) -> Option<String> {
    let i = a.iter().position(|x| x == name)?;
    let v = a.get(i + 1).cloned();
    a.drain(i..(i + 2).min(a.len()));
    v
}

/// `--name` present in `a` (and removed).
pub fn switch(a: &mut Vec<String>, name: &str) -> bool {
    a.iter().position(|x| x == name).map(|i| a.remove(i)).is_some()
}

/// The sentences of a text file, whitespace-normalised, 4..=60 tokens
/// long — the filter every Austen experiment used.
pub fn sentences_of(path: &str) -> Vec<String> {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    Tokenizer::split_sentences(&raw)
        .into_iter()
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| (4..=60).contains(&Tokenizer::tokenize(s).len()))
        .collect()
}

/// The three novels in the Austen corpus folder, as sentences. `dir` is the
/// folder that holds them (phiano's `data/corpora/austen`); the caller names
/// it, so this crate knows no data directory.
pub const AUSTEN: [&str; 3] = ["emma", "persuasion", "sense_and_sensibility"];

pub fn austen(dir: &str) -> Vec<String> {
    AUSTEN.iter().flat_map(|f| sentences_of(&format!("{dir}/{f}.txt"))).collect()
}

/// Every `.txt` under `dir` (READMEs aside), as `(name, sentences)`, sorted.
pub fn dir_sentences(dir: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "txt")).collect();
        paths.sort();
        for p in paths {
            let name = p.file_stem().unwrap().to_string_lossy().to_string();
            if name.starts_with("README") {
                continue;
            }
            out.push((name, sentences_of(p.to_str().unwrap())));
        }
    }
    out
}


/// Today as `YYYY-MM-DD` without a date crate (Howard Hinnant's civil-from-days).
pub fn chrono_free_date() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86_400;
    // civil from days (Howard Hinnant)
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_come_out_of_the_args() {
        let mut a: Vec<String> = ["x", "--n", "3", "--dry", "y"].iter().map(|s| s.to_string()).collect();
        assert_eq!(flag(&mut a, "--n").as_deref(), Some("3"));
        assert!(switch(&mut a, "--dry"));
        assert!(!switch(&mut a, "--dry"));
        assert_eq!(a, ["x", "y"]);
        assert!(sentences_of("no/such/file").is_empty());
    }
}
