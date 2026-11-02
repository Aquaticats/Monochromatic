//! What:
//!  The owner record inside every owner-lock directory (`owner.json`,
//!  schema version 1):
//!       its strict parser,
//!  its exact encoding,
//!  and the owner-liveness rule.
//! Why:
//!  The landing,
//!  reservation,
//!  hook,
//!  capture locks are directories holding this
//!      record.
//!  The incumbent and its hook dispatcher write and read the same record
//!      (`src/owner-lock/owner-lock-record.ts`),
//!  so both wrappers contend on one lock and each
//!      retires the other's dead lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = parseOwnerLockRecord(text); const alive = await ownerLockHolderIsAlive(record);
//! ```

/// Record field readers and the ordered writer.
use super::json_record::{
    ObjectWriter, number_equals, parse_object, safe_integer_field, string_field,
};
/// The rule for an owner that lives in another PID namespace.
use super::pid_namespace::foreign_owner_standing;
/// Birth identities and the existence probe.
use super::process_identity::{ProcessIdentityError, process_birth_identity};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Owner record filename inside a published lock directory.
pub const OWNER_LOCK_RECORD_FILENAME: &str = "owner.json";

/// Owner-lock record schema version.
pub const OWNER_LOCK_SCHEMA_VERSION: i64 = 1;

/// What:
///  The durable owner of one published lock.
///       A `struct` groups named fields;
///  `Option<String>` is "text or nothing".
/// Why:
///   The token distinguishes this acquisition from every other;
///  the PID and birth
///       identity prove liveness;
///  the reservation lock also names its transaction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnerLockRecord = { schemaVersion: 1; token: string; ownerPid: number; ownerBirthIdentity: string; transactionId?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerLockRecord {
    /// Unguessable token of this acquisition.
    pub token: String,
    /// Owning process ID.
    pub owner_pid: i64,
    /// Process-birth identity of the owner.
    pub owner_birth_identity: String,
    /// Commit transaction the lock is held for,
    ///  recorded by the landing reservation.
    pub transaction_id: Option<String>,
    /// PID namespace of the owner, or nothing for a record written before the field existed.
    ///  Optional on disk; readers that do not know the field ignore it.
    pub owner_pid_namespace: Option<String>,
}

/// What:
///  Parse an owner record's text,
///  or nothing when it is malformed.
/// Why:
///   The incumbent's `parseOwnerLockRecord` accepts exactly:
///  schema version 1,
///  a
///       non-empty token,
///  a safe-integer PID of at least 1,
///  a non-empty birth identity,
///  and
///       an optional non-empty transaction ID.
///  Anything else is malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseOwnerLockRecord(text: string): OwnerLockRecord; // throws when malformed
/// ```
pub fn parse_owner_lock_record(text: &str) -> Option<OwnerLockRecord> {
    let map: serde_json::Map<String, serde_json::Value> = parse_object(text)?;
    if !number_equals(&map, "schemaVersion", OWNER_LOCK_SCHEMA_VERSION) {
        return None;
    }
    let token: &str = string_field(&map, "token")?;
    let owner_pid: i64 = safe_integer_field(&map, "ownerPid")?;
    let identity: &str = string_field(&map, "ownerBirthIdentity")?;
    if token.is_empty() || owner_pid < 1 || identity.is_empty() {
        return None;
    }
    // A present `transactionId` must be non-empty text; an absent one is no transaction.
    let transaction_id: Option<String> = match map.get("transactionId") {
        None => None,
        Some(serde_json::Value::String(id)) if !id.is_empty() => Some(id.clone()),
        Some(_) => return None,
    };
    return Some(OwnerLockRecord {
        token: String::from(token),
        owner_pid,
        owner_birth_identity: String::from(identity),
        transaction_id,
        // A present field must be non-empty text; anything else leaves the PID to decide, as before the field.
        owner_pid_namespace: string_field(&map, "ownerPidNamespace")
            .filter(|namespace| { return !namespace.is_empty(); })
            .map(String::from),
    });
}

/// What:
///  Encode an owner record exactly as the incumbent's `writeCandidate` does:
///  compact
///       JSON in field order,
///  with a trailing newline.
/// Why:
///   Records written by either wrapper are byte-identical for the same values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify({ schemaVersion, token, ownerPid, ownerBirthIdentity, ...(transactionId && { transactionId }) })}\n`
/// ```
pub fn encode_owner_lock_record(record: &OwnerLockRecord) -> String {
    // `mut` allows appending one field at a time.
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .integer("schemaVersion", OWNER_LOCK_SCHEMA_VERSION)
        .string("token", record.token.as_str())
        .integer("ownerPid", record.owner_pid)
        .string("ownerBirthIdentity", record.owner_birth_identity.as_str());
    if let Some(id) = &record.transaction_id {
        writer.string("transactionId", id.as_str());
    }
    if let Some(namespace) = &record.owner_pid_namespace {
        writer.string("ownerPidNamespace", namespace.as_str());
    }
    return format!("{}\n", writer.finish());
}

/// What:
///  Whether a recorded owner still runs with the same process birth.
/// Why:
///   A PID that names no process,
///  a zombie,
///  or a younger process reusing the PID is a
///       dead owner.
///  The incumbent's `ownerLockHolderIsAlive` compares identities only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ownerLockHolderIsAlive(record: OwnerLockRecord): Promise<boolean>;
/// ```
pub fn owner_lock_holder_is_alive(record: &OwnerLockRecord, lock_directory: &Path) -> Result<bool, ProcessIdentityError> {
    // An owner of another PID namespace has a PID this process cannot interpret (issues 1781 and 1784).
    if let Some(standing) = foreign_owner_standing(
        record.owner_pid_namespace.as_deref(),
        lock_directory.join(OWNER_LOCK_RECORD_FILENAME).as_path(),
    ) {
        return Ok(standing);
    }
    // `match` unpacks the identity answer: a live process with the recorded birth is alive.
    match process_birth_identity(record.owner_pid)? {
        Some(current) => return Ok(current == record.owner_birth_identity),
        None => return Ok(false),
    }
}

/// Parser,
///  encoding and liveness controls stay out of the release executable.
#[cfg(test)]
#[path = "owner_lock_record_tests.rs"]
mod tests;
