//! Controls for what the release of the index lock installs:
//! a copy that is complete on the disk,
//! never an unchanged one.

use super::*;
use crate::service_fault::hooks;
use crate::service_index_lock::try_acquire;
use crate::service_land_start::witness_of;
use crate::test_support::{fixture, remove};

/// A function-like value with the right shape is a record in tests:
/// the shipped callers write a small struct that holds the state.
impl<Make: Fn() -> Result<(), std::io::Error>> Record for Make {
    fn record(&self) -> std::io::Result<()> {
        return self();
    }
}

/// An index file of `body` followed by a checksum tail of `tail`,
/// in a fresh fixture directory.
fn index_with(name: &str, body: &[u8], tail: &[u8]) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let index: PathBuf = root.join("index");
    let mut bytes: Vec<u8> = body.to_vec();
    bytes.extend_from_slice(tail);
    std::fs::write(&index, bytes).expect("index");
    return (root, index);
}

/// A copy that holds the live index's own bytes is not installed:
/// the live file stays the same file,
/// so its access list,
/// which a
///  copy made in another directory does not inherit,
/// is never replaced,
/// and no rename is spent on a no-op.
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

/// A copy that differs only before a skipped checksum (all zero bytes) is another index:
/// equal tails prove nothing then.
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

/// A changed copy is flushed to the disk before it becomes the index,
/// whichever way it was made,
/// and the directory and the
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
    assert!(at("sync-waited:built-index") < at("sync-waited:index-dir"), "the file is synced before its directory entry: {events:?}");
    assert!(at("sync-waited:index-dir") < at("recorded"), "{events:?}");
    remove(root.as_path());
}

/// A flush of the copy that fails installs nothing and records nothing:
/// the live index is as it was and the lock is given up.
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

/// The bytes of a POSIX ACL (`system.posix_acl_access`) that gives user 4242 read access beside the owner's read and write and
///  the group's read;
/// the same layout `subvolume_sync_tests.rs` uses,
/// set through the extended attribute so the gate image needs
///  no `setfacl`.
fn acl_with_named_user() -> Vec<u8> {
    let mut bytes: Vec<u8> = 2u32.to_le_bytes().to_vec();
    for (tag, bits, id) in [(1u16, 6u16, u32::MAX), (2, 4, 4242), (4, 4, u32::MAX), (0x10, 4, u32::MAX), (0x20, 0, u32::MAX)] {
        bytes.extend_from_slice(&tag.to_le_bytes());
        bytes.extend_from_slice(&bits.to_le_bytes());
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    return bytes;
}

/// Sets the access ACL of `path`;
/// false where the file system refuses extended attributes.
fn set_access_list(path: &Path, value: &[u8]) -> bool {
    let (path_text, name) = (std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path"), std::ffi::CString::new("system.posix_acl_access").expect("name"));
    // SAFETY: both strings are NUL-terminated and the value is readable for its length.
    let set: libc::c_int = unsafe { libc::lsetxattr(path_text.as_ptr(), name.as_ptr(), value.as_ptr().cast(), value.len(), 0) };
    if set != 0 {
        eprintln!("ACL CONTROL NOT RUN: this file system refuses the access ACL ({})", std::io::Error::last_os_error());
    }
    return set == 0;
}

/// The access ACL bytes of `path`,
/// or nothing when it has none.
fn access_list(path: &Path) -> Option<Vec<u8>> {
    let (path_text, name) = (std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path"), std::ffi::CString::new("system.posix_acl_access").expect("name"));
    let mut value: Vec<u8> = vec![0; 512];
    // SAFETY: the buffer is writable for its length and the strings are NUL-terminated.
    let read: isize = unsafe { libc::lgetxattr(path_text.as_ptr(), name.as_ptr(), value.as_mut_ptr().cast(), value.len()) };
    if read < 0 {
        return None;
    }
    value.truncate(usize::try_from(read).expect("length"));
    return Some(value);
}

/// A live index whose access list names another user,
/// replaced by a copy made in a private directory,
/// would get the list the
///  directory gives its files.
/// An unchanged copy replaces nothing,
/// so the list the live file has is the list it keeps.
/// The file
///  system that refuses the attribute skips the control with a loud line,
/// as the extended attribute control of the copy does.
#[test]
fn an_unchanged_copy_keeps_the_access_list_of_the_live_file() {
    let (root, index) = index_with("hold-acl", b"entries of the index", &[7; 20]);
    if !set_access_list(index.as_path(), acl_with_named_user().as_slice()) {
        remove(root.as_path());
        return;
    }
    let before: Option<Vec<u8>> = access_list(index.as_path());
    assert_eq!(before.as_deref(), Some(acl_with_named_user().as_slice()), "the list was set");
    let built: PathBuf = root.join("built");
    std::fs::copy(&index, &built).expect("copy");
    let lock = try_acquire(index.as_path(), 1).expect("lock");
    lock.release_with(built.as_path()).expect("released");
    assert_eq!(access_list(index.as_path()), before, "the live file keeps its access list");
    remove(root.as_path());
}
