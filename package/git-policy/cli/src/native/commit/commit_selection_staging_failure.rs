//! What:
//!  Why a start of the private staging ended:
//!  Git's own failure,
//!  or the caller's pathspec that matches nothing.
//! Why:
//!   Only one cause of these starts is the caller's input,
//!  so the two are kept apart:
//!  a Git failure stays the engine failure it was,
//!  and a refused pathspec ends the command with a finding of its own (issue 6283).
//!  A refusal Git prints for an ignore rule keeps its own sentence,
//!  because the sentence says the remedy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type SelectionStagingFailure = { kind: 'git'; failure: GitFailure } | { kind: 'pathspec'; tokens: Uint8Array[] };
//! ```

/// The sentence an ignored refusal keeps,
///  which says the remedy a person acts on.
use crate::commit_staged_additions::explain_ignored_refusal;
/// The Git failure a start reports,
///  and the sentence an ignored refusal keeps.
use crate::transaction_git::GitFailure;

/// What:
///  Why a start of the private staging ended:
///  Git's own failure,
/// or the caller's pathspec that matches nothing.
/// Why:
///   Only one cause of these starts is the caller's input,
/// so the two are kept apart:
/// a Git failure stays the engine failure it was,
///  and a refused pathspec ends the command with a finding of its own (issue 6283).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SelectionStagingFailure = { kind: 'git'; failure: GitFailure } | { kind: 'pathspec'; tokens: Uint8Array[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectionStagingFailure {
    /// A Git start failed for a reason that is not a pathspec the caller wrote naming nothing.
    Git(GitFailure),
    /// Git refused pathspecs the caller wrote because they match nothing.
    PathspecMatchesNothing(Vec<Vec<u8>>),
}

impl From<GitFailure> for SelectionStagingFailure {
    /// What:
    ///  A Git failure of the staging keeps the class it had before the classification existed.
    /// Why:
    ///  Every step that is not the classified refusal needs no change of report.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fromGitFailure = (failure) => ({ kind: 'git', failure });
    /// ```
    fn from(failure: GitFailure) -> Self {
        return Self::Git(failure);
    }
}

/// What:
///  `explain_ignored_refusal` for a start of the selection staging,
/// as the failure this file's callers take.
/// Why:
///  The ignored-refusal sentence still applies to a pathspec file this reader passes to Git,
///  and what remains of the failure is Git's,
/// which the caller reports as the engine failure it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explainAStagingFailure = (failure) => ({ kind: 'git', failure: explainIgnoredRefusal(failure) });
/// ```
pub(crate) fn explain_a_git_failure(failure: GitFailure) -> SelectionStagingFailure {
    return SelectionStagingFailure::Git(explain_ignored_refusal(failure));
}
