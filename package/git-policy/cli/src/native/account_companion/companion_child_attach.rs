//! What:
//!  The companion's descriptor attach onto its own slot,
//!  the mount facts read back after it,
//!  and the second child that moves the mount and unmounts it.
//! Why:
//!  The design attaches a second tmpfs through a file-system context and lands
//!  it on the descriptor the walk proved,
//!  then reads the kernel's own table to prove the mount is where it says
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "What it runs").
//!  The move and the unmount are a child of the child,
//!  which is safe because the first child is single-threaded by then,
//!  and the outer deadline kills the whole group it belongs to.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const id = mountOn(fsopen('tmpfs'), slot); expect(mountinfo).toName(id, slot);
//! spawn(() => { moveMount(slot, moveTarget); umount(moveTarget); expect(mountinfo).not.toName(id); });
//! ```

use super::companion_buffers::CompanionBuffers;
use super::companion_calls::{CompanionCall, Failure, NO_ERRNO, failed};
use super::companion_child_tree::Slots;
use super::companion_descriptor_calls::{fsconfig_create, fsmount, fsopen_tmpfs, move_mount_between_paths, move_mount_onto_descriptor};
use super::companion_mount_table::{mount_id_of, names_mount, names_mount_id, read_mount_table};
use super::companion_file_calls::{close, parent_death_signal};
use super::companion_mount_calls::unmount;
use super::companion_records::{StepRecord, send};
use super::companion_steps::CompanionStep;
use std::os::fd::RawFd;

/// What:
///  Attach a second tmpfs onto the held slot and prove it from the kernel's
///  table.
/// Why:
///  This is the arm whose descriptor calls the design's gate names,
///  and a mount that is not in the table where the descriptor put it is a
///  failure rather than a pass.
///
/// # Errors
/// A descriptor call refused,
///  or the read-back did not find the mount.
pub fn attach(buffers: &CompanionBuffers, slots: Slots, mount_table: &mut [u8]) -> Result<u64, Failure> {
    let context: RawFd = fsopen_tmpfs(buffers.tmpfs.as_c_str())?;
    let created: Result<(), Failure> = fsconfig_create(context);
    if created.is_err() {
        close(context);
        created?;
    }
    let mount: RawFd = fsmount(context)?;
    close(context);
    let mount_id: u64 = mount_id_of(mount)?;
    let moved: Result<(), Failure> = move_mount_onto_descriptor(mount, slots.slot);
    close(mount);
    moved?;
    let read: usize = read_mount_table(buffers.mountinfo.as_c_str(), mount_table)?;
    if !names_mount(&mount_table[..read], mount_id, buffers.slot.as_bytes()) {
        return Err(Failure { call: CompanionCall::MountFactsCheck, errno: NO_ERRNO });
    }
    return Ok(mount_id);
}

/// What:
///  Fork the child that moves the mounted slot and unmounts it,
///  and wait for it.
/// Why:
///  The mount is moved and unmounted by a process of its own,
///  and the outer deadline's group kill reaches it because it keeps the
///  group it was forked in.
///
/// # Errors
/// The fork refused,
///  or the child reported a failure of its own.
pub fn move_and_unmount(buffers: &CompanionBuffers, mount_id: u64, mount_table: &mut [u8], status: RawFd) -> Result<(), Failure> {
    // SAFETY: `fork` takes no argument, and this process is single-threaded by now.
    let child: libc::pid_t = unsafe { libc::fork() };
    if child < 0 {
        return Err(failed(CompanionCall::Fork));
    }
    if child == 0 {
        let ended: StepRecord = match child_move(buffers, mount_id, mount_table) {
            Ok(()) => StepRecord::passed(CompanionStep::MoveUnmount),
            Err(failure) => StepRecord::failed(CompanionStep::MoveUnmount, failure),
        };
        send(status, ended);
        super::companion_file_calls::exit_now(if ended.failure.is_none() { 0 } else { 1 });
    }
    let answered: Result<(), Failure> = wait_for(child);
    return answered;
}

/// What the second child does:
///  move the mount to the move target,
///  unmount it there,
///  and require the table to name it no more.
/// Why:
///  The unmount must not answer `EBUSY`,
///  and the table read is what proves the mount is gone rather than detached
///  from this path only.
fn child_move(buffers: &CompanionBuffers, mount_id: u64, mount_table: &mut [u8]) -> Result<(), Failure> {
    parent_death_signal()?;
    move_mount_between_paths(buffers.slot.as_c_str(), buffers.move_target.as_c_str())?;
    unmount(buffers.move_target.as_c_str())?;
    let read: usize = read_mount_table(buffers.mountinfo.as_c_str(), mount_table)?;
    if names_mount_id(&mount_table[..read], mount_id) {
        return Err(Failure { call: CompanionCall::MountFactsCheck, errno: NO_ERRNO });
    }
    return Ok(());
}

/// What:
///  Wait for one child,
///  answering the failure its exit status means.
/// Why:
///  The child wrote its own record,
///  so this only says whether the run goes on.
fn wait_for(child: libc::pid_t) -> Result<(), Failure> {
    let mut status: libc::c_int = 0;
    loop {
        // SAFETY: the status word is this call's own and `waitpid` writes it.
        let waited: libc::pid_t = unsafe { libc::waitpid(child, &mut status, 0) };
        if waited < 0 {
            let refusal: Failure = failed(CompanionCall::Fork);
            if refusal.errno == libc::EINTR {
                continue;
            }
            return Err(refusal);
        }
        break;
    }
    if libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0 {
        return Ok(());
    }
    return Err(Failure { call: CompanionCall::Fork, errno: NO_ERRNO });
}
