//! What a tree of text knows, and going on from it (docs/127): the rows
//! counted at once, the scope tree, and the next word read under the path
//! the words said so far are drawn into — story, chapter, paragraph, the
//! last word, the last two — D4 nested.

use super::self as prose;
use crate::relate::{Cast, Tally, Field, Order, ScopeTree, SAYS};

const TAU: f64 = 0.5;
/// The kinds a paragraph's rows post under (docs/127).
pub(super) const KINDS: &[&str] = &["has-word", "within", "pair", "in-word", "in-pair"];
/// The fewest of the asked conditions a cast's scope must say to be heard.
const HEARD: usize = 2;

pub struct Knowing {
    pub counts: Tally,
    pub tree: ScopeTree,
    /// Each paragraph's path and its words, in order (for what is not yet
    /// known, `unknown`); the rows themselves are counted and dropped.
    pub paragraphs: Vec<(String, Vec<String>)>,
    /// How many rows were counted.
    pub rows: usize,
    /// The scope path going on stays within (`--in text/story`): the cast
    /// starts there; the word levels still draw on the whole tree.
    pub within: Vec<String>,
    /// Read flat — the last word and the last two, no scopes, no cast —
    /// the comparison line of docs/135.
    pub flat: bool,
    /// Paragraphs held out of the learning (`learn_except`): path and words.
    pub held: Vec<(String, Vec<String>)>,
    /// Cast by phasor instead of by counts (docs/136): the field over the
    /// scopes, built once what was told is counted.
    pub field: Option<Field>,
    /// How the field's casts are ranked: by what was told (the base's now
    /// over was) or by attraction alone.
    pub order: Order,
    /// The base the levels move from, given the words said and the kind's
    /// values (docs/137: an operator's reading); the prior when `None`.
    pub base: Option<Base>,
    /// Without a `base`: move from the last word's kin reading (docs/138,
    /// `Tally::kin_base`) — what the words told the same company went on to.
    pub kin: bool,
}

pub type Base = Box<dyn Fn(&[String], &[String]) -> Option<Vec<f64>>>;

impl Knowing {
    /// Reads the whole tree at once: `(what was learned, (paragraph path, words told))`.
    pub fn learn(root: &std::path::Path) -> (Self, Vec<(String, usize)>) {
        Self::learn_except(root, &|_| false)
    }

    /// As `learn`, leaving out the paragraphs `held(path)` names; their
    /// words are kept in `held` to be asked about.
    pub fn learn_except(root: &std::path::Path, held: &dyn Fn(&str) -> bool) -> (Self, Vec<(String, usize)>) {
        let mut k = Self::fresh();
        let mut told = Vec::new();
        prose::for_each_paragraph(root, &mut |path, rows| {
            if held(&path) {
                k.held.push((path, Self::words(&rows)));
                return;
            }
            told.push((path.clone(), Self::words(&rows).len()));
            k.tell(&path, &rows);
        });
        (k, told)
    }

    /// Tells the tree what is new under `root` since it was read: the
    /// paragraphs whose paths it has not seen (the chapter a teacher appends
    /// to grows by a paragraph; nothing told is read again — docs/132's
    /// second wall). Returns how many paragraphs were told. A field over
    /// the scopes is built again, since the scopes changed.
    pub fn learn_new(&mut self, root: &std::path::Path) -> usize {
        let seen: std::collections::HashSet<String> = self.paragraphs.iter().chain(&self.held).map(|p| p.0.clone()).collect();
        let mut fresh: Vec<(String, Vec<phiecs::quest::Row>)> = Vec::new();
        prose::for_each_paragraph(root, &mut |path, rows| {
            if !seen.contains(&path) {
                fresh.push((path, rows));
            }
        });
        let n = fresh.len();
        for (path, rows) in fresh {
            let words: Vec<String> = rows.iter().filter(|r| r.kind == "has-word").map(|r| r.other.clone()).collect();
            self.counts.learn(&rows);
            self.tree.add_rows(&rows);
            self.rows += rows.len();
            self.paragraphs.push((path, words));
        }
        if let (Some(f), true) = (&self.field, n > 0) {
            self.field = Some(Field::seeded(&self.counts, &self.tree, f.space()));
        }
        n
    }

    /// Casts by the field of phasors with functions of length `space`,
    /// every word seeded from its spelling.
    pub fn with_field(mut self, space: usize) -> Self {
        self.field = Some(Field::seeded(&self.counts, &self.tree, space));
        self
    }

    /// Reads from a real-world prior instead of this text's own frequency
    /// (docs/157): a two-column `word<TAB>frequency` file (e.g. `wordfreq`'s,
    /// `data/lex/freq.tsv`). A word this text never said still has a rank, so
    /// the open stratum — a continuation never told here — is read by how
    /// common the word is in English, not by one text's counts. The levels
    /// still move from it where they post. Lines that don't parse are skipped;
    /// a word with no entry gets a small floor.
    pub fn with_real_prior(mut self, path: &std::path::Path) -> Self {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let freq: std::collections::HashMap<String, f64> = text
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .filter_map(|(w, f)| Some((w.to_string(), f.trim().parse::<f64>().ok()?)))
            .collect();
        self.base = Some(Box::new(move |_said: &[String], values: &[String]| {
            let raw: Vec<f64> = values.iter().map(|v| *freq.get(v).unwrap_or(&1e-9)).collect();
            let z: f64 = raw.iter().sum();
            (z > 0.0).then(|| raw.iter().map(|x| x / z).collect())
        }));
        self
    }

    /// The path the words said so far are drawn into, and the next word
    /// read under it, then under the last word, then under the last two.
    pub fn next(&self, said: &[String]) -> (Vec<String>, Vec<(String, f64)>, Vec<String>) {
        let mut recent: Vec<String> = said.iter().rev().take(6).filter(|w| w.chars().any(char::is_alphanumeric)).cloned().collect();
        recent.extend(said.windows(2).rev().take(3).map(|p| format!("pair:{} {}", p[0], p[1])));
        // The scope a cast finds is heard only when the words are inside it:
        // its last scope says, by itself, at least two of the asked
        // conditions and the last pairs of words (D3: the condition is
        // local — a paragraph that shares a few words with a new sentence is
        // not where that sentence is; the story's own paragraph says its
        // pairs). Otherwise the reading is the language's: the word and the
        // pair everywhere.
        let last_pairs: Vec<String> = said.windows(2).rev().take(3).map(|p| format!("pair:{} {}", p[0], p[1])).collect();
        let inside = |c: &Cast| c.own >= HEARD && c.path.last().is_some_and(|leaf| last_pairs.iter().all(|p| self.counts.count(Some(leaf), SAYS, p) >= 1.0));
        let cast = match (&self.field, self.flat) {
            (_, true) => Vec::new(),
            (Some(f), false) => f.cast_under(&self.counts, &self.tree, &self.within, &recent, 2, self.order),
            (None, false) => self.tree.cast_under(&self.counts, &self.within, &recent, f64::MIN, 2),
        };
        let path = cast.into_iter().next().filter(inside).map(|c| c.path).unwrap_or_default();
        // Coarse to fine: the path's scopes; the last word everywhere, then
        // under each scope of the path; the last two everywhere, then under
        // each scope. What this story goes on to outranks what the language does.
        let mut levels: Vec<Vec<String>> = path.iter().map(|s| vec![s.clone()]).collect();
        let inner: Vec<&String> = path.iter().skip(1).collect();
        if let Some(last) = said.last() {
            levels.push(vec![last.clone()]);
            levels.extend(inner.iter().map(|s| vec![format!("{s}|{last}")]));
        }
        if said.len() >= 2 {
            let pair = format!("pair:{} {}", said[said.len() - 2], said[said.len() - 1]);
            levels.push(vec![pair.clone()]);
            levels.extend(inner.iter().map(|s| vec![format!("{s}|{pair}")]));
        }
        let vp = self.counts.values_and_prior("next");
        let values = &vp.0;
        let base = match (&self.base, self.kin, said.last()) {
            (Some(b), _, _) => b(said, values),
            (None, true, Some(last)) => self.counts.kin_base(last, "next", values),
            _ => None,
        };
        let (s, posts) = match base {
            Some(base) => self.counts.read_levels_from(&levels, "next", values, TAU, base),
            None => self.counts.read_levels(&levels, "next", values, TAU),
        };
        let mut ranked: Vec<(String, f64)> = values.iter().cloned().zip(s).collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        (path, ranked, posts)
    }
}
