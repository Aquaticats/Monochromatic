//! Controls for the reading of the numbers a process has in nested PID namespaces.
//! Why:
//!  The answer decides whether a pid file's number can prove its writer gone, so each shape of table is pinned: a nested
//!  holder, the reader's own numbering, a process that ended, and a table that cannot be read completely.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(nestedHoldersOf(table, 7)).toEqual([9001]); expect(nestedHoldersOf(table, 9001)).toEqual([]);
//! ```
#![cfg(unix)]

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// Add one process to a fake table, with `status` as the text of its `status` file when there is one.
fn add_process(table: &Path, outer: i64, status: Option<&str>) {
    let process: PathBuf = table.join(outer.to_string());
    std::fs::create_dir_all(process.as_path()).expect("fake process");
    if let Some(text) = status {
        std::fs::write(process.join("status"), text).expect("status");
    }
}

/// A process listed under `outer` that is process `inner` of a nested namespace is a holder of `inner`, and of no other number.
#[test]
fn a_process_numbered_inside_a_nested_namespace_holds_that_number() {
    let root: PathBuf = fixture("nested-numbers-holder");
    add_process(root.as_path(), 9001, Some("Name:\tsh\nNSpid:\t9001\t7\n"));
    assert_eq!(nested_holders_of(root.as_path(), 7), Ok(vec![9001]));
    assert_eq!(nested_holders_of(root.as_path(), 8), Ok(Vec::new()));
    remove(root.as_path());
}

/// The reader's own number is the first column; a process that is in no nested namespace holds nothing nested.
#[test]
fn the_readers_own_number_is_not_a_nested_one() {
    let root: PathBuf = fixture("nested-numbers-own");
    add_process(root.as_path(), 9001, Some("Name:\tsh\nNSpid:\t9001\n"));
    assert_eq!(nested_holders_of(root.as_path(), 9001), Ok(Vec::new()));
    remove(root.as_path());
}

/// A process with no `status` file has ended since the listing and holds nothing.
#[test]
fn a_process_that_ended_during_the_listing_holds_nothing() {
    let root: PathBuf = fixture("nested-numbers-ended");
    add_process(root.as_path(), 9001, None);
    assert_eq!(nested_holders_of(root.as_path(), 7), Ok(Vec::new()));
    remove(root.as_path());
}

/// A `status` file without the line cannot be read as "not nested": the question stays open.
#[test]
fn a_status_without_the_namespace_line_leaves_the_question_open() {
    let root: PathBuf = fixture("nested-numbers-no-line");
    add_process(root.as_path(), 9001, Some("Name:\tsh\n"));
    assert!(nested_holders_of(root.as_path(), 7).is_err());
    remove(root.as_path());
}

/// A table that cannot be listed leaves the question open.
#[test]
fn a_table_that_cannot_be_listed_leaves_the_question_open() {
    let root: PathBuf = fixture("nested-numbers-no-table");
    assert!(nested_holders_of(root.join("absent").as_path(), 7).is_err());
    remove(root.as_path());
}

/// A `status` file that cannot be read, other than because the process ended, leaves the question open.
#[test]
fn an_unreadable_status_leaves_the_question_open() {
    use std::os::unix::fs::PermissionsExt as _;
    let root: PathBuf = fixture("nested-numbers-unreadable");
    add_process(root.as_path(), 9001, Some("Name:\tsh\nNSpid:\t9001\n"));
    let status: PathBuf = root.join("9001").join("status");
    std::fs::set_permissions(status.as_path(), std::fs::Permissions::from_mode(0o000)).expect("hide the status");
    let answer = nested_holders_of(root.as_path(), 7);
    std::fs::set_permissions(status.as_path(), std::fs::Permissions::from_mode(0o600)).expect("reveal the status");
    // A superuser reads the hidden file anyway, so the control has nothing to show for it.
    // SAFETY: `geteuid` reads the process credentials and cannot fail.
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipped: a superuser reads a file whose mode hides it");
    } else {
        assert!(answer.is_err(), "{answer:?}");
    }
    remove(root.as_path());
}
