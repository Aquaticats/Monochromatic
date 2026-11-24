//! What:
//!  The companion's child:
//!  its own process group and parent-death signal,
//!  its descriptor sweep,
//!  every step in the design's order,
//!  and the record each step writes.
//! Why:
//!  The child holds only the thread that forked it,
//!  so it runs fixed steps over buffers its parent made and ends in `_exit` on
//!  every path
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "The child starts").
//!  A shipped build makes no write in the host's mount namespace:
//!  the scratch tmpfs and the attach are both inside the child's own
//!  namespaces,
//! and they go when the child ends.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! run() { ownGroup(); pdeathsig(); sweep(); step('user-namespace', () => unshare(CLONE_NEWUSER)); ... step('move-unmount', () => moveAndUnmount()); _exit(0); }
//! ```

use super::companion_buffers::CompanionBuffers;
use super::companion_calls::{CompanionCall, Failure};
use super::companion_child_attach::{attach, move_and_unmount};
use super::companion_child_controls::run_the_controls;
use super::companion_child_tree::{Slots, make_the_tree, walk_to_the_slot};
use super::companion_file_calls::{close, close_range, exit_now, move_descriptor, open_for_write, parent_death_signal, parent_id, set_process_group, unshare_mount_namespace, unshare_user_namespace, write_all};
use super::companion_mount_calls::{make_mounts_private, mount_tmpfs};
use super::companion_records::{StepRecord, send};
use super::companion_steps::CompanionStep;
use std::os::fd::RawFd;

/// The descriptor a child's status write end is moved to,
/// below the range its sweep closes.
pub const CHILD_STATUS_DESCRIPTOR: RawFd = 4;

/// What:
///  What the parent hands the child.
/// Why:
///  The child reads no argument and no environment,
///  so a caller cannot aim it at another directory or another host state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildStart = { buffers: Buffers; status: number };
/// ```
pub struct ChildStart<'a> {
    /// Every name and line the child writes,
    /// made before the fork.
    pub buffers: &'a CompanionBuffers,
    /// The write end of the status pipe,
    /// as the child received it.
    pub status: RawFd,
}

/// What:
///  Run every step,
///  announcing each one,
///  and end this process.
/// Why:
///  The parent reads one record per step and judges the run from them,
///  so a step that is missing is a failure the parent can name.
///
/// Never returns:
///  the child ends after its last step or its first failed one.
pub fn run(start: &ChildStart<'_>, mount_table: &mut [u8]) -> ! {
    let status: RawFd = CHILD_STATUS_DESCRIPTOR;
    own_process(start.status);
    let buffers: &CompanionBuffers = start.buffers;
    check(status, buffers, CompanionStep::UserNamespace, unshare_user_namespace);
    write_one(status, buffers, CompanionStep::Setgroups, buffers.setgroups.as_c_str(), b"deny", CompanionCall::OpenSetgroups, CompanionCall::WriteSetgroups);
    write_one(status, buffers, CompanionStep::UidMap, buffers.uid_map.as_c_str(), buffers.uid_line.as_bytes(), CompanionCall::OpenUidMap, CompanionCall::WriteUidMap);
    write_one(status, buffers, CompanionStep::GidMap, buffers.gid_map.as_c_str(), buffers.gid_line.as_bytes(), CompanionCall::OpenGidMap, CompanionCall::WriteGidMap);
    check(status, buffers, CompanionStep::MountNamespace, unshare_mount_namespace);
    check(status, buffers, CompanionStep::PrivatePropagation, || return make_mounts_private(buffers.root.as_c_str()));
    check(status, buffers, CompanionStep::ScratchTmpfs, || return mount_tmpfs(buffers.tmpfs.as_c_str(), buffers.scratch.as_c_str(), buffers.tmpfs_options.as_c_str()));
    check(status, buffers, CompanionStep::ScratchTree, || return make_the_tree(buffers));
    let slots: Slots = guarded(status, buffers, CompanionStep::Walk, || return walk_to_the_slot(buffers));
    pass(status, buffers, CompanionStep::Walk);
    let mount_id: u64 = guarded(status, buffers, CompanionStep::Attach, || return attach(buffers, slots, mount_table));
    pass(status, buffers, CompanionStep::Attach);
    pass(status, buffers, CompanionStep::MountFacts);
    check(status, buffers, CompanionStep::RefusalControls, || return run_the_controls(buffers, slots));
    match move_and_unmount(buffers, mount_id, mount_table, status) {
        Ok(()) => exit_now(0),
        Err(failure) => end_failed(status, buffers, CompanionStep::MoveUnmount, failure),
    }
}

/// What:
///  Put this child in a group of its own,
///  carry its parent's death,
///  keep its status descriptor and drop every other one.
/// Why:
///  The group is what the deadline kills,
///  the death signal is what an outer kill of the wrapper leaves nothing
///  behind through,
///  and the sweep is what keeps a copy of another thread's pipe or lock out of
///  this child.
fn own_process(status: RawFd) {
    let _group: Result<(), Failure> = set_process_group(0);
    let _death: Result<(), Failure> = parent_death_signal();
    if parent_id() == 1 {
        exit_now(1);
    }
    let _kept: Result<(), Failure> = move_descriptor(status, CHILD_STATUS_DESCRIPTOR);
    let _closed_three: Result<(), Failure> = close_range(3, 3);
    let first_closed: u32 = u32::try_from(CHILD_STATUS_DESCRIPTOR + 1).unwrap_or(5);
    let _swept: Result<(), Failure> = close_range(first_closed, u32::MAX);
}

/// What:
///  Run one step's call,
///  announcing its outcome,
///  and end the child at its first failure.
/// Why:
///  Each step has one call and one record,
///  so the sequence reads as the design's list,
///  and the test-only fixture answers before the real call is made.
fn check(status: RawFd, buffers: &CompanionBuffers, step: CompanionStep, call: impl FnOnce() -> Result<(), Failure>) {
    if let Some(failure) = fixture_at(buffers, step) {
        end_failed(status, buffers, step, failure);
    }
    match call() {
        Ok(()) => pass(status, buffers, step),
        Err(failure) => end_failed(status, buffers, step, failure),
    }
}

/// What:
///  Run one step that answers a value,
///  and end the child at its failure,
///  which is how the walk,
/// the attach and the controls report.
/// Why:
///  A step with a value cannot use `check`,
///  and ending from inside a match arm keeps one shape for both.
fn guarded<T>(status: RawFd, buffers: &CompanionBuffers, step: CompanionStep, call: impl FnOnce() -> Result<T, Failure>) -> T {
    if let Some(failure) = fixture_at(buffers, step) {
        end_failed(status, buffers, step, failure);
    }
    return match call() {
        Ok(value) => value,
        Err(failure) => end_failed(status, buffers, step, failure),
    };
}

/// Write one step's passing record,
/// and sleep after it where the deadline arm asked.
fn pass(status: RawFd, buffers: &CompanionBuffers, step: CompanionStep) {
    send(status, StepRecord::passed(step));
    if buffers.hold == Some(step) {
        // Twice the shipped bound keeps the step long enough for the deadline to fire, and the wait is a sleep rather than a spin.
        let mut watched: [libc::pollfd; 0] = [];
        let _slept: libc::c_int = unsafe { libc::poll(watched.as_mut_ptr(), 0, 4_000) };
    }
}

/// What:
///  Write one failed step's record and end the child.
fn end_failed(status: RawFd, buffers: &CompanionBuffers, step: CompanionStep, failure: Failure) -> ! {
    let _ = buffers;
    send(status, StepRecord::failed(step, failure));
    exit_now(1);
}

/// What:
///  The failure one test-only fixture answers at this step,
///  or nothing.
/// Why:
///  M11's fixtures need a seam no syscall filter can give,
///  because a filter names a system call and cannot single out one write of
///  three.
fn fixture_at(buffers: &CompanionBuffers, step: CompanionStep) -> Option<Failure> {
    let fixture = buffers.fixture?;
    if fixture.step != step {
        return None;
    }
    return Some(Failure { call: first_call_of(step), errno: fixture.errno });
}

/// The call a step's record names when a fixture answers instead of it.
fn first_call_of(step: CompanionStep) -> CompanionCall {
    return match step {
        CompanionStep::UserNamespace => CompanionCall::UnshareUserNamespace,
        CompanionStep::Setgroups => CompanionCall::WriteSetgroups,
        CompanionStep::UidMap => CompanionCall::WriteUidMap,
        CompanionStep::GidMap => CompanionCall::WriteGidMap,
        CompanionStep::MountNamespace => CompanionCall::UnshareMountNamespace,
        CompanionStep::PrivatePropagation => CompanionCall::MountSetattr,
        CompanionStep::ScratchTmpfs => CompanionCall::MountTmpfs,
        CompanionStep::ScratchTree => CompanionCall::Mkdir,
        CompanionStep::Walk => CompanionCall::Openat2,
        CompanionStep::Attach => CompanionCall::FsMount,
        CompanionStep::MountFacts => CompanionCall::ReadMountinfo,
        CompanionStep::RefusalControls => CompanionCall::Openat2,
        CompanionStep::MoveUnmount => CompanionCall::Umount,
    };
}

/// What:
///  Write one mapping line:
///  open the file,
/// write the whole line,
/// close it.
/// Why:
///  The three setup writes take one shape,
///  and each names its own open and write calls so a skip names the right one.
fn write_one(status: RawFd, buffers: &CompanionBuffers, step: CompanionStep, path: &std::ffi::CStr, line: &[u8], open: CompanionCall, write: CompanionCall) {
    if let Some(failure) = fixture_at(buffers, step) {
        end_failed(status, buffers, step, failure);
    }
    let opened: RawFd = match open_for_write(path, open) {
        Ok(opened) => opened,
        Err(failure) => end_failed(status, buffers, step, failure),
    };
    let written: Result<(), Failure> = write_all(opened, line, write);
    close(opened);
    match written {
        Ok(()) => pass(status, buffers, step),
        Err(failure) => end_failed(status, buffers, step, failure),
    }
}
