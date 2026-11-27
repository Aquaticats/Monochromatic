//! What:
//!  The sentence for a source that cannot be shown unchanged:
//!  Git's status could not be read,
//!  another Git command is running there,
//!  an operation is unfinished,
//!  an entry no reset can undo is there,
//!  or the source really has uncommitted changes.
//! Why:
//!  This is the one cause that has to read Git's own status to tell a real change from a record of file times that is
//!  out of date,
//!  so it is the one place that asks Git,
//!  and every line it builds names the source and what makes the next add quick.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = explainUnclean({ context, root, subject, kind, reason });
//! ```

use crate::transaction_git::GitContext;
use crate::worktree_clean::CleanRefusalKind;
use crate::worktree_slow_route_words::{
    LEAD,
    SlowRoute,
    THEN_QUICK,
};
use crate::worktree_snapshot_source::{
    ChangedPaths,
    changed_paths,
    plain,
};
use std::path::Path;

/// What:
///  The sentence for a source whose Git status could not be read,
/// with the run that failed and what the person can do.
/// Why:
///   The status is what tells a real change from a record of file times that is out of date;
/// without it neither cause can be
///  named,
/// and the person can read Git's own error by running the same command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = statusFailed({ subject, root, reason });
/// ```
pub(crate) fn status_failed(
    subject: &str,
    root: &Path,
    reason: &str,
) -> SlowRoute {
    return SlowRoute {
        lead: LEAD,
        word: "status-failed",
        detail: reason.to_string(),
        sentence: format!(
            "Git could not list the uncommitted changes of {subject} ({}). Run `git -C {} status` to read its error and fix what it names, or wait if another Git command is running there, {THEN_QUICK}.",
            plain(reason),
            plain(root.to_string_lossy().as_ref())
        ),
    };
}

/// What:
///  The cause and the sentence for an unclean source,
/// chosen by the kind of the refusal.
/// Why:
///   A running command and an unfinished operation are told as such;
/// an entry no reset can undo is told as that;
/// otherwise
///  Git's own status decides between a real change and a record of file times that is out of date.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainUnclean({ context, root, subject, kind, reason });
/// ```
pub(crate) fn explain_unclean(
    context: &GitContext,
    root: &Path,
    subject: &str,
    kind: CleanRefusalKind,
    reason: &str,
) -> SlowRoute {
    match kind {
        CleanRefusalKind::OperationInProgress("index.lock") => {
            return SlowRoute {
                lead: LEAD,
                word: "git-running",
                detail: reason.to_string(),
                sentence: format!(
                    "another Git command is running in {subject} (its index.lock exists). Add again when it has finished, {THEN_QUICK}."
                ),
            };
        }
        CleanRefusalKind::OperationInProgress(name) => {
            return SlowRoute {
                lead: LEAD,
                word: "operation-unfinished",
                detail: reason.to_string(),
                sentence: format!(
                    "{subject} is in the middle of an operation ({name} exists in its Git directory). Finish or abort it, {THEN_QUICK}."
                ),
            };
        }
        CleanRefusalKind::NotPlain => {
            return SlowRoute {
                lead: LEAD,
                word: "not-restorable",
                detail: reason.to_string(),
                sentence: format!(
                    "{subject} has an index entry that no reset can undo, a skip-worktree, assume-unchanged, intent-to-add, unmerged or submodule entry ({}).",
                    plain(reason)
                ),
            };
        }
        CleanRefusalKind::Unreadable => {
            return SlowRoute {
                lead: LEAD,
                word: "unproven",
                detail: reason.to_string(),
                sentence: format!(
                    "{subject} could not be read to prove it unchanged ({}).",
                    plain(reason)
                ),
            };
        }
        CleanRefusalKind::StagedChange
        | CleanRefusalKind::FileChanged
        | CleanRefusalKind::StatStale
        | CleanRefusalKind::Racy
        | CleanRefusalKind::Untracked => {}
    }
    let listed: ChangedPaths = match changed_paths(context, root) {
        Ok(found) => found,
        Err(failure) => return status_failed(subject, root, failure.as_str()),
    };
    if let ChangedPaths {
        count,
        first: Some(first),
        ..
    } = listed
    {
        let paths: &str = if count == 1 { "path" } else { "paths" };
        return SlowRoute {
            lead: LEAD,
            word: "uncommitted-changes",
            detail: reason.to_string(),
            sentence: format!(
                "{subject} has uncommitted changes in {count} {paths} (for example {first}). Commit or stash them, {THEN_QUICK}."
            ),
        };
    }
    if matches!(kind, CleanRefusalKind::Racy | CleanRefusalKind::StatStale) {
        return SlowRoute {
            lead: LEAD,
            word: "stale-index",
            detail: reason.to_string(),
            sentence: format!(
                "Git's record of file times in {subject} is out of date. Run `git status` there once, {THEN_QUICK}."
            ),
        };
    }
    return SlowRoute {
        lead: LEAD,
        word: "unproven",
        detail: reason.to_string(),
        sentence: format!(
            "{subject} could not be proven unchanged ({}).",
            plain(reason)
        ),
    };
}
