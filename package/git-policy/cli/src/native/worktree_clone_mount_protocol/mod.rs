//! What:
//!  The protocol between the account's wrapper and the root-run clone-mount
//!  handler:
//!  the durable mount intent and the clone-local sweep nudge,
//!  the receipts the handler publishes under `/run`,
//!  the readiness record for one boot,
//!  and the wrapper's side that writes a request,
//!  reads a receipt or a readiness record and waits for a receipt.
//! Why:
//!  The design keeps the request channel to user properties and the answers to
//!  files under `/run`,
//!  so that a result never creates a second ZFS event
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  "Decided defaults";
//!  the build order is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 7).
//!  Both the wrapper and the root modes read the same codecs,
//!  so a value the handler writes is read by exactly the grammar the wrapper
//!  wrote it as.
//!  Nothing here mounts,
//!  unmounts or records on an intent's behalf:
//!  a reading returns a request or a refusal,
//!  and the steps that act on a request are later ones.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `root_owned_directory.rs`:
//!   the root-owned `/run` directory a record lives in,
//!   made when it is absent and proved through its own descriptor,
//!   which uses nothing in this folder and is used by the two writers below.
//! - `clone_mount_property_names.rs`:
//!   the two property names,
//!   the version and operation words and the intent's byte bound.
//! - `request_nonce.rs`:
//!   the 32-hex-digit nonce,
//!   its parse and its fresh value.
//! - `local_user_property.rs`:
//!   the property reply's `value` and `source` pair,
//!   and the local-versus-inherited decision.
//! - `clone_mount_intent.rs`:
//!   the intent grammar and the reading that separates a request from an
//!   inherited or malformed value.
//! - `clone_mount_sweep_nonce.rs`:
//!   the nudge value,
//!   which is never an instruction.
//! - `dataset_guid.rs`:
//!   the canonical numeric GUID a receipt's file is named by,
//!   which `clone_mount_receipt.rs` uses.
//! - `clone_mount_receipt.rs`:
//!   the receipt's text and its identity fields.
//! - `clone_mount_receipt_refusals.rs`:
//!   why a file is not a receipt.
//! - `clone_mount_receipt_parse.rs`:
//!   the receipt's parse,
//!   bounded before any line.
//! - `clone_mount_receipts_folder_refusals.rs`:
//!   why the directory or one receipt was refused.
//! - `clone_mount_receipts_folder_calls.rs`:
//!   the descriptor-relative opens,
//!   create,
//!   rename and directory sync.
//! - `clone_mount_receipts_folder.rs`:
//!   the root-owned receipts directory,
//!   the GUID-named file,
//!   its atomic publish and its directory-relative read.
//! - `clone_mount_readiness.rs`:
//!   the readiness record's boot,
//!   pool,
//!   checkout and sweep fields.
//! - `clone_mount_readiness_admission.rs`:
//!   the refusal that names the field not ready,
//!   and the comparison every attach makes.
//! - `clone_mount_readiness_parse.rs`:
//!   the record's parse and its own refusals.
//! - `clone_mount_readiness_file.rs`:
//!   the root-owned readiness directory,
//!   its atomic publish and its read.
//! - `clone_mount_intent_write.rs`:
//!   the wrapper's request and nudge payloads and the gated writes.
//! - `mount_request_wait.rs`:
//!   the wrapper's wait for a matching receipt,
//!   what it last saw,
//!   the provisional mount-request deadline and the timeout report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const intent = readIntent(clone.properties, clone.name);
//! const receipt = receipts.read(datasetGuid);
//! const ready = readReadiness(boot, pool, checkout);
//! ```

/// The root-owned `/run` directory a record lives in.
pub mod root_owned_directory;

/// The route's fixed property names,
/// words and bounds.
pub mod clone_mount_property_names;

/// The nonce both properties carry.
pub mod request_nonce;

/// One property's reply pair and the local-versus-inherited decision.
pub mod local_user_property;

/// The durable mount intent.
pub mod clone_mount_intent;

/// The clone-local sweep nudge.
pub mod clone_mount_sweep_nonce;

/// The canonical numeric dataset GUID a receipt is named by.
pub mod dataset_guid;

/// The receipt's text and its identity fields.
pub mod clone_mount_receipt;

/// Why a file is not a receipt this route wrote.
pub mod clone_mount_receipt_refusals;

/// The receipt's parse,
/// bounded before any line.
pub mod clone_mount_receipt_parse;

/// The root-owned results directory and its GUID-named files.
pub mod clone_mount_receipts_folder_refusals;

/// The descriptor-relative calls the receipts folder makes.
pub mod clone_mount_receipts_folder_calls;

/// The results directory,
/// its publish and its read.
pub mod clone_mount_receipts_folder;

/// The readiness record's boot,
/// pool,
/// checkout and sweep fields.
pub mod clone_mount_readiness;

/// Why an attach is not admitted,
/// and the one comparison that decides it.
pub mod clone_mount_readiness_admission;

/// The readiness record's parse.
pub mod clone_mount_readiness_parse;

/// The root-owned readiness directory and its file.
pub mod clone_mount_readiness_file;

/// The wrapper's intent and nudge writes behind the gate.
pub mod clone_mount_intent_write;

/// The wrapper's wait for a matching receipt and its deadline.
pub mod mount_request_wait;

/// Step 7's guest arms:
/// the intent refusals with no mount,
/// the nudge under the proposed grants,
/// and the property event to the product's receipt,
/// run in the throwaway guest the recipe prepares,
/// never in the release build.
#[cfg(all(test, target_os = "linux"))]
#[path = "mount_protocol_arm_tests.rs"]
mod mount_protocol_arm_tests;

/// The property names,
/// the version and operation words and the intent's byte bound.
pub use clone_mount_property_names::{
    DESIRED_STATE_PROPERTY, MAXIMUM_INTENT_BYTES, MOUNT_WORD, NONCE_HEX_DIGITS, SWEEP_NONCE_PROPERTY, UNMOUNT_WORD, VALUE_VERSION,
};
/// The nonce and its refusal.
pub use request_nonce::{NonceRefusal, RequestNonce};
/// The property reading and its refusal.
pub use local_user_property::{PropertyReading, PropertyRefusal};
/// The intent,
/// its reading and the text one request is written as.
pub use clone_mount_intent::{
    IntentReading, IntentRefusal, MountIntent, MountOperation, intent_text, parse_mount_intent, read_intent,
};
/// The nudge,
/// its reading and the text it is written as.
pub use clone_mount_sweep_nonce::{NudgeReading, NudgeRefusal, parse_sweep_nonce, read_sweep_nonce, sweep_nonce_text};
/// The dataset GUID and its refusal.
pub use dataset_guid::{DatasetGuid, DatasetGuidRefusal};
/// The receipt's shape,
/// text and outcome.
pub use clone_mount_receipt::{CloneMountReceipt, ReceiptMountIdentity, ReceiptOutcome};
/// Why a file is not a receipt.
pub use clone_mount_receipt_refusals::ReceiptRefusal;
/// The receipt's parse.
pub use clone_mount_receipt_parse::parse_receipt;
/// The results directory and why it or a receipt was refused.
pub use clone_mount_receipts_folder::ReceiptsFolder;
pub use clone_mount_receipts_folder_refusals::{DIRECTORY_MODE, RECEIPT_MODE, RESULTS_DIRECTORY, ReceiptsRefusal};
/// The readiness record and its four fields.
pub use clone_mount_readiness::{
    MAXIMUM_READINESS_BYTES, READINESS_DIRECTORY, READINESS_KEYS, READINESS_NAME, ReadinessExpectation, ReadinessRecord, ReadyState, SweepState,
};
/// The admission decision and its refusal.
pub use clone_mount_readiness_admission::{AdmissionRefusal, admit_attach};
/// The readiness record's parse and its refusal.
pub use clone_mount_readiness_parse::{ReadinessParseRefusal, parse_readiness};
/// The root-owned readiness directory and its file.
pub use clone_mount_readiness_file::{ReadinessFile, ReadinessFileRefusal};
/// The wrapper's payloads and its gated writer.
pub use clone_mount_intent_write::{CloneIntentWriter, IntentWriteRefusal, intent_property, nudge_property};
/// The wrapper's wait,
/// its stale-result rule and its provisional deadline.
pub use mount_request_wait::{
    LastObservation, MOUNT_REQUEST_POLL_INTERVAL, MountRequest, PROVISIONAL_MOUNT_REQUEST_DEADLINE, ReceiptStanding, StaleReason, WaitRefusal,
    standing_of, wait_for_answer,
};
