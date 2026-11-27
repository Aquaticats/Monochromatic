//! What:
//!  The finish of an interrupted unmount from the record's handle:
//!  the four states a killed removal leaves,
//!  the reading of a current record,
//!  and the call each state takes.
//! Why:
//!  The decision lets a later sweep finish an interrupted unmount from the
//!  record:
//!  a clone staged on the bind is unmounted there,
//!  and a bind left over an empty folder is unbound
//!  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "How a later sweep finishes an unmount").
//!  The recovery sits apart from the sweep that files a receipt,
//!  because the wrapper's arm reads its answer and nothing else does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = recoverAStagedUnmount(sweep, slot, cloneMountId);
//! ```

use super::staged_record_reading::{
    RecordReading,
    read_staged_record,
};
use super::staged_unmount_held::{
    HeldOutcome,
    HeldUnmount,
    unmount_the_staged_mount,
};
use super::staging_calls::StagingCalls;
use super::user_cache_pin::{
    UserCachePin,
    check_live_pin,
};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::Path;

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
///  What a later run found when it looked for an interrupted unmount.
/// Why:
///  The four kill points leave four distinct states,
///  and the next run must finish each from the record rather than refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecoveryOutcome = { kind: 'finished'; mountId } | { kind: 'bind-removed' } | { kind: 'not-staged'; why } | { kind: 'refused'; step; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryOutcome {
    /// The clone is gone and the bind is gone;
    /// a `done` receipt is owed.
    Finished {
        /// The clone's mount number when it was still there,
        /// or zero when the run only found the bind.
        mount_id: u64,
    },
    /// The bind covered an empty folder while the clone stood at its slot;
    /// the bind is now gone and the ordinary turn may bind again.
    BindRemoved,
    /// No current record describes a staged state.
    NotStaged,
    /// The state could not be finished,
    /// and the step says why.
    Refused {
        /// The step that refused.
        step: String,
        /// What refused.
        said: String,
    },
}

/// One topmost row at the folder path.
fn topmost_row(
    table: &[crate::fs_mount_options::MountFacts],
    path: &Path,
) -> Option<crate::fs_mount_options::MountFacts> {
    return table
        .iter()
        .filter(|facts| return facts.mount_point == path)
        .max_by_key(|facts| return facts.mount_id)
        .cloned();
}

/// What:
///  Finish an interrupted unmount before the normal turn runs.
/// Why:
///  The accepted staging design lets a later `unmount` intent's sweep complete
///  an interrupted removal from the record's handle:
///  a clone staged on the bind is unmounted there,
///  a bind left over an empty folder is unbound,
///  and a state with neither left files the receipt the waiting command reads.
///  The slot is the one path the record does not carry,
///  so the caller passes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = recoverAStagedUnmount(sweep, slot);
/// ```
pub fn recover_a_staged_unmount(
    sweep: &StagedSweep<'_>,
    slot: &Path,
    clone_mount_id: u64,
) -> RecoveryOutcome {
    let folder_path = sweep
        .cache_path
        .join(super::staging_folder_name::staging_folder_name(sweep.guid));
    let reading: RecordReading = read_staged_record(
        sweep.cache,
        sweep.guid,
        sweep.boot,
        folder_path.as_path(),
        sweep.calls,
    );
    let (record, folder): (
        super::staged_folder_record::StagedFolderRecord,
        std::fs::File,
    ) = match reading {
        RecordReading::Current {
            record,
            folder,
        } => (record, folder),
        RecordReading::Void {
            ..
        }
        | RecordReading::Unrecorded {
            ..
        }
        | RecordReading::Absent => return RecoveryOutcome::NotStaged,
        RecordReading::Refused {
            step,
            said,
        } => {
            return RecoveryOutcome::Refused {
                step,
                said,
            };
        }
    };
    if let Err(pin) = check_live_pin(sweep.cache, sweep.pin) {
        return RecoveryOutcome::Refused {
            step: String::from("user-cache-pin"),
            said: pin.message(),
        };
    }
    let table = match sweep.calls.mount_table() {
        Ok(table) => table,
        Err(refusal) => {
            return RecoveryOutcome::Refused {
                step: String::from(refusal.call.word()),
                said: refusal.message("the recovery could not read the mount table"),
            };
        }
    };
    let cache_facts = match sweep
        .calls
        .mount_facts(sweep.calls.mount_id(sweep.cache).unwrap_or(0))
    {
        Ok(facts) => facts,
        Err(refusal) => {
            return RecoveryOutcome::Refused {
                step: String::from(refusal.call.word()),
                said: refusal.message("the recovery could not read the cache's row"),
            };
        }
    };
    let Some(top) = topmost_row(table.as_slice(), folder_path.as_path()) else {
        return RecoveryOutcome::Finished {
            mount_id: clone_mount_id,
        };
    };
    if top.source == record.dataset && top.root == Path::new("/") {
        return match unmount_the_staged_mount(&HeldUnmount {
            cache: sweep.cache,
            folder: &folder,
            name: record.component.as_str(),
            clone_source: record.dataset.as_str(),
            calls: sweep.calls,
        }) {
            HeldOutcome::Unmounted {
                ..
            } => RecoveryOutcome::Finished {
                mount_id: top.mount_id,
            },
            HeldOutcome::Retained {
                step,
                said,
                ..
            } => RecoveryOutcome::Refused {
                step,
                said,
            },
        };
    }
    if top.source == cache_facts.source && top.root != Path::new("/") {
        let slot_source = table
            .iter()
            .find(|facts| return facts.mount_point == slot)
            .map(|facts| return facts.source.clone());
        let clone_at_slot: bool = slot_source.as_deref() == Some(record.dataset.as_str());
        if let Err(refusal) = sweep.calls.unmount_top(&folder) {
            return RecoveryOutcome::Refused {
                step: String::from(refusal.call.word()),
                said: refusal.message("the leftover bind could not be unbound"),
            };
        }
        return if clone_at_slot {
            RecoveryOutcome::BindRemoved
        } else {
            RecoveryOutcome::Finished {
                mount_id: clone_mount_id,
            }
        };
    }
    return RecoveryOutcome::Refused {
        step: String::from("pre-call-check"),
        said: format!(
            "the topmost mount at the staging folder is {} with source `{}`, which is neither the clone `{}` nor the cache bind `{}`",
            top.mount_id, top.source, record.dataset, cache_facts.source
        ),
    };
}
