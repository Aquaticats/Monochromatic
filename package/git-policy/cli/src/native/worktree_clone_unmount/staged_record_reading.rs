//! What:
//!  The order the staged-folder record is judged in:
//!  the boot id first and on its own,
//!  the handle second,
//!  and the folder's identity only after both,
//!  and the three answers a binding invocation and a later sweep need.
//! Why:
//!  A record from another boot proves that no invocation of this boot bound the
//!  folder,
//!  and its handle and namespace pair are not consulted
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staged-folder record").
//!  Of this boot,
//!  `ESTALE` with no mount row at the folder path voids,
//!  because that pair is the design's test and neither half proves the inode
//!  died;
//!  a failure other than `ESTALE` refuses rather than voiding,
//!  because the record may still be the only handle to a live mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reading = readStagedRecord(cache, guid, boot, folderPath, calls);
//! if (reading.kind === 'current') reuseFolder(reading.folder);
//! ```

use super::staged_folder_identity::{StagedFolderField, StagedFolderIdentity};
use super::staged_folder_record::{read_record_text, record_from_text, StagedFolderRecord, StagedFolderRefusal};
use super::staging_calls::{StagingCall, StagingCallRefusal, StagingCalls};
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{Path, PathBuf};

/// What:
///  What one read of the record beside a GUID's folder found.
/// Why:
///   A binding invocation replaces an absent,
///  unrecorded or void record before it binds and refuses a current one whose
///   identity moved;
///  a later sweep opens the folder only from a current record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordReading = { kind: 'absent' } | { kind: 'unrecorded'; why } | { kind: 'void'; why } | { kind: 'current'; record; folder } | { kind: 'refused'; step; said };
/// ```
pub enum RecordReading {
    /// No record stands.
    Absent,
    /// The file is there but is not one this route acts on.
    Unrecorded {
        /// Why it was read as unrecorded.
        why: String,
    },
    /// The record describes no folder this boot can act on.
    Void {
        /// Why it is void.
        why: String,
    },
    /// The record describes a folder that still exists and matches it.
    Current {
        /// The record.
        record: StagedFolderRecord,
        /// The folder reopened from its handle.
        folder: std::fs::File,
    },
    /// The record cannot be judged,
    /// so nothing is acted on it.
    Refused {
        /// The step the refusal names.
        step: String,
        /// What refused,
        /// in words.
        said: String,
    },
}

impl RecordReading {
    /// The word a sweep or an arm reports for this reading.
    pub fn word(&self) -> &'static str {
        return match self {
            RecordReading::Absent => "absent",
            RecordReading::Unrecorded { .. } => "unrecorded",
            RecordReading::Void { .. } => "void",
            RecordReading::Current { .. } => "current",
            RecordReading::Refused { .. } => "refused",
        };
    }

    /// The record a binding invocation replaces before it binds.
    pub fn is_replaceable(&self) -> bool {
        return matches!(self, RecordReading::Absent | RecordReading::Unrecorded { .. } | RecordReading::Void { .. });
    }
}

/// Whether this process's mount table has a row whose mount point is the folder's path.
///
/// Why:
///  The second half of the design's folder-gone test is the path key,
///  and a live bind or clone at the dentry leaves that row.
fn a_row_names_the_path(calls: &dyn StagingCalls, folder_path: &Path) -> Result<bool, StagingCallRefusal> {
    let table = calls.mount_table()?;
    for facts in table {
        if facts.mount_point == folder_path {
            return Ok(true);
        }
    }
    return Ok(false);
}

/// What:
///  Judge one record against this boot,
///  this process's mount table and the folder its handle reopens.
/// Why:
///  The order is the design's,
///  and the path key is the second test rather than a source comparison,
///  so a dead dentry is told from a live one the table can still see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readStagedRecord(cache, guid, boot, folderPath, calls);
/// ```
pub fn read_staged_record(cache: &std::fs::File, guid: DatasetGuid, boot: &str, folder_path: &Path, calls: &dyn StagingCalls) -> RecordReading {
    let text: String = match read_record_text(cache, guid) {
        Ok(Some(text)) => text,
        Ok(None) => return RecordReading::Absent,
        Err(StagedFolderRefusal::Unreadable { path, reason }) => return RecordReading::Refused { step: String::from(StagingCall::MountTable.word()), said: format!("{path} could not be read: {reason}") },
        Err(refusal) => return RecordReading::Unrecorded { why: refusal.message() },
    };
    let record: StagedFolderRecord = match record_from_text(text.as_str(), guid) {
        Ok(record) => record,
        Err(refusal) => return RecordReading::Unrecorded { why: refusal.message() },
    };
    if record.boot != boot {
        return RecordReading::Void { why: format!("the record names boot `{}`, and this boot is `{boot}`; its handle and namespace pair are not consulted", record.boot) };
    }
    let folder: std::fs::File = match calls.open_from_handle(cache, record.handle_type, record.handle.as_slice()) {
        Ok(folder) => folder,
        Err(refusal) if refusal.errno == libc::ESTALE => {
            return match a_row_names_the_path(calls, folder_path) {
                Ok(false) => RecordReading::Void { why: String::from("the handle answered ESTALE and no row of this process's mount table names the folder path, so the folder is gone") },
                Ok(true) => RecordReading::Refused { step: String::from(StagingCall::OpenHandle.word()), said: format!("the handle answered ESTALE while a mount row still names {}, so the failure does not prove the folder is gone", folder_path.display()) },
                Err(call) => RecordReading::Refused { step: String::from(StagingCall::MountTable.word()), said: call.message("the folder-gone test could not read this process's mount table") },
            };
        }
        Err(refusal) => return RecordReading::Refused { step: String::from(StagingCall::OpenHandle.word()), said: refusal.message("the record's handle could not be opened") },
    };
    let now: StagedFolderIdentity = match StagedFolderIdentity::of(&folder) {
        Ok(now) => now,
        Err(refusal) => return RecordReading::Refused { step: String::from(StagingCall::MountId.word()), said: refusal.message() },
    };
    let fields: Vec<StagedFolderField> = now.differences(&record.identity);
    if !fields.is_empty() {
        return RecordReading::Refused {
            step: String::from("identity"),
            said: format!(
                "the folder the record's handle reopens differs from the record in the {}; the record describes identity {} and the folder reads identity {}",
                StagedFolderIdentity::field_names(fields.as_slice()),
                record.identity.describe(),
                now.describe()
            ),
        };
    }
    return RecordReading::Current { record, folder };
}

/// What:
///  What a binding invocation does with one reading before it binds.
/// Why:
///  A replaceable record is rewritten from the folder the handler is about to
///  bind;
///  a current record is rewritten under the new nonce;
///  a refused record stops the bind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decision = bindingDecision(reading); // 'write-fresh' | 'rewrite' | 'refuse'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingDecision {
    /// Write a fresh record before the bind.
    WriteFresh,
    /// Rewrite the current record before the bind,
    /// keeping the folder it proves.
    Rewrite,
    /// Refuse before the bind.
    Refuse,
}

impl BindingDecision {
    /// The decision one reading gives.
    pub fn of(reading: &RecordReading) -> BindingDecision {
        return match reading {
            RecordReading::Absent | RecordReading::Unrecorded { .. } | RecordReading::Void { .. } => BindingDecision::WriteFresh,
            RecordReading::Current { .. } => BindingDecision::Rewrite,
            RecordReading::Refused { .. } => BindingDecision::Refuse,
        };
    }

    /// The word an arm reports.
    pub fn word(self) -> &'static str {
        return match self {
            BindingDecision::WriteFresh => "write-fresh",
            BindingDecision::Rewrite => "rewrite",
            BindingDecision::Refuse => "refuse",
        };
    }
}

/// The folder's path beneath the cache folder,
/// for the path-keyed test and a report.
pub fn folder_path_beneath(cache_path: &Path, component: &str) -> PathBuf {
    return cache_path.join(component);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staged_record_reading_tests.rs"]
mod tests;
