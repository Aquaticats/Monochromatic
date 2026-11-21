//! What:
//!  Arm 4's attach and arm 5's read-back:
//!  the context,
//!  the source,
//!  the read-only options,
//!  the exclusive create,
//!  the mount descriptor,
//!  its move onto the held slot descriptor,
//!  and the flags,
//!  source,
//!  filesystem and entries read back.
//! Why:
//!  The exclusive create is the freshness oracle:
//!  a matched extant superblock answers `EBUSY` before the dataset's mount
//!  path runs,
//!  and a kernel older than 6.6 refuses the command with `EOPNOTSUPP`,
//!  which the record keeps as a step that could not run
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms" and "Limits").
//!  The read-back proves the mount through the table,
//!  `fstatfs` and a bounded directory read,
//!  and never reads a file's content.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const attached = attachSnapshot(slotDescriptor, 'pool/dataset@ready'); // { mountId, mount }
//! ```

use super::probe_attach_verify::{AttachEvidence, verify_after_move};
use super::probe_call_error::CallError;
use super::probe_descriptor_calls::{NODEV_OPTION, NOSUID_OPTION, READ_ONLY_OPTION, SOURCE_OPTION, ZFS_FILESYSTEM, fsconfig_create, fsconfig_flag, fsconfig_text, fsopen, fsmount, move_mount_onto_descriptor};
use super::probe_mount_calls::mount_id_of;
use super::probe_errno_names::errno_of;
use std::os::fd::AsRawFd;

/// What:
///  Why the attach stopped,
///  naming the step and the errno.
/// Why:
///  The worker writes one status line per step,
///  so the answer is the step that failed and the errno it answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachFailure = { step: Step; errno?: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachFailure {
    /// The step that failed.
    pub step: super::probe_arms::ProbeStep,
    /// The errno the call answered,
    /// where it answered one.
    pub errno: Option<i32>,
}

impl AttachFailure {
    /// This failure as a status line's answer.
    pub fn errno_name(&self) -> Option<&'static str> {
        return self.errno.and_then(errno_of);
    }
}

/// The step one call is,
/// so a failure names it.
fn failed(step: super::probe_arms::ProbeStep, error: CallError) -> AttachFailure {
    return AttachFailure { step, errno: Some(error.errno) };
}

/// What:
///  Attach the configured source read-only through the held slot descriptor.
/// Why:
///  Every call is one step,
///  and the order is the design's:
///  context,
///  source,
///  options,
///  exclusive create,
///  mount descriptor,
///  move.
///
/// # Errors
/// A call failed,
///  the read-back refused,
///  or the mount could not be read through `fstatfs`.
pub fn attach_snapshot(slot: &std::os::fd::OwnedFd, slot_path: &std::path::Path, source: &str) -> Result<AttachEvidence, AttachFailure> {
    use super::probe_arms::ProbeStep;
    let context = fsopen(ZFS_FILESYSTEM).map_err(|error| return failed(ProbeStep::AttachContext, error))?;
    fsconfig_text(context.as_raw_fd(), SOURCE_OPTION, source).map_err(|error| return failed(ProbeStep::AttachSource, error))?;
    for option in [READ_ONLY_OPTION, NOSUID_OPTION, NODEV_OPTION] {
        fsconfig_flag(context.as_raw_fd(), option).map_err(|error| return failed(ProbeStep::AttachOptions, error))?;
    }
    fsconfig_create(context.as_raw_fd(), true).map_err(|error| return failed(ProbeStep::AttachExclusiveCreate, error))?;
    let mount = fsmount(context.as_raw_fd()).map_err(|error| return failed(ProbeStep::AttachMountDescriptor, error))?;
    let mount_id: u64 = match mount_id_of(mount.as_raw_fd()) {
        Ok(number) => number,
        Err(_) => return Err(AttachFailure { step: ProbeStep::AttachMountDescriptor, errno: Some(libc::ENOENT) }),
    };
    move_mount_onto_descriptor(mount.as_raw_fd(), slot.as_raw_fd()).map_err(|error| return failed(ProbeStep::AttachPlace, error))?;
    // The mount descriptor holds a reference on the mount, and a reference the worker keeps would make the unmount arm answer EBUSY.
    drop(mount);
    return verify_after_move(slot_path, mount_id, source).map_err(|failure| return AttachFailure { step: ProbeStep::VerifyMount, errno: failure.errno });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_attach_arm_tests.rs"]
mod tests;
