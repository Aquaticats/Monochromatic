//! What:
//!  The landing lease:
//!  a capability a running wrapper passes to the
//!       Git it starts,
//!  so a nested wrapper invocation from `rebase --exec` proceeds
//!       under its ancestor instead of waiting for it.
//! Why:
//!  Without a lease a nested invocation would wait for the landing lock its own
//!      ancestor holds,
//!  forever.
//!  Both wrappers format and validate the same lease
//!      (`src/index-lock/landing-lease.ts`).
//!  The wrapper runs no external hook, so it passes no preparation lease any more.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (await hasValidLandingLease({ environment, lockDirectory })) { /* skip the landing lock */ }
//! ```

/// Record readers and the ordered writer.
use super::json_record::{ObjectWriter, parse_object, string_field};
/// Owner records of published locks.
use super::owner_lock_record::{PublishedOwner, read_owner_lock_record};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::Path;

/// Environment variable carrying the landing lease to a forwarded index writer.
pub const LANDING_LEASE_VARIABLE: &str = "CLI_GIT_LANDING_LEASE";

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
