//! What:
//!  Why a file is not a holder scan's result:
//!  it is past the parse's bound,
//!  its version or kind is another record's,
//!  a line is missing,
//!  a key is out of order or another key's,
//!  a count or a value does not fit its own rule,
//!  and the two typed fields that have their own refusals.
//! Why:
//!  A reader acts on this file,
//!  so a half-written,
//!  reordered or foreign result is refused rather than read as a clear one,
//!  and each cause names the line and what to change
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type HolderScanRefusal = { kind: 'too-large' | 'key' | 'count' | 'value' | 'version' | 'kind' | ...; ... };
//! ```

use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuidRefusal;
use crate::worktree_clone_mount_protocol::request_nonce::NonceRefusal;

/// What:
///  Why one text is not a holder scan's result.
/// Why:
///   Each variant names the field a person looks at,
///  and the reader's answer never falls back to a default:
///  a missing count and a wrong one are different things.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { kind: 'too-large'; bytes; bound } | { kind: 'key'; number; expected; found } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HolderScanRefusal {
    /// The text is longer than any result this route writes.
    TooLarge {
        /// How many bytes it holds.
        bytes: usize,
        /// The bound the writer and the reader share.
        bound: usize,
    },
    /// The text ended before one line.
    MissingLine {
        /// The line's key.
        key: &'static str,
    },
    /// The text held a line after its last one.
    TrailingLine {
        /// The line as read.
        found: String,
    },
    /// The first line is not this result's version.
    Version {
        /// The line as read.
        found: String,
    },
    /// The kind line names another record.
    Kind {
        /// The value as read.
        found: String,
    },
    /// A line's key is not the one this place takes.
    Key {
        /// The line's one-based place in the text.
        number: usize,
        /// The key this place takes.
        expected: &'static str,
        /// The key as read.
        found: String,
    },
    /// A count line holds no canonical decimal count.
    Count {
        /// The key.
        key: &'static str,
        /// The value as read.
        shown: String,
    },
    /// A value is empty or does not fit its own rule.
    Value {
        /// The key.
        key: &'static str,
        /// The value as read.
        shown: String,
    },
    /// The dataset GUID does not parse as one.
    Dataset(DatasetGuidRefusal),
    /// The nonce does not parse as one.
    Nonce(NonceRefusal),
    /// The time is not ISO 8601 with an offset.
    Time {
        /// The value as read.
        value: String,
    },
}

impl HolderScanRefusal {
    /// The refusal in words,
    /// naming the line and the way forward.
    pub fn message(&self) -> String {
        return match self {
            HolderScanRefusal::TooLarge { bytes, bound } => format!(
                "this result is {bytes} bytes, and a holder scan's result is at most {bound}. Nothing was read from it. Ways forward: read the file again from the handler that wrote it; a result this large is not one this route wrote."
            ),
            HolderScanRefusal::MissingLine { key } => format!(
                "this result ends before its `{key}` line, so it was cut short. Nothing was read from it. Ways forward: write and read the result again through one handler run; a file cut short is not a result."
            ),
            HolderScanRefusal::TrailingLine { found } => format!(
                "this result holds a line after its last field: `{found}`. Nothing was read from it. Ways forward: read a result this route wrote; no trailing line belongs to one."
            ),
            HolderScanRefusal::Version { found } => format!(
                "this result begins `{found}`, and every holder scan's result begins with its own version. Nothing was read from it. Ways forward: read a result this release's handler wrote."
            ),
            HolderScanRefusal::Kind { found } => format!(
                "this file's kind is `{found}`, and a holder scan's result says it is a clone-holder-scan. Nothing was read from it. Ways forward: read the holder scan's own result rather than a receipt or a readiness record that shares its folder's grammar."
            ),
            HolderScanRefusal::Key { number, expected, found } => format!(
                "line {number} of this result begins `{found}`, and that place takes `{expected}`. Nothing was read from it. Ways forward: read a result this route wrote; its lines are in one fixed order and a reordered file is not read."
            ),
            HolderScanRefusal::Count { key, shown } => format!(
                "the `{key}` line holds `{shown}`, and a count here is one or more decimal digits with no sign and no leading zero. Nothing was read from it. Ways forward: read a result this route wrote."
            ),
            HolderScanRefusal::Value { key, shown } => format!(
                "the `{key}` line holds `{shown}`, which is not a value that field takes. Nothing was read from it. Ways forward: read a result this route wrote, whose every field is checked as it is written."
            ),
            HolderScanRefusal::Dataset(refusal) => refusal.message(),
            HolderScanRefusal::Nonce(refusal) => refusal.message(),
            HolderScanRefusal::Time { value } => format!(
                "the result's time is `{value}`, and a time here is ISO 8601 with a date, a second and an offset. Nothing was read from it. Ways forward: read a result this route wrote."
            ),
        };
    }
}
