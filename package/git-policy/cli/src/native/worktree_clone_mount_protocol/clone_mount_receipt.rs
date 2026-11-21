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

/// The largest receipt a reader accepts,
/// which is far more than the 15 lines below need.
pub const MAXIMUM_RECEIPT_BYTES: usize = 4096;

/// The dataset-name bound OpenZFS itself uses.
pub const MAXIMUM_DATASET_NAME_BYTES: usize = 255;

/// The path bound one canonical slot path may use.
pub const MAXIMUM_PATH_BYTES: usize = 4095;

/// The bound one step or errno word may use.
pub const MAXIMUM_WORD_BYTES: usize = 64;

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
}

impl ReceiptOutcome {
    /// The word the receipt carries.
    pub fn word(self) -> &'static str {
        return match self {
            ReceiptOutcome::Done => "done",
            ReceiptOutcome::Refused => "refused",
            ReceiptOutcome::Failed => "failed",
        };
    }

    /// Read one outcome word.
    pub fn from_word(word: &str) -> Option<Self> {
        return match word {
            "done" => Some(ReceiptOutcome::Done),
            "refused" => Some(ReceiptOutcome::Refused),
            "failed" => Some(ReceiptOutcome::Failed),
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
    /// When the result was filed,
    /// as ISO 8601 with offset.
    pub time: String,
}

/// The receipt's keys in the order the file holds them.
pub const RECEIPT_KEYS: [&str; 15] = [
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

impl CloneMountReceipt {
    /// The file's whole text,
    /// its final newline included.
    pub fn text(&self) -> String {
        let mount_id: String = self.mount.as_ref().map_or(String::from(ABSENT_WORD), |mount| return mount.mount_id.to_string());
        let device: String = self.mount.as_ref().map_or(String::from(ABSENT_WORD), |mount| return mount.device.to_string());
        let inode: String = self.mount.as_ref().map_or(String::from(ABSENT_WORD), |mount| return mount.inode.to_string());
        let source: &str = self.mount.as_ref().map_or(ABSENT_WORD, |mount| return mount.source.as_str());
        let canonical_path: &str = self.mount.as_ref().map_or(ABSENT_WORD, |mount| return mount.canonical_path.as_str());
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
            format!("failed-step {}", self.failed_step.as_deref().unwrap_or(ABSENT_WORD)),
            format!("errno {}", self.errno.as_deref().unwrap_or(ABSENT_WORD)),
            format!("boot {}", self.boot),
            format!("time {}", self.time),
        ];
        return format!("{}\n", lines.join("\n"));
    }
}
