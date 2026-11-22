//! What:
//!  Why a file is not a receipt this route wrote:
//!  the byte bound,
//!  the line count,
//!  a key at the wrong place,
//!  a value that does not fit its field,
//!  and the mount fields a done result alone carries.
//! Why:
//!  A receipt is read by a caller that then acts on its answer,
//!  so every deviation has a named cause and nothing is defaulted
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The words tell the caller where the handler's own account is and that
//!  nothing acted on the file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ReceiptRefusal extends Error {}
//! ```

use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuidRefusal;
use crate::worktree_clone_mount_protocol::request_nonce::NonceRefusal;

/// What:
///  Why a file is not a receipt this route wrote.
/// Why:
///   A receipt is read by a caller that then acts on its answer,
///  so every deviation has a named cause and nothing is defaulted:
///  a wrong key order,
///  a value that does not fit its field and a missing or extra line are all
///  refusals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReceiptRefusal = { kind: 'too-large' | 'line-count' | 'key' | 'value' | 'dataset' | 'nonce' | 'operation' | 'outcome' | 'mount-field' | 'step' | 'errno' | 'boot' | 'time'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiptRefusal {
    /// The file is longer than a receipt may be.
    TooLarge {
        /// How many bytes were read.
        bytes: usize,
        /// The bound.
        bound: usize,
    },
    /// The file is not text.
    NotText,
    /// The file does not have the receipt's line count.
    LineCount {
        /// How many lines it holds.
        found: usize,
        /// How many a receipt holds.
        expected: usize,
    },
    /// A line does not begin with the key at that place.
    Key {
        /// The one-based line.
        number: usize,
        /// The key that belongs there.
        expected: &'static str,
        /// The key as read.
        found: String,
    },
    /// A value does not fit its field.
    Value {
        /// The field's key.
        key: &'static str,
        /// What was found,
        /// as a person reads it.
        shown: String,
    },
    /// The dataset GUID field is not a GUID.
    Dataset(DatasetGuidRefusal),
    /// The nonce field is not a nonce.
    Nonce(NonceRefusal),
    /// A done receipt lacks a mount field,
    /// or another outcome carries one.
    MountField {
        /// The field's key.
        key: &'static str,
        /// Whether the field should have been present.
        wanted: bool,
    },
    /// The time field is not ISO 8601 with an offset.
    Time {
        /// The value as read.
        value: String,
    },
}

impl ReceiptRefusal {
    /// The refusal in words,
    /// naming the file's field and the way forward.
    pub fn message(&self) -> String {
        let reason: String = match self {
            ReceiptRefusal::TooLarge { bytes, bound } => format!("the file is {bytes} bytes and a receipt is at most {bound}"),
            ReceiptRefusal::NotText => String::from("the file is not UTF-8 text"),
            ReceiptRefusal::LineCount { found, expected } => format!("the file holds {found} lines and a receipt holds exactly {expected}"),
            ReceiptRefusal::Key { number, expected, found } => format!("line {number} begins with `{found}` and a receipt has `{expected}` there"),
            ReceiptRefusal::Value { key, shown } => format!("`{key}` holds `{shown}`, which does not fit that field"),
            ReceiptRefusal::Dataset(refusal) => refusal.message(),
            ReceiptRefusal::Nonce(refusal) => refusal.message(),
            ReceiptRefusal::MountField { key, wanted } => format!(
                "`{key}` {} on a receipt of this outcome",
                if *wanted { "is missing" } else { "is present" }
            ),
            ReceiptRefusal::Time { value } => format!("`time {value}` is not an ISO 8601 time with an offset"),
        };
        return format!(
            "the file under the clone-mount results directory is not a receipt this route wrote: {reason}. Nothing was matched, mounted or unmounted on its answer. Ways forward: read the handler's journal (`journalctl -u reconcile-git-worktree-clone-mounts.service`) for the result it filed, and let the next sweep replace the file; do not hand-edit it."
        );
    }
}
