//! What:
//!  One probe of the account companion's `setgroups` step,
//!  for `doctor`:
//!  the child that makes the companion's own first two calls,
//!  the parent's bounded wait,
//!  and what the answer means for a host.
//! Why:
//!  The companion's `deny` write is refused on some hosts before it is reached,
//!  at the open of `/proc/self/setgroups` for write,
//!  which asks `CAP_SYS_ADMIN` inside the new user namespace
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "The account companion's group drop on Ubuntu 24.04").
//!  A person cannot see that from outside,
//!  and `doctor` may not write anything or claim a slot,
//!  so the fact is measured the way an add measures it:
//!  a child of its own makes `unshare(CLONE_NEWUSER)`,
//!  opens the file for write,
//!  writes `deny`,
//!  and answers over a pipe in the companion's own record shape.
//!  The probe touches no mount,
//!  no ZFS state and no file of the person's,
//!  and its child ends however the run goes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const drop = theGroupDrop(); // { kind: 'written' } | { kind: 'skipped', errno } | { kind: 'answered', call, errno } | { kind: 'unread', reason }
//! ```

use super::companion_calls::{
    CompanionCall,
    Failure,
};
use super::companion_deadline::COMPANION_DEADLINE;
use super::companion_file_calls::{
    close,
    exit_now,
    open_for_write,
    parent_death_signal,
    parent_id,
    unshare_user_namespace,
    write_all,
};
use super::companion_process_calls::{
    Exit,
    fork,
    kill_group,
    pid_descriptor,
    pipe_pair,
    read_available,
    reap_now,
    ready_within,
    set_process_group,
};
use super::companion_records::{
    Decoded,
    RECORD_BYTES,
    StepRecord,
    decode_all,
    send,
};
use super::companion_steps::CompanionStep;
use std::os::fd::{
    AsRawFd,
    OwnedFd,
};

/// How many bytes the status pipe can hold here:
/// one record for the namespace and one for the `setgroups` step.
const RECORD_ROOM: usize = 2 * RECORD_BYTES;

/// The word the child writes to deny the group drop.
const DENY: &[u8] = b"deny";

/// The file whose write open asks for the capability.
const SETGROUPS_PATH: &std::ffi::CStr = c"/proc/self/setgroups";

/// What:
///  What one probe of the companion's `setgroups` step answered.
/// Why:
///  `doctor` prints a line only for the skip the owner added on 2026-11-26,
///  and the other three answers are told apart so a control can hold each:
///  a host that wrote the `deny` needs no line,
///  a host that answered elsewhere reports it in the add's own line,
///  and a probe that could not be read must say so rather than pass for a
///  passing host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GroupDrop = { kind: 'written' } | { kind: 'skipped'; errno: number } | { kind: 'answered'; call: Call; errno: number } | { kind: 'unread'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupDrop {
    /// The `deny` write was made,
    /// so an add's companion passes this step on this host.
    Written,
    /// The open answered `EACCES`,
    /// which is the skip the owner added.
    Skipped {
        /// The number the open answered.
        errno: i32,
    },
    /// The host answered somewhere else,
    /// so its own line is the add's and this probe prints none.
    Answered {
        /// The call that answered.
        call: CompanionCall,
        /// The number it answered.
        errno: i32,
    },
    /// The probe could not be run or its answer could not be read.
    Unread {
        /// Why.
        reason: String,
    },
}

/// What:
///  Probe this host for the companion's group drop.
/// Why:
///  This is the read `doctor` makes,
///  and the controls hand their own answers instead, so no control forks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const drop = (() => { const [read, write] = pipe(); if (fork() === 0) child(write); return wait(read); })();
/// ```
pub fn the_group_drop() -> GroupDrop {
    let (reading, writing): (OwnedFd, OwnedFd) = match pipe_pair() {
        Ok(ends) => ends,
        Err(error) => {
            return GroupDrop::Unread {
                reason: format!(
                    "its status pipe could not be made (errno {})",
                    error.raw_os_error().unwrap_or(libc::EIO)
                ),
            };
        }
    };
    let child: libc::pid_t = match fork() {
        Ok(Some(pid)) => pid,
        Ok(None) => {
            // The child keeps the write end only, and ends from this call.
            close(reading.as_raw_fd());
            run_the_group_drop(writing.as_raw_fd());
        }
        Err(error) => {
            return GroupDrop::Unread {
                reason: format!(
                    "its child could not be started (errno {})",
                    error.raw_os_error().unwrap_or(libc::EAGAIN)
                ),
            };
        }
    };
    let _group = set_process_group(child);
    drop(writing);
    return wait_for_the_group_drop(child, &reading);
}

/// What:
///  Take a group and a parent's death of its own,
///  make the companion's first two calls in its order,
///  and end with one record per step.
/// Why:
///  The order is the fact being measured:
///  a host that refuses the open never reaches the write,
///  and a companion that wrote first would ask the kernel a different question.
///
/// Never returns:
///  the child ends after its second record.
fn run_the_group_drop(status: libc::c_int) -> ! {
    // The child takes its own group and its parent's death, as the companion's child does.
    let _group: Result<(), Failure> = super::companion_file_calls::set_process_group(0);
    let _death: Result<(), Failure> = parent_death_signal();
    if parent_id() == 1 {
        exit_now(1);
    }
    let namespace: Result<(), Failure> = unshare_user_namespace();
    match namespace {
        Ok(()) => send(status, StepRecord::passed(CompanionStep::UserNamespace)),
        Err(failure) => {
            send(
                status,
                StepRecord::failed(CompanionStep::UserNamespace, failure),
            );
            exit_now(1);
        }
    }
    let opened: libc::c_int =
        match open_for_write(SETGROUPS_PATH, CompanionCall::OpenSetgroups) {
            Ok(opened) => opened,
            Err(failure) => {
                send(
                    status,
                    StepRecord::failed(CompanionStep::Setgroups, failure),
                );
                exit_now(1);
            }
        };
    let written: Result<(), Failure> = write_all(opened, DENY, CompanionCall::WriteSetgroups);
    close(opened);
    match written {
        Ok(()) => {
            send(status, StepRecord::passed(CompanionStep::Setgroups));
            exit_now(0);
        }
        Err(failure) => {
            send(
                status,
                StepRecord::failed(CompanionStep::Setgroups, failure),
            );
            exit_now(1);
        }
    }
}

/// What:
///  Wait for the child under the companion's own bound,
///  read its records,
///  end it if it outlasted that bound,
///  and judge what it wrote.
/// Why:
///  `doctor` may not wait with no end,
///  and a child the kernel does not release is handed to the crate's one
///  reaper rather than waited on here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = poll(pidfd, COMPANION_DEADLINE) ? readAll(read) : (kill(-child), 'unread');
/// ```
fn wait_for_the_group_drop(
    child: libc::pid_t,
    reading: &OwnedFd,
) -> GroupDrop {
    let ready: bool = pid_descriptor(child)
        .and_then(|descriptor| return ready_within(&descriptor, COMPANION_DEADLINE))
        .unwrap_or_default();
    if !ready {
        let _killed = kill_group(child);
        if reap_now(child).unwrap_or(Exit::StillRunning) == Exit::StillRunning {
            // The crate's own reaper collects a child the kernel has not released, whenever it does.
            crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon_pid(child);
        }
        return GroupDrop::Unread {
            reason: format!(
                "it did not answer within {} ms",
                COMPANION_DEADLINE.as_millis()
            ),
        };
    }
    let mut bytes: [u8; RECORD_ROOM] = [0; RECORD_ROOM];
    let mut read: usize = 0;
    while read < bytes.len() {
        let Some(more) = read_available(reading, &mut bytes[read..]).ok() else {
            break;
        };
        if more == 0 {
            break;
        }
        read += more;
    }
    let ended: Exit = reap_now(child).unwrap_or(Exit::StillRunning);
    if ended == Exit::StillRunning {
        crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon_pid(child);
    }
    return the_answer(decode_all(&bytes[..read]));
}

/// What:
///  What the records the child wrote mean for this host.
/// Why:
///  The judgement is one function of the records,
///  so every answer is held by a control without a fork,
///  and only the step and call the owner named is the skip:
///  a refusal at any other step is that step's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failed = records.find((r) => r.failure); return !failed ? 'written' : skipped(failed) ?? 'answered';
/// ```
fn the_answer(decoded: Decoded) -> GroupDrop {
    if decoded.unreadable {
        return GroupDrop::Unread {
            reason: String::from("its child wrote an answer this build cannot read"),
        };
    }
    if decoded.records.is_empty() {
        return GroupDrop::Unread {
            reason: String::from("its child wrote no record"),
        };
    }
    let Some(failed_record) = decoded
        .records
        .iter()
        .find(|record| return record.failure.is_some())
    else {
        // Every record passed, so the child made both calls: the namespace and the deny write.
        if decoded.records.len() != 2 {
            return GroupDrop::Unread {
                reason: format!(
                    "its child wrote {} passing records where a whole run writes two",
                    decoded.records.len()
                ),
            };
        }
        return GroupDrop::Written;
    };
    let Some(failure) = failed_record.failure else {
        return GroupDrop::Unread {
            reason: String::from("a record was read as failed and holds no failure"),
        };
    };
    if failed_record.step == CompanionStep::Setgroups
        && failure.call == CompanionCall::OpenSetgroups
        && failure.errno == libc::EACCES
    {
        return GroupDrop::Skipped {
            errno: failure.errno,
        };
    }
    return GroupDrop::Answered {
        call: failure.call,
        errno: failure.errno,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_group_drop_probe_tests.rs"]
mod tests;
