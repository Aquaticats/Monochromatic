//! What:
//!  How the service adds a review entry to the shadow log of a worktree that people and agents are editing at the same
//!  time: append only, never create, never rewrite, once per commit, with a bounded repair of entries another writer
//!  overwrote.
//! Why:
//!  `doc/audit/standards-review-gate-shadow.md` is a tracked file that sessions edit while the service appends to it, and
//!  no lock may be taken. One `O_APPEND` write of the whole entry cannot clobber anyone's content and cannot interleave
//!  with another append. What an append cannot prevent is another writer that read the file before it and writes the
//!  whole file back after it, which loses the entry. Each entry is therefore also kept as a plain file in the Git
//!  directory, and every delivery checks the latest few entries are still in the log and appends the missing ones again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!(await readFile(log)).includes(sha12)) await appendFile(log, text); await repairRecent(log, spool);
//! ```

use super::private_storage::{create_private_directory_all, write_private_file_unsynced};
use super::standards_review_entry::SHORT_SHA_LENGTH;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Spooled entries the repair checks against the log on each delivery.
pub const REPAIR_WINDOW: usize = 16;

/// Spooled entries kept; older ones are removed so the spool stays small.
pub const SPOOL_KEPT: usize = 4 * REPAIR_WINDOW;

/// What a delivery did.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivery {
    /// The entry was appended.
    Appended,
    /// The log already names this commit, so nothing was added.
    AlreadyThere,
}

/// What:
///  The spool directory of one worktree under a Git directory: entries of different worktrees are never mixed, because
///  each worktree's log holds the history of its own branch.
/// Why:
///   A worktree on another branch lacks commits that a sibling worktree made, and must not be given their entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'cli-git/review-entries', fnv1a(repository))
/// ```
pub fn spool_directory(common_dir: &Path, repository: &str) -> PathBuf {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in repository.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    return common_dir.join("cli-git").join("review-entries").join(format!("{hash:016x}"));
}

/// The text the log is searched for: the first characters of the commit name, which survive a reflow of the entry.
fn token(sha: &str) -> String {
    return sha.chars().take(SHORT_SHA_LENGTH).collect();
}

/// Whether the log names the commit already; a log that cannot be read names nothing.
pub fn names_commit(log: &Path, sha: &str) -> bool {
    return std::fs::read_to_string(log).is_ok_and(|text| return text.contains(token(sha).as_str()));
}

/// One whole-entry append to a log that must already exist.
fn append_existing(log: &Path, text: &str) -> std::io::Result<()> {
    let mut file: std::fs::File = std::fs::OpenOptions::new().append(true).create(false).open(log)?;
    return file.write_all(text.as_bytes());
}

/// The spooled entries, newest first, as (commit token, path, text).
fn spooled(spool: &Path) -> Vec<(String, PathBuf, String)> {
    let mut found: Vec<(SystemTime, String, PathBuf)> = Vec::new();
    if let Ok(listing) = std::fs::read_dir(spool) {
        for entry in listing.flatten() {
            let modified: SystemTime = entry.metadata().and_then(|meta| return meta.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
            found.push((modified, entry.file_name().to_string_lossy().into_owned(), entry.path()));
        }
    }
    found.sort_by(|left, right| return right.0.cmp(&left.0).then_with(|| return right.1.cmp(&left.1)));
    return found
        .into_iter()
        .map(|(_, name, path)| return (name, path.clone(), std::fs::read_to_string(path).unwrap_or_default()))
        .collect();
}

/// What:
///  Append again the newest spooled entries whose commit the log no longer names, and remove spooled entries beyond
///  `SPOOL_KEPT`. Returns how many entries were appended again.
/// Why:
///   This is the repair for an append another writer overwrote; it looks at a bounded number of entries per call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of spooled.slice(0, window)) if (!log.includes(entry.token)) await appendFile(log, entry.text);
/// ```
pub fn repair_recent(log: &Path, spool: &Path) -> usize {
    let entries: Vec<(String, PathBuf, String)> = spooled(spool);
    let mut present: String = std::fs::read_to_string(log).unwrap_or_default();
    let mut repaired: usize = 0;
    for (name, _, text) in entries.iter().take(REPAIR_WINDOW) {
        if text.is_empty() || present.contains(name.as_str()) {
            continue;
        }
        if append_existing(log, text.as_str()).is_ok() {
            present.push_str(text.as_str());
            repaired += 1;
        }
    }
    for (_, path, _) in entries.iter().skip(SPOOL_KEPT) {
        let _removed = std::fs::remove_file(path);
    }
    return repaired;
}

/// What:
///  Deliver one entry for `sha` to `log`: nothing if the log already names the commit, otherwise keep the entry in the
///  spool, append it, then repair the latest few entries.
/// Why:
///   Keeping the commit-name test first makes a repeated item (a crash after the append, before the item was settled)
///   add nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deliver({ log, spool, sha, text })
/// ```
pub fn deliver(log: &Path, spool: Option<&Path>, sha: &str, text: &str) -> Result<Delivery, String> {
    let current: String = std::fs::read_to_string(log).map_err(|error| return format!("reading {} failed: {error}", log.display()))?;
    let name: String = token(sha);
    if current.contains(name.as_str()) {
        return Ok(Delivery::AlreadyThere);
    }
    if let Some(directory) = spool {
        create_private_directory_all(directory).map_err(|error| return format!("creating the entry spool failed: {error}"))?;
        let kept: std::io::Result<()> = write_private_file_unsynced(directory.join(name.as_str()).as_path(), text.as_bytes());
        if let Err(error) = kept && error.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(format!("keeping the entry failed: {error}"));
        }
    }
    append_existing(log, text).map_err(|error| return format!("appending to {} failed: {error}", log.display()))?;
    if let Some(directory) = spool {
        let _repaired = repair_recent(log, directory);
    }
    return Ok(Delivery::Appended);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_append_tests.rs"]
mod tests;
