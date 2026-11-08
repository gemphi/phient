//! Several Phi-4 runs of the builders' game side by side (docs/107–108):
//! the first run (Ollama's default sampling), temperature 0, and the prompt
//! that says who fetches and who takes note. Students here are the fast ones
//! (bag of words and Phical counts), five folds by call.

use super::follow::{consistency, load, read_of, student};

pub fn compare(files: &[String], epochs: usize, seed: u64) -> String {
    let mut s = String::from("## Phi-4 runs of the builders' game, side by side\n\n| run | replies read | exact (game's terms) | act | same answer to the same call | our rules predict Phi-4 | bag-of-words student | counts student |\n|---|---|---|---|---|---|---|---|\n");
    for f in files {
        let asked = load(f);
        let read = read_of(&asked);
        let n = read.len().max(1) as f64;
        let exact = read.iter().filter(|x| x.2 == x.0.action).count() as f64 / n;
        let act = read.iter().filter(|x| x.2[0] == x.0.action[0]).count() as f64 / n;
        let (same, pairs) = consistency(&read);
        let (_, st) = student(&read, &[], epochs, seed);
        let get = |k: &str| st.iter().find(|x| x.0.starts_with(k)).map_or(0.0, |x| x.1);
        let name = std::path::Path::new(f).file_stem().map_or(f.clone(), |x| x.to_string_lossy().into_owned());
        s += &format!(
            "| {name} | {} of {} | {:.1}% | {:.1}% | {:.1}% ({same} of {pairs}) | {:.1}% | {:.1}% | {:.1}% |\n",
            read.len(),
            asked.len(),
            100.0 * exact,
            100.0 * act,
            100.0 * same as f64 / pairs.max(1) as f64,
            100.0 * get("our rules"),
            100.0 * get("bag of words"),
            100.0 * get("Phical counts")
        );
    }
    s
}
