//! What:
//!  One removal's unmount turn:
//!  the checks,
//!  the staging folder and its record,
//!  the bind and its propagation change,
//!  the move,
//!  the placement proof,
//!  the busy answer with its checked reattach,
//!  and the two descriptor unmounts.
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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = unmountTheClone(request);
//! ```

use super::reattach::{roll_back_to_slot, ReattachRequest, ReattachResult};
use super::slot_walk_calls::{DatasetGuids, SlotWalk};
use super::staged_folder_identity::StagedFolderIdentity;
use super::staged_folder_record::{write_record, StagedFolderRecord};
use super::staged_record_reading::{read_staged_record, BindingDecision, RecordReading};
use super::staging_calls::{StagingCallRefusal, StagingCalls};
use super::staging_folder_name::staging_folder_name;
use super::unmount_checks::{verify_the_slot, CheckedSlot, ExpectedMount, SlotRefusal};
use super::user_cache_pin::{check_live_pin, UserCachePin};
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::os::unix::fs::MetadataExt;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  Everything one unmount turn reads and acts on.
/// Why:
///  The facts and the seams travel together,
///  so the caller cannot pass the slot of one clone with the GUID of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnmountRequest = { checkout; slot; nonce; expected; pin; cache; cachePath; boot; calls; walk; guids };
/// ```
pub struct UnmountRequest<'a> {
    /// The verified checkout.
    pub checkout: &'a Path,
    /// The clone's slot name.
    pub slot: &'a str,
    /// The intent's nonce.
    pub nonce: &'a str,
    /// The mount the removal expects.
    pub expected: &'a ExpectedMount,
    /// The slot folder's identity the store recorded.
    pub slot_identity: &'a StagedFolderIdentity,
    /// The `user-cache` pin.
    pub pin: &'a UserCachePin,
    /// The account's cache descriptor.
    pub cache: &'a std::fs::File,
    /// The cache folder's path.
    pub cache_path: &'a Path,
    /// This boot's identifier.
    pub boot: &'a str,
    /// The mount calls.
    pub calls: &'a dyn StagingCalls,
    /// The walks.
    pub walk: &'a dyn SlotWalk,
    /// The dataset GUID read.
    pub guids: &'a dyn DatasetGuids,
}

/// What:
///  What one turn did.
/// Why:
///  The wrapper and the receipt need one word and the mount numbers,
///  and a person needs the step and errno when nothing was unmounted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnmountOutcome = { kind: 'unmounted'; mountId; bindGone } | { kind: 'rolled-back'; ... } | { kind: 'retained' | 'refused'; step; errno; said; mount? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnmountOutcome {
    /// The clone and the bind are gone.
    Unmounted {
        /// The clone's mount number.
        mount_id: u64,
        /// The slot's path.
        slot: PathBuf,
        /// Whether the bind was unbound too.
        bind_gone: bool,
    },
    /// The clone is back at its slot under the checked reattach,
    /// and the bind may be gone.
    Reattached {
        /// The clone's mount number.
        mount_id: u64,
        /// The slot's path.
        slot: PathBuf,
        /// Whether the bind was unbound too.
        bind_gone: bool,
        /// The clone root's device.
        device: u64,
        /// The clone root's inode.
        inode: u64,
        /// Why the mount was moved back.
        cause: ReattachCause,
    },
    /// A mount stays where a step left it.
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
    /// Nothing was moved,
    /// and the step says why.
    Refused {
        /// The step that refused.
        step: String,
        /// The kernel's errno,
        /// or zero.
        errno: i32,
        /// What refused.
        said: String,
        /// The row when one was read.
        mount: Option<MountFacts>,
    },
}

/// What:
///  Why a mount was moved back to its slot.
/// Why:
///  The accepted decision gives one of the two a refusal receipt and no other,
///  so the caller must be able to tell them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReattachCause = 'misplaced-rolled-back' | 'busy';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReattachCause {
    /// The move landed on a mount the account stacked,
    /// so the rollback moved it back.
    MisplacedRolledBack,
    /// The unmount answered `EBUSY`,
    /// so the checked reattach moved it back.
    Busy,
}

impl ReattachCause {
    /// The word a report carries.
    pub fn word(self) -> &'static str {
        return match self {
            ReattachCause::MisplacedRolledBack => "misplaced-rolled-back",
            ReattachCause::Busy => "busy-reattached",
        };
    }
}

impl UnmountOutcome {
    /// The word a receipt carries.
    pub fn word(&self) -> &'static str {
        return match self {
            UnmountOutcome::Unmounted { .. } => "done",
            UnmountOutcome::Reattached { .. } => "reattached",
            UnmountOutcome::Retained { .. } | UnmountOutcome::Refused { .. } => "refused",
        };
    }

    /// Whether the clone's mount is absent.
    pub fn unmounted(&self) -> bool {
        return matches!(self, UnmountOutcome::Unmounted { .. });
    }

    /// Whether a mount stays where it was.
    pub fn retained(&self) -> bool {
        return matches!(self, UnmountOutcome::Retained { .. });
    }
}

/// One refused read as a refused outcome.
fn refused(step: &str, said: String) -> UnmountOutcome {
    return UnmountOutcome::Refused { step: String::from(step), errno: 0, said, mount: None };
}

/// One call's refusal as a refused outcome.
fn call_refused(step: &str, refusal: StagingCallRefusal, mount: Option<MountFacts>) -> UnmountOutcome {
    return UnmountOutcome::Refused { step: String::from(step), errno: refusal.errno, said: refusal.message(step), mount };
}

/// One slot refusal as a refused outcome.
fn slot_refused(refusal: SlotRefusal) -> UnmountOutcome {
    return UnmountOutcome::Refused { step: refusal.step, errno: 0, said: refusal.said, mount: refusal.observed };
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
    let checked: CheckedSlot = match verify_the_slot(request.slot, request.expected, request.checkout, request.calls, request.walk, request.guids) {
        Ok(checked) => checked,
        Err(refusal) => return slot_refused(refusal),
    };
    let name: String = component_name(request.expected.guid);
    let staging: std::fs::File = match crate::fs_beneath::open_directory_at(request.cache, OsStr::new(name.as_str())) {
        Ok(staging) => staging,
        Err(error) => return refused("staging-folder", format!("the staging folder {name} could not be opened without crossing a mount: {error}. Nothing was bound or moved, and the wrapper creates that folder before a removal asks the handler to bind it")),
    };
    if let Err(pin) = check_live_pin(request.cache, request.pin) {
        return refused("user-cache-pin", pin.message());
    }
    if let Err(outcome) = record_the_bind(request, &staging, name.as_str()) {
        return outcome;
    }
    let bind: std::fs::File = match request.calls.bind_the_folder(&staging, request.cache, OsStr::new(name.as_str())) {
        Ok(bind) => bind,
        Err(refusal) => return call_refused("bind", refusal, None),
    };
    if let Err(refusal) = request.calls.make_private(&bind) {
        return UnmountOutcome::Retained { step: String::from("make-private"), errno: refusal.errno, said: refusal.message("the bind was made and could not be made private"), mount: None };
    }
    if let Err(refusal) = request.calls.move_clone(&checked.target, &bind) {
        return UnmountOutcome::Retained { step: String::from("move-mount"), errno: refusal.errno, said: refusal.message("the bind stands and the clone is still at its slot"), mount: Some(checked.facts) };
    }
    // The slot's descriptor is on the clone's mount, so it would hold the clone busy. Every read the route still needs is
    // by name under the cache descriptor, which crosses to the bind's dentry rather than pinning the clone.
    drop(checked.target);
    let bind_id: u64 = match request.calls.mount_id(&bind) {
        Ok(bind_id) => bind_id,
        Err(refusal) => return UnmountOutcome::Retained { step: String::from("mount-id"), errno: refusal.errno, said: refusal.message("the bind's own mount could not be read"), mount: None },
    };
    // The bind's descriptor would hold the bind busy, so it is released before the unmount; the staging folder's
    // descriptor, taken before the bind, is the one the two calls act through.
    drop(bind);
    let topmost: (std::fs::File, u64, MountFacts) = match topmost_at(request.calls, request.cache, name.as_str()) {
        Ok(topmost) => topmost,
        Err(refusal) => return UnmountOutcome::Retained { step: String::from(refusal.call.word()), errno: refusal.errno, said: refusal.message("the move's placement could not be read"), mount: None },
    };
    if topmost.1 != request.expected.mount_id || topmost.2.source != request.expected.source {
        return UnmountOutcome::Retained {
            step: String::from("post-move-check"),
            errno: 0,
            said: format!("the topmost mount at the staging folder is {} with source `{}`, and the clone is mount {} with source `{}`. The handler unbinds nothing and removes nothing.", topmost.1, topmost.2.source, request.expected.mount_id, request.expected.source),
            mount: Some(topmost.2),
        };
    }
    if topmost.2.parent_id != bind_id {
        let staged: &std::fs::File = &topmost.0;
        let moved_back: ReattachResult = reattach(request, staged);
        return match moved_back {
            ReattachResult::Reattached { mount_id, slot } => {
                let bind_gone: bool = unbind_if_empty(request.calls, request.cache, name.as_str(), &staging, bind_id);
                log_rolled_back(request, mount_id);
                UnmountOutcome::Reattached { mount_id, slot, bind_gone, device: request.expected.device, inode: request.expected.inode, cause: ReattachCause::MisplacedRolledBack }
            }
            ReattachResult::Refused { step, said } => UnmountOutcome::Retained { step, errno: 0, said, mount: Some(topmost.2) },
        };
    }
    return unmount_the_pair(request, &staging, bind_id, name.as_str(), topmost.0);
}

/// Record the bind before it is made,
/// replacing an absent,
/// unrecorded or void record and refusing a current record whose identity moved.
#[allow(clippy::result_large_err, reason = "the outcome carries the mount row and the step a person acts on")]
fn record_the_bind(request: &UnmountRequest<'_>, staging: &std::fs::File, name: &str) -> Result<(), UnmountOutcome> {
    let folder_facts: std::fs::Metadata = match staging.metadata() {
        Ok(facts) => facts,
        Err(error) => return Err(refused("staging-folder", format!("the staging folder {name} could not be read: {error}"))),
    };
    let cache_facts: std::fs::Metadata = match request.cache.metadata() {
        Ok(facts) => facts,
        Err(error) => return Err(refused("staging-folder", format!("the cache folder could not be read: {error}"))),
    };
    let entry_count: usize = match crate::entries_of_one_open_folder::entry_names(staging) {
        Ok(names) => names.len(),
        Err(error) => return Err(refused("staging-folder", format!("the staging folder {name} could not be listed: {error}"))),
    };
    if folder_facts.uid() != cache_facts.uid() {
        return Err(refused("staging-folder", format!("the staging folder {name} is owned by {}, and the account's cache by {}", folder_facts.uid(), cache_facts.uid())));
    }
    if folder_facts.mode() & 0o7777 != 0o700 {
        return Err(refused("staging-folder", format!("the staging folder {name} has mode {:04o}, and the route records 0700", folder_facts.mode() & 0o7777)));
    }
    if entry_count != 0 {
        return Err(refused("staging-folder", format!("the staging folder {name} holds {entry_count} entries, and a staging folder is empty before its bind")));
    }
    let reading: RecordReading = read_staged_record(request.cache, request.expected.guid, request.boot, request.cache_path.join(name).as_path(), request.calls);
    let identity: StagedFolderIdentity = match StagedFolderIdentity::of(staging) {
        Ok(identity) => identity,
        Err(refusal) => return Err(refused("staged-record", refusal.message())),
    };
    if let RecordReading::Current { record, .. } = &reading {
        let moved = identity.differences(&record.identity);
        if !moved.is_empty() {
            return Err(refused(
                "staged-record",
                format!("the folder the record describes differs from the folder this invocation opened in the {}, so the record is kept and the bind refused", StagedFolderIdentity::field_names(moved.as_slice())),
            ));
        }
    }
    if BindingDecision::of(&reading) == BindingDecision::Refuse {
        let said: String = match reading {
            RecordReading::Refused { step, said } => format!("the staged record refused at its {step} step: {said}"),
            _ => String::from("the staged record refused"),
        };
        return Err(refused("staged-record", said));
    }
    let (handle_type, handle): (i32, Vec<u8>) = match request.calls.name_handle(staging) {
        Ok(handle) => handle,
        Err(refusal) => return Err(call_refused("name-handle", refusal, None)),
    };
    let (namespace_device, namespace_inode): (u64, u64) = match request.calls.namespace_pair() {
        Ok(pair) => pair,
        Err(refusal) => return Err(call_refused("namespace", refusal, None)),
    };
    let record: StagedFolderRecord = StagedFolderRecord {
        component: String::from(name),
        dataset: request.expected.source.clone(),
        guid: request.expected.guid,
        identity,
        handle_type,
        handle,
        nonce: String::from(request.nonce),
        namespace_device,
        namespace_inode,
        boot: String::from(request.boot),
    };
    return write_record(request.cache, &record).map_err(|refusal| return refused("staged-record", refusal.message()));
}

/// One topmost mount at the staging name.
fn topmost_at(calls: &dyn StagingCalls, cache: &std::fs::File, name: &str) -> Result<(std::fs::File, u64, MountFacts), StagingCallRefusal> {
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
fn unmount_the_pair(request: &UnmountRequest<'_>, held: &std::fs::File, bind_id: u64, name: &str, staged: std::fs::File) -> UnmountOutcome {
    // A descriptor on the clone would hold it busy; the reattach path reopens the folder by name after the kernel says
    // EBUSY, so nothing needs to keep this one alive.
    drop(staged);
    let call: Result<(), StagingCallRefusal> = request.calls.unmount_top(held);
    if let Err(refusal) = call {
        if refusal.errno == libc::EBUSY {
            let reopened: std::fs::File = match request.calls.open_by_name(request.cache, std::ffi::OsStr::new(name)) {
                Ok(reopened) => reopened,
                Err(open) => return UnmountOutcome::Retained { step: String::from(open.call.word()), errno: open.errno, said: open.message("the busy clone could not be reopened for its reattach"), mount: None },
            };
            let reopened_id: u64 = match request.calls.mount_id(&reopened) {
                Ok(id) => id,
                Err(read_failure) => return UnmountOutcome::Retained { step: String::from("mount-id"), errno: read_failure.errno, said: read_failure.message("the busy answer's mount could not be read"), mount: None },
            };
            let reopened_row = request.calls.mount_facts(reopened_id).ok();
            if reopened_id != request.expected.mount_id || reopened_row.as_ref().is_some_and(|row| return row.source != request.expected.source || row.root != std::path::Path::new("/")) {
                return UnmountOutcome::Retained {
                    step: String::from("pre-call-check"),
                    errno: 0,
                    said: format!("the mount the busy call answered for is {reopened_id} of source `{}`, and the clone is mount {} of `{}`. Nothing was moved.", reopened_row.as_ref().map_or_else(|| return String::from("unread"), |row| return row.source.clone()), request.expected.mount_id, request.expected.source),
                    mount: reopened_row,
                };
            }
            return match reattach(request, &reopened) {
                ReattachResult::Reattached { mount_id, slot } => {
                    let bind_gone: bool = unbind_if_empty(request.calls, request.cache, name, held, bind_id);
                    log_rolled_back(request, mount_id);
                    UnmountOutcome::Reattached { mount_id, slot, bind_gone, device: request.expected.device, inode: request.expected.inode, cause: ReattachCause::Busy }
                }
                ReattachResult::Refused { step, said } => UnmountOutcome::Retained { step, errno: 0, said, mount: None },
            };
        }
        return UnmountOutcome::Retained { step: String::from("unmount"), errno: refusal.errno, said: refusal.message("the clone's unmount refused"), mount: None };
    }
    let after: (std::fs::File, u64, MountFacts) = match topmost_at(request.calls, request.cache, name) {
        Ok(after) => after,
        Err(refusal) => return UnmountOutcome::Retained { step: String::from(refusal.call.word()), errno: refusal.errno, said: refusal.message("the clone's unmount could not be followed by a read of the bind"), mount: None },
    };
    if after.1 != bind_id {
        return UnmountOutcome::Retained {
            step: String::from("post-unmount"),
            errno: 0,
            said: format!("the clone's unmount returned, and the topmost mount at the staging folder is {} rather than the bind {bind_id}. The handler unbinds nothing and reports the row.", after.1),
            mount: Some(after.2),
        };
    }
    drop(after.0);
    if let Err(refusal) = request.calls.unmount_top(held) {
        return UnmountOutcome::Retained { step: String::from("unbind"), errno: refusal.errno, said: refusal.message("the bind's unmount refused"), mount: None };
    }
    return UnmountOutcome::Unmounted { mount_id: request.expected.mount_id, slot: request.expected.canonical_slot.clone(), bind_gone: true };
}

/// The checked move back of a staged clone to its slot.
fn reattach(request: &UnmountRequest<'_>, staged: &std::fs::File) -> ReattachResult {
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
fn unbind_if_empty(calls: &dyn StagingCalls, cache: &std::fs::File, name: &str, held: &std::fs::File, bind_id: u64) -> bool {
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
fn log_rolled_back(request: &UnmountRequest<'_>, mount_id: u64) {
    crate::diagnostic_log::debug("worktreeCloneUnmount", format!("the clone `{}` whose mount {mount_id} landed on a stacked mount is back at slot `{}`", request.expected.source, request.slot).as_str());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unmount_turn_tests.rs"]
mod tests;
