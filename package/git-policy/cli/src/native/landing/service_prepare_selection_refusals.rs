//! What:
//!  The findings a preparation ends with when the selection the command line itself makes cannot be committed,
//! and `None` for a selection the private staging can take.
//! Why:
//!  Each of these is the person's own input meeting a rule,
//! not a failure of the engine or of the transaction,
//! so each ends the command as a finding with its own code and exit `1`,
//!  and each is decided before the private index is built or any `git add` runs.
//!  The preparation asks these here rather than carrying their shapes,
//! which keeps one place naming what a selection can refuse
//!  (`service_prepare.rs` builds the index and asks this module first).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = namedIgnoredEnding(...) ?? stagedDeletionEnding(...);
//! ```

use crate::commit_index_selection::IndexSelection;
use crate::commit_named_ignored_files::{NAMED_IGNORED_CODE, NAMED_IGNORED_CORE_ID, named_ignored_files, named_ignored_message};
use crate::commit_staged_deletions::{STAGED_DELETION_CODE, STAGED_DELETION_CORE_ID, staged_deletion_message, staged_deletions_present_on_disk};
use crate::commit_transaction::TransactionResult;
use crate::policy_events::PolicyEvent;
use crate::transaction_git::{GitContext, GitFailure};
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The ending of a commit whose selection names a file an ignore rule refuses,
/// and `None` for a selection the private staging can take.
/// Why:
///  The decision comes from the named elements themselves,
/// so it never depends on the words of a Git message,
///  and the finding names every rejected path with both ways forward
/// (`commit_named_ignored_files.rs`,
/// issue 6278).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = namedIgnoredFiles(selection).length > 0 ? found(NAMED_IGNORED, message) : null;
/// ```
pub(crate) fn named_ignored_ending(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    selection: &IndexSelection<'_>,
) -> Result<Option<TransactionResult>, GitFailure> {
    let ignored: Vec<Vec<u8>> = named_ignored_files(context, cwd, real_index, selection)?;
    if ignored.is_empty() {
        return Ok(None);
    }
    return Ok(Some(ending(NAMED_IGNORED_CORE_ID, NAMED_IGNORED_CODE, named_ignored_message(ignored.as_slice()))));
}

/// What:
///  The ending of an explicit-path commit whose selection names a path a deletion is staged for while an entry is
/// still on disk,
/// and `None` for a selection the pathspec can record.
/// Why:
///  A pathspec selects worktree content,
/// so the deletion is not what it would record,
///  and the finding names the paths and both ways forward
/// (`commit_staged_deletions.rs`,
/// issue 472).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const present = stagedDeletionsPresentOnDisk(...); return present.length > 0 ? found(STAGED_DELETION, message) : null;
/// ```
pub(crate) fn staged_deletion_ending(
    context: &GitContext,
    cwd: &Path,
    repository_root: &Path,
    real_index: &Path,
    base_revision: &str,
    pathspecs: &[OsString],
) -> Result<Option<TransactionResult>, GitFailure> {
    let present: Vec<Vec<u8>> = staged_deletions_present_on_disk(context, cwd, repository_root, real_index, base_revision, pathspecs)?;
    if present.is_empty() {
        return Ok(None);
    }
    return Ok(Some(ending(STAGED_DELETION_CORE_ID, STAGED_DELETION_CODE, staged_deletion_message(present.as_slice()))));
}

/// What:
///  The result a refusal ends the command with:
/// one core finding,
/// nothing landed,
/// and exit `1`.
/// Why:
///   Every selection refusal reports the same way,
/// so no reader has to learn two shapes for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ events: [{ type: 'core-finding', coreId, code, message }], exitCode: 1, landedOid: null, usesLfs: false })
/// ```
fn ending(core_id: &'static str, code: &'static str, message: String) -> TransactionResult {
    return TransactionResult {
        events: vec![PolicyEvent::CoreFinding { core_id, code, message }],
        exit_code: 1,
        landed_oid: None,
        uses_lfs: false,
    };
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "service_prepare_selection_refusals_tests.rs"]
mod tests;
