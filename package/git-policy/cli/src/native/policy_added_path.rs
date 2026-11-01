//! What: Whether a direct fix may change a tracked file it did not select, and the
//!       `patch-conflict` message when it may not.
//! Why: `SPEC.md` ("Added paths") admits such a file only when `HEAD`, the real index, the
//!      private index and the worktree all hold the blob the correction was computed
//!      against, so the fix discards nobody's change. The reasons and the remedy are the
//!      installed wrapper's words (`commit-transaction-added-paths.ts`, `assertAddablePath`
//!      and its direct-fix message), checked in its order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await assertAddablePath({ ..., path, oid, lifecycle: 'direct-fix' }); // throws AddedPathPreconditionError
//! ```

/// The modes and object names the precondition compares.
use super::candidate_object::{CandidateMode, ObjectId};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The reason `HEAD` does not hold the file the correction was computed against.
pub const ADDED_PATH_HEAD_REASON: &str =
    "HEAD does not hold it as the ordinary file the fix was computed against";

/// The reason the index holds a different entry than `HEAD`.
pub const ADDED_PATH_STAGED_REASON: &str = "it has staged changes";

/// The reason the worktree has no copy of the file.
pub const ADDED_PATH_MISSING_REASON: &str = "its worktree copy is missing";

/// The reason the worktree copy differs from `HEAD`.
pub const ADDED_PATH_UNSTAGED_REASON: &str = "its worktree copy has unstaged changes";

/// What: An index or tree entry: its mode and object.
/// Why:  Entries are compared field by field, as the installed wrapper compares mode text
///       and object names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { mode: CandidateMode; object: ObjectId };
/// ```
pub type Entry = (CandidateMode, ObjectId);

/// What: The direct-fix `patch-conflict` message for a file and a reason.
/// Why:  It names both ways out: select the file, or restore it to `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directFixMessage = ({ path, reason }) => `A policy fix needs to change ${path}, ...`;
/// ```
pub fn added_path_message(path: &str, reason: &str) -> String {
    return format!(
        "A policy fix needs to change {path}, which this fix did not select, but {reason}. \
         Include {path} in the fix pathspecs so the fix applies to its worktree copy, \
         or restore it to match HEAD (git restore --staged --worktree -- {path}), \
         then run the fix again."
    );
}

/// What: Whether a mode is an ordinary file.
/// Why:  Only regular and executable files take a full-content correction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOrdinary = (mode) => mode === 'regular' || mode === 'executable';
/// ```
fn is_ordinary(mode: CandidateMode) -> bool {
    return mode == CandidateMode::Regular || mode == CandidateMode::Executable;
}

/// What: `HEAD`'s mode for the file, or why `HEAD` and the index rule it out. `index` is
///       the private index's entry, whose object the correction was computed against; for
///       a file the fix did not select it equals the real index's entry.
/// Why:  `HEAD` must hold the same object as an ordinary file; the index must then hold it
///       with `HEAD`'s mode. The mode is what the worktree copy is checked against next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function headMode(head: Entry | undefined, index: Entry): CandidateMode; // throws the reason
/// ```
pub fn head_mode(head: Option<&Entry>, index: &Entry) -> Result<CandidateMode, &'static str> {
    let Some((mode, object)) = head else {
        return Err(ADDED_PATH_HEAD_REASON);
    };
    if *object != index.1 || !is_ordinary(*mode) {
        return Err(ADDED_PATH_HEAD_REASON);
    }
    if index.0 != *mode {
        return Err(ADDED_PATH_STAGED_REASON);
    }
    return Ok(*mode);
}

/// What: Whether the file's executable bit matches `mode`. On systems without one, it does.
/// Why:  A worktree copy whose bit differs from `HEAD` has an unstaged mode change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const executableMatches = ((stat.mode & 0o100) !== 0) === (mode === 'executable');
/// ```
fn executable_matches(metadata: &std::fs::Metadata, mode: CandidateMode) -> bool {
    #[cfg(unix)]
    {
        /// Unix permission bits.
        use std::os::unix::fs::PermissionsExt;
        return (metadata.permissions().mode() & 0o100 != 0) == (mode == CandidateMode::Executable);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (metadata, mode);
        return true;
    }
}

/// What: Why the worktree copy rules out the file, if it does. `Err` is a read the system
///       refused for another reason than the file being absent.
/// Why:  The copy must be a plain file, not a link, with `HEAD`'s executable bit and bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function worktreeRefusal(file: string, mode, headBytes): Promise<string | undefined>;
/// ```
pub fn worktree_refusal(
    file: &Path,
    mode: CandidateMode,
    head_bytes: &[u8],
) -> Result<Option<&'static str>, std::io::Error> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(file) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Some(ADDED_PATH_MISSING_REASON));
        }
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || !executable_matches(&metadata, mode) {
        return Ok(Some(ADDED_PATH_UNSTAGED_REASON));
    }
    if std::fs::read(file)? != head_bytes {
        return Ok(Some(ADDED_PATH_UNSTAGED_REASON));
    }
    return Ok(None);
}

/// Precondition and message controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_added_path_tests.rs"]
mod tests;
