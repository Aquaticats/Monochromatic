//! Controls for the record of a sweep a stopped push owed:
//! the marker is written into the repository's own Git directory,
//! the repository's roots find it,
//! and settling it judges the locks and takes the marker away.

use super::*;
use std::time::Duration;

/// A ledger over one disposable Git directory with an orphan staging file,
/// whose judgement looks at an empty process table of its own.
/// Why:
///  The sweep's own removal is what a control must see,
/// and the real judgement refuses to decide from the processes of the whole
///  machine (a Git of another account vetoes it),
/// so the control gives it a table with nothing in it.
fn a_ledger_over_a_staging_file(name: &str) -> (std::path::PathBuf, LockLedger, std::path::PathBuf, Judge) {
    let root: std::path::PathBuf = crate::test_support::fixture(name);
    let git_dir: std::path::PathBuf = root.join("git-dir");
    let proc_root: std::path::PathBuf = root.join("proc");
    std::fs::create_dir_all(git_dir.as_path()).expect("a disposable Git directory");
    std::fs::create_dir_all(proc_root.as_path()).expect("an empty disposable process table");
    let staging: std::path::PathBuf = git_dir.join("packed-refs.new");
    std::fs::write(staging.as_path(), b"an orphan staging file of a killed Git").expect("the staging file");
    let mut command: std::process::Command = std::process::Command::new("true");
    command.env("GIT_DIR", git_dir.as_os_str());
    let judge: Judge = Judge {
        proc_root,
        margin: Duration::ZERO,
        boot: None,
        local_file_systems: ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "overlay", "zfs", "bcachefs"].into_iter().map(String::from).collect(),
        scan_scope: Some(Vec::new()),
    };
    let ledger: LockLedger = LockLedger::before_start(&command).with_judge(judge.clone());
    return (root, ledger, staging, judge);
}

/// What:
///  A recorded sweep is a file inside the repository's own Git directory,
/// and the roots of that repository find it.
/// Why:
///   A record the next reader cannot find is no record:
/// the reader has the Git directory and not this process's memory.
#[test]
fn a_recorded_sweep_is_a_file_the_repositorys_roots_find() {
    let (root, ledger, staging, _judge) = a_ledger_over_a_staging_file("owed-sweep-record");
    let roots: Vec<std::path::PathBuf> = ledger.roots().to_vec();
    assert!(!any_owed(roots.as_slice()), "nothing is owed before a stop records one");
    let written: Vec<std::path::PathBuf> = record(&ledger, "the control recorded it");
    assert_eq!(written.len(), roots.len(), "one marker per root of the repository: {written:?}");
    for marker in &written {
        assert!(marker.is_file(), "the marker is there: {}", marker.display());
        assert_eq!(marker, &marker_in(marker.parent().and_then(|folder| { return folder.parent(); }).expect("the git directory")), "the marker is at its own path");
        let said: String = std::fs::read_to_string(marker.as_path()).expect("the marker's words");
        assert!(said.contains("the control recorded it"), "the marker carries why it is there: {said}");
    }
    assert!(any_owed(roots.as_slice()), "the repository's roots now find the record");
    assert_eq!(owed_markers_of(roots.as_slice()), written, "and the markers are the ones written");
    assert!(staging.exists(), "recording owes the sweep and removes nothing");
    crate::test_support::remove(root.as_path());
}

/// What:
///  Settling an owed sweep judges the locks of the repository and takes the marker away,
/// and a second settlement has nothing to do.
/// Why:
///   The judgement is the duty the marker carries:
/// a marker that outlived its judgement would send every later reader
///  through the walk again,
/// and a judgement that ran must not be repeated for nothing.
#[test]
fn settling_an_owed_sweep_judges_the_locks_and_takes_the_marker_away() {
    let (root, ledger, staging, judge) = a_ledger_over_a_staging_file("owed-sweep-settled");
    let roots: Vec<std::path::PathBuf> = ledger.roots().to_vec();
    let _written: Vec<std::path::PathBuf> = record(&ledger, "the control recorded it");
    let settled: Option<Outcome> = settle_if_owed(roots.as_slice(), &judge);
    let outcome: Outcome = settled.expect("the record was there, so the judgement ran");
    assert_eq!(outcome.cleared.len(), 1, "the orphan staging file the sweep would have removed is gone: {outcome:?}");
    assert!(!staging.exists(), "the staging file is not there any more");
    assert!(!any_owed(roots.as_slice()), "and the marker went with the judgement");
    assert!(settle_if_owed(roots.as_slice(), &judge).is_none(), "a second reader finds nothing owing and judges nothing");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A judged lock the judgement will not remove is named with the command that clears it,
/// which is what a reader of the record says to the person.
#[test]
fn a_lock_the_judgement_will_not_remove_is_named_with_its_command() {
    let (root, ledger, staging, judge) = a_ledger_over_a_staging_file("owed-sweep-named");
    let roots: Vec<std::path::PathBuf> = ledger.roots().to_vec();
    let git_dir: std::path::PathBuf = roots.first().expect("a root").clone();
    std::fs::create_dir_all(git_dir.join("refs").join("heads").as_path()).expect("a refs directory");
    let held: std::path::PathBuf = git_dir.join("refs").join("heads").join("main.lock");
    std::fs::write(held.as_path(), b"").expect("a lock nothing proves stale");
    let _written: Vec<std::path::PathBuf> = record(&ledger, "the control recorded it");
    let outcome: Outcome = settle(roots.as_slice(), &judge);
    let notice: String = outcome.notice();
    assert!(notice.contains("main.lock"), "the notice names the lock it leaves: {notice}");
    assert!(notice.contains("rm -- "), "and the command that clears it: {notice}");
    assert!(held.exists(), "a lock nothing proves stale is never removed");
    assert!(!staging.exists(), "the staging file it can prove is removed in the same judgement");
    assert!(!any_owed(roots.as_slice()), "the marker went with the judgement it carried the duty to");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A ledger with no roots records nothing,
/// which is what a caller that could not find the repository has.
/// Why:
///   The line a stop says must not promise a record that no file carries:
/// a stop with no repository root says so
///  in its own words instead (`push/stop_of_a_push_and_sweep_of_its_locks.rs`).
#[test]
fn a_ledger_with_no_roots_records_nothing() {
    let ledger: LockLedger = LockLedger::none();
    assert!(record(&ledger, "the control recorded it").is_empty(), "no root, no marker");
    assert!(!any_owed(ledger.roots()), "and nothing is owed anywhere");
}
