//! What:
//!  The records a push of a branch makes no longer true,
//!  and the step that removes them once such a push succeeded.
//! Why:
//!  A successful push of a branch carries every earlier commit of it,
//!  so an earlier failure is no longer true;
//!  a record written after that push began is about a newer commit,
//!  which the older push says nothing about,
//!  so only the records written no later than the push's start go.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function clearPushFailuresBefore(gitDir: string, branchRef: string, cutoff: Date): void;
//! ```

/// Import the record's directory and the part of its name that belongs to one branch.
use super::push_failure_notice_record_text::{
    branch_prefix,
    notice_directory,
};
/// Import the suffix of the name a command holds a record under,
/// because a claim belongs to the command holding it.
use crate::push_failure_notice_claimed_by_one_command::CLAIM_SUFFIX;
/// Import the borrowed path type the records' directory is named with.
use std::path::Path;

/// What:
///  Removes the records of one branch,
/// after a push of that branch succeeded.
/// Why:
///   A successful push of the branch carries every earlier commit of it,
/// so an earlier failure is no longer true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearPushFailures(gitDir, branchRef)
/// ```
pub fn clear_push_failures(
    git_dir: &Path,
    branch_ref: &str,
) {
    clear_push_failures_before(git_dir, branch_ref, std::time::SystemTime::now());
}

/// What:
///  Removes the records of one branch that were written no later than `cutoff`,
/// after a push of that branch succeeded.
/// Why:
///   A push carries the commits that existed when it began.
/// A record written after that moment is about a newer commit,
/// which
///  another instance of the service may have pushed and failed to push while this push's result was still being handled;
/// the
///  older push says nothing about it,
/// so it stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearPushFailuresBefore(gitDir, branchRef, pushBegan)
/// ```
pub fn clear_push_failures_before(
    git_dir: &Path,
    branch_ref: &str,
    cutoff: std::time::SystemTime,
) {
    let Ok(listing) = std::fs::read_dir(notice_directory(git_dir)) else {
        return;
    };
    let prefix: String = branch_prefix(branch_ref);
    for entry in listing.flatten() {
        let older: bool = match entry.metadata() {
            Ok(meta) => match meta.modified() {
                Ok(written) => written <= cutoff,
                Err(_) => false,
            },
            Err(_) => false,
        };
        let name: String = entry.file_name().to_string_lossy().into_owned();
        // A claim belongs to the command that holds it, which shows it and takes it with it.
        if older && name.starts_with(prefix.as_str()) && !name.ends_with(CLAIM_SUFFIX) {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}
