//! What:
//!  Controls of the mount calls on a machine that mounts nothing:
//!  a bind of a folder that is not there is refused with its errno,
//!  an unmount of a path that is not a mount point is refused,
//!  and the mount table of this process reads as this namespace's own.
//! Why:
//!  The probe's three mount calls are covered by the guest arms,
//!  and this control holds the one thing that must be true without
//!  authority:
//!  a refused call answers its errno rather than a made-up success.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(bindMount('/no/such/source', '/no/such/target').call).toBe('mount bind');
//! ```

use super::{SCRATCH_TMPFS_OPTIONS, bind_mount, read_own_mount_table, unmount};
use crate::testing::test_support::fixture;
use std::path::PathBuf;

/// A bind whose source is not there is refused,
/// and its call is named.
#[test]
fn a_refused_bind_names_its_call() {
    let root: PathBuf = fixture("start-probe-mount-bind");
    let refusal = bind_mount(root.join("no-such-source").as_path(), root.as_path()).expect_err("the bind is refused");
    assert_eq!(refusal.call, "mount bind");
}

/// An unmount of a path that is not a mount point is refused,
/// and the refusal names `umount2`.
#[test]
fn an_unmount_of_a_plain_folder_is_refused() {
    let root: PathBuf = fixture("start-probe-mount-unmount");
    let refusal = unmount(root.as_path()).expect_err("the unmount is refused");
    assert_eq!(refusal.call, "umount2");
    assert!(matches!(refusal.errno, libc::EINVAL | libc::EPERM), "{}", refusal.errno);
}

/// The scratch tmpfs names a size and an inode count,
/// so it can hold only what the probe needs.
#[test]
fn the_scratch_tmpfs_is_bounded() {
    assert!(SCRATCH_TMPFS_OPTIONS.contains("size=64k"));
    assert!(SCRATCH_TMPFS_OPTIONS.contains("nr_inodes=32"));
}

/// This process's own table reads and holds at least its root mount.
#[test]
fn the_own_table_reads() {
    let table = read_own_mount_table().expect("the table reads");
    assert!(!table.is_empty());
}
