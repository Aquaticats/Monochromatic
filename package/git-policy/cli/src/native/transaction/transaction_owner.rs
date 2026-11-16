//! What:
//!  The owner record of one commit-transaction directory (`owner.json`,
//!  schema version 2)
//!       and the liveness rule that decides whether recovery may touch the transaction.
//! Why:
//!  The owner record is written before the directory is published,
//!  so recovery can skip a
//!      live owner's transaction at every phase.
//!  The record is read by a later run of this wrapper,
//!  so its schema version is exact:
//!  a record of another version is refused rather than misread.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owner = parseTransactionOwner(bytes); const liveness = await classifyTransactionOwner(owner);
//! ```

/// Record field readers,
///  the fatal decoder and the ordered writer.
use crate::json_record::{
    ObjectWriter, decode_fatal, number_equals, parse_object, safe_integer_field, string_field,
};
/// Birth identities and the existence probe.
use crate::process_identity::{
    ProcessIdentityError, birth_identity_value, current_birth_identity, process_birth_identity,
};
/// The namespace an owner record names, the rule for an owner of another one, and the bound on a birth that names no value.
use crate::pid_namespace::{
    ForeignStanding, UnreadableOwnerStanding, UnreadablePart, current_pid_namespace, foreign_owner_standing,
    unreadable_owner_standing,
};

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
    /// PID namespace of the owner,
    /// or nothing for a record written before the field existed.
    ///  Optional on disk;
    /// readers that do not know the field ignore it.
    pub owner_pid_namespace: Option<String>,
}

/// What:
///  Whether a recorded owner still runs.
/// Why:
///   Only a dead owner's transaction may be recovered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TransactionOwnerLiveness = 'alive' | 'dead' | { cannotBeTold: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnerLiveness {
    /// The PID names the original process,
    /// or nothing proves it names another one:
    /// a record whose birth is spelled another way than this reader can compare
    ///  counts as its owner's,
    /// because a record in one spelling proves nothing about another.
    Alive,
    /// The process exited or its PID was reused.
    Dead,
    /// Neither, past the patience, and the text is the words a person reads:
    ///  the record, its owner, its age and the exact command that removes it.
    /// Why:
    ///  A record whose holder cannot be told may belong to a merge that runs, and the wrapper goes on beside a
    ///  live merge's record (it decides the kept copies and leaves the record whole). So this answer is read as
    ///  "leave the record alone and tell the person", never as a stop:
    ///  owner,
    ///  2026-11-16,
    ///  "A: stays, command says how (Recommended)",
    ///  and the wrapper's rule that a commit does not fail because of its own bookkeeping.
    CannotBeTold(String),
}

/// What:
///  Why an owner's liveness could not be decided.
/// Why:
///   A refused probe (for example another account's process) must not let recovery touch a transaction that may be
///  live;
/// the caller decides whether the refusal means "alive" (permission) or stops recovery (anything else).
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
    /// The record is neither alive nor dead, and the text is the error to stop with:
    /// it names another PID namespace and its measured age is past the patience,
    /// or its own time cannot be trusted (`pid_namespace.rs`,
    /// `RefusedForeignRecord`),
    /// or its birth identity is no value this command can read and its age is past the patience
    /// (`pid_namespace.rs`,
    /// `RefusedUntellableRecord`).
    RecordRefused(String),
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
        owner_pid_namespace: match string_field(&map, "ownerPidNamespace") {
            Some(namespace) if !namespace.is_empty() => Some(String::from(namespace)),
            _ => None,
        },
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
        return crate::process_identity::signal_probe(pid).map_err(OwnerLivenessError::Probe);
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
///  The recorded birth and the one the process has now are compared by the value each names,
///  so a leading zero or the platform's tag in another case is the same birth:
///  compared as text, that spelling read as another process, which is a proved-dead owner,
///  and a dead owner settles a merge that runs:
///  the record goes,
///  the refs that keep its commit go,
///  its kept copies are put back or removed beside its paths,
///  and its branch is read as one nobody is working on.
///  A text that names no value at all is neither a live owner's nor a dead one's,
///  and a caller that has the record reads its age
///  (`classify_transaction_owner_record`,
///  `unreadable_owner_standing`).
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
    return match process_birth_identity(owner_pid).map_err(OwnerLivenessError::Identity)? {
        Some(current) => match (birth_identity_value(owner_identity), birth_identity_value(current.as_str())) {
            (Some(recorded), Some(live)) => Ok(if recorded == live { OwnerLiveness::Alive } else { OwnerLiveness::Dead }),
            // A birth that names no value is no proof that the owner ended: what it holds stays, and the record's own age
            //  is the bound a caller with the record applies.
            _ => Ok(OwnerLiveness::Alive),
        },
        None => Ok(OwnerLiveness::Dead),
    };
}

/// What:
///  Classify a parsed owner record,
/// trusting its PID only inside the PID namespace that wrote it.
/// Why:
///   An owner of another namespace has a PID this process cannot interpret (issues 1781 and 1784),
/// so it counts as
///  alive until its record file `owner_path` outlives the patience;
/// the same rule serves recovery,
/// the lease check
///  and the reservation queue.
/// A record whose birth identity names no value is the same state:
/// inside the patience it stands for an owner that may run,
/// and past it the answer is `OwnerLiveness::CannotBeTold` with the words,
/// which the caller that reads merge records prints before it goes on
/// (`merge_isolation_recovery_records.rs`),
/// as the wrapper already goes on beside a live merge's record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await classifyTransactionOwnerRecord(owner, ownerPath);
/// ```
pub fn classify_transaction_owner_record(
    owner: &TransactionOwner,
    owner_path: &std::path::Path,
) -> Result<OwnerLiveness, OwnerLivenessError> {
    match foreign_owner_standing(owner.owner_pid_namespace.as_deref(), owner_path) {
        Some(ForeignStanding::Standing) => return Ok(OwnerLiveness::Alive),
        // The record of another namespace that its owner removed is a finished owner's: nothing of it holds anything, and a
        //  command that read the parsed copy before it went on is not stopped over a file that is not there
        //  (cross-vendor review of 2026-11-09, finding 3).
        Some(ForeignStanding::Finished) => return Ok(OwnerLiveness::Dead),
        Some(ForeignStanding::Refused(refused)) => return Err(OwnerLivenessError::RecordRefused(refused.describe(owner.owner_pid))),
        None => {}
    }
    let liveness: OwnerLiveness = match classify_transaction_owner(owner.owner_pid, owner.owner_identity.as_str()) {
        Ok(found) => found,
        // What: a process the platform will not report on is a holder that cannot be told, like a birth that names no
        //       value: the record's own age is the bound, and past it the words name the record.
        // Why:  A command of a person's must not stop for a record it was not asked about while nothing proves that
        //       record's owner ended; read as dead, the record would settle a merge that runs.
        Err(OwnerLivenessError::Identity(unreadable)) => {
            return match unreadable_owner_standing(owner_path, UnreadablePart::Process(unreadable.to_string()), owner.owner_pid) {
                UnreadableOwnerStanding::MayRun => Ok(OwnerLiveness::Alive),
                UnreadableOwnerStanding::Ended => Ok(OwnerLiveness::Dead),
                UnreadableOwnerStanding::Refused(words) => Ok(OwnerLiveness::CannotBeTold(words)),
            };
        }
        Err(other) => return Err(other),
    };
    // What: a birth that names no value is neither a live owner's nor a gone one's, and the record's own age is the bound.
    // Why:  Read as alive the record stays settled as an owner's for as long as any process holds its number, which is for
    //       ever; read as dead it settles a merge that runs. Past the patience the person is told which record it is and
    //       the command that removes it.
    if liveness == OwnerLiveness::Alive && birth_identity_value(owner.owner_identity.as_str()).is_none() {
        return match unreadable_owner_standing(owner_path, UnreadablePart::Birth(owner.owner_identity.clone()), owner.owner_pid) {
            UnreadableOwnerStanding::MayRun => Ok(OwnerLiveness::Alive),
            UnreadableOwnerStanding::Ended => Ok(OwnerLiveness::Dead),
            UnreadableOwnerStanding::Refused(words) => Ok(OwnerLiveness::CannotBeTold(words)),
        };
    }
    return Ok(liveness);
}

/// Owner record and liveness controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_owner_tests.rs"]
mod tests;
