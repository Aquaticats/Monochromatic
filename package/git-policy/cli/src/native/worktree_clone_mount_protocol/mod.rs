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
//!  Both executables read the same codecs,
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
//! - `clone_mount_receipt.rs`:
//!   the receipt's text,
//!   its identity fields and its parse.
//! - `clone_mount_receipts_folder.rs`:
//!   the root-owned receipts directory,
//!   the GUID-named file,
//!   its atomic publish and its directory-relative read.
//! - `clone_mount_readiness.rs`:
//!   the readiness record's boot,
//!   pool,
//!   checkout and sweep fields,
//!   and the refusal that names the field not ready.
//! - `clone_mount_intent_write.rs`:
//!   the wrapper's request and nudge writes through the binding.
//! - `mount_request_wait.rs`:
//!   the wrapper's wait for a matching receipt,
//!   the mount-request deadline and the timeout report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const intent = readIntent(clone.properties, clone.name);
//! const receipt = receipts.read(datasetGuid);
//! const ready = readReadiness(boot, pool, checkout);
//! ```

/// The route's fixed property names, words and bounds.
pub mod clone_mount_property_names;

/// The nonce both properties carry.
pub mod request_nonce;

/// One property's reply pair and the local-versus-inherited decision.
pub mod local_user_property;

/// The durable mount intent.
pub mod clone_mount_intent;

/// The clone-local sweep nudge.
pub mod clone_mount_sweep_nonce;

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
