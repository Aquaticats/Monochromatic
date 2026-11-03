//! Controls for what the release of the index lock installs: a copy that is complete on the disk, never an unchanged one.

use super::*;
use crate::service_fault::hooks;
use crate::service_index_lock::try_acquire;
use crate::service_land_start::witness_of;
use crate::test_support::{fixture, remove};

/// An index file of `body` followed by a checksum tail of `tail`, in a fresh fixture directory.
fn index_with(name: &str, body: &[u8], tail: &[u8]) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let index: PathBuf = root.join("index");
    let mut bytes: Vec<u8> = body.to_vec();
    bytes.extend_from_slice(tail);
    std::fs::write(&index, bytes).expect("index");
    return (root, index);
}

/// A copy that holds the live index's own bytes is not installed: the live file stays the same file, so its access list, which a
///  copy made in another directory does not inherit, is never replaced, and no rename is spent on a no-op.
#[test]
fn an_unchanged_copy_leaves_the_live_file_in_place() {
    let (root, index) = index_with("hold-unchanged", b"entries of the index", &[7; 20]);
    let built: PathBuf = root.join("built");
    std::fs::copy(&index, &built).expect("copy");
    let before = witness_of(index.as_path()).expect("witness");
    let lock = try_acquire(index.as_path(), 1).expect("lock");
    let recorded = std::cell::Cell::new(false);
    lock.release_with_record(built.as_path(), &|| { recorded.set(true); return Ok(()); }).expect("released");
    assert_eq!(witness_of(index.as_path()).expect("witness"), before, "the live file is the same file");
    assert!(recorded.get(), "the completion is recorded all the same");
    assert!(!root.join("index.lock").exists(), "the lock is given up");
    remove(root.as_path());
}

/// A copy that differs only before a skipped checksum (all zero bytes) is another index: equal tails prove nothing then.
#[test]
fn a_copy_with_a_skipped_checksum_is_compared_in_full() {
    let (root, index) = index_with("hold-skipped-checksum", b"entries of the index", &[0; 32]);
    let built: PathBuf = root.join("built");
    std::fs::write(&built, [b"entries of the INDEX".as_slice(), &[0; 32]].concat()).expect("copy");
    let lock = try_acquire(index.as_path(), 1).expect("lock");
    lock.release_with(built.as_path()).expect("released");
    assert!(std::fs::read(&index).expect("index").starts_with(b"entries of the INDEX"), "the changed copy was installed");
    remove(root.as_path());
}

/// A changed copy is flushed to the disk before it becomes the index, whichever way it was made, and the directory and the
///  completion record follow.
#[test]
fn a_changed_copy_is_synced_before_it_is_installed() {
    let (root, index) = index_with("hold-sync-order", b"entries of the index", &[7; 20]);
    let built: PathBuf = root.join("built");
    std::fs::write(&built, [b"entries of the OTHER".as_slice(), &[9; 20]].concat()).expect("copy");
    hooks::take_events();
    let lock = try_acquire(index.as_path(), 1).expect("lock");
    lock.release_with_record(built.as_path(), &|| { crate::service_fault::note("recorded"); return Ok(()); }).expect("released");
    let events: Vec<String> = hooks::take_events();
    let at = |name: &str| -> usize { return events.iter().position(|event| { return event == name; }).unwrap_or(usize::MAX); };
    assert!(at("sync-built-index") < at("sync-index-dir"), "the file is synced before its directory entry: {events:?}");
    assert!(at("sync-index-dir") < at("recorded"), "{events:?}");
    remove(root.as_path());
}

/// A flush of the copy that fails installs nothing and records nothing: the live index is as it was and the lock is given up.
#[test]
fn a_failed_sync_of_the_copy_installs_nothing() {
    let (root, index) = index_with("hold-sync-fails", b"entries of the index", &[7; 20]);
    let built: PathBuf = root.join("built");
    std::fs::write(&built, [b"entries of the OTHER".as_slice(), &[9; 20]].concat()).expect("copy");
    let before = witness_of(index.as_path()).expect("witness");
    hooks::fail_at("sync-built-index");
    let lock = try_acquire(index.as_path(), 1).expect("lock");
    let recorded = std::cell::Cell::new(false);
    let released = lock.release_with_record(built.as_path(), &|| { recorded.set(true); return Ok(()); });
    hooks::clear_failures();
    assert!(released.is_err());
    assert_eq!(witness_of(index.as_path()).expect("witness"), before, "the live index is as it was");
    assert!(!recorded.get(), "nothing is recorded");
    assert!(!root.join("index.lock").exists(), "the lock is given up");
    remove(root.as_path());
}

/// The access list of the live file, as `getfacl` prints it without the file name; nothing when the tool or the file system
///  cannot give one.
fn access_list(path: &Path) -> Option<String> {
    let output = std::process::Command::new("getfacl").arg("--numeric").arg("--absolute-names").arg("--omit-header").arg(path).output().ok()?;
    return output.status.success().then(|| { return String::from_utf8_lossy(output.stdout.as_slice()).into_owned(); });
}

/// A live index whose access list names another user, replaced by a copy made in a private directory, would get the list the
///  directory gives its files. An unchanged copy replaces nothing, so the list the live file has is the list it keeps.
#[test]
fn an_unchanged_copy_keeps_the_access_list_of_the_live_file() {
    let (root, index) = index_with("hold-acl", b"entries of the index", &[7; 20]);
    let set = std::process::Command::new("setfacl").args(["--modify", "user:65534:r--"]).arg(&index).status();
    if !set.is_ok_and(|status| { return status.success(); }) {
        remove(root.as_path());
        return;
    }
    let before: Option<String> = access_list(index.as_path());
    assert!(before.as_ref().is_some_and(|text| { return text.contains("user:65534:r--"); }), "the list was set: {before:?}");
    let built: PathBuf = root.join("built");
    std::fs::copy(&index, &built).expect("copy");
    let lock = try_acquire(index.as_path(), 1).expect("lock");
    lock.release_with(built.as_path()).expect("released");
    assert_eq!(access_list(index.as_path()), before, "the live file keeps its access list");
    remove(root.as_path());
}
