//! What:
//!  The note of the last Git step that failed:
//! what it keeps,
//! what drops it and what takes it away.
//! Why:
//!  A note that outlives the step that made it names a step that did not stop the command,
//! and one that survives being taken reaches a second record.

/// The ledger and its one note.
use super::*;

/// The status a control's step exits with.
const REFUSED_STATUS: i32 = 128;

/// A note keeps the step and its status until it is taken,
/// and taking it leaves nothing behind.
#[test]
fn a_note_is_taken_once() {
    clear_failed_step();
    note_failed_step("git add", Some(REFUSED_STATUS));
    let taken: FailedStep = drain_failed_step().expect("the note just written");
    assert_eq!(taken.name, "git add");
    assert_eq!(taken.status, Some(REFUSED_STATUS));
    assert_eq!(drain_failed_step(), None, "a record never reads the note another record stated");
}

/// A step that could not start keeps no status,
/// so a reader never reads zero as the status of a step that never exited.
#[test]
fn a_step_that_never_exited_keeps_no_status() {
    clear_failed_step();
    note_failed_step("git diff-index", None);
    assert_eq!(drain_failed_step(), Some(FailedStep { name: String::from("git diff-index"), status: None }));
}

/// Another step starting drops the note,
/// so a step that failed and let the command go on is never the reason a later stop gives.
#[test]
fn another_step_starting_drops_the_note() {
    clear_failed_step();
    note_failed_step("git add", Some(REFUSED_STATUS));
    clear_failed_step();
    assert_eq!(drain_failed_step(), None);
}

/// A name carrying a path,
/// a quote or a line break is written with those characters replaced,
/// so no note can put a path or a second line into a record.
#[test]
fn a_note_never_carries_a_path_or_a_line_break() {
    clear_failed_step();
    note_failed_step("git add -- /home/someone/a b.txt\n{\"k\":1}", Some(REFUSED_STATUS));
    let taken: FailedStep = drain_failed_step().expect("the note just written");
    assert!(!taken.name.contains('/') && !taken.name.contains('\n') && !taken.name.contains('"'), "{}", taken.name);
    assert!(taken.name.starts_with("git add"), "{}", taken.name);
}
