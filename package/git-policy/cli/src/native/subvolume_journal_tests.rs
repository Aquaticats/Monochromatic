//! Controls for the migration journal: an empty start, the states and their predicates, the atomic write, the reload,
//!  every shape of a file that is not a journal, and the deletion.

use super::*;
use crate::test_support::{fixture, remove};

/// A fresh journal path in a fixture.
fn journal_path(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let path: PathBuf = parent.join(".journal.json");
    return (parent, path);
}

/// No file means an empty journal that is not persisted and answers "not begun" to every step.
#[test]
fn a_missing_file_is_an_empty_journal() {
    let (parent, path) = journal_path("journal-empty");
    let journal: Journal = Journal::load(path.as_path()).expect("load");
    assert!(!journal.is_persisted() && journal.state("copy").is_none());
    assert!(!journal.is_done("copy") && !journal.has_begun("copy"));
    assert_eq!(journal.path(), path.as_path());
    remove(parent.as_path());
}

/// `started` has begun but is not done; `done` is both; `rolled-back` is neither.
#[test]
fn the_states_answer_the_predicates() {
    let (parent, path) = journal_path("journal-states");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("copy", "started").expect("mark");
    assert!(journal.has_begun("copy") && !journal.is_done("copy"));
    journal.mark("copy", "done").expect("mark");
    assert!(journal.has_begun("copy") && journal.is_done("copy"));
    journal.mark("exchange", "rolled-back").expect("mark");
    assert!(!journal.has_begun("exchange") && !journal.is_done("exchange"));
    assert_eq!(journal.state("exchange"), Some("rolled-back"));
    assert!(journal.is_persisted());
    remove(parent.as_path());
}

/// What was marked is read back by a new load, and no temporary file stays beside the journal.
#[test]
fn a_marked_step_survives_a_reload_and_leaves_no_temporary_file() {
    let (parent, path) = journal_path("journal-reload");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("create", "done").expect("mark");
    journal.mark("copy", "started").expect("mark");
    let reloaded: Journal = Journal::load(path.as_path()).expect("reload");
    assert!(reloaded.is_persisted() && reloaded.is_done("create") && reloaded.has_begun("copy") && !reloaded.is_done("copy"));
    assert_eq!(reloaded.started, journal.started);
    let names: Vec<String> = std::fs::read_dir(parent.as_path()).expect("listing").flatten().map(|item| { return item.file_name().to_string_lossy().into_owned(); }).collect();
    assert_eq!(names, vec![String::from(".journal.json")]);
    remove(parent.as_path());
}

/// Every file that is not a journal of this format is an error that names the file; nothing is guessed.
#[test]
fn a_file_that_is_not_a_journal_is_refused_by_name() {
    let (parent, path) = journal_path("journal-broken");
    let cases: [(&str, &str); 6] = [
        ("not json", "is not valid JSON"),
        ("{\"schema\": 2, \"started\": \"x\", \"steps\": {}}", "unknown schema version"),
        ("{\"started\": \"x\", \"steps\": {}}", "unknown schema version"),
        ("{\"schema\": 1, \"steps\": {}}", "has no start time"),
        ("{\"schema\": 1, \"started\": \"x\"}", "has no steps"),
        ("{\"schema\": 1, \"started\": \"x\", \"steps\": {\"copy\": {}}}", "step without a state"),
    ];
    for (text, why) in cases {
        std::fs::write(path.as_path(), text).expect("write");
        let error: MigrateError = Journal::load(path.as_path()).expect_err("refused");
        let shown: String = error.to_string();
        assert!(shown.contains(why) && shown.contains(path.to_str().expect("path")), "{text}: {shown}");
    }
    remove(parent.as_path());
}

/// A path that cannot be read as a file (a directory) is an input and output error, not an empty journal.
#[test]
fn an_unreadable_journal_is_an_error_not_an_empty_one() {
    let (parent, path) = journal_path("journal-directory");
    std::fs::create_dir(path.as_path()).expect("a directory where the journal would be");
    let error: MigrateError = Journal::load(path.as_path()).expect_err("refused");
    assert!(error.to_string().contains("cannot read the journal"), "{error}");
    remove(parent.as_path());
}

/// A write that cannot happen is reported with the journal's path and leaves the memory state recorded.
#[test]
fn a_journal_that_cannot_be_written_is_reported() {
    let parent: PathBuf = fixture("journal-unwritable");
    let path: PathBuf = parent.join("absent-directory").join(".journal.json");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    let error: MigrateError = journal.mark("copy", "started").expect_err("no directory to write in");
    assert!(error.to_string().contains("cannot write the journal"), "{error}");
    assert!(!journal.is_persisted());
    remove(parent.as_path());
}

/// Deleting removes the file and forgets the steps; deleting twice is not an error.
#[test]
fn delete_forgets_the_steps_and_is_repeatable() {
    let (parent, path) = journal_path("journal-delete");
    let mut journal: Journal = Journal::load(path.as_path()).expect("load");
    journal.mark("copy", "done").expect("mark");
    journal.delete().expect("delete");
    assert!(!path.exists() && !journal.is_persisted() && !journal.is_done("copy"));
    journal.delete().expect("a second delete");
    remove(parent.as_path());
}

/// A path that is a non-empty directory cannot be deleted as a file, and the error names it.
#[test]
fn a_journal_path_that_cannot_be_removed_is_reported() {
    let (parent, path) = journal_path("journal-undeletable");
    std::fs::create_dir(path.as_path()).expect("directory");
    std::fs::write(path.join("inside"), b"x").expect("file");
    let mut journal: Journal = Journal { path: path.clone(), started: String::from("x"), states: BTreeMap::new(), times: BTreeMap::new(), persisted: true };
    let error: MigrateError = journal.delete().expect_err("a directory is not a file");
    assert!(error.to_string().contains("cannot delete the journal"), "{error}");
    remove(parent.as_path());
}

/// Each step keeps the time it was recorded at when a later step is marked, and a record written without a time reads as
///  having none.
#[test]
fn a_step_keeps_its_own_time_and_an_old_record_has_none() {
    let directory = crate::test_support::fixture("journal-times");
    let path = directory.join("journal.json");
    let mut journal: Journal = Journal::load(path.as_path()).expect("empty");
    assert_eq!(journal.seconds_of("copy"), None);
    journal.mark("copy", "done").expect("mark");
    let first: i64 = journal.seconds_of("copy").expect("time");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    journal.mark("delta1", "done").expect("mark");
    let reloaded: Journal = Journal::load(path.as_path()).expect("reload");
    assert_eq!(reloaded.seconds_of("copy"), Some(first), "marking another step does not move this step's time");
    assert!(reloaded.seconds_of("delta1") > Some(first));
    std::fs::write(path.as_path(), br#"{"schema":1,"started":"x","steps":{"copy":{"state":"done","at":"y"}}}"#).expect("an older journal");
    assert_eq!(Journal::load(path.as_path()).expect("load").seconds_of("copy"), None);
    crate::test_support::remove(directory.as_path());
}
