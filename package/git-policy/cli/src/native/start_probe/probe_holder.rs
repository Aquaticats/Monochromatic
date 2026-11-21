//! What:
//!  Arm 6:
//!  the mode's own child that unshares a mount namespace of its own,
//!  makes it private and attaches the same source read-only there,
//!  with the plain create rather than the exclusive one.
//! Why:
//!  The namespace arm needs a second namespace that really holds the source
//!  while the worker's mount exists,
//!  and reusing the worker's superblock is exactly what this arm wants
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms").
//!  The mode forks it rather than the worker,
//!  so it inherits no descriptor on the worker's mount and no mount it did not
//!  make,
//!  and its own pipe ends and parent-death signal keep it from outliving the
//!  thread that forked it.
//!  It writes one line per step,
//!  waits on its control pipe until the mode stops it,
//!  and ends in `_exit`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! unshare(CLONE_NEWNS); private(); attach(source, holder, { exclusive: false }); report(); waitForStop(); _exit(0);
//! ```

use super::probe_arms::ProbeStep;
use super::probe_child_prelude::{parent_death_signal, parent_id, CHILD_FIRST_KEPT_DESCRIPTOR};
use super::probe_descriptor_calls::{NODEV_OPTION, NOSUID_OPTION, READ_ONLY_OPTION, SOURCE_OPTION, ZFS_FILESYSTEM, fsconfig_create, fsconfig_flag, fsconfig_text, fsopen, fsmount, move_mount_onto_descriptor};
use super::probe_mount_calls::{make_mounts_private, mount_id_of, read_own_mount_table, unshare_mount_namespace};
use super::probe_pipe_lines::write_all;
use super::probe_status_line::StepLine;
use std::os::fd::{AsRawFd, RawFd};
use std::path::Path;

/// What:
///  Attach the source in this child's own namespace,
///  report the step,
///  and wait for the mode's stop byte.
/// Why:
///  The holder's mount must exist while the worker scans and be gone after,
///  so its life is the mode's decision and its steps are two lines.
///
/// Never returns:
///  the child ends itself after the stop byte or on any failure.
pub fn run(control: RawFd, status: RawFd, source: &str, holder: &Path) -> ! {
    let ending = |line: StepLine| -> ! {
        let _written = write_all(status, line.encode().as_bytes());
        crate::start_probe::probe_child_prelude::exit_now(1);
    };
    if parent_death_signal().is_err() || parent_id() == 1 {
        crate::start_probe::probe_child_prelude::exit_now(1);
    }
    let _ = super::probe_child_prelude::close_from(CHILD_FIRST_KEPT_DESCRIPTOR);
    if let Err(error) = unshare_mount_namespace() {
        ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno)));
    }
    if let Err(error) = make_mounts_private() {
        ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno)));
    }
    let context = match fsopen(ZFS_FILESYSTEM) {
        Ok(context) => context,
        Err(error) => ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno))),
    };
    if let Err(error) = fsconfig_text(context.as_raw_fd(), SOURCE_OPTION, source) {
        ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno)));
    }
    for option in [READ_ONLY_OPTION, NOSUID_OPTION, NODEV_OPTION] {
        if let Err(error) = fsconfig_flag(context.as_raw_fd(), option) {
            ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno)));
        }
    }
    if let Err(error) = fsconfig_create(context.as_raw_fd(), false) {
        ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno)));
    }
    let mount = match fsmount(context.as_raw_fd()) {
        Ok(mount) => mount,
        Err(error) => ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno))),
    };
    let mount_id: u64 = mount_id_of(mount.as_raw_fd()).unwrap_or(0);
    let held: std::os::fd::OwnedFd = match super::probe_open_beneath::open_path_handle(holder) {
        Ok(held) => held,
        Err(error) => ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno))),
    };
    if let Err(error) = move_mount_onto_descriptor(mount.as_raw_fd(), held.as_raw_fd()) {
        ending(StepLine::failed_number(ProbeStep::HolderAttach, Some(error.errno)));
    }
    drop(mount);
    let listed: bool = match read_own_mount_table() {
        Ok(table) => table.iter().any(|facts| return facts.mount_id == mount_id && facts.source == source && facts.filesystem_type == ZFS_FILESYSTEM),
        Err(_) => false,
    };
    if !listed {
        ending(StepLine::failed(ProbeStep::HolderAttach, None));
    }
    let _attach_written = write_all(status, StepLine::passed(ProbeStep::HolderAttach).encode().as_bytes());
    let mut byte: [u8; 1] = [0];
    // SAFETY: the buffer is this call's own and the call writes at most its length; the read waits for the mode's stop.
    let _read: libc::ssize_t = unsafe { libc::read(control, byte.as_mut_ptr().cast::<libc::c_void>(), 1) };
    let _wait_written = write_all(status, StepLine::passed(ProbeStep::HolderWait).encode().as_bytes());
    crate::start_probe::probe_child_prelude::exit_now(0);
}
