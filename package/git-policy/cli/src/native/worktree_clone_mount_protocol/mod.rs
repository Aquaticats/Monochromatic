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
//! - `staging_folder_component.rs`:
//!   the component every staging folder of a removed clone begins with,
//!   and the question whether a mount point is one.
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
//! - `clone_mount_receipts_folder_read.rs`:
//!   the directory opened and proved,
//!   and the read of one dataset's receipt.
//! - `clone_mount_receipts_folder.rs`:
//!   the root-owned receipts directory,
//!   the GUID-named file,
//!   and its atomic publish.
//! - `clone_mount_readiness.rs`:
//!   the readiness record's boot,
//!   pool,
//!   checkout and sweep fields.
//! - `clone_mount_readiness_admission.rs`:
//!   the refusal that names the field not ready,
//!   and the comparison every attach makes.
//! - `clone_mount_readiness_parse.rs`:
//!   the record's parse and its own refusals.
//! - `clone_mount_readiness_file_refusals.rs`:
//!   why the readiness directory or its record was refused,
//!   and the sentence a person reads.
//! - `clone_mount_readiness_file_publish.rs`:
//!   the staging name one publish writes under,
//!   and the write of one whole record.
//! - `clone_mount_readiness_file_read.rs`:
//!   the directory opened and proved,
//!   and the read of one whole record.
//! - `clone_mount_readiness_file.rs`:
//!   the root-owned readiness directory and its record:
//!   where one boot's readiness lives and how it is written.
//! - `clone_mount_intent_write.rs`:
//!   the wrapper's request and nudge payloads and the gated writes.
//! - `unanswered_request_entry.rs`:
//!   one line of the folder of unanswered requests:
//!   its fields,
//!   its text and its parse.
//! - `unanswered_request_names.rs`:
//!   one attempt's name,
//!   the staging name a writer links it from,
//!   and the number the next claim takes.
//! - `unanswered_requests_refusals.rs`:
//!   why the folder refused one call:
//!   the folder,
//!   a file that is not an entry,
//!   or a text that is not one.
//! - `unanswered_requests_one_entry_read.rs`:
//!   the read of one entry:
//!   no link followed,
//!   the account's own,
//!   one page.
//! - `unanswered_requests_folder.rs`:
//!   the folder in the Git common directory:
//!   one `stat`,
//!   the publish of a wait's entry,
//!   the helper's recorded outcome,
//!   and the removal of a clone's attempts.
//! - `unanswered_requests_nudge.rs`:
//!   the gaps,
//!   the claim whose link decides the one owner,
//!   and the outcome a helper records,
//!   compiled only under the nudge switch.
//! - `unanswered_requests_before_a_command.rs`:
//!   the check one forwarded command makes,
//!   one `stat` when nothing waits,
//!   with no call site until its wiring lands.
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

/// The component every staging folder of a removed clone begins with.
pub mod staging_folder_component;

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

/// One line of the folder of unanswered requests:
/// its fields,
/// its text and its parse.
pub mod unanswered_request_entry;

/// One attempt's name,
/// the staging name a writer links it from,
/// and the number the next claim takes.
pub mod unanswered_request_names;

/// Why the folder of unanswered requests refused one call,
/// and what a person reads.
pub mod unanswered_requests_refusals;

/// The folder of unanswered requests in the Git common directory:
/// one `stat`,
/// the publish of a wait's entry,
/// the helper's recorded outcome,
/// and the removal of a clone's attempts.
pub mod unanswered_requests_folder;

/// The nudge a command asks for on a waiting clone's behalf,
/// compiled only where the `zfs-lost-event-nudge` switch is on,
/// which is off until step 22 sizes its gaps with M8.
#[cfg(feature = "zfs-lost-event-nudge")]
pub(crate) mod unanswered_requests_nudge;

/// The check one forwarded command makes in its repository's folder of
/// unanswered requests:
/// one `stat` when nothing waits,
/// and the receipt read that removes an answered entry.
/// It has no call site yet:
/// a use of this folder by `invocation/` is a new use inside a loop the layout
/// allow list tolerates,
/// which no line may record,
/// so the forwarded path's one line waits for the round that untangles it.
pub mod unanswered_requests_before_a_command;

/// The read of one entry of the folder:
/// no link followed,
/// the account's own,
/// one page.
pub mod unanswered_requests_one_entry_read;

/// Step 7's guest arms:
/// the intent refusals with no mount,
/// the nudge under the proposed grants,
/// and the property event to the product's receipt,
/// run in the throwaway guest the recipe prepares,
/// never in the release build.
#[cfg(all(test, target_os = "linux"))]
#[path = "mount_protocol_arm_tests.rs"]
mod mount_protocol_arm_tests;

/// The intent,
/// its reading and the text one request is written as.
pub use clone_mount_intent::{
    IntentReading,
    IntentRefusal,
    MountIntent,
    MountOperation,
    intent_text,
    parse_mount_intent,
    read_intent,
};
/// The wrapper's payloads and its gated writer.
pub use clone_mount_intent_write::{
    CloneIntentWriter,
    IntentWriteRefusal,
    intent_property,
    nudge_property,
};
/// The property names,
/// the version and operation words and the intent's byte bound.
pub use clone_mount_property_names::{
    DESIRED_STATE_PROPERTY,
    MAXIMUM_INTENT_BYTES,
    MOUNT_WORD,
    NONCE_HEX_DIGITS,
    SWEEP_NONCE_PROPERTY,
    UNMOUNT_WORD,
    VALUE_VERSION,
};
/// The readiness record and its four fields.
pub use clone_mount_readiness::{
    MAXIMUM_READINESS_BYTES,
    READINESS_DIRECTORY,
    READINESS_KEYS,
    READINESS_NAME,
    ReadinessExpectation,
    ReadinessRecord,
    ReadyState,
    SweepState,
};
/// The admission decision and its refusal.
pub use clone_mount_readiness_admission::{
    AdmissionRefusal,
    admit_attach,
};
/// The root-owned readiness directory and its file.
pub use clone_mount_readiness_file::{
    ReadinessFile,
    ReadinessFileRefusal,
};
/// The readiness record's parse and its refusal.
pub use clone_mount_readiness_parse::{
    ReadinessParseRefusal,
    parse_readiness,
};
/// The receipt's shape,
/// text and outcome.
pub use clone_mount_receipt::{
    CloneMountReceipt,
    ReceiptMountIdentity,
    ReceiptOutcome,
};
/// The receipt's parse.
pub use clone_mount_receipt_parse::parse_receipt;
/// Why a file is not a receipt.
pub use clone_mount_receipt_refusals::ReceiptRefusal;
/// The results directory and why it or a receipt was refused.
pub use clone_mount_receipts_folder::ReceiptsFolder;
pub use clone_mount_receipts_folder_refusals::{
    DIRECTORY_MODE,
    RECEIPT_MODE,
    RESULTS_DIRECTORY,
    ReceiptsRefusal,
};
/// The nudge,
/// its reading and the text it is written as.
pub use clone_mount_sweep_nonce::{
    NudgeReading,
    NudgeRefusal,
    parse_sweep_nonce,
    read_sweep_nonce,
    sweep_nonce_text,
};
/// The dataset GUID and its refusal.
pub use dataset_guid::{
    DatasetGuid,
    DatasetGuidRefusal,
};
/// The property reading and its refusal.
pub use local_user_property::{
    PropertyReading,
    PropertyRefusal,
};
/// The wrapper's wait,
/// its stale-result rule and its provisional deadline.
pub use mount_request_wait::{
    LastObservation,
    MOUNT_REQUEST_POLL_INTERVAL,
    MountRequest,
    PROVISIONAL_MOUNT_REQUEST_DEADLINE,
    ReceiptStanding,
    StaleReason,
    WaitRefusal,
    standing_of,
    wait_for_answer,
};
/// The nonce and its refusal.
pub use request_nonce::{
    NonceRefusal,
    RequestNonce,
};
