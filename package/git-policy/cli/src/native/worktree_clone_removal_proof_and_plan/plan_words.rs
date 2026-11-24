//! What:
//!  The words a destruction plan's lines are made of:
//!  the header line,
//!  the eighteen keys in the order it writes them,
//!  the snapshot outcome words,
//!  the head's words,
//!  and the two words that stand for an absent value.
//! Why:
//!  The project accepted one `key=value` format for the three removal-side records
//!  (`doc/decision/cli-git-zfs-removal-side-records-and-the-adds-record-accepted.md`,
//!  "Choice 2"),
//!  and one line per attempt for a snapshot's history (its "Choice 32").
//!  The header line and the key order are the plan's whole grammar,
//!  so they live in one file both the writer and the reader take,
//!  and the header string is step 10's own constant rather than a second copy:
//!  the add already refuses a removal's plan by reading it
//!  (`worktree_clone_binding_record/plan_folder_read.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const keys = ['run', 'owner', ...] as const;
//! type Outcome = 'planned' | 'destroyed' | 'refused' | 'intact' | 'unknown';
//! ```

/// The first line every destruction plan carries,
/// taken from the add's own reader so one string serves both.
pub use crate::worktree_clone_binding_record::plan_folder_read::PLAN_HEADER;

/// The plan's keys in the order its header writes them.
///
/// Why:
///  One order refuses a repeated key, a missing key and a key written out of place with
///  one rule, so no reader has to guess which of two lines is the record's.
pub const PLAN_KEYS_IN_ORDER: [&str; 18] = [
    "run",
    "owner",
    "started",
    "cloneDataset",
    "cloneGuid",
    "cloneCreatetxg",
    "originDataset",
    "originDatasetGuid",
    "originSnapshot",
    "originSnapshotGuid",
    "originSnapshotCreatetxg",
    "registrationIdHex",
    "admin",
    "targetHex",
    "unmountNonce",
    "unmountMountId",
    "head",
    "attempts",
];

/// The one attempt kind this grammar records.
///
/// Why:
///  The decided record carries no bookmark entry, so a reader that meets another kind
///  refuses the record rather than skipping a line it does not understand.
pub const SNAPSHOT_KIND: &str = "snapshot";

/// The key every attempt line begins with.
pub const ATTEMPT_KEY: &str = "tuple";

/// The key that ends every attempt line,
/// which is last so a snapshot name may hold the space the kernel's charset allows.
pub const ATTEMPT_NAME_KEY: &str = "name";

/// The word standing for an absent mount id,
/// which is what an unmount receipt and a forced receipt carry.
pub const NO_MOUNT_ID: &str = "none";

/// The errno of a line that reports no errno.
pub const NO_ERRNO: &str = "-";

/// What:
///  One snapshot's state on one line of its history.
/// Why:
///  The merge rule reads a snapshot's current state as the outcome on its highest
///  attempt number, so the five words have to be one closed set both the writer and the
///  reader take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = 'planned' | 'destroyed' | 'refused' | 'intact' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotOutcome {
    /// The plan's first write recorded the snapshot among the whole verified inventory.
    Planned,
    /// The removal proved the object gone,
    /// which is the only end a name's history has.
    Destroyed,
    /// The kernel's per-name result carried an error for this snapshot.
    Refused,
    /// The request returned non-zero and this name was not among the failures,
    /// or a later read still found the object.
    Intact,
    /// The request's outcome cannot be attributed to names at all.
    Unknown,
}

impl SnapshotOutcome {
    /// The word the line carries.
    pub fn word(self) -> &'static str {
        return match self {
            SnapshotOutcome::Planned => "planned",
            SnapshotOutcome::Destroyed => "destroyed",
            SnapshotOutcome::Refused => "refused",
            SnapshotOutcome::Intact => "intact",
            SnapshotOutcome::Unknown => "unknown",
        };
    }

    /// The outcome one stored word names.
    pub fn parse(word: &str) -> Option<SnapshotOutcome> {
        return match word {
            "planned" => Some(SnapshotOutcome::Planned),
            "destroyed" => Some(SnapshotOutcome::Destroyed),
            "refused" => Some(SnapshotOutcome::Refused),
            "intact" => Some(SnapshotOutcome::Intact),
            "unknown" => Some(SnapshotOutcome::Unknown),
            _ => None,
        };
    }
}

/// What:
///  The state of the clone's own head destroy,
///  which is the one state the header carries in flight.
/// Why:
///  A snapshot's attempt has no in-flight marker,
///  so a kill inside its request is settled by proof rather than by a word;
///  the head's single request is recorded before it is made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeadWord = 'present' | 'requested' | 'gone' | 'refused' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadWord {
    /// The clone still stands.
    Present,
    /// The head destroy was recorded and submitted.
    Requested,
    /// The clone was proven gone.
    Gone,
    /// The head request was refused and the clone stands.
    Refused,
    /// The head request failed without a per-name answer.
    Unknown,
}

impl HeadWord {
    /// The word the header carries.
    pub fn word(self) -> &'static str {
        return match self {
            HeadWord::Present => "present",
            HeadWord::Requested => "requested",
            HeadWord::Gone => "gone",
            HeadWord::Refused => "refused",
            HeadWord::Unknown => "unknown",
        };
    }

    /// The head word one stored word names.
    pub fn parse(word: &str) -> Option<HeadWord> {
        return match word {
            "present" => Some(HeadWord::Present),
            "requested" => Some(HeadWord::Requested),
            "gone" => Some(HeadWord::Gone),
            "refused" => Some(HeadWord::Refused),
            "unknown" => Some(HeadWord::Unknown),
            _ => None,
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "plan_words_tests.rs"]
mod tests;
