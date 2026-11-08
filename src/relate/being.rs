//! Beings told from quests and code: the constructors that need `phient`.
//! The struct is `phiecs::being::Being`; these are free functions over it.
use super::read_quests;
use phiecs::agent::Agent;
use phiecs::being::{persona_files, Being};
use phiecs::persona::Persona;
use std::path::Path;

/// A persona told everything its quest folders hold:
/// `<root>/<slug>/<seed>/relations.tsv`, as `persona_quest` writes them.
/// With no folders yet it is `Being::new`, and `told()` says so with 0.
pub fn from_quests(root: &Path, persona: Persona) -> Being {
    let (rows, _) = read_quests(&root.join(persona.slug()));
    let mut being = Being::new(persona);
    being.tell(&rows);
    // A persona's `sources` are also told: a directory of `.b` files is
    // read by the BCPL reader (docs/168), so a language's persona is
    // told from its own code rather than quested from an oracle.
    for src in being.persona.sources.clone() {
        let dir = Path::new(&src);
        if dir.is_dir() {
            // The coarse grain: what each function declares, gets and
            // calls. The prose reader's occurrence rows are a corpus's
            // grain, not a persona's.
            being.tell(&crate::code::bcpl::read_coarse(dir));
        }
    }
    being
}

/// A persona read from its file and told from `root`.
pub fn from_file(root: &Path, file: &Path) -> Result<Being, String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(from_quests(root, Persona::from_md(&text)?))
}

/// The being called `who` under `dir`, told from `root`: an exact name or
/// slug first (`bertrand-russell`), else the one slug with `who` as a word
/// (`feynman`); two or more is an error naming them (`russell`).
pub fn find(root: &Path, dir: &Path, who: &str) -> Result<Being, String> {
    let who = who.trim().to_lowercase().replace(' ', "-");
    let mut files = persona_files(dir);
    for sub in ["stage", "code"] {
        files.extend(persona_files(&dir.join(sub)));
    }
    let all: Vec<Persona> = files
        .iter()
        .filter_map(|f| std::fs::read_to_string(f).ok().and_then(|t| Persona::from_md(&t).ok()))
        .collect();
    if let Some(p) = all.iter().find(|p| p.slug() == who || p.name.to_lowercase() == who.replace('-', " ")) {
        return Ok(from_quests(root, p.clone()));
    }
    let hits: Vec<&Persona> = all.iter().filter(|p| p.slug().split('-').any(|w| w == who)).collect();
    match hits.as_slice() {
        [p] => Ok(from_quests(root, (*p).clone())),
        [] => Err(format!("no persona called {who} under {}", dir.display())),
        many => Err(format!("{who} is {}: say which", many.iter().map(|p| p.slug()).collect::<Vec<_>>().join(" or "))),
    }
}
