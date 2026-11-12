//! The tests of the non-blocking take of a profile directory's build locks:
//! a name that is not a regular file is refused instead of opened,
//! so a pipe at a lock file never holds the pass that takes the lock.

use std::{fs, path::PathBuf};

use crate::lean_lock_probe::{LOCK_FILE_NAMES, LockOutcome, lock_profile_directory};
use crate::lean_test_support::{make_pipe, make_profile, within};
use crate::test_scratch::scratch;

/// What:
/// `fn empty_lock_table(root: &Path) -> PathBuf` is the path of an empty kernel lock table for a test.
/// Why:
/// The probe reads the table after it takes every lock,
/// and a test of the take itself needs one that names no file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emptyLockTable(root: string): string
/// ```
fn empty_lock_table(root: &std::path::Path) -> PathBuf {
    let table: PathBuf = root.join("locks");
    fs::write(&table, b"").expect("an empty lock table");
    return table;
}

/// A pipe at a Cargo lock file is refused, not opened:
/// the open of a pipe waits for a writer that nothing promises,
/// and a profile directory whose lock cannot be taken is skipped and reported instead.
#[test]
fn a_pipe_at_a_lock_file_is_unlockable_and_never_waited_for() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let lock: PathBuf = profile.join(LOCK_FILE_NAMES[0]);
    fs::remove_file(&lock).expect("the regular lock file the fixture wrote");
    make_pipe(&lock);
    let table: PathBuf = empty_lock_table(root.path());
    let probe_profile: PathBuf = profile.clone();
    let outcome: LockOutcome = within(move || { return lock_profile_directory(&probe_profile, &table); });
    match outcome {
        LockOutcome::Unlockable { file, reason } => {
            assert_eq!(file, lock);
            assert!(reason.contains("not a regular file"), "{reason}");
        }
        other => panic!("a pipe at a lock file is no lock this probe may take: {other:?}"),
    }
    assert!(lock.exists(), "the lock files are never renamed or unlinked");
}

/// A regular lock file is still taken exactly as before:
/// the open a probe shares with the readers of a build tree does not change what a lock file is.
#[test]
fn a_regular_lock_file_is_still_taken_exclusively() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let table: PathBuf = empty_lock_table(root.path());
    let probe_profile: PathBuf = profile.clone();
    let outcome: LockOutcome = within(move || { return lock_profile_directory(&probe_profile, &table); });
    match outcome {
        LockOutcome::Held(held) => assert_eq!(held.names(), [profile.join(LOCK_FILE_NAMES[0])]),
        other => panic!("a regular lock file is taken: {other:?}"),
    }
}
