//! What:
//!  The unmount of a staged mount from the descriptor a process holds,
//!  and the later sweep that holds that descriptor again from the record's
//!  file handle.
//! Why:
//!  The design's interface is "the unmount from the held descriptor"
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The unmount call from staging"):
//!  the first `umount2` takes the clone,
//!  the second takes the bind,
//!  and each call checks the topmost mount before it and the transition after
//!  it.
//!  A later sweep has no descriptor of its own,
//!  so it opens the folder from the record's handle and then makes the same
//!  calls.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = unmountTheStagedMount({ cache, name, cloneSource, calls });
//! const swept = finishAStagedUnmount({ cache, guid, boot, calls });
//! ```

use super::staged_record_reading::{read_staged_record, RecordReading};
use super::staging_calls::{StagingCallRefusal, StagingCalls};
use super::staging_folder_name::staging_record_name;
use super::user_cache_pin::{check_live_pin, UserCachePin};
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::ffi::OsStr;
use std::path::Path;

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
#[allow(clippy::large_enum_variant, reason = "the retained alternative names the step, words and row a person acts on")]
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
    let top: (std::fs::File, u64, MountFacts) = match topmost(request.calls, request.cache, request.name) {
        Ok(top) => top,
        Err(refusal) => return retained(refusal, None),
    };
    let clone_mount: u64 = top.1;
    if top.2.source != request.clone_source {
        return HeldOutcome::Retained {
            step: String::from("pre-call-check"),
            errno: 0,
            said: format!("the topmost mount at the staging folder is {} with source `{}`, and the clone is `{}`. Nothing was unmounted.", top.1, top.2.source, request.clone_source),
            mount: Some(top.2),
        };
    }
    drop(top.0);
    if let Err(refusal) = request.calls.unmount_top(request.folder) {
        return retained(refusal, None);
    }
    let after: (std::fs::File, u64, MountFacts) = match topmost(request.calls, request.cache, request.name) {
        Ok(after) => after,
        Err(refusal) => return retained(refusal, None),
    };
    if after.1 == clone_mount {
        return HeldOutcome::Retained { step: String::from("post-call-check"), errno: 0, said: format!("the clone's unmount returned and mount {clone_mount} is still topmost"), mount: Some(after.2) };
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
                return HeldOutcome::Retained { step: String::from("post-call-check"), errno: 0, said: format!("the bind's unmount returned and mount {bind_mount} is still topmost"), mount: Some(facts) };
            }
            HeldOutcome::Unmounted { clone_mount, bind_mount }
        }
        Err(refusal) => retained(refusal, None),
    };
}

/// One topmost mount by name.
fn topmost(calls: &dyn StagingCalls, cache: &std::fs::File, name: &str) -> Result<(std::fs::File, u64, MountFacts), StagingCallRefusal> {
    let folder: std::fs::File = calls.open_by_name(cache, OsStr::new(name))?;
    let mount_id: u64 = calls.mount_id(&folder)?;
    let facts: MountFacts = calls.mount_facts(mount_id)?;
    return Ok((folder, mount_id, facts));
}

/// One call's refusal as a retained outcome.
fn retained(refusal: StagingCallRefusal, mount: Option<MountFacts>) -> HeldOutcome {
    return HeldOutcome::Retained { step: String::from(refusal.call.word()), errno: refusal.errno, said: refusal.message("the staged unmount refused"), mount };
}

/// What:
///  One later sweep's reading of a record.
/// Why:
///  The sweep opens the folder from the handle and makes the held-descriptor
///  calls,
///  so a fresh process finishes what a killed handler left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Sweep = { cache; cachePath; guid; boot; pin; calls };
/// ```
pub struct StagedSweep<'a> {
    /// The account's cache descriptor.
    pub cache: &'a std::fs::File,
    /// The account's cache folder.
    pub cache_path: &'a Path,
    /// The clone's GUID.
    pub guid: DatasetGuid,
    /// This boot's identifier.
    pub boot: &'a str,
    /// The `user-cache` pin.
    pub pin: &'a UserCachePin,
    /// The mount calls.
    pub calls: &'a dyn StagingCalls,
}

/// What:
///  What one sweep found.
/// Why:
///  The wrapper and `doctor` need one word and the record's folder name,
///  and a retained mount needs the step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepOutcome = { kind: 'unmounted' | 'empty' | 'absent' | 'void' | 'unrecorded' | 'refused' | 'retained'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SweepOutcome {
    /// The record's folder is gone and no mount row names it.
    FolderGone,
    /// The record is void,
    /// unrecorded or absent,
    /// and a sweep is a non-binding pass.
    NotCurrent {
        /// The reading's word.
        reading: String,
        /// Why,
        /// where the reading says.
        why: String,
    },
    /// The folder holds no mount.
    Empty,
    /// The clone and its bind left.
    Unmounted {
        /// The clone's mount number.
        clone_mount: u64,
        /// The bind's mount number.
        bind_mount: u64,
    },
    /// A mount stays,
    /// and the step says what refused.
    Retained {
        /// The step that refused.
        step: String,
        /// What refused.
        said: String,
    },
}

impl SweepOutcome {
    /// The word a receipt or an arm carries.
    pub fn word(&self) -> &'static str {
        return match self {
            SweepOutcome::FolderGone => "folder-gone",
            SweepOutcome::NotCurrent { .. } => "not-current",
            SweepOutcome::Empty => "empty",
            SweepOutcome::Unmounted { .. } => "unmounted",
            SweepOutcome::Retained { .. } => "retained",
        };
    }
}

/// What:
///  Finish an interrupted removal from the record's handle.
/// Why:
///  A sweep without the record's handle cannot make the call,
///  and a record from another boot is settled by the wrapper rather than
///  rewritten here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = finishAStagedUnmount(sweep);
/// ```
pub fn finish_a_staged_unmount(sweep: &StagedSweep<'_>) -> SweepOutcome {
    let folder_path = sweep.cache_path.join(super::staging_folder_name::staging_folder_name(sweep.guid));
    let reading: RecordReading = read_staged_record(sweep.cache, sweep.guid, sweep.boot, folder_path.as_path(), sweep.calls);
    let (record, folder): (super::staged_folder_record::StagedFolderRecord, std::fs::File) = match reading {
        RecordReading::Current { record, folder } => (record, folder),
        RecordReading::Void { why } => return SweepOutcome::NotCurrent { reading: String::from("void"), why },
        RecordReading::Unrecorded { why } => return SweepOutcome::NotCurrent { reading: String::from("unrecorded"), why },
        RecordReading::Absent => return SweepOutcome::NotCurrent { reading: String::from("absent"), why: String::from("no staged-folder record stands") },
        RecordReading::Refused { step, said } => return SweepOutcome::Retained { step, said },
    };
    let cache_mount: u64 = match sweep.calls.mount_id(sweep.cache) {
        Ok(mount) => mount,
        Err(refusal) => return SweepOutcome::Retained { step: String::from(refusal.call.word()), said: refusal.message("the cache's mount could not be read") },
    };
    let held_mount: u64 = match sweep.calls.mount_id(&folder) {
        Ok(mount) => mount,
        Err(refusal) => return SweepOutcome::Retained { step: String::from(refusal.call.word()), said: refusal.message("the record's folder's mount could not be read") },
    };
    if held_mount != cache_mount {
        return SweepOutcome::Retained { step: String::from("open-handle"), said: format!("the record's handle reopens the folder on mount {held_mount}, and the cache descriptor is on mount {cache_mount}; the handle names no folder on the cache") };
    }
    // The handle names the folder's inode on the cache mount, so it cannot see a mount covering that dentry. The
    // mount table is the read that says whether anything stands there.
    let table = match sweep.calls.mount_table() {
        Ok(table) => table,
        Err(refusal) => return SweepOutcome::Retained { step: String::from(refusal.call.word()), said: refusal.message("the sweep's mount table could not be read") },
    };
    if !table.iter().any(|facts| return facts.mount_point == folder_path) {
        return SweepOutcome::Empty;
    }
    if let Err(pin) = check_live_pin(sweep.cache, sweep.pin) {
        return SweepOutcome::Retained { step: String::from("user-cache-pin"), said: pin.message() };
    }
    let name: String = record.component;
    match unmount_the_staged_mount(&HeldUnmount { cache: sweep.cache, folder: &folder, name: name.as_str(), clone_source: record.dataset.as_str(), calls: sweep.calls }) {
        HeldOutcome::Unmounted { clone_mount, bind_mount } => return SweepOutcome::Unmounted { clone_mount, bind_mount },
        HeldOutcome::Retained { step, said, .. } => return SweepOutcome::Retained { step, said },
    }
}

/// The record's name,
/// which the wrapper unlinks after the folder's `rmdir`.
pub fn record_name(guid: DatasetGuid) -> String {
    return staging_record_name(guid);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staged_unmount_tests.rs"]
mod tests;
