//! What:
//!  A full disk, or a disk that fails a flush, at each call of an explicit-path commit that writes, creates, renames,
//!  links or syncs.
//! Why:
//!  The README promises that a write can fail with no space left in the middle of a landing and the wrapper still
//!  recovers.
//!  Every call of the commit is failed in turn (`binary_fault_support.rs` has the method), and after each the next
//!  ordinary command must succeed, the commit must be landed or not made, and nothing may stay behind.
//!  A call that fails after the branch moved is the dangerous one: a record that a failed write left torn must not
//!  stop recovery from completing the landing once space is back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const n of range(1, countCalls('write'))) await injectAndExpectRecovery({ call: 'write', error: 'ENOSPC', n });
//! ```

use super::fault::{Effect, sweep};

/// Every `write` of the wrapper's own thread fails with no space left on the device.
#[test]
fn a_full_disk_at_any_write_of_the_wrapper_leaves_a_recoverable_repository() {
    sweep("enospc-write", "write", Effect::Fail("ENOSPC"), false);
}

/// Every `write` of the wrapper and of the Git processes it starts fails with no space left on the device.
#[test]
fn a_full_disk_at_any_write_of_the_wrapper_or_git_leaves_a_recoverable_repository() {
    sweep("enospc-write-tree", "write", Effect::Fail("ENOSPC"), true);
}

/// Every directory creation fails with no space left (no free inode or no room for the entry).
#[test]
fn a_full_disk_at_any_directory_creation_leaves_a_recoverable_repository() {
    sweep("enospc-mkdir", "mkdir", Effect::Fail("ENOSPC"), false);
}

/// Every rename, the step that publishes a record or retires a transaction, fails with no space left.
#[test]
fn a_full_disk_at_any_rename_leaves_a_recoverable_repository() {
    sweep("enospc-rename", "rename", Effect::Fail("ENOSPC"), false);
}

/// Every symbolic link creation fails with no space left.
#[test]
fn a_full_disk_at_any_symbolic_link_leaves_a_recoverable_repository() {
    sweep("enospc-symlink", "symlink", Effect::Fail("ENOSPC"), false);
}

/// Every hard link creation fails with no space left.
#[test]
fn a_full_disk_at_any_hard_link_leaves_a_recoverable_repository() {
    sweep("enospc-linkat", "linkat", Effect::Fail("ENOSPC"), false);
}

/// Every flush fails with an input or output error, the way a disk that lost cached data reports it.
#[test]
fn a_failed_flush_at_any_sync_leaves_a_recoverable_repository() {
    sweep("eio-fsync", "fsync", Effect::Fail("EIO"), false);
}
