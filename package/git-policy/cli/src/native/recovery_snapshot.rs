//! What:
//!  The check that an index snapshot still holds the bytes the landing wrote,
//!  made before recovery trusts it.
//! Why:
//!  Neither snapshot is synced when the landing writes it,
//!  while the landing record that names them is.
//!  A power loss can persist the record and the moved branch and leave a snapshot empty,
//!  cut short or filled with bytes that never reached the disk.
//!  Installing such a post-index leaves a real index every later command rejects,
//!  and trusting such a pre-landing snapshot would compare the real index with the wrong bytes.
//!  The check recomputes the length and hash the landing recorded;
//!  a record without them (written before they existed) is trusted as before.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (recorded && !sameDigest(await digestFile(snapshot), recorded)) throw new CommitTransactionRecoveryError(message);
//! ```

/// The length and hash of a snapshot.
use super::content_digest::{ContentDigest, digest_file};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// The records the message describes.
use super::transaction_journal::{LandingOperation, LandingRecord, PreparingRecord, landing_record_filename};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Which snapshot a check is about,
///  as the words the message uses.
/// Why:
///   The two snapshots fail for different reasons and the person needs to know which file to look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotRole = 'pre-landing' | 'post-index';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotRole {
    /// The copy of the real index taken before the landing.
    PreLanding,
    /// The index the landing computed to install.
    Post,
}

impl SnapshotRole {
    /// What:
    ///  The sentence that says what recovery could not do because the snapshot differs.
    /// Why:
    ///   A damaged post-index is not installed;
    ///  a damaged pre-landing snapshot cannot prove the real index unchanged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// role === 'post-index' ? 'recovery did not install it as the real index' : '...'
    /// ```
    fn consequence(self) -> &'static str {
        match self {
            SnapshotRole::PreLanding => {
                return "recovery cannot prove from it that the real index is unchanged since the landing began, so it \
                        installed nothing";
            }
            SnapshotRole::Post => return "recovery did not install it as the real index",
        }
    }

    /// What:
    ///  The name the message gives the snapshot.
    /// Why:
    ///   The file names alone do not say which of the two is which.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// role === 'post-index' ? 'post-index' : 'pre-landing'
    /// ```
    fn label(self) -> &'static str {
        match self {
            SnapshotRole::PreLanding => return "pre-landing index snapshot",
            SnapshotRole::Post => return "post-index snapshot",
        }
    }
}

/// What:
///  Fail closed when a snapshot differs from the digest its landing record holds.
///  Does nothing when the record holds none.
/// Why:
///   See the module comment;
///  the message names the snapshot, the landing record and the transaction directory,
///  says what recovery left unchanged, and lists the steps that finish the commit by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await verifySnapshot({ directory, preparing, landing, snapshot, role, recorded });
/// ```
pub fn verify_snapshot(
    directory: &Path,
    preparing: &PreparingRecord,
    landing: &LandingRecord,
    snapshot: &Path,
    role: SnapshotRole,
    recorded: Option<ContentDigest>,
) -> Result<(), RecoveryError> {
    let Some(expected) = recorded else {
        return Ok(());
    };
    let found: ContentDigest = match digest_file(snapshot) {
        Ok(digest) => digest,
        Err(error) => return Err(io_failure("reading the index snapshot", snapshot, &error)),
    };
    if found == expected {
        return Ok(());
    }
    let landing_file: String = landing_record_filename(landing.attempt);
    // The pinned link is named `<snapshot>.recovery`; the person looks for the original name.
    let named: String = match snapshot.file_name().and_then(|name| { return name.to_str(); }) {
        Some(name) => directory.join(name.strip_suffix(".recovery").unwrap_or(name)).display().to_string(),
        None => snapshot.display().to_string(),
    };
    let branch_state: String = match (landing.operation, landing.new_oid.as_deref()) {
        (LandingOperation::Commit, Some(oid)) => {
            format!("Commit {oid} is already on {}", preparing.target_ref)
        }
        _ => format!("{} was not moved by this normalization", preparing.target_ref),
    };
    return Err(RecoveryError(format!(
        "The {} {} does not match the length and hash that {landing_file} recorded for it \
         (recorded {} bytes with hash {}, found {} bytes with hash {}), so {}. \
         {branch_state}, and recovery changed neither the branch nor the real index {}. \
         Recovery retained the transaction at {}. \
         To finish by hand, first check that no Git command is running in this repository. \
         Then remove {}.lock (if it exists), {} (if it exists) and {}, \
         run `git status`, and if the index does not match HEAD, run `git reset`, \
         which makes the index match HEAD and leaves the worktree files unchanged. \
         Every guarded command stops until {} is removed.",
        role.label(),
        named,
        expected.length,
        expected.hash_text(),
        found.length,
        found.hash_text(),
        role.consequence(),
        preparing.real_index_path,
        directory.display(),
        preparing.real_index_path,
        preparing.shadow_path,
        directory.display(),
        directory.display(),
    )));
}

/// Snapshot verification controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_snapshot_tests.rs"]
mod tests;
