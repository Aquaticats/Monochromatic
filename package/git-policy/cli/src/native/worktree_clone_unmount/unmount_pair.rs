//! What:
//!  The two unmounts one staged pair takes:
//!  the read of the topmost mount before and after each call,
//!  the clone's unmount,
//!  the bind's unbind when the clone has left,
//!  the bind's own unmount,
//!  and the checked move back of a clone that landed on a stack the account
//!  made.
//! Why:
//!  A mount the account stacks between a check and its call is the one the call
//!  takes down,
//!  so each call is preceded by a read of the topmost mount and followed by
//!  one,
//!  and a clone that landed on top of a stack is moved back before anything
//!  else is unbound
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The unmount call from staging").
//!  The pair's calls sit apart from the turn's first checks,
//!  so a reader of the turn reads the order of a removal in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unmounted = unmountThePair(request, held, bindId, name, staged);
//! ```

use super::reattach::{
    ReattachRequest,
    ReattachResult,
    roll_back_to_slot,
};
use super::staging_calls::{
    StagingCallRefusal,
    StagingCalls,
};
use super::unmount_turn_answers::{
    ReattachCause,
    UnmountOutcome,
};
use super::unmount_turn_request::UnmountRequest;
use crate::fs_mount_options::MountFacts;
use std::ffi::OsStr;

/// One topmost mount at the staging name.
pub(super) fn topmost_at(
    calls: &dyn StagingCalls,
    cache: &std::fs::File,
    name: &str,
) -> Result<(std::fs::File, u64, MountFacts), StagingCallRefusal> {
    let folder: std::fs::File = calls.open_by_name(cache, OsStr::new(name))?;
    let mount_id: u64 = calls.mount_id(&folder)?;
    let facts: MountFacts = calls.mount_facts(mount_id)?;
    return Ok((folder, mount_id, facts));
}

/// What:
///  Unmount the clone and then the bind,
///  with the topmost check before each call and the transition check after it.
/// Why:
///  A mount the account stacks between a check and its call is the one the call
///  takes down,
///  and the transition check is what catches it.
pub(super) fn unmount_the_pair(
    request: &UnmountRequest<'_>,
    held: &std::fs::File,
    bind_id: u64,
    name: &str,
    staged: std::fs::File,
) -> UnmountOutcome {
    // A descriptor on the clone would hold it busy; the reattach path reopens the folder by name after the kernel says
    // EBUSY, so nothing needs to keep this one alive.
    drop(staged);
    let call: Result<(), StagingCallRefusal> = request.calls.unmount_top(held);
    if let Err(refusal) = call {
        if refusal.errno == libc::EBUSY {
            let reopened: std::fs::File = match request
                .calls
                .open_by_name(request.cache, std::ffi::OsStr::new(name))
            {
                Ok(reopened) => reopened,
                Err(open) => {
                    return UnmountOutcome::Retained {
                        step: String::from(open.call.word()),
                        errno: open.errno,
                        said: open.message("the busy clone could not be reopened for its reattach"),
                        mount: None,
                    };
                }
            };
            let reopened_id: u64 = match request.calls.mount_id(&reopened) {
                Ok(id) => id,
                Err(read_failure) => {
                    return UnmountOutcome::Retained {
                        step: String::from("mount-id"),
                        errno: read_failure.errno,
                        said: read_failure.message("the busy answer's mount could not be read"),
                        mount: None,
                    };
                }
            };
            let reopened_row = request.calls.mount_facts(reopened_id).ok();
            if reopened_id != request.expected.mount_id
                || reopened_row.as_ref().is_some_and(|row| {
                    return row.source != request.expected.source
                        || row.root != std::path::Path::new("/");
                })
            {
                return UnmountOutcome::Retained {
                    step: String::from("pre-call-check"),
                    errno: 0,
                    said: format!(
                        "the mount the busy call answered for is {reopened_id} of source `{}`, and the clone is mount {} of `{}`. Nothing was moved.",
                        reopened_row.as_ref().map_or_else(
                            || return String::from("unread"),
                            |row| return row.source.clone()
                        ),
                        request.expected.mount_id,
                        request.expected.source
                    ),
                    mount: reopened_row,
                };
            }
            return match reattach(request, &reopened) {
                ReattachResult::Reattached {
                    mount_id,
                    slot,
                } => {
                    let bind_gone: bool =
                        unbind_if_empty(request.calls, request.cache, name, held, bind_id);
                    log_rolled_back(request, mount_id);
                    UnmountOutcome::Reattached {
                        mount_id,
                        slot,
                        bind_gone,
                        device: request.expected.device,
                        inode: request.expected.inode,
                        cause: ReattachCause::Busy,
                    }
                }
                ReattachResult::Refused {
                    step,
                    said,
                } => UnmountOutcome::Retained {
                    step,
                    errno: 0,
                    said,
                    mount: None,
                },
            };
        }
        return UnmountOutcome::Retained {
            step: String::from("unmount"),
            errno: refusal.errno,
            said: refusal.message("the clone's unmount refused"),
            mount: None,
        };
    }
    let after: (std::fs::File, u64, MountFacts) =
        match topmost_at(request.calls, request.cache, name) {
            Ok(after) => after,
            Err(refusal) => {
                return UnmountOutcome::Retained {
                    step: String::from(refusal.call.word()),
                    errno: refusal.errno,
                    said: refusal
                        .message("the clone's unmount could not be followed by a read of the bind"),
                    mount: None,
                };
            }
        };
    if after.1 != bind_id {
        return UnmountOutcome::Retained {
            step: String::from("post-unmount"),
            errno: 0,
            said: format!(
                "the clone's unmount returned, and the topmost mount at the staging folder is {} rather than the bind {bind_id}. The handler unbinds nothing and reports the row.",
                after.1
            ),
            mount: Some(after.2),
        };
    }
    drop(after.0);
    if let Err(refusal) = request.calls.unmount_top(held) {
        return UnmountOutcome::Retained {
            step: String::from("unbind"),
            errno: refusal.errno,
            said: refusal.message("the bind's unmount refused"),
            mount: None,
        };
    }
    return UnmountOutcome::Unmounted {
        mount_id: request.expected.mount_id,
        slot: request.expected.canonical_slot.clone(),
        bind_gone: true,
    };
}

/// The checked move back of a staged clone to its slot.
pub(super) fn reattach(
    request: &UnmountRequest<'_>,
    staged: &std::fs::File,
) -> ReattachResult {
    let move_back = ReattachRequest {
        checkout: request.checkout,
        slot: request.slot,
        clone_source: request.expected.source.as_str(),
        clone_mount_id: request.expected.mount_id,
        slot_identity: request.slot_identity,
        slot_parent_mount_id: request.expected.parent_mount_id,
        clone_at_staging: staged,
        calls: request.calls,
        walk: request.walk,
    };
    return roll_back_to_slot(&move_back);
}

/// Unbind the bind when the topmost mount is the recorded bind.
pub(super) fn unbind_if_empty(
    calls: &dyn StagingCalls,
    cache: &std::fs::File,
    name: &str,
    held: &std::fs::File,
    bind_id: u64,
) -> bool {
    let Ok((folder, mount_id, _facts)) = topmost_at(calls, cache, name) else {
        return false;
    };
    if mount_id != bind_id {
        return false;
    }
    drop(folder);
    return calls.unmount_top(held).is_ok();
}

/// A debug line for a move back,
/// kept here so both callers say the same words.
pub(super) fn log_rolled_back(
    request: &UnmountRequest<'_>,
    mount_id: u64,
) {
    crate::diagnostic_log::debug(
        "worktreeCloneUnmount",
        format!(
            "the clone `{}` whose mount {mount_id} landed on a stacked mount is back at slot `{}`",
            request.expected.source, request.slot
        )
        .as_str(),
    );
}
