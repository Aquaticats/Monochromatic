//! What:
//!  Arm 2's reads over the trees the probe owns:
//!  the no-crossing walk to the slot and the identity it records,
//!  the four refusal controls,
//!  and the read-only open of the checkout root.
//! Why:
//!  The walk the product uses is judged here over a tree nobody else can
//!  rename into,
//!  and each refusal must name the component or field the way the measured
//!  refusals did
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms" and M5).
//!  The success control sits beside every refusal:
//!  the slot the walk reaches is opened and its identity recorded,
//!  so a walk that refuses everything fails its own arm.
//!  Nothing here reads a directory entry of the checkout,
//!  which is what the read-only root open proves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const evidence = walkAndRefuse(scratch, slot, moveTarget, checkout); // { slotIdentity, checkoutIdentity }
//! ```

use super::probe_call_error::CallError;
use super::probe_open_beneath::open_directory_beneath;
use super::probe_errno_names::errno_of;
use crate::fs_beneath::{ObjectId, identity_of, open_directory_at, open_root, symlink_at};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The directory inside the scratch tmpfs the refusal controls plant in.
pub const REFUSAL_TREE: &str = "refusal";
/// The symbolic link that must refuse with `ELOOP`.
pub const LINK_NAME: &str = "link";
/// The folder the bind mount is planted on,
/// across a walk component.
pub const BOUND_NAME: &str = "bound";
/// The folder whose identity is recorded and then replaced.
pub const REPLACED_NAME: &str = "replaced";
/// The name the replaced folder is moved to,
/// which is how its identity changes under the recorded one.
pub const REPLACED_ELSEWHERE_NAME: &str = "replaced-elsewhere";

/// What:
///  What the walk and the refusals proved:
///  the slot's identity and the checkout root's.
/// Why:
///   The slot's identity is the one the changed-identity control compares
///  against,
///  and the checkout's says the root itself opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkEvidence = { slot: ObjectId; checkout: ObjectId };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WalkEvidence {
    /// The slot folder's identity.
    pub slot: ObjectId,
    /// The checkout root's identity.
    pub checkout: ObjectId,
}

/// What:
///  Why the walk or one of its controls did not answer as the design expects.
/// Why:
///   A refusal control that attached,
///  or refused with another errno,
///  is this arm's failure,
///  and the sentence names which control and what it answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkFailure = { kind: 'call' | 'control'; call: string; errno?: number; control?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WalkFailure {
    /// One call failed.
    Call {
        /// The call.
        call: &'static str,
        /// The errno it answered.
        errno: Option<i32>,
    },
    /// A refusal control did not refuse,
    /// or refused with another errno.
    Control {
        /// The control's name.
        control: &'static str,
        /// The errno it answered,
        /// where it answered one.
        errno: Option<i32>,
    },
}

impl WalkFailure {
    /// The errno's name,
    /// where the failure carries one this build names.
    pub fn errno_name(&self) -> Option<&'static str> {
        return match self {
            WalkFailure::Call { errno: Some(number), .. } | WalkFailure::Control { errno: Some(number), .. } => errno_of(*number),
            WalkFailure::Call { errno: None, .. } | WalkFailure::Control { errno: None, .. } => None,
        };
    }

    /// The failure in words,
    /// naming the control and what it answered.
    pub fn message(&self) -> String {
        return match self {
            WalkFailure::Call { call, errno } => format!("the probe's walk call `{call}` failed (errno {errno:?})."),
            WalkFailure::Control { control, errno } => format!("the probe's refusal control `{control}` did not refuse as designed (errno {errno:?})."),
        };
    }
}

/// The errno of one failed call,
/// as the refusal compares it.
fn errno_of_call(error: &std::io::Error) -> Option<i32> {
    return error.raw_os_error();
}

/// What:
///  Create the slot,
///  the move target and the refusal tree inside the scratch tmpfs.
/// Why:
///   Nothing the probe's children move or unmount exists before this,
///  and every name is inside the filesystem the probe owns.
///
/// # Errors
/// A directory could not be made.
pub fn make_scratch_tree(scratch: &Path, slot: &Path, move_target: &Path) -> Result<(), WalkFailure> {
    for path in [slot.to_path_buf(), move_target.to_path_buf()] {
        if let Err(error) = std::fs::create_dir_all(path.as_path()) {
            return Err(WalkFailure::Call { call: "mkdir", errno: errno_of_call(&error) });
        }
    }
    let refusal: PathBuf = scratch.join(REFUSAL_TREE);
    let bound: PathBuf = refusal.join(BOUND_NAME);
    let bound_slot: PathBuf = bound.join("slot");
    for path in [refusal.as_path(), bound.as_path(), bound_slot.as_path()] {
        if let Err(error) = std::fs::create_dir_all(path) {
            return Err(WalkFailure::Call { call: "mkdir", errno: errno_of_call(&error) });
        }
    }
    return Ok(());
}

/// What:
///  Walk from the scratch root to the slot and record its identity.
/// Why:
///  The walk is the product's own resolution bits over a tree the probe owns,
///  and its answer is the identity the changed-identity control compares.
///
/// # Errors
/// The root or the slot could not be opened as a folder.
pub fn walk_to_slot(scratch: &Path, slot: &Path) -> Result<ObjectId, WalkFailure> {
    let root: std::fs::File = open_root(scratch).map_err(|error| return WalkFailure::Call { call: "open_root", errno: errno_of_call(&error) })?;
    let slot_name: &OsStr = slot.file_name().ok_or(WalkFailure::Call { call: "slot name", errno: None })?;
    let opened: std::fs::File = open_directory_at(&root, slot_name).map_err(|error| return WalkFailure::Call { call: "open_directory_at", errno: errno_of_call(&error) })?;
    return identity_of(&opened).map_err(|error| return WalkFailure::Call { call: "identity_of", errno: errno_of_call(&error) });
}

/// What:
///  Run the four refusal controls inside the probe's own tree,
///  each of which must refuse and must refuse with the design's own answer.
/// Why:
///  A control that attached would make the walk's refusals unproven,
///  and the success control is the walk above,
///  which opened the slot before these ran.
///
/// # Errors
/// A call failed or a control did not refuse as designed.
pub fn refusal_controls(scratch: &Path, slot: &Path) -> Result<(), WalkFailure> {
    let root: std::fs::File = open_root(scratch).map_err(|error| return WalkFailure::Call { call: "open_root", errno: errno_of_call(&error) })?;
    let refusal: std::fs::File = open_directory_at(&root, OsStr::new(REFUSAL_TREE)).map_err(|error| return WalkFailure::Call { call: "open_directory_at", errno: errno_of_call(&error) })?;
    symlink_at(slot, &refusal, OsStr::new(LINK_NAME)).map_err(|error| return WalkFailure::Call { call: "symlink_at", errno: errno_of_call(&error) })?;
    match open_directory_at(&refusal, OsStr::new(LINK_NAME)) {
        Ok(_) => return Err(WalkFailure::Control { control: "link", errno: None }),
        Err(error) if errno_of_call(&error) == Some(libc::ELOOP) => {}
        Err(error) => return Err(WalkFailure::Control { control: "link", errno: errno_of_call(&error) }),
    }
    match open_directory_beneath(&root, "../escape") {
        Ok(_) => return Err(WalkFailure::Control { control: "escape", errno: None }),
        Err(CallError { errno, .. }) if errno == libc::EXDEV => {}
        Err(CallError { errno, .. }) => return Err(WalkFailure::Control { control: "escape", errno: Some(errno) }),
    }
    super::probe_mount_calls::bind_mount(slot, bound_slot_of(scratch).as_path()).map_err(|error| return WalkFailure::Call { call: error.call, errno: Some(error.errno) })?;
    match open_beneath_components(&root, [REFUSAL_TREE, BOUND_NAME, "slot"]) {
        Ok(_) => return Err(WalkFailure::Control { control: "crossing", errno: None }),
        Err(errno) if errno == Some(libc::EXDEV) => {}
        Err(errno) => return Err(WalkFailure::Control { control: "crossing", errno }),
    }
    let replaced: PathBuf = scratch.join(REFUSAL_TREE).join(REPLACED_NAME);
    std::fs::create_dir_all(replaced.as_path()).map_err(|error| return WalkFailure::Call { call: "mkdir", errno: errno_of_call(&error) })?;
    let recorded: ObjectId = identity_of(&open_directory_at(&refusal, OsStr::new(REPLACED_NAME)).map_err(|error| return WalkFailure::Call { call: "open_directory_at", errno: errno_of_call(&error) })?)
        .map_err(|error| return WalkFailure::Call { call: "identity_of", errno: errno_of_call(&error) })?;
    std::fs::rename(replaced.as_path(), scratch.join(REFUSAL_TREE).join(REPLACED_ELSEWHERE_NAME).as_path())
        .map_err(|error| return WalkFailure::Call { call: "rename", errno: errno_of_call(&error) })?;
    std::fs::create_dir_all(replaced.as_path()).map_err(|error| return WalkFailure::Call { call: "mkdir", errno: errno_of_call(&error) })?;
    let now: ObjectId = identity_of(&open_directory_at(&refusal, OsStr::new(REPLACED_NAME)).map_err(|error| return WalkFailure::Call { call: "open_directory_at", errno: errno_of_call(&error) })?)
        .map_err(|error| return WalkFailure::Call { call: "identity_of", errno: errno_of_call(&error) })?;
    return if now == recorded { Err(WalkFailure::Control { control: "changed identity", errno: None }) } else { Ok(()) };
}

/// What:
///  Open the checkout root read-only and record its identity.
/// Why:
///  The checkout is not the probe's to change,
///  and this open proves the root itself is reachable and reads no entry.
///
/// # Errors
/// The root could not be opened or its identity read.
pub fn open_checkout(checkout: &Path) -> Result<ObjectId, WalkFailure> {
    let root: std::fs::File = open_root(checkout).map_err(|error| return WalkFailure::Call { call: "open_root checkout", errno: errno_of_call(&error) })?;
    return identity_of(&root).map_err(|error| return WalkFailure::Call { call: "identity_of checkout", errno: errno_of_call(&error) });
}

/// The bound control's mount point inside the scratch tmpfs.
fn bound_slot_of(scratch: &Path) -> PathBuf {
    return scratch.join(REFUSAL_TREE).join(BOUND_NAME).join("slot");
}

/// Walk one component at a time beneath the held root and answer the errno of
///  the first refusal.
fn open_beneath_components(root: &std::fs::File, components: [&str; 3]) -> Result<(), Option<i32>> {
    let mut held: std::fs::File = root.try_clone().map_err(|error| return errno_of_call(&error))?;
    for component in components {
        match open_directory_at(&held, OsStr::new(component)) {
            Ok(next) => held = next,
            Err(error) => return Err(errno_of_call(&error)),
        }
    }
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_walk_arm_tests.rs"]
mod tests;
