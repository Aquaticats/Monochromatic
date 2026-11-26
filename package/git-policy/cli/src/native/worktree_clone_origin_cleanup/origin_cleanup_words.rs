//! What:
//!  The words the origin cleanup record and its folder are made of:
//!  the header line,
//!  the fifteen keys in the order the record writes them,
//!  the two states,
//!  the four block reasons,
//!  the folder's name,
//!  and the record's own name prefix.
//! Why:
//!  The project accepted one `key=value` format for the three removal-side records
//!  (`doc/decision/cli-git-zfs-removal-side-records-and-the-adds-record-accepted.md`,
//!  "Choice 2"),
//!  and this record's fields are its design's
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The fields and what proves the origin later").
//!  One file holds the closed sets,
//!  so a reader that meets a word outside them refuses the record rather than guessing,
//!  and the folder's name sits here because `doctor` finds the record by it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const header = '# monochromatic-git-policy-cli origin cleanup, version 1';
//! const keys = ['state', 'startedAt', ...] as const;
//! type State = 'owed' | 'blocked';
//! ```

/// The first line every origin cleanup record carries.
pub const ORIGIN_CLEANUP_HEADER: &str = "# monochromatic-git-policy-cli origin cleanup, version 1";

/// The folder inside the wrapper's own folder in the common Git directory,
/// which holds one record per leftover origin snapshot.
///
/// Why:
///  It sits beside the binding's home and the plans folder,
///  so a repository's rename machinery moves it with them,
///  and only this inner name is new.
pub const ORIGIN_CLEANUPS_FOLDER: &str = "origin-cleanups";

/// The prefix of every origin snapshot the add makes,
/// which is also this record's file name prefix.
///
/// Why:
///  The record is keyed by the snapshot's own name,
///  so one grammar spells the name and the file.
pub const ORIGIN_SNAPSHOT_PREFIX: &str = "git-worktree-clone-origin-";

/// The digits of the add's nonce a snapshot name carries,
/// which is the request nonce's own width.
pub const ORIGIN_SNAPSHOT_NONCE_DIGITS: usize = 32;

/// The bytes of one record's file name.
pub const ORIGIN_CLEANUP_NAME_BYTES: usize =
    ORIGIN_SNAPSHOT_PREFIX.len() + ORIGIN_SNAPSHOT_NONCE_DIGITS;

/// The plan's keys in the order its record writes them.
///
/// Why:
///  One order refuses a repeated key,
///  a missing key and a key written out of place with one rule,
///  so no reader has to guess which of two lines is the record's.
pub const ORIGIN_CLEANUP_KEYS_IN_ORDER: &[&str] = &[
    "state",
    "startedAt",
    "attemptedAt",
    "blockedReason",
    "blockedErrno",
    "numClones",
    "userRefs",
    "originDataset",
    "originDatasetGuid",
    "originSnapshot",
    "originSnapshotGuid",
    "originSnapshotCreatetxg",
    "cloneDataset",
    "cloneGuid",
    "slotPath",
];

/// What:
///  The two states a record stores.
/// Why:
///  Completion is not a state:
///  it is the record's absence,
///  and only an absence proof removes the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = 'owed' | 'blocked';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupState {
    /// The record exists and no attempt has recorded an outcome.
    Owed,
    /// An attempt ran and could not finish.
    Blocked,
}

impl CleanupState {
    /// The word the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            CleanupState::Owed => "owed",
            CleanupState::Blocked => "blocked",
        };
    }

    /// The state one stored word names.
    pub fn parse(word: &str) -> Option<CleanupState> {
        return match word {
            "owed" => Some(CleanupState::Owed),
            "blocked" => Some(CleanupState::Blocked),
            _ => None,
        };
    }
}

/// What:
///  Why an attempt could not finish.
/// Why:
///  The four reasons are a closed set a person acts on differently:
///  a dependency clears when the clone or hold goes,
///  a deferral when its owner destroys the snapshot or the mark goes,
///  an unproved identity when the pool answers the record's objects again,
///  and a refusal when the pool lets a later request through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reason = 'dependency' | 'deferred' | 'unproved' | 'refused';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockReason {
    /// A live clone or user hold pins the snapshot.
    Dependency,
    /// The snapshot is marked for deferred destroy.
    Deferred,
    /// The live objects are not the record's,
    /// so nothing is proven.
    Unproved,
    /// The destroy request was refused,
    /// or its absence proof still found the snapshot.
    Refused,
}

impl BlockReason {
    /// The code the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            BlockReason::Dependency => "dependency",
            BlockReason::Deferred => "deferred",
            BlockReason::Unproved => "unproved",
            BlockReason::Refused => "refused",
        };
    }

    /// The reason one stored code names.
    pub fn parse(word: &str) -> Option<BlockReason> {
        return match word {
            "dependency" => Some(BlockReason::Dependency),
            "deferred" => Some(BlockReason::Deferred),
            "unproved" => Some(BlockReason::Unproved),
            "refused" => Some(BlockReason::Refused),
            _ => None,
        };
    }

    /// The clause a line about this reason begins with.
    pub fn clause(self) -> &'static str {
        return match self {
            BlockReason::Dependency => "it is still pinned",
            BlockReason::Deferred => "it is marked for deferred destroy",
            BlockReason::Unproved => "the objects the record names are not the ones the pool has",
            BlockReason::Refused => "the pool refused the request",
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_words_tests.rs"]
mod tests;
