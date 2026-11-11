//! Verifies the lock an origin stop and a move share:
//! one holder at a time,
//! a released gate is free again,
//! a bounded wait ends with the answer that another holder has it,
//! and the gate file it creates carries no permission for anyone else.

/// The gate under test.
use crate::adoption_gate::{acquire_within, adoption_gate_path, try_acquire};
/// Unix file mode of the gate.
use std::os::unix::fs::PermissionsExt;
/// Fixture paths.
use std::path::PathBuf;
/// Bounded waits.
use std::time::{Duration, Instant};

/// A directory of this test,
/// emptied first.
fn gate_root(name: &str) -> PathBuf {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-gate-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("make the gate directory");
    return directory;
}

/// The user this test takes the gate for.
fn own_uid() -> u32 {
    // SAFETY: getuid has no preconditions.
    return unsafe { libc::getuid() };
}

/// One holder is let in,
/// and a second one is answered at once that the gate is taken.
#[test]
fn one_holder_is_let_in_and_the_next_is_answered_at_once() {
    let root = gate_root("exclusive");
    let uid = own_uid();
    let first = try_acquire(&root, uid).expect("the first ask answers").expect("the gate is free");
    let started = Instant::now();
    assert!(try_acquire(&root, uid).expect("the second ask answers").is_none(), "a second holder was let in");
    assert!(started.elapsed() < Duration::from_secs(1), "the refused ask waited");
    drop(first);
    assert!(try_acquire(&root, uid).expect("the third ask answers").is_some(), "the released gate stayed taken");
}

/// A wait for a held gate ends at its limit,
/// which is what keeps a sweep from holding a watcher's start.
#[test]
fn a_wait_ends_at_its_limit_while_another_holder_has_the_gate() {
    let root = gate_root("wait");
    let uid = own_uid();
    let held = try_acquire(&root, uid).expect("the first ask answers").expect("the gate is free");
    let started = Instant::now();
    let waited = acquire_within(&root, uid, Duration::from_millis(100)).expect("the wait answers");
    let elapsed = started.elapsed();
    assert!(waited.is_none(), "the wait answered a gate another holder had");
    assert!(elapsed >= Duration::from_millis(100), "the wait ended before its limit: {elapsed:?}");
    assert!(elapsed < Duration::from_secs(2), "the wait outlived its limit: {elapsed:?}");
    drop(held);
    assert!(acquire_within(&root, uid, Duration::from_millis(100)).expect("the second wait answers").is_some(), "a freed gate was not taken");
}

/// The gate file is named for the watched user,
/// so two users' managers,
/// whose scopes never meet,
/// never wait for each other.
#[test]
fn the_gate_is_named_for_the_user_and_only_its_owner_may_open_it() {
    let root = gate_root("path");
    let first = adoption_gate_path(&root, 1000);
    let second = adoption_gate_path(&root, 1001);
    assert_ne!(first, second);
    let _held = try_acquire(&root, 1000).expect("the ask answers").expect("the gate is free");
    let mode = std::fs::metadata(&first).expect("the gate file exists").permissions().mode();
    assert_eq!(mode & 0o777, 0o600, "the gate file is readable by others: {mode:o}");
}
