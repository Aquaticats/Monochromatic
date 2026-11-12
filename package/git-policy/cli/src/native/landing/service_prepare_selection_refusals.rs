//! What:
//!  The findings a preparation ends with when the selection the command line itself makes cannot be committed,
//! and `None` for a selection the private staging can take.
//! Why:
//!  Each of these is the person's own input meeting a rule,
//! not a failure of the engine or of the transaction,
//! so each ends the command as a finding with its own code and exit `1`,
//!  and each decision is made either before the private index is built
//! or on the one staging start whose pathspecs the caller wrote,
//! never on a pre-check of every commit's waiting path.
//!  The preparation asks these here rather than carrying their shapes,
//! which keeps one place naming what a selection can refuse
//!  (`service_prepare.rs` builds the index and asks this module first and last).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = namedIgnoredEnding(...) ?? stagedDeletionEnding(...) ?? pathspecMatchingNothingEnding(...);
//! ```

use crate::commit_index_selection::IndexSelection;
use crate::commit_named_ignored_files::{NAMED_IGNORED_CODE, NAMED_IGNORED_CORE_ID, named_ignored_files, named_ignored_message};
use crate::commit_pathspec_matching_nothing::{PATHSPEC_MATCHING_NOTHING_CODE, PATHSPEC_MATCHING_NOTHING_CORE_ID, pathspec_matching_nothing_message};
use crate::commit_selection_staging::selection_elements;
use crate::commit_staged_deletions::{STAGED_DELETION_CODE, STAGED_DELETION_CORE_ID, staged_deletion_message, staged_deletions_present_on_disk};
use crate::commit_transaction::TransactionResult;
use crate::policy_events::PolicyEvent;
use crate::transaction_git::{GitContext, GitFailure};
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
    worktree_root: &Path,
    real_index: &Path,
    selection: &IndexSelection<'_>,
) -> Result<Option<TransactionResult>, GitFailure> {
    let ignored: Vec<Vec<u8>> = named_ignored_files(context, cwd, worktree_root, real_index, selection)?;
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
///  The selection is asked as one thing,
/// so the paths a `--pathspec-from-file` names reach this guard exactly as the paths on the command line do:
///  a guard reading only the command line let the file's paths through,
/// and the commit then recorded the worktree bytes over a deletion the person had staged.
///  A pathspec file this reader cannot take is left to Git,
/// which reports it when the private staging opens it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const elements = selectionElements(selection) ?? []; const present = stagedDeletionsPresentOnDisk(..., elements); return present.length > 0 ? found(STAGED_DELETION, message) : null;
/// ```
pub(crate) fn staged_deletion_ending(
    context: &GitContext,
    cwd: &Path,
    repository_root: &Path,
    real_index: &Path,
    base_revision: &str,
    selection: &IndexSelection<'_>,
) -> Result<Option<TransactionResult>, GitFailure> {
    let Some(elements) = selection_elements(selection) else { return Ok(None) };
    let present: Vec<Vec<u8>> = staged_deletions_present_on_disk(context, cwd, repository_root, real_index, base_revision, elements.as_slice())?;
    if present.is_empty() {
        return Ok(None);
    }
    return Ok(Some(ending(STAGED_DELETION_CORE_ID, STAGED_DELETION_CODE, staged_deletion_message(present.as_slice()))));
}

/// What:
///  The ending of a commit whose own pathspec Git refused for matching nothing,
/// and the finding that names the token the caller wrote (issue 6283).
/// Why:
///  Native Git decides which pathspecs are valid,
/// so the refusal is read from Git's words at the staging start that carried the caller's own tokens,
///  and it is the caller's input rather than a failure of the engine:
/// the commit was not made and nothing was staged by it.
///  A token that begins with a hyphen also says where options belong,
/// which is the shape issue 6283 reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = found('explicit-path-commit', 'pathspec-matches-nothing', message(refused));
/// ```
pub(crate) fn pathspec_matching_nothing_ending(refused: &[Vec<u8>]) -> TransactionResult {
    return ending(PATHSPEC_MATCHING_NOTHING_CORE_ID, PATHSPEC_MATCHING_NOTHING_CODE, pathspec_matching_nothing_message(refused));
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
