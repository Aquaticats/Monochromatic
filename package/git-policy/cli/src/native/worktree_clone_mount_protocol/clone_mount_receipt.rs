//! What:
//!  The result the root handler files for one clone:
//!  the dataset GUID the file is named by,
//!  the request's operation and nonce,
//!  the outcome,
//!  the mount identity that outcome observed,
//!  the failing step and errno where one applies,
//!  and the boot and time the result belongs to.
//! Why:
//!  The project kept atomic `/run` receipts rather than root-written
//!  acknowledgement properties,
//!  so a result creates no second ZED event
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  "Decided defaults").
//!  The nonce is matched as data,
//!  never as part of a name,
//!  and a waiter that finds another operation,
//!  another nonce or another boot is holding a stale result rather than its
//!  answer.
//!  The keys are fixed and every value is checked,
//!  so a half-written or foreign file cannot be read as a result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = receipt.text(); // 'v1\nkind clone-mount-result\ndataset ...\n'
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::{
    MOUNT_WORD,
    NONCE_HEX_DIGITS,
    UNMOUNT_WORD,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The receipt's format version,
/// which every reader checks first.
pub const RECEIPT_VERSION: &str = "v1";

/// The receipt's kind,
/// which tells it from the readiness record beside it.
pub const RECEIPT_KIND: &str = "clone-mount-result";

/// The word an inapplicable field carries.
pub const ABSENT_WORD: &str = "none";

/// The receipt's keys in the order the file holds them.
pub const RECEIPT_KEYS: &[&str] = &[
    "v1",
    "kind",
    "dataset",
    "nonce",
    "operation",
    "outcome",
    "mount-id",
    "device",
    "inode",
    "source",
    "canonical-path",
    "failed-step",
    "errno",
    "boot",
    "time",
];

/// The digits a `u64` mount count,
/// device or inode can take,
/// which is `u64::MAX`'s own width.
pub const MAXIMUM_DECIMAL_DIGITS: usize = 20;

/// The bound the kernel's own boot identifier uses,
/// a canonical UUID's 36 characters.
pub const MAXIMUM_BOOT_BYTES: usize = 36;

/// The bound one operation word may use.
pub const MAXIMUM_OPERATION_BYTES: usize = if MOUNT_WORD.len() > UNMOUNT_WORD.len() {
    MOUNT_WORD.len()
} else {
    UNMOUNT_WORD.len()
};

/// The bound one outcome word may use.
pub const MAXIMUM_OUTCOME_BYTES: usize = 9;

/// The bound one time may use:
/// the date,
/// the second,
/// nine fraction digits and the offset.
pub const MAXIMUM_TIME_BYTES: usize = 35;

/// The dataset-name bound OpenZFS itself uses.
pub const MAXIMUM_DATASET_NAME_BYTES: usize = 255;

/// The path bound one canonical slot path may use,
/// which is the kernel's `PATH_MAX` without its terminator.
pub const MAXIMUM_PATH_BYTES: usize = 4095;

/// The bound one step or errno word may use.
pub const MAXIMUM_WORD_BYTES: usize = 64;

/// The most bytes one line of the receipt takes:
/// its key,
/// its separator and the longest value that field holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lineBytes = (key, bound) => key.length + 1 + bound;
/// ```
const fn line_bytes(
    key: &str,
    bound: usize,
) -> usize {
    return key.len() + 1 + bound;
}

/// What:
///  The largest receipt a reader accepts,
///  from the bound of every field the writer fills and the line each field is
///  written on.
/// Why:
///   A file bound typed in beside field bounds can fall below them,
///  and then a receipt the writer can write is one the reader refuses,
///  silently,
///  after the handler already reported the result.
///  Derived this way,
///  every receipt the writer builds fits,
///  and `publish` proves the reader reads it back.
///  It refuses a receipt it cannot read back,
///  so nothing is filed for a caller to wait out.
/// The mount fields take `none` on a result that observed no mount,
/// which their own bounds already cover.
pub const MAXIMUM_RECEIPT_BYTES: usize = RECEIPT_KEYS[0].len()
    + line_bytes(RECEIPT_KEYS[1], RECEIPT_KIND.len())
    + line_bytes(RECEIPT_KEYS[2], MAXIMUM_DECIMAL_DIGITS)
    + line_bytes(RECEIPT_KEYS[3], NONCE_HEX_DIGITS)
    + line_bytes(RECEIPT_KEYS[4], MAXIMUM_OPERATION_BYTES)
    + line_bytes(RECEIPT_KEYS[5], MAXIMUM_OUTCOME_BYTES)
    + line_bytes(RECEIPT_KEYS[6], MAXIMUM_DECIMAL_DIGITS)
    + line_bytes(RECEIPT_KEYS[7], MAXIMUM_DECIMAL_DIGITS)
    + line_bytes(RECEIPT_KEYS[8], MAXIMUM_DECIMAL_DIGITS)
    + line_bytes(RECEIPT_KEYS[9], MAXIMUM_DATASET_NAME_BYTES)
    + line_bytes(RECEIPT_KEYS[10], MAXIMUM_PATH_BYTES)
    + line_bytes(RECEIPT_KEYS[11], MAXIMUM_WORD_BYTES)
    + line_bytes(RECEIPT_KEYS[12], MAXIMUM_WORD_BYTES)
    + line_bytes(RECEIPT_KEYS[13], MAXIMUM_BOOT_BYTES)
    + line_bytes(RECEIPT_KEYS[14], MAXIMUM_TIME_BYTES)
    + RECEIPT_KEYS.len();

/// What:
///  What the handler did about one request.
/// Why:
///   A waiter must tell an observed effect from a refusal and from a failed
///  step,
///  and each report names a different next action.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReceiptOutcome = 'done' | 'refused' | 'failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptOutcome {
    /// The operation's effect was observed.
    Done,
    /// The handler refused before acting.
    Refused,
    /// A step failed.
    Failed,
    /// The unmount stopped short and the clone was put back at its slot,
    /// or the turn refused before it moved anything.
    Cancelled,
}

impl ReceiptOutcome {
    /// The word the receipt carries.
    pub fn word(self) -> &'static str {
        return match self {
            ReceiptOutcome::Done => "done",
            ReceiptOutcome::Refused => "refused",
            ReceiptOutcome::Failed => "failed",
            ReceiptOutcome::Cancelled => "cancelled",
        };
    }

    /// Read one outcome word.
    pub fn from_word(word: &str) -> Option<Self> {
        return match word {
            "done" => Some(ReceiptOutcome::Done),
            "refused" => Some(ReceiptOutcome::Refused),
            "failed" => Some(ReceiptOutcome::Failed),
            "cancelled" => Some(ReceiptOutcome::Cancelled),
            _ => None,
        };
    }
}

/// What:
///  The mount one done result observed:
///  the kernel's mount id and the attached folder's device and inode,
///  the dataset it is a mount of,
///  and the canonical path it was found at.
/// Why:
///   A receipt that named only "done" would not let the caller confirm the
///  mount it sees is the one the handler attached,
///  and the design requires the matching receipt to be confirmed by the
///  caller's mount view
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Waiting and refusal").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReceiptMountIdentity = { mountId; device; inode; source; canonicalPath };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptMountIdentity {
    /// The kernel's mount number.
    pub mount_id: u64,
    /// The attached folder's device.
    pub device: u64,
    /// The attached folder's inode.
    pub inode: u64,
    /// The ZFS source the mount table names.
    pub source: String,
    /// The canonical slot path the mount was found at.
    pub canonical_path: String,
}

/// What:
///  One whole receipt.
/// Why:
///   The values are exactly the fields the file carries,
///  so a writer and a reader share one shape and a waiter compares the
///  request's fields with the file's without guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneMountReceipt = { dataset; operation; nonce; outcome; mount?; failedStep?; errno?; boot; time };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneMountReceipt {
    /// The dataset whose GUID names the file.
    pub dataset: DatasetGuid,
    /// The operation the request asked for.
    pub operation: MountOperation,
    /// The request's nonce.
    pub nonce: RequestNonce,
    /// What the handler did.
    pub outcome: ReceiptOutcome,
    /// The mount a done result observed.
    pub mount: Option<ReceiptMountIdentity>,
    /// The step a refusal or failure names.
    pub failed_step: Option<String>,
    /// The errno a failure names.
    pub errno: Option<String>,
    /// The boot the result belongs to,
    /// as the kernel's own identifier.
    pub boot: String,
    /// When this answer was first filed,
    /// as ISO 8601 with offset;
    /// a standing clone's unchanged answer is not rewritten,
    /// so a later sweep does not move this time.
    pub time: String,
}

impl CloneMountReceipt {
    /// The file's whole text,
    /// its final newline included.
    pub fn text(&self) -> String {
        let mount_id: String = self
            .mount
            .as_ref()
            .map_or(String::from(ABSENT_WORD), |mount| {
                return mount.mount_id.to_string();
            });
        let device: String = self
            .mount
            .as_ref()
            .map_or(String::from(ABSENT_WORD), |mount| {
                return mount.device.to_string();
            });
        let inode: String = self
            .mount
            .as_ref()
            .map_or(String::from(ABSENT_WORD), |mount| {
                return mount.inode.to_string();
            });
        let source: &str = self
            .mount
            .as_ref()
            .map_or(ABSENT_WORD, |mount| return mount.source.as_str());
        let canonical_path: &str = self
            .mount
            .as_ref()
            .map_or(ABSENT_WORD, |mount| return mount.canonical_path.as_str());
        let lines: [String; 15] = [
            String::from(RECEIPT_VERSION),
            format!("kind {RECEIPT_KIND}"),
            format!("dataset {}", self.dataset.file_name()),
            format!("nonce {}", self.nonce.as_str()),
            format!("operation {}", self.operation.word()),
            format!("outcome {}", self.outcome.word()),
            format!("mount-id {mount_id}"),
            format!("device {device}"),
            format!("inode {inode}"),
            format!("source {source}"),
            format!("canonical-path {canonical_path}"),
            format!(
                "failed-step {}",
                self.failed_step.as_deref().unwrap_or(ABSENT_WORD)
            ),
            format!("errno {}", self.errno.as_deref().unwrap_or(ABSENT_WORD)),
            format!("boot {}", self.boot),
            format!("time {}", self.time),
        ];
        return format!("{}\n", lines.join("\n"));
    }
}
