//! The language machine's REPL (docs/165): McCarthy's Lisp over Russell's
//! algebra over the stories' sets and the personas' worlds.
//!
//! ```text
//! cargo run --release -p phient --example lisp -- [--text data/text] [--bcpl ext/bcpl/cintcode/com] [--quests quests --personas crates/phinet/personas]… [-e "(form)"] [file.lisp]
//! ```
//!
//! With no `-e` and no file it reads forms from stdin, one per line.
//! `--personas` may be given more than once (both rings).

use phient::relate::trace;
use phient::study::lab::flag;
use phient::relate::lisp::{Machine, Stage};
use phiecs::being::persona_files;
use phient::relate::being::from_file;
use std::io::BufRead;
use std::path::PathBuf;

fn main() {
    let mut a: Vec<String> = std::env::args().skip(1).collect();
    let text = flag(&mut a, "--text");
    let quests = flag(&mut a, "--quests").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("quests"));
    let mut rings = Vec::new();
    while let Some(d) = flag(&mut a, "--personas") {
        rings.push(PathBuf::from(d));
    }
    let mut forms = Vec::new();
    while let Some(e) = flag(&mut a, "-e") {
        forms.push(e);
    }
    let bcpl = flag(&mut a, "--bcpl");
    let mut m = Machine::new();
    let mut tree = Vec::new();
    if let Some(t) = &text {
        tree.extend(phient::prose::read_tree(&PathBuf::from(t)));
    }
    if let Some(b) = &bcpl {
        tree.extend(phient::code::bcpl::read_tree(&PathBuf::from(b)));
    }
    if !tree.is_empty() {
        m = m.with_scopes(trace::scopes_of(&tree));
        eprintln!("; {} scopes from {}{}", m.host.scopes.len(), text.as_deref().unwrap_or(""), bcpl.as_deref().map(|b| format!(" + {b}")).unwrap_or_default());
    }
    let mut beings = Vec::new();
    for r in &rings {
        for f in persona_files(r) {
            if let Ok(b) = from_file(&quests, &f) {
                beings.push(b);
            }
        }
    }
    if !beings.is_empty() {
        eprintln!("; {} personas", beings.len());
        m = m.with_beings(beings);
    }
    for f in &a {
        let src = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{f}: {e}"));
        forms.push(src);
    }
    if forms.is_empty() {
        eprintln!("; phinet lisp — (scopes) (beings) (held who kind) (next \"scope:…\") (closure r) … ctrl-d to end");
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            match m.run(&line) {
                Ok(v) => println!("{v}"),
                Err(e) => println!("; error: {e}"),
            }
        }
        return;
    }
    for src in forms {
        match m.run(&src) {
            Ok(v) => println!("{v}"),
            Err(e) => {
                eprintln!("; error: {e}");
                std::process::exit(1);
            }
        }
    }
}
