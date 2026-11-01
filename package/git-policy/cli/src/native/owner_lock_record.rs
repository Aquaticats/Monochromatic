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
//!  capture and push locks are directories holding this
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
/// Bounded no-follow reads of the record file.
use super::private_storage::{ReadRefusal, read_regular_file};
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

/// Largest owner record read;
///  a record is about 150 bytes,
///  so anything near this is corrupt.
pub const OWNER_LOCK_RECORD_LIMIT: u64 = 64 * 1024;

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
///  What reading a published lock found.
///  `Busy` means the lock vanished while it was
///       read,
///  so the caller must look again.
/// Why:
///   A release or retirement renames the directory away at any moment;
///  that is a race to
///       retry,
///  not corruption.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadOwner = OwnerLockRecord | typeof LOCK_BUSY;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublishedOwner {
    /// The lock holds this owner.
    Owner(OwnerLockRecord),
    /// The lock or its record vanished during the read.
    Busy,
}

/// What:
///  Why a lock's record could not be judged.
/// Why:
///   A malformed or unreadable record must stop the acquirer:
///  it might belong to a live
///       owner,
///  and the wrapper never deletes a lock it cannot prove dead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class OwnerLockError extends Error {}
/// ```
#[derive(Debug)]
pub enum OwnerRecordError {
    /// The record exists but is not a valid owner record.
    Malformed(std::path::PathBuf),
    /// The record could not be read.
    Unreadable(std::path::PathBuf, std::io::Error),
}

/// What:
///  Read the owner of a published lock directory.
/// Why:
///   A missing record or directory means the lock vanished (`ENOENT`,
///  `ENOTDIR`),
///  which
///       the incumbent reports as busy so the caller retries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readOwnerLockRecord(lockDirectory: string): Promise<OwnerLockRecord | typeof LOCK_BUSY>;
/// ```
pub fn read_owner_lock_record(lock_directory: &Path) -> Result<PublishedOwner, OwnerRecordError> {
    let path: std::path::PathBuf = lock_directory.join(OWNER_LOCK_RECORD_FILENAME);
    let bytes: Vec<u8> = match read_regular_file(path.as_path(), OWNER_LOCK_RECORD_LIMIT) {
        Ok(read) => read,
        Err(ReadRefusal::Missing) => return Ok(PublishedOwner::Busy),
        Err(ReadRefusal::NotRegular | ReadRefusal::TooLarge) => {
            return Err(OwnerRecordError::Malformed(path));
        }
        Err(ReadRefusal::Io(error)) => return Err(OwnerRecordError::Unreadable(path, error)),
    };
    // Node's `readFile(path, 'utf8')` replaces invalid bytes before `JSON.parse`.
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    match parse_owner_lock_record(text.as_ref()) {
        Some(record) => return Ok(PublishedOwner::Owner(record)),
        None => return Err(OwnerRecordError::Malformed(path)),
    }
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
