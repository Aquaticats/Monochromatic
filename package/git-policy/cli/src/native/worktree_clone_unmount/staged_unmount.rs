//! What:
//!  The later sweep of a staged folder:
//!  the reading of one record from the cache,
//!  the two mount numbers that must be the cache's and the folder's,
//!  the pin check,
//!  the held-descriptor unmount it hands the pair to,
//!  and the word the wrapper and `doctor` answer with.
//! Why:
//!  A sweep has no descriptor of its own,
//!  so it opens the folder from the record's handle and then makes the same
//!  calls a held descriptor makes
//!  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "How a later sweep finishes an unmount").
//!  A record from another boot is settled by the wrapper rather than rewritten
//!  here,
//!  and a sweep over a void or absent record is a non-binding pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = finishAStagedUnmount(sweep);
//! const record = recordName(guid);
//! ```

use super::staged_record_reading::{
    RecordReading,
    read_staged_record,
};
use super::staging_folder_name::staging_record_name;
use super::user_cache_pin::check_live_pin;
use crate::worktree_clone_mount_protocol::DatasetGuid;

/// The held-descriptor unmount and the answer it leaves.
pub use super::staged_unmount_held::{
    HeldOutcome,
    HeldUnmount,
    unmount_the_staged_mount,
};

/// The finish of an interrupted unmount and the answer it leaves.
pub use super::staged_unmount_recovery::{
    RecoveryOutcome,
    StagedSweep,
    recover_a_staged_unmount,
};

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
            SweepOutcome::NotCurrent {
                ..
            } => "not-current",
            SweepOutcome::Empty => "empty",
            SweepOutcome::Unmounted {
                ..
            } => "unmounted",
            SweepOutcome::Retained {
                ..
            } => "retained",
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
            why,
        } => {
            return SweepOutcome::NotCurrent {
                reading: String::from("void"),
                why,
            };
        }
        RecordReading::Unrecorded {
            why,
        } => {
            return SweepOutcome::NotCurrent {
                reading: String::from("unrecorded"),
                why,
            };
        }
        RecordReading::Absent => {
            return SweepOutcome::NotCurrent {
                reading: String::from("absent"),
                why: String::from("no staged-folder record stands"),
            };
        }
        RecordReading::Refused {
            step,
            said,
        } => {
            return SweepOutcome::Retained {
                step,
                said,
            };
        }
    };
    let cache_mount: u64 = match sweep.calls.mount_id(sweep.cache) {
        Ok(mount) => mount,
        Err(refusal) => {
            return SweepOutcome::Retained {
                step: String::from(refusal.call.word()),
                said: refusal.message("the cache's mount could not be read"),
            };
        }
    };
    let held_mount: u64 = match sweep.calls.mount_id(&folder) {
        Ok(mount) => mount,
        Err(refusal) => {
            return SweepOutcome::Retained {
                step: String::from(refusal.call.word()),
                said: refusal.message("the record's folder's mount could not be read"),
            };
        }
    };
    if held_mount != cache_mount {
        return SweepOutcome::Retained {
            step: String::from("open-handle"),
            said: format!(
                "the record's handle reopens the folder on mount {held_mount}, and the cache descriptor is on mount {cache_mount}; the handle names no folder on the cache"
            ),
        };
    }
    // The handle names the folder's inode on the cache mount, so it cannot see a mount covering that dentry. The
    // mount table is the read that says whether anything stands there.
    let table = match sweep.calls.mount_table() {
        Ok(table) => table,
        Err(refusal) => {
            return SweepOutcome::Retained {
                step: String::from(refusal.call.word()),
                said: refusal.message("the sweep's mount table could not be read"),
            };
        }
    };
    if !table
        .iter()
        .any(|facts| return facts.mount_point == folder_path)
    {
        return SweepOutcome::Empty;
    }
    if let Err(pin) = check_live_pin(sweep.cache, sweep.pin) {
        return SweepOutcome::Retained {
            step: String::from("user-cache-pin"),
            said: pin.message(),
        };
    }
    let name: String = record.component;
    match unmount_the_staged_mount(&HeldUnmount {
        cache: sweep.cache,
        folder: &folder,
        name: name.as_str(),
        clone_source: record.dataset.as_str(),
        calls: sweep.calls,
    }) {
        HeldOutcome::Unmounted {
            clone_mount,
            bind_mount,
        } => {
            return SweepOutcome::Unmounted {
                clone_mount,
                bind_mount,
            };
        }
        HeldOutcome::Retained {
            step,
            said,
            ..
        } => {
            return SweepOutcome::Retained {
                step,
                said,
            };
        }
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
