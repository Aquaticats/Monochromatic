//! What:
//!  Arm 2's reads over the trees the probe owns:
//!  the no-crossing walk to the slot and the identity it records,
//!  and the read-only open of the checkout root.
//! Why:
//!  The walk the product uses is judged here over a tree nobody else can
//!  rename into,
//!  and this walk is the success control the four refusals sit beside:
//!  the slot it reaches is opened and its identity recorded,
//!  so a walk that refuses everything fails its own arm
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms" and M5).
//!  Nothing here reads a directory entry of the checkout,
//!  which is what the read-only root open proves.
//!  The four refusal controls are in `probe_walk_refusal_controls.rs`,
//!  beside this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! makeScratchTree(scratch, slot, moveTarget); const evidence = walkToSlot(scratch, slot); const checkout = openCheckout(path);
//! ```
use super::probe_errno_names::errno_of;
use crate::fs_beneath::{
    ObjectId,
    identity_of,
    open_directory_at,
    open_root,
};
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

/// The directory inside the scratch tmpfs the refusal controls plant in.
pub const REFUSAL_TREE: &str = "refusal";
/// The folder the bind mount is planted on,
/// across a walk component.
pub const BOUND_NAME: &str = "bound";

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
        return self.errno_number().and_then(errno_of);
    }

    /// The errno itself,
    /// so a number this build does not name still reaches the record.
    pub fn errno_number(&self) -> Option<i32> {
        return match self {
            WalkFailure::Call {
                errno,
                ..
            }
            | WalkFailure::Control {
                errno,
                ..
            } => *errno,
        };
    }

    /// The failure in words,
    /// naming the control and what it answered.
    pub fn message(&self) -> String {
        return match self {
            WalkFailure::Call {
                call,
                errno,
            } => format!("the probe's walk call `{call}` failed (errno {errno:?})."),
            WalkFailure::Control {
                control,
                errno,
            } => format!(
                "the probe's refusal control `{control}` did not refuse as designed (errno {errno:?})."
            ),
        };
    }
}

/// The errno of one failed call,
/// as the refusal compares it.
pub(super) fn errno_of_call(error: &std::io::Error) -> Option<i32> {
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
pub fn make_scratch_tree(
    scratch: &Path,
    slot: &Path,
    move_target: &Path,
) -> Result<(), WalkFailure> {
    for path in [slot.to_path_buf(), move_target.to_path_buf()] {
        if let Err(error) = std::fs::create_dir_all(path.as_path()) {
            return Err(WalkFailure::Call {
                call: "mkdir",
                errno: errno_of_call(&error),
            });
        }
    }
    let refusal: PathBuf = scratch.join(REFUSAL_TREE);
    let bound: PathBuf = refusal.join(BOUND_NAME);
    let bound_slot: PathBuf = bound.join("slot");
    for path in [refusal.as_path(), bound.as_path(), bound_slot.as_path()] {
        if let Err(error) = std::fs::create_dir_all(path) {
            return Err(WalkFailure::Call {
                call: "mkdir",
                errno: errno_of_call(&error),
            });
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
pub fn walk_to_slot(
    scratch: &Path,
    slot: &Path,
) -> Result<ObjectId, WalkFailure> {
    let root: std::fs::File = open_root(scratch).map_err(|error| {
        return WalkFailure::Call {
            call: "open_root",
            errno: errno_of_call(&error),
        };
    })?;
    let slot_name: &OsStr = slot.file_name().ok_or(WalkFailure::Call {
        call: "slot name",
        errno: None,
    })?;
    let opened: std::fs::File = open_directory_at(&root, slot_name).map_err(|error| {
        return WalkFailure::Call {
            call: "open_directory_at",
            errno: errno_of_call(&error),
        };
    })?;
    return identity_of(&opened).map_err(|error| {
        return WalkFailure::Call {
            call: "identity_of",
            errno: errno_of_call(&error),
        };
    });
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
    let root: std::fs::File = open_root(checkout).map_err(|error| {
        return WalkFailure::Call {
            call: "open_root checkout",
            errno: errno_of_call(&error),
        };
    })?;
    return identity_of(&root).map_err(|error| {
        return WalkFailure::Call {
            call: "identity_of checkout",
            errno: errno_of_call(&error),
        };
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_walk_arm_tests.rs"]
mod tests;
