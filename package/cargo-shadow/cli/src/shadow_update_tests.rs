//! The unit tests of the stand-in lockfile that a full update of a root without a lockfile uses.

use std::fs;

use crate::shadow_update::run_with_placeholder_lock;
use crate::test_scratch::scratch;

const REAL_LOCK: &str = "version = 4\n\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\n";

/// Review finding (dry run): a dry run restores the stand-in and succeeds, and must leave no lockfile behind.
#[test]
fn a_successful_dry_run_leaves_no_lockfile_behind() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");

    let code: i32 = run_with_placeholder_lock(&lock, true, &mut || { return 0; });

    assert_eq!(code, 0);
    assert!(!lock.exists(), "the stand-in of a dry run was left behind");
}

/// A real update keeps the lockfile it produced.
#[test]
fn a_successful_update_keeps_the_lockfile_it_produced() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    let written = lock.clone();

    let code: i32 = run_with_placeholder_lock(&lock, false, &mut || {
        fs::write(&written, REAL_LOCK).expect("cargo writes the lock");
        return 0;
    });

    assert_eq!(code, 0);
    assert_eq!(fs::read_to_string(&lock).expect("read"), REAL_LOCK);
}

/// A refused update removes its own stand-in.
#[test]
fn a_refused_update_removes_its_own_stand_in() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");

    let code: i32 = run_with_placeholder_lock(&lock, false, &mut || { return 1; });

    assert_eq!(code, 1);
    assert!(!lock.exists());
}

/// Review finding (cleanup deletes another writer's file): another writer publishes a lockfile while the update runs, and
/// the update then fails. The cleanup removes only the stand-in it wrote.
#[test]
fn a_refused_update_does_not_delete_a_lockfile_another_writer_published() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    let theirs = lock.clone();

    let code: i32 = run_with_placeholder_lock(&lock, false, &mut || {
        fs::write(&theirs, REAL_LOCK).expect("the other writer");
        return 1;
    });

    assert_eq!(code, 1);
    assert_eq!(fs::read_to_string(&lock).expect("the other writer's lock survives"), REAL_LOCK);
}

/// Review finding (truncation): a lockfile that already exists is never replaced by the stand-in.
#[test]
fn an_existing_lockfile_is_never_touched() {
    let root = scratch();
    let lock = root.path().join("Cargo.lock");
    fs::write(&lock, REAL_LOCK).expect("an existing lock");

    let code: i32 = run_with_placeholder_lock(&lock, true, &mut || { return 0; });

    assert_eq!(code, 0);
    assert_eq!(fs::read_to_string(&lock).expect("read"), REAL_LOCK);
}
