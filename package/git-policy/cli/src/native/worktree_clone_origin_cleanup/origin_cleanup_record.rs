//! What:
//!  The origin cleanup record as a value:
//!  its identity lines,
//!  its state lines,
//!  the pool one destroy names,
//!  and the bytes it publishes.
//! Why:
//!  The record carries the origin snapshot past the clone and the registration,
//!  so its identity lines are written once and never change,
//!  only its state lines are replaced,
//!  and a blocked record carries the two counts or the errno a person acts on
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The lines, the encodings and the order the reader requires").
//!  The total is one record with no free-text field:
//!  a field a person can write into is a field that steers a report.
//!  This file writes one record;
//!  the parse it must agree with is `origin_cleanup_parse.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = OriginCleanupRecord.owed({ origin, clone, slotPath });
//! const bytes = record.encode(); // throws when a field is past its bound
//! ```

use crate::worktree_clone_binding_record::binding_encoding::path_as_hexadecimal;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_origin_cleanup::origin_cleanup_bounds::field_maximum_bytes;
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::{CleanupRefusal, CleanupRefusalCause};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::{
    BlockReason, CleanupState, ORIGIN_CLEANUP_HEADER, ORIGIN_CLEANUP_KEYS_IN_ORDER, ORIGIN_SNAPSHOT_PREFIX,
};
use std::path::{Path, PathBuf};

/// What:
///  One blocked attempt's reason and the numbers a person acts on.
/// Why:
///  The state lines come first so a person reading a blocked record sees the reason at the top,
///  and the counts travel with the reason code because a dependency clears when they reach zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Block = { reason; errno; clones; holds };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CleanupBlock {
    /// Why the attempt could not finish.
    pub reason: BlockReason,
    /// The errno of a refused request,
    /// `0` when no request was made.
    pub errno: u32,
    /// The live clone count the attempt read.
    pub clones: u64,
    /// The live user hold count the attempt read.
    pub holds: u64,
}

impl CleanupBlock {
    /// The detail a printed line reports,
    /// which is what decides whether the reason changed.
    pub fn detail(&self) -> String {
        return format!("{} clone(s), {} hold(s), errno {}", self.clones, self.holds, self.errno);
    }
}

/// What:
///  One origin snapshot's cleanup record,
///  as a value.
/// Why:
///  The writer,
///  the removal's attempt,
///  the look and `doctor` all read this one shape,
///  so a field one of them needs is necessarily in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Record = { state; startedAt; attemptedAt?; block?; origin; clone; slotPath };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OriginCleanupRecord {
    /// Whether an attempt has recorded an outcome.
    pub state: CleanupState,
    /// When the writing command first wrote the record.
    pub started_at: String,
    /// When the last attempt ran,
    /// absent until one recorded an outcome.
    pub attempted_at: Option<String>,
    /// The reason and counts of a blocked attempt,
    /// present exactly when the state is blocked.
    pub block: Option<CleanupBlock>,
    /// The checkout dataset's full name as the add recorded it.
    pub origin_dataset: String,
    /// The checkout dataset's GUID.
    pub origin_dataset_guid: DatasetGuid,
    /// The origin snapshot's own name,
    /// which is also the record's file name.
    pub origin_snapshot: String,
    /// The origin snapshot's GUID.
    pub origin_snapshot_guid: DatasetGuid,
    /// The transaction group that created the origin snapshot.
    pub origin_snapshot_createtxg: u64,
    /// The clone's full dataset name.
    pub clone_dataset: String,
    /// The clone's GUID,
    /// absent exactly when no clone was made.
    pub clone_guid: Option<DatasetGuid>,
    /// The slot's path.
    pub slot_path: PathBuf,
}

impl OriginCleanupRecord {
    /// What:
    ///  A record whose cleanup no attempt has attempted yet.
    /// Why:
    ///  This is the moment the design fixes:
    ///  the removal publishes it after the pre-delete refusals and before its first destroy request,
    ///  because the record is what carries the cleanup past the clone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const record = OriginCleanupRecord.owed({ origin, clone, slotPath, startedAt });
    /// ```
    pub fn owed(
        origin_dataset: &str,
        origin_dataset_guid: DatasetGuid,
        origin_snapshot: &str,
        origin_snapshot_guid: DatasetGuid,
        origin_snapshot_createtxg: u64,
        clone_dataset: &str,
        clone_guid: DatasetGuid,
        slot_path: &Path,
        started_at: &str,
    ) -> OriginCleanupRecord {
        return OriginCleanupRecord {
            state: CleanupState::Owed,
            started_at: String::from(started_at),
            attempted_at: None,
            block: None,
            origin_dataset: String::from(origin_dataset),
            origin_dataset_guid,
            origin_snapshot: String::from(origin_snapshot),
            origin_snapshot_guid,
            origin_snapshot_createtxg,
            clone_dataset: String::from(clone_dataset),
            clone_guid: Some(clone_guid),
            slot_path: slot_path.to_path_buf(),
        };
    }

    /// What:
    ///  The same record with one attempt's blocked outcome on it.
    /// Why:
    ///  A rewrite re-emits the whole record with the state lines changed,
    ///  so the identity lines cannot be rewritten by a later attempt.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const blocked = record.blocked({ reason, errno, clones, holds }, attemptedAt);
    /// ```
    pub fn blocked(&self, block: CleanupBlock, attempted_at: &str) -> OriginCleanupRecord {
        let mut next: OriginCleanupRecord = self.clone();
        next.state = CleanupState::Blocked;
        next.attempted_at = Some(String::from(attempted_at));
        next.block = Some(block);
        return next;
    }

    /// The origin snapshot's full name,
    /// as a destroy request names it.
    pub fn full_snapshot_name(&self) -> String {
        return format!("{}@{}", self.origin_dataset, self.origin_snapshot);
    }

    /// The pool the destroy request names,
    /// which is the origin dataset's first `/`-separated component.
    ///
    /// # Throws
    /// A dataset name with no component,
    /// which no record may carry.
    pub fn pool(&self) -> Result<String, CleanupRefusal> {
        let Some(first) = self.origin_dataset.split('/').next() else {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "originDataset", String::from("names no pool")));
        };
        if first.is_empty() {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "originDataset", String::from("names no pool")));
        }
        return Ok(String::from(first));
    }

    /// What:
    ///  The record's bytes,
    ///  its key order and its fields checked against the bounds they must satisfy.
    /// Why:
    ///  A writer that published past a bound would leave a record its own reader refuses,
    ///  and a blocked record without its counts would be one a person cannot act on.
    ///
    /// # Throws
    /// The field that is past its bound,
    /// the state line its own state forbids or requires,
    /// or a value that is not the spelling its kind allows.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const bytes = record.encode();
    /// ```
    pub fn encode(&self) -> Result<Vec<u8>, CleanupRefusal> {
        let values: [Option<String>; 15] = [
            Some(String::from(self.state.word())),
            Some(self.started_at.clone()),
            self.attempted_at.clone(),
            self.block.map(|held| return String::from(held.reason.word())),
            self.block.map(|held| return held.errno.to_string()),
            self.block.map(|held| return held.clones.to_string()),
            self.block.map(|held| return held.holds.to_string()),
            Some(self.origin_dataset.clone()),
            Some(self.origin_dataset_guid.file_name()),
            Some(self.origin_snapshot.clone()),
            Some(self.origin_snapshot_guid.file_name()),
            Some(self.origin_snapshot_createtxg.to_string()),
            Some(self.clone_dataset.clone()),
            self.clone_guid.map(|guid| return guid.file_name()),
            Some(path_as_hexadecimal(self.slot_path.as_path())),
        ];
        let mut text: String = String::from(ORIGIN_CLEANUP_HEADER);
        text.push('\n');
        for (place, key) in ORIGIN_CLEANUP_KEYS_IN_ORDER.iter().enumerate() {
            let Some(value) = &values[place] else { continue };
            let widest: usize = field_maximum_bytes(key).unwrap_or(0);
            if value.len() > widest {
                return Err(CleanupRefusal::at(CleanupRefusalCause::FieldTooWide, key, value.clone()));
            }
            text.push_str(key);
            text.push('=');
            text.push_str(value.as_str());
            text.push('\n');
        }
        self.refuse_a_state_its_lines_do_not_say()?;
        return Ok(text.into_bytes());
    }

    /// The state lines a record's own state forbids or requires.
    fn refuse_a_state_its_lines_do_not_say(&self) -> Result<(), CleanupRefusal> {
        if self.origin_snapshot.len() != ORIGIN_SNAPSHOT_PREFIX.len() + 32 {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "originSnapshot", self.origin_snapshot.clone()));
        }
        match (self.state, self.attempted_at.is_some(), self.block.is_some()) {
            (CleanupState::Owed, false, false) => Ok(()),
            (CleanupState::Blocked, true, true) => Ok(()),
            (state, attempted, block) => Err(CleanupRefusal::at(
                CleanupRefusalCause::UnexpectedKey,
                "state",
                format!("`{}` with attemptedAt={attempted} and blockedReason={block}", state.word()),
            )),
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_record_tests.rs"]
mod tests;
