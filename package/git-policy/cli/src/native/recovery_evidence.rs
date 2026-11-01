//! What:
//!  Durable evidence a dead transaction left:
//!  its numbered attempt records,
//!  the real
//!       `index.lock` files those records prove it created,
//!  and Git's PID file naming it.
//! Why:
//!  Recovery releases exactly what the dead owner held and nothing else
//!      (`src/policy-engine/commit-transaction-recovery-evidence.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await releaseRecordedLocks({ directory, names, realIndexPath, ownerPid });
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// Exact reads,
///  owned-lock release and the PID file path.
use super::recovery_files::{
    LockRelease, lock_pid_path, lock_release_name, read_recovery_file, release_owned_lock,
    remove_file_if_present,
};
/// The failure of a journal record that cannot be read, with its path and ways out named.
use super::recovery_record_failure::{RecordKind, name_record_failure};
/// The index-lock record parser.
use super::transaction_journal_parse::parse_index_lock;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Landing record filename prefix.
pub const LANDING_RECORD_PREFIX: &str = "landing-";
/// Index-lock record filename prefix.
pub const INDEX_LOCK_RECORD_PREFIX: &str = "index-lock-";
/// Numbered record suffix.
pub const RECORD_SUFFIX: &str = ".json";

/// What:
///  The ascending attempt numbers of one numbered record kind among directory entries.
/// Why:
///   `attemptNumbers`:
///  `<prefix><n>.json` with `n` a positive safe integer.
///  The incumbent
///       reads `n` with `Number`,
///  which also accepts spellings no wrapper writes (whitespace,
///       a sign,
///  exponents,
///  `0x`);
///  here `n` must be ASCII digits,
///  the only form either wrapper
///       writes (recorded in `doc/handover/cli-git-native-transactions.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attemptNumbers({ names, prefix }): readonly number[];
/// ```
pub fn attempt_numbers(names: &[String], prefix: &str) -> Vec<i64> {
    // `mut` allows collecting the numbers one at a time.
    let mut attempts: Vec<i64> = Vec::new();
    for name in names {
        let Some(middle) = name.strip_prefix(prefix).and_then(strip_record_suffix) else {
            continue;
        };
        if let Some(attempt) = decimal_attempt(middle) {
            attempts.push(attempt);
        }
    }
    attempts.sort_unstable();
    return attempts;
}

/// What:
///  The text before the record suffix.
/// Why:
///   A named function keeps the chain free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (rest: string) => rest.endsWith('.json') ? rest.slice(0, -5) : undefined
/// ```
fn strip_record_suffix(rest: &str) -> Option<&str> {
    return rest.strip_suffix(RECORD_SUFFIX);
}

/// What:
///  The positive safe integer an all-digit text spells,
///  or nothing.
/// Why:
///   Leading zeros read as `Number` reads them;
///  an empty text,
///  a zero,
///  a non-digit or a
///       value beyond 2^53 - 1 is no attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^[0-9]+$/.test(text) && Number.isSafeInteger(Number(text)) && Number(text) > 0 ? Number(text) : undefined
/// ```
pub fn decimal_attempt(text: &str) -> Option<i64> {
    if text.is_empty() || !text.bytes().all(is_ascii_digit) {
        return None;
    }
    let value: i64 = text.parse::<i64>().ok()?;
    if !(1..=super::json_record::MAX_SAFE_INTEGER).contains(&value) {
        return None;
    }
    return Some(value);
}

/// What:
///  Whether a byte is an ASCII digit.
/// Why:
///   A named predicate keeps the scan free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte >= 0x30 && byte <= 0x39
/// ```
fn is_ascii_digit(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
///  The entry names of a transaction directory.
/// Why:
///   Attempt records are found by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readdir(directory)
/// ```
pub fn directory_names(directory: &Path) -> Result<Vec<String>, RecoveryError> {
    let listing: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(error) => return Err(io_failure("listing", directory, &error)),
    };
    let mut names: Vec<String> = Vec::new();
    for item in listing {
        match item {
            Ok(entry) => names.push(entry.file_name().to_string_lossy().into_owned()),
            Err(error) => return Err(io_failure("listing", directory, &error)),
        }
    }
    return Ok(names);
}

/// What:
///  Whether a transaction directory holds a landing record.
/// Why:
///   `hasLandingRecord`:
///  such a transaction is recovered only under the landing lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function hasLandingRecord(directory: string): Promise<boolean>;
/// ```
pub fn has_landing_record(directory: &Path) -> Result<bool, RecoveryError> {
    let names: Vec<String> = directory_names(directory)?;
    return Ok(!attempt_numbers(names.as_slice(), LANDING_RECORD_PREFIX).is_empty());
}

/// What:
///  Whether a transaction entered the landing critical section:
///  it recorded a real
///       `index.lock` it created,
///  or a landing attempt.
/// Why:
///   `enteredLanding`:
///  a dead owner past this point may still hold `index.lock`,
///  which only
///       a recovery under the landing lock may release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function enteredLanding(directory: string): Promise<boolean>;
/// ```
pub fn entered_landing(directory: &Path) -> Result<bool, RecoveryError> {
    let names: Vec<String> = directory_names(directory)?;
    return Ok(
        !attempt_numbers(names.as_slice(), LANDING_RECORD_PREFIX).is_empty()
            || !attempt_numbers(names.as_slice(), INDEX_LOCK_RECORD_PREFIX).is_empty(),
    );
}

/// What:
///  Remove Git's PID file beside the real index lock when it still names the dead owner.
/// Why:
///   `removeDeadPidFile`:
///  the transaction wrote `pid <owner>\n` there;
///  any other content
///       belongs to someone else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeDeadPidFile({ realIndexPath, ownerPid }): Promise<void>;
/// ```
pub fn remove_dead_pid_file(real_index: &Path, owner_pid: i64) -> Result<(), RecoveryError> {
    let pid_path: PathBuf = lock_pid_path(real_index);
    let text: Vec<u8> = match std::fs::read(&pid_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_failure("reading", pid_path.as_path(), &error)),
    };
    if text == format!("pid {owner_pid}\n").as_bytes() {
        remove_file_if_present(pid_path.as_path())?;
    }
    return Ok(());
}

/// What:
///  Release every real `index.lock` the transaction's lock records prove it created,
///  then
///       its PID file.
/// Why:
///   `releaseRecordedLocks`:
///  attempts are checked in order against the one lock path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function releaseRecordedLocks({ directory, names, realIndexPath, ownerPid }): Promise<void>;
/// ```
pub fn release_recorded_locks(
    directory: &Path,
    names: &[String],
    real_index: &Path,
    owner_pid: i64,
) -> Result<(), RecoveryError> {
    // `as_os_str().to_owned()` copies the raw path so the suffix can be appended.
    let mut lock_name: std::ffi::OsString = real_index.as_os_str().to_owned();
    lock_name.push(".lock");
    let lock_path: PathBuf = PathBuf::from(lock_name);
    for attempt in attempt_numbers(names, INDEX_LOCK_RECORD_PREFIX) {
        let record_path: PathBuf = directory.join(format!(
            "{INDEX_LOCK_RECORD_PREFIX}{attempt}{RECORD_SUFFIX}"
        ));
        let record: super::transaction_journal::IndexLockRecord =
            parse_index_lock(read_recovery_file(record_path.as_path())?.as_slice())
                .map_err(|error| { return name_record_failure(error, record_path.as_path(), RecordKind::IndexLock); })?;
        let released: LockRelease = release_owned_lock(&record.lock, lock_path.as_path())?;
        debug(
            "releaseRecordedLocks",
            format!("attempt {attempt} lock {}", lock_release_name(released)).as_str(),
        );
    }
    return remove_dead_pid_file(real_index, owner_pid);
}

/// Evidence controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_evidence_tests.rs"]
mod tests;
