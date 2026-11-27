//! What:
//!  Arm 8's forked child and the worker's side of it:
//!  the two pipes it is given,
//!  the relay that passes each of its step lines on as the child writes them,
//!  and the wait that ends it.
//! Why:
//!  The child is the process that moves the mount and unmounts it,
//!  so the mode can name arm 8's failing step only if that step's line
//!  reaches it before the next step starts,
//!  and the worker relays while the child lives rather than after it ends.
//!  The child stays in the worker's process group,
//!  so the mode's group kill at the deadline reaches it too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const child = forkUnmountChild(slot, moveTarget, mountId, hold); const failed = child.relayUntilEof(status); waitBlocking(child.pid);
//! ```

use super::probe_arms::ProbeStep;
use super::probe_child_prelude::{
    CHILD_CONTROL_DESCRIPTOR,
    CHILD_STATUS_DESCRIPTOR,
    close_from,
    move_descriptor,
};
use super::probe_pipe_lines::read_one_blocking;
use super::probe_processes::{
    fork,
    pipe_pair,
    raw,
};
use super::probe_status_line::{
    LineFeed,
    StepAnswer,
    StepLine,
};
use super::probe_worker_step_lines::say;
use std::os::fd::{
    OwnedFd,
    RawFd,
};
use std::path::Path;

/// One forked unmount child and its status pipe's read end.
pub(super) struct UnmountChild {
    /// The child's pid.
    pub(super) pid: libc::pid_t,
    /// The read end of the child's status pipe.
    reports: OwnedFd,
    /// The write end of the child's control pipe,
    /// kept open so a hold in the child waits rather than reading its end.
    _control: OwnedFd,
}

impl UnmountChild {
    /// What:
    ///  Relay every line the child writes,
    ///  as it writes it,
    ///  and answer whether one of them failed.
    /// Why:
    ///  The mode can name which of arm 8's steps was in flight only if the
    ///  step's line reaches it before the next step starts,
    ///  so the worker relays while the child lives rather than after it ends.
    ///  The child holds its status pipe open until `_exit`,
    ///  so the end of the pipe is the child's own end.
    pub(super) fn relay_until_eof(
        &mut self,
        status: RawFd,
    ) -> bool {
        let mut feed: LineFeed = LineFeed::new();
        let mut failed: bool = false;
        loop {
            let byte: u8 = match read_one_blocking(raw(&self.reports)) {
                Ok(byte) => byte,
                Err(_) => break,
            };
            for parsed in feed.push(&[byte]) {
                match parsed {
                    Ok(step_line) => {
                        if matches!(step_line.answer, StepAnswer::Failed { .. }) {
                            failed = true;
                        }
                        say(status, step_line);
                    }
                    Err(_refusal) => say(status, StepLine::failed(ProbeStep::Move, None)),
                }
            }
        }
        return failed;
    }
}

/// Fork the unmount child with its two pipes.
pub(super) fn fork_unmount_child(
    slot: &Path,
    move_target: &Path,
    mount_id: u64,
    hold: Option<ProbeStep>,
) -> Result<UnmountChild, Option<&'static str>> {
    let control = pipe_pair().map_err(|error| return error.errno_name())?;
    let reports = pipe_pair().map_err(|error| return error.errno_name())?;
    match fork() {
        Err(error) => return Err(error.errno_name()),
        Ok(None) => {
            let _ = move_descriptor(raw(&control.read), CHILD_CONTROL_DESCRIPTOR);
            let _ = move_descriptor(raw(&reports.write), CHILD_STATUS_DESCRIPTOR);
            let _ = close_from(5);
            super::probe_unmount_child::run(
                CHILD_STATUS_DESCRIPTOR,
                slot,
                move_target,
                mount_id,
                hold,
            );
        }
        Ok(Some(pid)) => {
            // The child stays in the worker's own process group,
            // so the mode's group kill at the deadline reaches it too.
            drop(control.read);
            drop(reports.write);
            return Ok(UnmountChild {
                pid,
                reports: reports.read,
                _control: control.write,
            });
        }
    }
}
