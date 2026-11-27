//! What:
//!  The unmount of a staged mount from the descriptor a process holds:
//!  the two `umount2` calls,
//!  the topmost read before each one,
//!  the transition check after it,
//!  and the answer a refusal leaves.
//! Why:
//!  The design's interface is "the unmount from the held descriptor"
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The unmount call from staging"):
//!  the first `umount2` takes the clone,
//!  the second takes the bind,
//!  and each call checks the topmost mount before it and the transition after
//!  it,
//!  so a mount the account stacked between a check and its call is never the
//!  one a call takes down.
//!  The calls sit apart from the two sweeps that make them,
//!  because a held descriptor and a record's handle reach the same two calls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = unmountTheStagedMount({ cache, folder, name, cloneSource, calls });
//! ```

use super::staging_calls::{
    StagingCallRefusal,
    StagingCalls,
};
use crate::fs_mount_options::MountFacts;
use std::ffi::OsStr;

/// What:
///  One staged mount to take down from a held folder descriptor.
/// Why:
///  The clone's source and the folder's name are all the caller knows,
///  and the calls read the rest live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldUnmount = { cache; folder; name; cloneSource; calls };
/// ```
pub struct HeldUnmount<'a> {
    /// The account's cache descriptor.
    pub cache: &'a std::fs::File,
    /// The held folder descriptor,
    /// on the topmost mount at the folder.
    pub folder: &'a std::fs::File,
    /// The folder's name beneath the cache.
    pub name: &'a str,
    /// The clone's dataset source.
    pub clone_source: &'a str,
    /// The mount calls.
    pub calls: &'a dyn StagingCalls,
}

/// What:
///  What the held-descriptor unmount did.
/// Why:
///  The wrapper needs to know both mounts left,
///  and a refusal names the step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldOutcome = { kind: 'unmounted'; clone: boolean; bind: boolean } | { kind: 'retained'; step; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "the retained alternative names the step, words and row a person acts on"
)]
pub enum HeldOutcome {
    /// The clone and the bind left.
    Unmounted {
        /// The clone's mount number.
        clone_mount: u64,
        /// The bind's mount number.
        bind_mount: u64,
    },
    /// Something is left,
    /// and the step says what refused.
    Retained {
        /// The step that refused.
        step: String,
        /// The kernel's errno,
        /// or zero.
        errno: i32,
        /// What refused.
        said: String,
        /// The topmost row when one was read.
        mount: Option<MountFacts>,
    },
}

/// What:
///  Take down the topmost mount,
///  check what is now topmost,
///  and take that down too when it is the bind.
/// Why:
///  The first call takes the clone,
///  and the second takes the bind only when the transition check proved the
///  bind is what the call will now reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = unmountTheStagedMount(request);
/// ```
pub fn unmount_the_staged_mount(request: &HeldUnmount<'_>) -> HeldOutcome {
    let top: (std::fs::File, u64, MountFacts) =
        match topmost(request.calls, request.cache, request.name) {
            Ok(top) => top,
            Err(refusal) => return retained(refusal, None),
        };
    let clone_mount: u64 = top.1;
    if top.2.source != request.clone_source {
        return HeldOutcome::Retained {
            step: String::from("pre-call-check"),
            errno: 0,
            said: format!(
                "the topmost mount at the staging folder is {} with source `{}`, and the clone is `{}`. Nothing was unmounted.",
                top.1, top.2.source, request.clone_source
            ),
            mount: Some(top.2),
        };
    }
    drop(top.0);
    if let Err(refusal) = request.calls.unmount_top(request.folder) {
        return retained(refusal, None);
    }
    let after: (std::fs::File, u64, MountFacts) =
        match topmost(request.calls, request.cache, request.name) {
            Ok(after) => after,
            Err(refusal) => return retained(refusal, None),
        };
    if after.1 == clone_mount {
        return HeldOutcome::Retained {
            step: String::from("post-call-check"),
            errno: 0,
            said: format!("the clone's unmount returned and mount {clone_mount} is still topmost"),
            mount: Some(after.2),
        };
    }
    let bind_mount: u64 = after.1;
    drop(after.0);
    if let Err(refusal) = request.calls.unmount_top(request.folder) {
        return retained(refusal, None);
    }
    let final_read = topmost(request.calls, request.cache, request.name);
    return match final_read {
        Ok((folder, mount_id, facts)) => {
            drop(folder);
            if mount_id == bind_mount {
                return HeldOutcome::Retained {
                    step: String::from("post-call-check"),
                    errno: 0,
                    said: format!(
                        "the bind's unmount returned and mount {bind_mount} is still topmost"
                    ),
                    mount: Some(facts),
                };
            }
            HeldOutcome::Unmounted {
                clone_mount,
                bind_mount,
            }
        }
        Err(refusal) => retained(refusal, None),
    };
}

/// One topmost mount by name.
fn topmost(
    calls: &dyn StagingCalls,
    cache: &std::fs::File,
    name: &str,
) -> Result<(std::fs::File, u64, MountFacts), StagingCallRefusal> {
    let folder: std::fs::File = calls.open_by_name(cache, OsStr::new(name))?;
    let mount_id: u64 = calls.mount_id(&folder)?;
    let facts: MountFacts = calls.mount_facts(mount_id)?;
    return Ok((folder, mount_id, facts));
}

/// One call's refusal as a retained outcome.
fn retained(
    refusal: StagingCallRefusal,
    mount: Option<MountFacts>,
) -> HeldOutcome {
    return HeldOutcome::Retained {
        step: String::from(refusal.call.word()),
        errno: refusal.errno,
        said: refusal.message("the staged unmount refused"),
        mount,
    };
}
