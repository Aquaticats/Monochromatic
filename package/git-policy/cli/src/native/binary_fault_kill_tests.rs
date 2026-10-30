//! What:
//!  The wrapper killed with `SIGKILL` at each call of an explicit-path commit that writes, creates, renames, removes
//!  or syncs.
//! Why:
//!  The journals and recovery exist for a process killed at any instruction, and the phase markers of
//!  `binary_landing_crash_tests.rs` and the damage table of `binary_recovery_tearing_tests.rs` reach only the
//!  boundaries between phases.
//!  A kill at the `n`th call of a system call reaches every state a record can be in, including a file that was
//!  created and not yet written, a directory not yet renamed into place and a lock not yet released.
//!  After each kill the next ordinary command must succeed without a person repairing anything, the commit must be
//!  landed or not made, and nothing may stay behind (`binary_fault_support.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const n of range(1, countCalls('fsync'))) await killAtCallAndExpectRecovery({ call: 'fsync', n });
//! ```

use super::fault::{Effect, sweep};

/// A kill before each `write`: a record created and not yet written stays as an empty file.
#[test]
fn a_kill_at_any_write_leaves_a_recoverable_repository() {
    sweep("kill-write", "write", Effect::Kill, false);
}

/// A kill of the wrapper or of a Git process it started, before a `write`: the process group of a closing terminal
///  or an out-of-memory kill takes either.
#[test]
fn a_kill_of_the_wrapper_or_git_at_any_write_leaves_a_recoverable_repository() {
    sweep("kill-write-tree", "write", Effect::Kill, true);
}

/// A kill of the wrapper or of a Git process it started, before a flush.
#[test]
fn a_kill_of_the_wrapper_or_git_at_any_sync_leaves_a_recoverable_repository() {
    sweep("kill-fsync-tree", "fsync", Effect::Kill, true);
}

/// A kill before each flush: every journal barrier of the commit is a place the process can die.
#[test]
fn a_kill_at_any_sync_leaves_a_recoverable_repository() {
    sweep("kill-fsync", "fsync", Effect::Kill, false);
}

/// A kill before each rename: a record, a transaction directory or a lock published or retired half way.
#[test]
fn a_kill_at_any_rename_leaves_a_recoverable_repository() {
    sweep("kill-rename", "rename", Effect::Kill, false);
}

/// A kill before each directory creation.
#[test]
fn a_kill_at_any_directory_creation_leaves_a_recoverable_repository() {
    sweep("kill-mkdir", "mkdir", Effect::Kill, false);
}

/// A kill before each removal of a directory or file at a path relative to a directory handle: the cleanup after a landing.
#[test]
fn a_kill_at_any_removal_leaves_a_recoverable_repository() {
    sweep("kill-unlinkat", "unlinkat", Effect::Kill, false);
}

/// A kill before each removal of a file.
#[test]
fn a_kill_at_any_file_removal_leaves_a_recoverable_repository() {
    sweep("kill-unlink", "unlink", Effect::Kill, false);
}
