//! What:
//!  Controls of the mark:
//!  a trigger writes it,
//!  a second trigger's write is the same request,
//!  the look finds it,
//!  the removal leaves nothing,
//!  a removal of a mark that is not there is fine,
//!  and a name that is no file at all is reported rather than refused.
//! Why:
//!  Presence is the whole meaning,
//!  so every one of these states has to end in the answer the taker's steps 4
//!  and 7 read
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What a taker does";
//!  veto entry V6).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeTheMark(folder); expect(theMarkIsThere(folder).kind).toBe('present');
//! ```

use super::{
    MarkReading,
    PASS_MARK_MODE,
    PASS_MARK_TEXT,
    SweepsFolder,
    remove_the_mark,
    the_mark_is_there,
    write_the_mark,
};
use crate::test_support::fixture;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// This test's folder,
/// which the thread's fixtures remove however the control ends.
fn folder(name: &str) -> SweepsFolder {
    let root: PathBuf = fixture(name);
    return SweepsFolder::at(root.join("sweeps"), rustix::process::geteuid().as_raw());
}

/// A folder with no mark answers that no trigger asked for a pass.
#[test]
fn a_folder_with_no_mark_asks_for_no_pass() {
    let folder: SweepsFolder = folder("mark-absent");
    let reading: MarkReading = the_mark_is_there(&folder);
    assert_eq!(reading, MarkReading::Absent);
    assert!(!reading.asks_for_a_pass());
}

/// A written mark is one file at the designed mode holding its one line,
/// and the look finds it.
#[test]
fn a_written_mark_is_one_file_holding_its_line() {
    let folder: SweepsFolder = folder("mark-present");
    write_the_mark(&folder).expect("a mark this account may write");
    let facts: std::fs::Metadata = std::fs::metadata(folder.pass_mark_path()).expect("the mark");
    assert!(facts.is_file(), "the mark is a file");
    assert_eq!(facts.permissions().mode() & 0o7777, PASS_MARK_MODE);
    assert_eq!(
        std::fs::read_to_string(folder.pass_mark_path()).expect("the mark's line"),
        format!("{PASS_MARK_TEXT}\n")
    );
    let reading: MarkReading = the_mark_is_there(&folder);
    assert_eq!(reading, MarkReading::Present);
    assert!(reading.asks_for_a_pass());
}

/// A second trigger's mark is the request the first one made.
#[test]
fn a_second_write_of_a_mark_is_the_same_request() {
    let folder: SweepsFolder = folder("mark-twice");
    write_the_mark(&folder).expect("the first mark");
    write_the_mark(&folder).expect("a mark that is already there is the same request");
    assert_eq!(the_mark_is_there(&folder), MarkReading::Present);
    assert_eq!(
        std::fs::read_dir(folder.directory())
            .expect("the folder")
            .flatten()
            .count(),
        1,
        "one mark, never two"
    );
}

/// Removing the mark leaves nothing,
/// and removing it twice asks for nothing the second time.
#[test]
fn removing_the_mark_leaves_nothing() {
    let folder: SweepsFolder = folder("mark-removed");
    write_the_mark(&folder).expect("a mark");
    remove_the_mark(&folder).expect("the mark goes");
    assert_eq!(the_mark_is_there(&folder), MarkReading::Absent);
    remove_the_mark(&folder).expect("a mark that is not there is the state wanted");
}

/// A name that is a folder cannot be removed,
/// and the report says which name it was,
/// which is what a pass writes a line about and runs past.
#[test]
fn a_mark_that_is_no_file_is_reported() {
    let folder: SweepsFolder = folder("mark-is-a-folder");
    folder.prepare().expect("the folder");
    std::fs::create_dir_all(folder.pass_mark_path()).expect("a folder where the mark goes");
    assert_eq!(the_mark_is_there(&folder), MarkReading::Present);
    let refusal: String = remove_the_mark(&folder).expect_err("a folder cannot be unlinked");
    assert!(
        refusal.contains("could not be removed"),
        "the report says what failed: {refusal}"
    );
    assert!(
        refusal.contains("another-pass-is-due"),
        "the report names the mark: {refusal}"
    );
}

/// A folder that could not be made is a report from the write,
/// and never a panic.
#[test]
fn a_mark_in_a_folder_that_cannot_be_made_is_reported() {
    let root: PathBuf = fixture("mark-unwritable-folder");
    let blocker: PathBuf = root.join("sweeps");
    std::fs::write(blocker.as_path(), b"not a folder").expect("a file where the folder goes");
    let folder: SweepsFolder = SweepsFolder::at(blocker, rustix::process::geteuid().as_raw());
    let refusal: String = write_the_mark(&folder).expect_err("a file is no folder");
    assert!(
        refusal.contains("cannot be used"),
        "the report says the folder cannot be used: {refusal}"
    );
}
