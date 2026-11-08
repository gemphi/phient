//! `band` — the band as a TEMPLATE (docs/199). A template's job is to
//! CONTAIN what comes next, not to be it (docs/198 measured the band
//! losing 4.9 points of top-1 while handing back the right grammatical
//! class). So this measures containment instead: at every position, read
//! the group BEFORE the sentence is told, and record how large it is and
//! where the true next word falls in it.
//!
//! Run once per read mode; the per-position records line up across runs
//! because the stream and the telling order are the same, so a band of
//! median size n can be scored against the crowd read's top-n — the only
//! fair control, since a bigger group contains more by accident.
//!
//! ```text
//! PHI_READ=bound:0.125 cargo run --release -p phient --example band -- <corpus> --out /tmp/b125.tsv
//! ```

use phient::lineup::Ecs;
use phient::prose::tokens;
use phient::study::Reader;
use std::io::Write;

fn sentences(path: &str) -> Vec<Vec<String>> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .split(|c| matches!(c, '.' | '!' | '?' | '\n'))
        .map(tokens)
        .map(|w| w.into_iter().filter(|t| t.chars().any(char::is_alphanumeric)).collect::<Vec<_>>())
        .filter(|w| w.len() >= 4)
        .collect()
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = a.first().cloned() else {
        eprintln!("which corpus?");
        return;
    };
    let flag = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let lines: usize = flag("--lines").and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
    let out = flag("--out").unwrap_or_else(|| "/tmp/band.tsv".into());

    let stream: Vec<Vec<String>> = sentences(&path).into_iter().take(lines).collect();
    let mode = std::env::var("PHI_READ").unwrap_or_else(|_| "count".into());
    let mut person = Ecs::new();
    let mut file = std::io::BufWriter::new(std::fs::File::create(&out).expect("out"));
    writeln!(file, "# mode\t{mode}\tcorpus\t{path}").ok();
    writeln!(file, "size\trank").ok();

    let t0 = std::time::Instant::now();
    let (mut asked, mut held) = (0usize, 0usize);
    for words in &stream {
        // Predict first, learn after: every position is read before this
        // sentence moves a single phasor.
        for j in 1..words.len() {
            let read = person.ranked(&words[..j]);
            let rank = read.iter().position(|(w, _)| *w == words[j]);
            asked += 1;
            if rank.is_some() {
                held += 1;
            }
            writeln!(file, "{}\t{}", read.len(), rank.map_or(-1i64, |r| r as i64)).ok();
        }
        person.tell(words);
    }
    file.flush().ok();
    let share = 100.0 * held as f64 / asked.max(1) as f64;
    println!("# band as template — {} — {}", path, mode);
    println!("{} sentences, {asked} positions read before being told, in {:.0} s", stream.len(), t0.elapsed().as_secs_f64());
    println!("the group HELD the true next word at {share:.1}% of positions ({held}/{asked})");
    println!("per-position records: {out}");
}
