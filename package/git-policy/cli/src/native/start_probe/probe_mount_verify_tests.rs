//! What:
//!  Controls of the read-back's comparison:
//!  the mount the attach asked for,
//!  a mount ID the table does not list,
//!  another filesystem,
//!  another source,
//!  and a missing flag.
//! Why:
//!  The arm reports the flags it asked for and reads them back,
//!  so a table that lacks one must fail the arm rather than pass it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! verifyMount(table, { mountId: 41, source: 'pool/ds@ready' });
//! ```

use super::{ExpectedMount, VerifyFailure, verify_mount};
use crate::fs_mount_options::parse_mount_table;

/// The mount line the probe's attach would make.
const THE_MOUNT: &str = "41 30 0:32 / /var/tmp/probe/slot ro,nosuid,nodev,relatime - zfs zpcachyos/nosnap/git-worktree-probe@ready ro,xattr,noacl\n";

/// The table the arm read back holds the mount it asked for.
#[test]
fn the_mount_the_attach_asked_for_passes() {
    let table = parse_mount_table(THE_MOUNT.as_bytes());
    let mount = verify_mount(table.as_slice(), ExpectedMount { mount_id: 41, source: "zpcachyos/nosnap/git-worktree-probe@ready" }).expect("the mount is the one asked for");
    assert_eq!(mount.mount_id, 41);
    assert_eq!(mount.filesystem, "zfs");
    assert!(mount.options.contains(&String::from("ro")));
}

/// Every wrong fact is named.
#[test]
fn every_wrong_fact_is_named() {
    let table = parse_mount_table(THE_MOUNT.as_bytes());
    assert_eq!(verify_mount(table.as_slice(), ExpectedMount { mount_id: 42, source: "x" }).err(), Some(VerifyFailure::NotListed { expected: 42 }));
    assert_eq!(
        verify_mount(table.as_slice(), ExpectedMount { mount_id: 41, source: "other@ready" }).err(),
        Some(VerifyFailure::OtherSource { found: String::from("zpcachyos/nosnap/git-worktree-probe@ready"), expected: String::from("other@ready") })
    );
    let other_filesystem: &str = "41 30 0:32 / /var/tmp/probe/slot ro,nosuid,nodev,relatime - tmpfs tmpfs ro\n";
    let other_table = parse_mount_table(other_filesystem.as_bytes());
    assert_eq!(verify_mount(other_table.as_slice(), ExpectedMount { mount_id: 41, source: "tmpfs" }).err(), Some(VerifyFailure::OtherFilesystem { found: String::from("tmpfs") }));
}

/// A missing flag fails the arm with that flag's name.
#[test]
fn a_missing_flag_is_named() {
    let missing_nodev: &str = "41 30 0:32 / /var/tmp/probe/slot ro,nosuid,relatime - zfs zpcachyos/nosnap/git-worktree-probe@ready ro\n";
    let table = parse_mount_table(missing_nodev.as_bytes());
    assert_eq!(
        verify_mount(table.as_slice(), ExpectedMount { mount_id: 41, source: "zpcachyos/nosnap/git-worktree-probe@ready" }).err(),
        Some(VerifyFailure::MissingFlag { flag: "nodev" })
    );
}
