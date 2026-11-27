//! What:
//!  Arm 2's four refusal controls:
//!  the walk that must not follow a symbolic link,
//!  the one that must not leave the tree through `..`,
//!  the one that must not cross into another mount,
//!  and the one that must not open a folder replaced under its recorded
//!  identity.
//! Why:
//!  Each refusal is what the product's walk promises a caller,
//!  and each one must refuse with the answer the measured refusal gave,
//!  so a control that attached,
//!  or refused with another errno,
//!  fails this arm rather than passing it
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms" and M5).
//!  The success control sits beside these in `probe_walk_arm.rs`:
//!  the walk it takes reached the slot and recorded its identity before these
//!  ran,
//!  so a walk that refuses everything fails its own arm.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! refuseALink(scratch, slot); refuseAnEscape(scratch); refuseACrossing(scratch); refuseAChangedIdentity(scratch);
//! ```

use super::probe_call_error::CallError;
use super::probe_open_beneath::open_directory_beneath;
use super::probe_walk_arm::{
    BOUND_NAME,
    REFUSAL_TREE,
    WalkFailure,
    errno_of_call,
};
use crate::fs_beneath::{
    ObjectId,
    identity_of,
    open_directory_at,
    open_root,
    symlink_at,
};
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

/// The symbolic link that must refuse with `ELOOP`.
pub const LINK_NAME: &str = "link";
/// The folder whose identity is recorded and then replaced.
pub const REPLACED_NAME: &str = "replaced";
/// The name the replaced folder is moved to,
/// which is how its identity changes under the recorded one.
pub const REPLACED_ELSEWHERE_NAME: &str = "replaced-elsewhere";
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
pub fn refusal_controls(
    scratch: &Path,
    slot: &Path,
) -> Result<(), WalkFailure> {
    let root: std::fs::File = open_root(scratch).map_err(|error| {
        return WalkFailure::Call {
            call: "open_root",
            errno: errno_of_call(&error),
        };
    })?;
    let refusal: std::fs::File =
        open_directory_at(&root, OsStr::new(REFUSAL_TREE)).map_err(|error| {
            return WalkFailure::Call {
                call: "open_directory_at",
                errno: errno_of_call(&error),
            };
        })?;
    symlink_at(slot, &refusal, OsStr::new(LINK_NAME)).map_err(|error| {
        return WalkFailure::Call {
            call: "symlink_at",
            errno: errno_of_call(&error),
        };
    })?;
    match open_directory_at(&refusal, OsStr::new(LINK_NAME)) {
        Ok(_) => {
            return Err(WalkFailure::Control {
                control: "link",
                errno: None,
            });
        }
        Err(error) if errno_of_call(&error) == Some(libc::ELOOP) => {}
        Err(error) => {
            return Err(WalkFailure::Control {
                control: "link",
                errno: errno_of_call(&error),
            });
        }
    }
    match open_directory_beneath(&root, "../escape") {
        Ok(_) => {
            return Err(WalkFailure::Control {
                control: "escape",
                errno: None,
            });
        }
        Err(CallError {
            errno,
            ..
        }) if errno == libc::EXDEV => {}
        Err(CallError {
            errno,
            ..
        }) => {
            return Err(WalkFailure::Control {
                control: "escape",
                errno: Some(errno),
            });
        }
    }
    super::probe_mount_calls::bind_mount(slot, bound_slot_of(scratch).as_path()).map_err(
        |error| {
            return WalkFailure::Call {
                call: error.call,
                errno: Some(error.errno),
            };
        },
    )?;
    match open_beneath_components(&root, &[REFUSAL_TREE, BOUND_NAME, "slot"]) {
        Ok(_) => {
            return Err(WalkFailure::Control {
                control: "crossing",
                errno: None,
            });
        }
        Err(errno) if errno == Some(libc::EXDEV) => {}
        Err(errno) => {
            return Err(WalkFailure::Control {
                control: "crossing",
                errno,
            });
        }
    }
    let replaced: PathBuf = scratch.join(REFUSAL_TREE).join(REPLACED_NAME);
    std::fs::create_dir_all(replaced.as_path()).map_err(|error| {
        return WalkFailure::Call {
            call: "mkdir",
            errno: errno_of_call(&error),
        };
    })?;
    let recorded: ObjectId = identity_of(
        &open_directory_at(&refusal, OsStr::new(REPLACED_NAME)).map_err(|error| {
            return WalkFailure::Call {
                call: "open_directory_at",
                errno: errno_of_call(&error),
            };
        })?,
    )
    .map_err(|error| {
        return WalkFailure::Call {
            call: "identity_of",
            errno: errno_of_call(&error),
        };
    })?;
    std::fs::rename(
        replaced.as_path(),
        scratch
            .join(REFUSAL_TREE)
            .join(REPLACED_ELSEWHERE_NAME)
            .as_path(),
    )
    .map_err(|error| {
        return WalkFailure::Call {
            call: "rename",
            errno: errno_of_call(&error),
        };
    })?;
    std::fs::create_dir_all(replaced.as_path()).map_err(|error| {
        return WalkFailure::Call {
            call: "mkdir",
            errno: errno_of_call(&error),
        };
    })?;
    let now: ObjectId = identity_of(
        &open_directory_at(&refusal, OsStr::new(REPLACED_NAME)).map_err(|error| {
            return WalkFailure::Call {
                call: "open_directory_at",
                errno: errno_of_call(&error),
            };
        })?,
    )
    .map_err(|error| {
        return WalkFailure::Call {
            call: "identity_of",
            errno: errno_of_call(&error),
        };
    })?;
    return if now == recorded {
        Err(WalkFailure::Control {
            control: "changed identity",
            errno: None,
        })
    } else {
        Ok(())
    };
}

/// The bound control's mount point inside the scratch tmpfs.
fn bound_slot_of(scratch: &Path) -> PathBuf {
    return scratch.join(REFUSAL_TREE).join(BOUND_NAME).join("slot");
}

/// Walk one component at a time beneath the held root and answer the errno of
///  the first refusal.
fn open_beneath_components(
    root: &std::fs::File,
    components: &[&str],
) -> Result<(), Option<i32>> {
    let mut held: std::fs::File = root
        .try_clone()
        .map_err(|error| return errno_of_call(&error))?;
    for component in components {
        match open_directory_at(&held, OsStr::new(component)) {
            Ok(next) => held = next,
            Err(error) => return Err(errno_of_call(&error)),
        }
    }
    return Ok(());
}
