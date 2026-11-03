//! The unit tests of compare-before-replace and of recovery from a killed run.

use std::fs;
use std::path::Path;

use crate::crates::LockSnapshot;
use crate::journal::{JOURNAL_DIR, begin, finish, recover, restore_if_unchanged};
use crate::test_scratch::scratch;

const CRATE_DIR: &str = "package/j/one";

fn snapshot(source: &str) -> LockSnapshot {
    return LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: source.to_string() };
}

fn repository(lock: &str) -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), lock).expect("lock");
    return root;
}

fn lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("read");
}

#[test]
fn a_lock_that_still_holds_what_the_run_produced_is_replaced_by_the_snapshot() {
    let root = repository("produced");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert!(warnings.is_empty());
    assert_eq!(lock(root.path()), "original");
}

#[test]
fn a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning() {
    let root = repository("written by someone else");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert_eq!(lock(root.path()), "written by someone else");
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains(CRATE_DIR) && warnings[0].contains("another process"));
}

#[test]
fn a_lock_that_already_equals_the_snapshot_needs_nothing_and_no_temporary_file_is_left() {
    let root = repository("original");

    let warnings = restore_if_unchanged(root.path(), &[snapshot("original")], &[snapshot("produced")]).expect("restores");

    assert!(warnings.is_empty());
    let leftovers: Vec<String> = fs::read_dir(root.path().join(CRATE_DIR))
        .expect("list")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().to_string(); })
        .collect();
    assert_eq!(leftovers, vec!["Cargo.lock".to_string()]);
}

/// A process id that cannot exist on Linux (above the kernel's maximum).
const DEAD_PID: u32 = 4_194_305;

fn plant_journal(root: &Path, original: &str) {
    begin(root, &[snapshot(original)]).expect("begin");
    let ours = root.join(JOURNAL_DIR).join(std::process::id().to_string());
    fs::rename(&ours, root.join(JOURNAL_DIR).join(DEAD_PID.to_string())).expect("pose as a killed process");
}

#[test]
fn a_killed_run_whose_lock_holds_refused_versions_is_saved_aside_and_the_snapshot_goes_back() {
    let root = repository("unchecked output of the killed run");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });

    assert_eq!(lock(root.path()), "original");
    assert_eq!(
        fs::read_to_string(root.path().join(CRATE_DIR).join("Cargo.lock.refused")).expect("the saved lock"),
        "unchecked output of the killed run"
    );
    assert_eq!(messages.len(), 1);
    assert!(!root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists());
}

#[test]
fn a_killed_run_whose_lock_passes_the_policy_is_kept() {
    let root = repository("good output");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return false; }, &|_before, _live| { return Vec::new(); });

    assert_eq!(lock(root.path()), "good output");
    assert!(!root.path().join(CRATE_DIR).join("Cargo.lock.refused").exists());
    assert_eq!(messages.len(), 1);
}

#[test]
fn a_journal_of_a_running_process_and_our_own_journal_are_not_touched() {
    let root = repository("live lock");
    plant_journal(root.path(), "original");

    let messages = recover(root.path(), &|_pid| { return true; }, &|_before, _live| { return vec!["never asked".to_string()]; });

    assert!(messages.is_empty());
    assert_eq!(lock(root.path()), "live lock");
    assert!(root.path().join(JOURNAL_DIR).join(DEAD_PID.to_string()).exists());
}

#[test]
fn finish_removes_this_processs_journal() {
    let root = repository("x");
    begin(root.path(), &[snapshot("x")]).expect("begin");

    finish(root.path());

    assert!(!root.path().join(JOURNAL_DIR).join(std::process::id().to_string()).exists());
}
