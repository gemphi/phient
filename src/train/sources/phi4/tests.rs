//! The directory-pinned Phi-4 ingest (R3.14, R3.15, R3.16): a directory with
//! a vocabulary is read, an empty one reports the exact path it looked for and
//! leaves the facet untouched.

use super::*;
use crate::train::trainer::Trainer;
use phiecs::phasor::facet::Facet;

/// A fresh temporary directory named by `tag`, emptied if it lingered from a
/// previous run.
fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("phient_phi4_{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Write a minimal `vocab.json` of a few alphabetic tokens into `dir`.
fn write_vocab(dir: &Path) {
    let json = r#"{"the":0,"cat":1,"sat":2,"on":3,"mat":4}"#;
    fs::write(dir.join(VOCAB_FILE), json).expect("write vocab.json");
}

#[test]
fn a_directory_with_a_vocabulary_is_read_from_that_directory_alone() {
    let dir = temp_dir("present");
    write_vocab(&dir);
    let source = Phi4Source::at(&dir).expect("a directory holding vocab.json is read");
    assert_eq!(source.model_dir, dir, "the source is pinned to the directory given");

    let mut facet = Facet::new();
    let trainer = Trainer::new(0.05);
    let summary = source.learn_into_facet(&mut facet, &trainer, "main");
    // The five alphabetic tokens were read into the named facet (R3.16).
    assert_eq!(summary.vocab_tokens_loaded, 5);
    assert_eq!(summary.facet, "main");
    assert_eq!(summary.final_vocabulary_size, facet.vocabulary_size());
    assert!(facet.vocabulary_size() >= 5, "the words landed in the facet");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn an_empty_directory_reports_the_path_it_looked_for_and_changes_no_facet() {
    let dir = temp_dir("empty");
    // The facet as it stands before any ingest is attempted.
    let mut facet = Facet::new();
    facet.get_or_init("already");
    let before = facet.vocabulary_size();

    // The empty directory holds no vocab.json, so `at` refuses by the exact
    // path it looked for and constructs no source (R3.15).
    let looked_for = dir.join(VOCAB_FILE);
    match Phi4Source::at(&dir) {
        Err(Phi4Error::NoVocabulary { looked_for: named }) => {
            assert_eq!(named, looked_for, "the error names the exact path looked for");
        }
        Err(Phi4Error::Unreadable { path, reason }) => {
            panic!("expected NoVocabulary, got Unreadable {{{}}}: {reason}", path.display());
        }
        Ok(_) => panic!("an empty directory must report NoVocabulary, not a source"),
    }
    // Nothing was ingested: the facet is left exactly as it was.
    assert_eq!(facet.vocabulary_size(), before, "a refused ingest changed the facet");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_error_names_the_missing_vocabulary_path_in_its_message() {
    let dir = temp_dir("message");
    let err = match Phi4Source::at(&dir) {
        Err(e) => e,
        Ok(_) => panic!("no vocab.json, yet a source was built"),
    };
    let said = err.to_string();
    assert!(said.contains("vocab.json"), "the message names the vocabulary file: {said}");
    let _ = fs::remove_dir_all(&dir);
}