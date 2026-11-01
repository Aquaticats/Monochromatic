//! What:
//!  The preparation and landing leases:
//!  capabilities a running wrapper passes to the
//!       Git it starts,
//!  so a nested wrapper invocation from a hook or `rebase --exec` proceeds
//!       under its ancestor instead of waiting for it.
//! Why:
//!  Without a lease a nested invocation would wait for the hook lock or landing lock its own
//!      ancestor holds,
//!  forever.
//!  Both wrappers format and validate the same leases
//!      (`src/hook-dispatch/preparation-lease.ts`,
//!  `src/index-lock/landing-lease.ts`),
//!  because a
//!      hook started by one wrapper may run the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (await hasValidInheritedLease(process.env)) { /* skip recovery and the hook lock */ }
//! ```

/// Record readers and the ordered writer.
use super::json_record::{ObjectWriter, decode_fatal, parse_object, string_field};
/// Owner records of published locks.
use super::owner_lock_record::{PublishedOwner, read_owner_lock_record};
/// Bounded no-follow reads.
use super::private_storage::read_regular_file;
/// Transaction owner records and their liveness.
use super::transaction_owner::{
    OwnerLiveness, classify_transaction_owner_record, parse_transaction_owner,
};
/// The owner record filename of a transaction directory.
use super::transaction_registry::OWNER_FILENAME;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Environment variable carrying the preparation lease to hook descendants.
pub const PREPARATION_LEASE_VARIABLE: &str = "CLI_GIT_PREPARATION_LEASE";

/// Environment variable carrying the landing lease to a forwarded index writer.
pub const LANDING_LEASE_VARIABLE: &str = "CLI_GIT_LANDING_LEASE";

/// Largest plan or owner record a lease check reads.
const LEASE_RECORD_LIMIT: u64 = 1024 * 1024;

/// What:
///  Format a preparation lease naming the transaction directory and a fresh token.
/// Why:
///   `formatPreparationLease`:
///  `JSON.stringify({ directory, token })`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// formatPreparationLease({ directory, token })
/// ```
pub fn format_preparation_lease(directory: &str, token: &str) -> String {
    // `mut` allows appending one field at a time.
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer.string("directory", directory).string("token", token);
    return writer.finish();
}

/// What:
///  Format a landing lease naming the held lock directory and its owner token.
/// Why:
///   `formatLandingLease`:
///  `JSON.stringify({ lockDirectory, token })`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// formatLandingLease({ lockDirectory, token })
/// ```
pub fn format_landing_lease(lock_directory: &str, token: &str) -> String {
    // `mut` allows appending one field at a time.
    let mut writer: ObjectWriter = ObjectWriter::new();
    writer
        .string("lockDirectory", lock_directory)
        .string("token", token);
    return writer.finish();
}

/// What:
///  The value of one variable in an injected environment,
///  when present and not empty.
/// Why:
///   An empty lease is no lease,
///  as `(lease === undefined) || (lease === '')`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lease = environment[name]; if (!lease) return undefined;
/// ```
fn nonempty_variable(environment: &[(OsString, OsString)], name: &str) -> Option<String> {
    // The last entry wins, as a process environment has one value per name.
    let mut found: Option<&OsString> = None;
    for (key, value) in environment {
        if key == name {
            found = Some(value);
        }
    }
    let text: &str = found?.to_str()?;
    if text.is_empty() {
        return None;
    }
    return Some(String::from(text));
}

/// What:
///  Read a small record as text,
///  or nothing when it is missing,
///  unsafe or not UTF-8.
/// Why:
///   Every failure to read makes a lease invalid;
///  it is never an error of the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await readFile(path, 'utf8'); } catch { return undefined; }
/// ```
fn read_text(path: &Path) -> Option<String> {
    let bytes: Vec<u8> = read_regular_file(path, LEASE_RECORD_LIMIT).ok()?;
    return Some(String::from(decode_fatal(bytes.as_slice())?));
}

/// What:
///  Whether the environment carries a valid preparation lease of a live outer transaction.
/// Why:
///   The incumbent's `hasValidInheritedLease`:
///  the lease names a directory whose
///       `hooks/plan.json` still carries exactly this lease,
///  and whose owner is alive.
///  Any
///       malformed,
///  stale or foreign lease is invalid,
///  never an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function hasValidInheritedLease(environment): Promise<boolean>;
/// ```
pub fn has_valid_preparation_lease(environment: &[(OsString, OsString)]) -> bool {
    let Some(lease) = nonempty_variable(environment, PREPARATION_LEASE_VARIABLE) else {
        return false;
    };
    let Some(value) = parse_object(lease.as_str()) else {
        return false;
    };
    let Some(directory) = string_field(&value, "directory") else {
        return false;
    };
    let transaction: PathBuf = PathBuf::from(directory);
    let Some(plan_text) = read_text(transaction.join("hooks").join("plan.json").as_path()) else {
        return false;
    };
    let Some(plan) = parse_object(plan_text.as_str()) else {
        return false;
    };
    if string_field(&plan, "lease") != Some(lease.as_str()) {
        return false;
    }
    let owner_path: PathBuf = transaction.join(OWNER_FILENAME);
    let Ok(owner_bytes) = read_regular_file(
        owner_path.as_path(),
        LEASE_RECORD_LIMIT,
    ) else {
        return false;
    };
    let Some(owner) = parse_transaction_owner(owner_bytes.as_slice()) else {
        return false;
    };
    return matches!(
        classify_transaction_owner_record(&owner, owner_path.as_path()),
        Ok(OwnerLiveness::Alive)
    );
}

/// What:
///  Whether the environment carries a lease for `lock_directory` that its owner still holds.
/// Why:
///   The incumbent's `hasValidLandingLease`:
///  the lease must name exactly this lock,
///  and the
///       lock's current owner token must be the lease's token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function hasValidLandingLease({ environment, lockDirectory }): Promise<boolean>;
/// ```
pub fn has_valid_landing_lease(
    environment: &[(OsString, OsString)],
    lock_directory: &Path,
) -> bool {
    let Some(lease) = nonempty_variable(environment, LANDING_LEASE_VARIABLE) else {
        return false;
    };
    let Some(value) = parse_object(lease.as_str()) else {
        return false;
    };
    let (Some(named), Some(token)) = (
        string_field(&value, "lockDirectory"),
        string_field(&value, "token"),
    ) else {
        return false;
    };
    // The incumbent compares the texts exactly, so a differently spelled path is another lock.
    if lock_directory.to_str() != Some(named) {
        return false;
    }
    match read_owner_lock_record(lock_directory) {
        Ok(PublishedOwner::Owner(record)) => return record.token == token,
        Ok(PublishedOwner::Busy) | Err(_) => return false,
    }
}

/// What:
///  The owner token a landing lease in the environment carries.
/// Why:
///   A nested landing holds the ancestor's lock by that token, never by the lease text around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.parse(environment.CLI_GIT_LANDING_LEASE).token
/// ```
pub fn landing_lease_token(environment: &[(OsString, OsString)]) -> Option<String> {
    let lease: String = nonempty_variable(environment, LANDING_LEASE_VARIABLE)?;
    let value = parse_object(lease.as_str())?;
    return string_field(&value, "token").map(String::from);
}

/// Lease formatting and validation controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_lease_tests.rs"]
mod tests;
