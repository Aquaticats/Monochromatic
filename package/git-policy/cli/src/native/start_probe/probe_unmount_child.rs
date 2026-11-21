//! What:
//!  Arm 8's second process:
//!  the child that moves the mount from the slot to the move target and
//!  unmounts it there by path,
//!  which must not answer `EBUSY`.
//! Why:
//!  A child's close cannot release what the worker holds,
//!  so the worker closes every descriptor on the mount before this fork,
//!  and the child then holds nothing of the mount when it unmounts
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms").
//!  The child sets its own parent-death signal and rechecks the parent that
//!  signal follows,
//!  so it goes when the worker dies,
//!  and it reports each step on the pipe its worker owns.
//!  The child reads the mount table rather than the mounted tree,
//!  which is how it verifies the move without holding the mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! moveMountBetween(slot, moveTarget); verifyMountPoint(moveTarget); umount2(moveTarget, 0); _exit(0);
//! ```

use super::probe_arms::ProbeStep;
use super::probe_child_prelude::{parent_death_signal, parent_id, CHILD_FIRST_KEPT_DESCRIPTOR};
use super::probe_descriptor_calls::move_mount_between_paths;
use super::probe_mount_calls::{read_own_mount_table, unmount};
use super::probe_pipe_lines::write_all;
use super::probe_status_line::StepLine;
use std::path::Path;

/// What:
///  Run the move and the unmount,
///  writing one line per step and ending in `_exit`.
/// Why:
///  Every step's line reaches the worker,
///  which relays it to the mode,
///  so a timed-out record can name which of the three was in flight.
///
/// # Errors
/// Never returns:
///  the child ends itself on every path.
pub fn run(status: std::os::fd::RawFd, slot: &Path, move_target: &Path, expected_mount_id: u64, hold: Option<ProbeStep>) -> ! {
    if parent_death_signal().is_err() || parent_id() == 1 {
        crate::start_probe::probe_child_prelude::exit_now(1);
    }
    let _ = super::probe_child_prelude::close_from(CHILD_FIRST_KEPT_DESCRIPTOR);
    match move_mount_between_paths(slot, move_target) {
        Ok(()) => {
            say(status, StepLine::passed(ProbeStep::Move));
            if hold == Some(ProbeStep::Move) {
                std::thread::sleep(super::probe_deadline::PROBE_DEADLINE * 2);
            }
        }
        Err(error) => {
            say(status, StepLine::failed_number(ProbeStep::Move, Some(error.errno)));
            crate::start_probe::probe_child_prelude::exit_now(1);
        }
    }
    let placed: bool = match read_own_mount_table() {
        Ok(table) => table.iter().any(|facts| return facts.mount_id == expected_mount_id && facts.mount_point == move_target),
        Err(_) => false,
    };
    if !placed {
        say(status, StepLine::failed(ProbeStep::Unmount, None));
        crate::start_probe::probe_child_prelude::exit_now(1);
    }
    if hold == Some(ProbeStep::Unmount) {
        std::thread::sleep(super::probe_deadline::PROBE_DEADLINE * 2);
    }
    match unmount(move_target) {
        Ok(()) => say(status, StepLine::passed(ProbeStep::Unmount)),
        Err(error) => {
            say(status, StepLine::failed_number(ProbeStep::Unmount, Some(error.errno)));
            crate::start_probe::probe_child_prelude::exit_now(1);
        }
    }
    crate::start_probe::probe_child_prelude::exit_now(0);
}

/// Write one step's line,
/// exactly one per step.
fn say(status: std::os::fd::RawFd, line: StepLine) {
    let _written = write_all(status, line.encode().as_bytes());
}
