//! The grammar's roles (docs/170): BCPL's closed vocabulary of structure.
//! Six roles, read straight off the grammar (Crick 2004, "A GCC Front End
//! for BCPL", App. C BNF + App. D operator precedence) — nothing hand-tuned
//! per word. This is the closed kind-set over the open names (docs/169 #2).

/// BCPL's reserved words — the ONE copy lives with the compiler
/// ([`phiclm::token::KEYWORDS`], docs/178); this is a view of it.
pub use phiclm::token::KEYWORDS;

/// The grammatical role a token plays. One of six, so the floor for
/// guessing the next role is 1-in-6 (~17%).
pub fn of(t: &str) -> &'static str {
    let c = t.chars().next().unwrap_or(' ');
    if KEYWORDS.contains(&t) {
        "keyword"
    } else if c.is_ascii_digit() {
        "number"
    } else if t == "\"" || t == "'" {
        "string"
    } else if c.is_alphabetic() || c == '_' {
        "name"
    } else if "()[]{}$".contains(c) {
        "bracket"
    } else {
        "operator"
    }
}

/// How many distinct roles — the denominator of the floor.
pub const COUNT: usize = 6;
