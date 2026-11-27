//! What:
//!  One removal's unmount turn:
//!  the checks,
//!  the staging folder and its record,
//!  the bind and its propagation change,
//!  the move,
//!  the placement proof,
//!  the busy answer with its checked reattach,
//!  and the hand-over of the pair's two unmounts to `unmount_pair.rs`.
//! Why:
//!  The order is the design's and every step fails closed
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "What a writer can settle without asking":
//!  write the record,
//!  bind,
//!  make private,
//!  move,
//!  verify,
//!  unmount the clone,
//!  unbind,
//!  then the wrapper's `rmdir`).
//!  A refusal before the bind makes no mount call;
//!  a refusal after it keeps the mount and names the step,
//!  and the only move back is the checked reattach of the busy answer.
//!  The turn keeps the order of a removal in one place;
//!  the answers it builds,
//!  the facts it is given and the pair's calls sit in
//!  the three files beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = unmountTheClone(request);
//! ```

use super::reattach::ReattachResult;
use super::staging_folder_name::staging_folder_name;
use super::unmount_checks::{
    CheckedSlot,
    SlotRefusal,
    verify_the_slot,
};
use super::unmount_pair::{
    log_rolled_back,
    reattach,
    topmost_at,
    unbind_if_empty,
    unmount_the_pair,
};
use super::unmount_turn_answers::{
    call_refused,
    refused,
};
use super::user_cache_pin::check_live_pin;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::ffi::OsStr;

/// What one turn did,
/// and the words it answers with.
pub use super::unmount_turn_answers::UnmountOutcome;

/// Why a mount was moved back to its slot.
pub use super::unmount_turn_answers::ReattachCause;

/// Everything one unmount turn reads and acts on.
pub use super::unmount_turn_request::UnmountRequest;

/// One slot refusal as a refused outcome.
fn slot_refused(refusal: SlotRefusal) -> UnmountOutcome {
    return UnmountOutcome::Refused {
        step: refusal.step,
        errno: 0,
        said: refusal.said,
        mount: refusal.observed,
    };
}

/// The staging folder's component as a name.
fn component_name(guid: DatasetGuid) -> String {
    return staging_folder_name(guid);
}

/// What:
///  Run one removal's unmount.
/// Why:
///  Every check precedes the bind,
///  and every later failure names its step and keeps the mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = unmountTheClone(request);
/// ```
pub fn unmount_the_clone(request: &UnmountRequest<'_>) -> UnmountOutcome {
    let checked: CheckedSlot = match verify_the_slot(
        request.slot,
        request.expected,
        request.checkout,
        request.calls,
        request.walk,
        request.guids,
    ) {
        Ok(checked) => checked,
        Err(refusal) => return slot_refused(refusal),
    };
    let name: String = component_name(request.expected.guid);
    let staging: std::fs::File = match crate::fs_beneath::open_directory_at(
        request.cache,
        OsStr::new(name.as_str()),
    ) {
        Ok(staging) => staging,
        Err(error) => {
            return refused(
                "staging-folder",
                format!(
                    "the staging folder {name} could not be opened without crossing a mount: {error}. Nothing was bound or moved, and the wrapper creates that folder before a removal asks the handler to bind it"
                ),
            );
        }
    };
    if let Err(outcome) =
        super::unmount_record::check_the_folders_shape(request, &staging, name.as_str())
    {
        return outcome;
    }
    if let Err(pin) = check_live_pin(request.cache, request.pin) {
        return refused("user-cache-pin", pin.message());
    }
    if let Err(outcome) = super::unmount_record::record_the_bind(request, &staging, name.as_str()) {
        return outcome;
    }
    let bind: std::fs::File =
        match request
            .calls
            .bind_the_folder(&staging, request.cache, OsStr::new(name.as_str()))
        {
            Ok(bind) => bind,
            Err(refusal) => return call_refused("bind", refusal, None),
        };
    if let Err(refusal) = request.calls.make_private(&bind) {
        return UnmountOutcome::Retained {
            step: String::from("make-private"),
            errno: refusal.errno,
            said: refusal.message("the bind was made and could not be made private"),
            mount: None,
        };
    }
    if let Err(refusal) = request.calls.move_clone(&checked.target, &bind) {
        return UnmountOutcome::Retained {
            step: String::from("move-mount"),
            errno: refusal.errno,
            said: refusal.message("the bind stands and the clone is still at its slot"),
            mount: Some(checked.facts),
        };
    }
    // The slot's descriptor is on the clone's mount, so it would hold the clone busy. Every read the route still needs is
    // by name under the cache descriptor, which crosses to the bind's dentry rather than pinning the clone.
    drop(checked.target);
    let bind_id: u64 = match request.calls.mount_id(&bind) {
        Ok(bind_id) => bind_id,
        Err(refusal) => {
            return UnmountOutcome::Retained {
                step: String::from("mount-id"),
                errno: refusal.errno,
                said: refusal.message("the bind's own mount could not be read"),
                mount: None,
            };
        }
    };
    // The bind's descriptor would hold the bind busy, so it is released before the unmount; the staging folder's
    // descriptor, taken before the bind, is the one the two calls act through.
    drop(bind);
    let topmost: (std::fs::File, u64, MountFacts) =
        match topmost_at(request.calls, request.cache, name.as_str()) {
            Ok(topmost) => topmost,
            Err(refusal) => {
                return UnmountOutcome::Retained {
                    step: String::from(refusal.call.word()),
                    errno: refusal.errno,
                    said: refusal.message("the move's placement could not be read"),
                    mount: None,
                };
            }
        };
    if topmost.1 != request.expected.mount_id || topmost.2.source != request.expected.source {
        return UnmountOutcome::Retained {
            step: String::from("post-move-check"),
            errno: 0,
            said: format!(
                "the topmost mount at the staging folder is {} with source `{}`, and the clone is mount {} with source `{}`. The handler unbinds nothing and removes nothing.",
                topmost.1, topmost.2.source, request.expected.mount_id, request.expected.source
            ),
            mount: Some(topmost.2),
        };
    }
    if topmost.2.parent_id != bind_id {
        let staged: &std::fs::File = &topmost.0;
        let moved_back: ReattachResult = reattach(request, staged);
        return match moved_back {
            ReattachResult::Reattached {
                mount_id,
                slot,
            } => {
                let bind_gone: bool = unbind_if_empty(
                    request.calls,
                    request.cache,
                    name.as_str(),
                    &staging,
                    bind_id,
                );
                log_rolled_back(request, mount_id);
                UnmountOutcome::Reattached {
                    mount_id,
                    slot,
                    bind_gone,
                    device: request.expected.device,
                    inode: request.expected.inode,
                    cause: ReattachCause::MisplacedRolledBack,
                }
            }
            ReattachResult::Refused {
                step,
                said,
            } => UnmountOutcome::Retained {
                step,
                errno: 0,
                said,
                mount: Some(topmost.2),
            },
        };
    }
    return unmount_the_pair(request, &staging, bind_id, name.as_str(), topmost.0);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unmount_turn_tests.rs"]
mod tests;
