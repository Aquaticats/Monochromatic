//! What:
//!  The owner record of one commit-transaction directory (`owner.json`,
//!  schema version 2)
//!       and the liveness rule that decides whether recovery may touch the transaction.
//! Why:
//!  The owner record is written before the directory is published,
//!  so recovery can skip a
//!      live owner's transaction at every phase.
//!  Both wrappers write and judge the same record
//!      (`src/policy-engine/commit-transaction-owner.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owner = parseTransactionOwner(bytes); const liveness = await classifyTransactionOwner(owner);
//! ```

/// Record field readers,
///  the fatal decoder and the ordered writer.
use super::json_record::{
    ObjectWriter, decode_fatal, number_equals, parse_object, safe_integer_field, string_field,
};
/// Birth identities and the existence probe.
use super::process_identity::{
    ProcessIdentityError, current_birth_identity, process_birth_identity,
};
/// The namespace an owner record names and the rule for an owner of another one.
use super::pid_namespace::{current_pid_namespace, foreign_owner_standing};

/// Owner record schema;
///  version 2 marks the per-transaction directory layout.
pub const TRANSACTION_OWNER_SCHEMA_VERSION: i64 = 2;

/// What:
///  The durable owner of one transaction directory.
/// Why:
///   The PID and birth identity prove liveness;
///  `created_at`,
///  the invocation start time
///       as ISO-8601 text,
///  orders dead transactions for recovery and grants the reservation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransactionOwnerRecord = { schemaVersion: 2; transactionId: string; ownerPid: number; ownerIdentity: string; createdAt: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionOwner {
    /// Transaction ID,
    ///  equal to the directory name.
    pub transaction_id: String,
    /// Wrapper process owning the transaction.
    pub owner_pid: i64,
    /// Process-birth identity of the owner.
    pub owner_identity: String,
    /// ISO-8601 invocation start time.
    pub created_at: String,
    /// PID namespace of the owner, or nothing for a record written before the field existed.
    ///  Optional on disk; readers that do not know the field ignore it.
    pub owner_pid_namespace: Option<String>,
}

/// What:
///  Whether a recorded owner still runs.
/// Why:
///   Only a dead owner's transaction may be recovered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransactionOwnerLiveness = 'alive' | 'dead';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerLiveness {
    /// The PID names the original process.
    Alive,
    /// The process exited or its PID was reused.
    Dead,
}

/// What:
///  Why an owner's liveness could not be decided.
/// Why:
///   A refused probe (for example another account's process) must not let recovery touch a transaction that may be
///  live; the caller decides whether the refusal means "alive" (permission) or stops recovery (anything else).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommitTransactionRecoveryError extends Error {}
/// ```
#[derive(Debug)]
pub enum OwnerLivenessError {
    /// The existence probe was refused.
    Probe(std::io::Error),
    /// The birth identity could not be read.
    Identity(ProcessIdentityError),
}

/// What:
///  The current process's owner record for a fresh transaction.
/// Why:
///   Written into the staging directory before publication.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function createTransactionOwnerRecord({ transactionId, createdAt }): Promise<TransactionOwnerRecord>;
/// ```
pub fn current_transaction_owner(
    transaction_id: &str,
    created_at: &str,
) -> Result<TransactionOwner, ProcessIdentityError> {
    return Ok(TransactionOwner {
        transaction_id: String::from(transaction_id),
        owner_pid: i64::from(std::process::id()),
        owner_identity: current_birth_identity()?,
        created_at: String::from(created_at),
        owner_pid_namespace: current_pid_namespace(),
    });
}

/// What:
///  Encode an owner record as compact JSON in the incumbent's field order,
///  with a newline.
/// Why:
///   `encodeTransactionOwner` writes exactly these bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify({ schemaVersion, transactionId, ownerPid, ownerIdentity, createdAt })}\n`
/// ```
pub fn encode_transaction_owner(owner: &TransactionOwner) -> String {
    // `mut` allows appending one field at a time.
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", TRANSACTION_OWNER_SCHEMA_VERSION)
        .string("transactionId", owner.transaction_id.as_str())
        .integer("ownerPid", owner.owner_pid)
        .string("ownerIdentity", owner.owner_identity.as_str())
        .string("createdAt", owner.created_at.as_str());
    if let Some(namespace) = &owner.owner_pid_namespace {
        writer.string("ownerPidNamespace", namespace.as_str());
    }
    return format!("{}\n", writer.finish());
}

/// What:
///  Parse owner record bytes,
///  or nothing when they are not a valid record.
/// Why:
///   The incumbent's `parseTransactionOwner` decodes with a fatal UTF-8 decoder and accepts
///       schema version 2,
///  text `transactionId` (possibly empty,
///  which the directory-name check
///       then rejects),
///  a safe-integer PID of at least 1,
///  a non-empty identity and text
///       `createdAt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseTransactionOwner(bytes: Uint8Array): TransactionOwnerRecord; // throws when malformed
/// ```
pub fn parse_transaction_owner(bytes: &[u8]) -> Option<TransactionOwner> {
    let map: serde_json::Map<String, serde_json::Value> = parse_object(decode_fatal(bytes)?)?;
    if !number_equals(&map, "schemaVersion", TRANSACTION_OWNER_SCHEMA_VERSION) {
        return None;
    }
    let transaction_id: &str = string_field(&map, "transactionId")?;
    let owner_pid: i64 = safe_integer_field(&map, "ownerPid")?;
    let owner_identity: &str = string_field(&map, "ownerIdentity")?;
    let created_at: &str = string_field(&map, "createdAt")?;
    if owner_pid < 1 || owner_identity.is_empty() {
        return None;
    }
    return Some(TransactionOwner {
        transaction_id: String::from(transaction_id),
        owner_pid,
        owner_identity: String::from(owner_identity),
        created_at: String::from(created_at),
        // A present field must be non-empty text; anything else leaves the PID to decide, as before the field.
        owner_pid_namespace: string_field(&map, "ownerPidNamespace")
            .filter(|namespace| { return !namespace.is_empty(); })
            .map(String::from),
    });
}

/// What:
///  Whether the process `pid` exists,
///  as the incumbent's `processIsAlive` asks first.
///       On systems without signals,
///  existence is left to the identity probe.
/// Why:
///   `kill(pid, 0)` answering `ESRCH` is a dead owner;
///  any other refusal is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processIsAlive(pid: number): boolean;
/// ```
fn process_exists(pid: i64) -> Result<bool, OwnerLivenessError> {
    #[cfg(unix)]
    {
        return super::process_identity::signal_probe(pid).map_err(OwnerLivenessError::Probe);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = pid;
        return Ok(true);
    }
}

/// What:
///  Classify a recorded owner:
///  alive only while its PID still names the same process birth.
/// Why:
///   An exited process,
///  a zombie,
///  or a younger process reusing the PID is a dead owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function classifyTransactionOwner({ ownerPid, ownerIdentity }): Promise<'alive' | 'dead'>;
/// ```
pub fn classify_transaction_owner(
    owner_pid: i64,
    owner_identity: &str,
) -> Result<OwnerLiveness, OwnerLivenessError> {
    if !process_exists(owner_pid)? {
        return Ok(OwnerLiveness::Dead);
    }
    match process_birth_identity(owner_pid).map_err(OwnerLivenessError::Identity)? {
        Some(current) if current == owner_identity => return Ok(OwnerLiveness::Alive),
        Some(_) | None => return Ok(OwnerLiveness::Dead),
    }
}

/// What:
///  Classify a parsed owner record, trusting its PID only inside the PID namespace that wrote it.
/// Why:
///   An owner of another namespace has a PID this process cannot interpret (issues 1781 and 1784), so it counts as
///  alive until its record file `owner_path` outlives the patience; the same rule serves recovery, the lease check
///  and the reservation queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await classifyTransactionOwnerRecord(owner, ownerPath);
/// ```
pub fn classify_transaction_owner_record(
    owner: &TransactionOwner,
    owner_path: &std::path::Path,
) -> Result<OwnerLiveness, OwnerLivenessError> {
    if let Some(standing) = foreign_owner_standing(owner.owner_pid_namespace.as_deref(), owner_path) {
        return Ok(if standing { OwnerLiveness::Alive } else { OwnerLiveness::Dead });
    }
    return classify_transaction_owner(owner.owner_pid, owner.owner_identity.as_str());
}

/// Owner record and liveness controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_owner_tests.rs"]
mod tests;
