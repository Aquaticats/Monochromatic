//! What:
//!  Whether a published transaction directory without an owner record is abandoned,
//!  and the message for one that is not.
//! Why:
//!  A power loss can persist the rename that published a transaction directory and the bytes of `owner.json`
//!  but not the name `owner.json` in it.
//!  Nothing then says which process runs the transaction,
//!  so the PID and birth identity that decide every other directory are gone.
//!  The wrapper already judges an owner it cannot probe by age:
//!  an owner of another PID namespace counts as alive until its record outlives `FOREIGN_OWNER_PATIENCE`
//!  (`pid_namespace.rs`).
//!  The same rule serves here:
//!  a directory is abandoned once nothing in it has changed for the patience
//!  and no process has a file or working directory in it.
//!  A live wrapper writes its records within minutes,
//!  and a wrapper cannot lose its own `owner.json` name while it runs,
//!  so the age bound can fail only for a directory whose record someone removed by hand and whose wrapper then sat idle
//!  for longer than the patience.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const standing = await ownerlessStanding(directory); // 'abandoned' | { unproven: reason }
//! ```

/// The patience an owner that cannot be probed is given.
use super::pid_namespace::{FOREIGN_OWNER_PATIENCE, age_outlives_patience};
/// Processes that use a path.
use super::process_use::{UseScan, UseTarget, processes_using};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// Wall-clock times.
use std::time::SystemTime;

/// Most entries the age walk reads before it gives up on proving the directory old.
///  A transaction directory holds a few dozen records at most.
const ENTRY_LIMIT: usize = 4096;

/// What:
///  What a directory without an owner record is.
/// Why:
///   Recovery treats `Abandoned` like a dead owner's directory and fails closed on `Unproven`, naming the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnerlessStanding = 'abandoned' | { unproven: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnerlessStanding {
    /// Nothing in the directory changed for the patience and no process uses it.
    Abandoned,
    /// The directory may still be in use; the text says what is not proven.
    Unproven(String),
}

/// What:
///  The newest modification time of a directory and everything in it,
///  or why the walk stopped.
/// Why:
///   A live wrapper writes records anywhere in its directory,
///  so the age of the directory is the age of its newest entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.max(...(await walk(directory)).map((entry) => entry.mtimeMs))
/// ```
fn newest_change(directory: &Path) -> Result<Result<SystemTime, String>, RecoveryError> {
    let mut newest: SystemTime = SystemTime::UNIX_EPOCH;
    let mut pending: Vec<PathBuf> = vec![directory.to_path_buf()];
    let mut seen: usize = 0;
    while let Some(path) = pending.pop() {
        seen += 1;
        if seen > ENTRY_LIMIT {
            return Ok(Err(format!("It holds more than {ENTRY_LIMIT} entries, too many to date")));
        }
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(&path) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_failure("inspecting", path.as_path(), &error)),
        };
        let modified: SystemTime = match metadata.modified() {
            Ok(found) => found,
            Err(error) => return Ok(Err(format!("The change time of {} is unreadable ({error})", path.display()))),
        };
        newest = newest.max(modified);
        if metadata.is_dir() {
            let listing: std::fs::ReadDir = match std::fs::read_dir(&path) {
                Ok(found) => found,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(io_failure("listing", path.as_path(), &error)),
            };
            for item in listing.flatten() {
                pending.push(item.path());
            }
        }
    }
    return Ok(Ok(newest));
}

/// What:
///  The description of the processes found using a directory.
/// Why:
///   The message names the process the person has to deal with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scan.users.map((user) => `PID ${user.pid} (${user.command})`).join(', ')
/// ```
fn describe_users(scan: &UseScan) -> String {
    let mut named: Vec<String> = Vec::new();
    for user in &scan.users {
        match &user.command {
            Some(command) => named.push(format!("PID {} ({command})", user.pid)),
            None => named.push(format!("PID {}", user.pid)),
        }
    }
    return named.join(", ");
}

/// What:
///  Judge a published transaction directory that has no owner record,
///  with the age of its newest entry against `now` and the processes found using it.
/// Why:
///   The two proofs are separate so a control can set the clock and the scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownerlessStandingAt({ directory, now, scan })
/// ```
pub fn ownerless_standing_with(
    directory: &Path,
    now: SystemTime,
    scan: fn(&Path) -> UseScan,
) -> Result<OwnerlessStanding, RecoveryError> {
    let newest: SystemTime = match newest_change(directory)? {
        Ok(found) => found,
        Err(reason) => return Ok(OwnerlessStanding::Unproven(reason)),
    };
    let Ok(age) = now.duration_since(newest) else {
        return Ok(OwnerlessStanding::Unproven(String::from("Its newest entry is dated in the future")));
    };
    if !age_outlives_patience(age) {
        return Ok(OwnerlessStanding::Unproven(format!(
            "Its newest entry changed {} seconds ago",
            age.as_secs()
        )));
    }
    let found: UseScan = scan(directory);
    if !found.users.is_empty() {
        return Ok(OwnerlessStanding::Unproven(format!(
            "{} has a file or working directory in it",
            describe_users(&found)
        )));
    }
    return Ok(OwnerlessStanding::Abandoned);
}

/// What:
///  The processes using the canonical form of a directory.
/// Why:
///   Links in `/proc` resolve to canonical paths, so the directory is compared in that form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// processesUsing({ inside: await realpath(directory) })
/// ```
fn scan_canonical(directory: &Path) -> UseScan {
    let canonical: PathBuf = std::fs::canonicalize(directory).unwrap_or_else(|_| { return directory.to_path_buf(); });
    return processes_using(&UseTarget::Inside(canonical.as_path()));
}

/// What:
///  Judge a published transaction directory that has no owner record,
///  by the clock and the host's process table.
/// Why:
///   The production judgement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ownerlessStanding(directory)
/// ```
pub fn ownerless_standing(directory: &Path) -> Result<OwnerlessStanding, RecoveryError> {
    return ownerless_standing_with(directory, SystemTime::now(), scan_canonical);
}

/// What:
///  The failure for a directory whose owner is not proven gone.
/// Why:
///   The diagnostic names the directory, says why nothing is recovered, when cli-git recovers it by itself,
///  and the one manual way out with the condition under which it is safe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new CommitTransactionRecoveryError(`Transaction owner record is missing: ${directory}. ...`)
/// ```
pub fn ownerless_refusal(directory: &Path, reason: &str) -> RecoveryError {
    let minutes: u64 = FOREIGN_OWNER_PATIENCE.as_secs() / 60;
    return RecoveryError(format!(
        "Transaction owner record is missing: {dir}. A published transaction directory names the cli-git process that \
         runs it in `owner.json`, so cli-git cannot tell whether that process is still running, and every guarded \
         command stops (exit 2) until this directory is dealt with. {reason}. cli-git recovers the directory by \
         itself, with no action from you, once nothing in it has changed for {minutes} minutes and no process has a \
         file or working directory in it. To continue sooner, first confirm that no cli-git or git command is running \
         in this repository (for example with `ps`), then remove the directory with `rm -r {dir}`. Remove it by hand \
         only when it holds no `landing-<n>.json` record: with one, the commit may already have moved the branch, and \
         removing the directory skips the index installation that cli-git's own recovery performs, so wait for the \
         automatic recovery instead. The usual cause is a power loss that kept the file `owner.json` but not its name \
         in the directory.",
        dir = directory.display()
    ));
}

/// Ownerless-directory controls stay out of the release executable.
#[cfg(test)]
#[path = "ownerless_directory_tests.rs"]
mod tests;
