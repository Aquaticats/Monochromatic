//! What:
//!  The plain sentence that tells a person why a `git worktree add` was not made as a snapshot, and the places it goes:
//!  one line on standard error unless `--quiet` was given, one warning entry in the journal, and the debug log that already
//!  holds the detail.
//! Why:
//!  A snapshot of the main checkout takes seconds and Git's checkout with the ignored-state copy takes minutes. The choice
//!  between them was made silently, so two slow adds could not be explained afterwards. The sentence names the cause in the
//!  person's words (a change, a running Git command, a stale index, a missing subvolume) and what makes the next add quick,
//!  without the names of the ladder's steps. A filesystem that is not btrfs gets no sentence: nothing the person does there
//!  would make the add a snapshot, so the line would only repeat itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = explainRefusal({ context, identity, refusal });
//! route?.tell(plan); // stderr unless quiet, and the journal
//! ```

use crate::client_journal::record_entry;
use crate::diagnostic_log::debug;
use crate::fs_kind::{FilesystemKind, filesystem_kind, is_subvolume_root};
use crate::journald_native::Priority;
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_clean::CleanRefusalKind;
use crate::worktree_snapshot_source::{ChangedPaths, Refusal, RefusalKind, changed_paths, plain};
use std::io::Write;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSlowRoute";

/// The start of the line on standard error and of the journal message; the cause and the remedy follow.
pub const LEAD: &str = "monochromatic-git-policy-cli: this worktree was not made as a snapshot, so it took longer, because ";

/// What:
///  What kept the add from being a snapshot, as a sentence for the person and a word for the journal.
/// Why:
///   One sentence and one word per cause, chosen by what the cause is and never by the ladder's step that found it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlowRoute = { word: string; detail: string; sentence: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlowRoute {
    /// The journal field `CLI_GIT_SLOW_CAUSE`: a short word a reader can filter on.
    word: &'static str,
    /// The journal field `CLI_GIT_SLOW_DETAIL`: the check's own reason, as the debug log has it.
    detail: String,
    /// The rest of the line after `LEAD`: the cause, then what makes the next add quick.
    sentence: String,
}

impl SlowRoute {
    /// The journal word of the cause.
    pub fn word(&self) -> &'static str {
        return self.word;
    }

    /// The check's own reason.
    pub fn detail(&self) -> &str {
        return self.detail.as_str();
    }

    /// The whole line, without its newline.
    pub fn line(&self) -> String {
        return format!("{LEAD}{}", self.sentence);
    }

    /// What:
    ///  Tell the person on standard error unless the add was quiet, and tell the journal either way.
    /// Why:
    ///   `--quiet` silences the terminal and nothing else; the journal is where the cause is found afterwards. A closed
    ///  standard error is no failure of the add.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (!plan.quiet) console.error(route.line()); journal.warn(route.line(), fields);
    /// ```
    pub fn tell(&self, plan: &AddPlan) {
        let line: String = self.line();
        debug(TAG, format!("{}: told ({}; {})", plan.destination.display(), self.word, self.detail).as_str());
        if !plan.quiet {
            // A closed standard error leaves nowhere to print; the journal entry below still records the cause.
            let _written = std::io::stderr().write_all(format!("{line}\n").as_bytes());
        }
        let fields: [(&str, String); 3] = [
            ("CLI_GIT_WORKTREE", plan.destination.to_string_lossy().into_owned()),
            ("CLI_GIT_SLOW_CAUSE", self.word.to_string()),
            ("CLI_GIT_SLOW_DETAIL", self.detail.clone()),
        ];
        record_entry(Priority::Warning, line.as_str(), &fields);
    }
}

/// What:
///  How the source is called in a sentence: the main checkout, or the worktree the person ran the add from.
/// Why:
///   The person knows which checkout they sit in; the sentence names it the way they would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind === 'main' ? 'the main checkout' : 'this worktree'
/// ```
fn subject(identity: &WorktreeIdentity) -> &'static str {
    return match identity {
        WorktreeIdentity::MainWorktree { .. } => "the main checkout",
        _ => "this worktree",
    };
}

/// The closing words of a remedy: what the next add becomes.
const THEN_QUICK: &str = "and the next add is a snapshot that takes seconds";

/// What:
///  The sentence for a source whose Git status could not be read, with the run that failed and what the person can do.
/// Why:
///   The status is what tells a real change from a record of file times that is out of date; without it neither cause can be
///  named, and the person can read Git's own error by running the same command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = statusFailed({ subject, root, reason });
/// ```
fn status_failed(subject: &str, root: &Path, reason: &str) -> SlowRoute {
    return SlowRoute {
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
///  The cause and the sentence for an unclean source, chosen by the kind of the refusal.
/// Why:
///   A running command and an unfinished operation are told as such; an entry no reset can undo is told as that; otherwise
///  Git's own status decides between a real change and a record of file times that is out of date.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainUnclean({ context, root, subject, kind, reason });
/// ```
fn explain_unclean(context: &GitContext, root: &Path, subject: &str, kind: CleanRefusalKind, reason: &str) -> SlowRoute {
    match kind {
        CleanRefusalKind::OperationInProgress("index.lock") => {
            return SlowRoute { word: "git-running", detail: reason.to_string(), sentence: format!("another Git command is running in {subject} (its index.lock exists). Add again when it has finished, {THEN_QUICK}.") };
        }
        CleanRefusalKind::OperationInProgress(name) => {
            return SlowRoute { word: "operation-unfinished", detail: reason.to_string(), sentence: format!("{subject} is in the middle of an operation ({name} exists in its Git directory). Finish or abort it, {THEN_QUICK}.") };
        }
        CleanRefusalKind::NotPlain => {
            return SlowRoute { word: "not-restorable", detail: reason.to_string(), sentence: format!("{subject} has an index entry that no reset can undo, a skip-worktree, assume-unchanged, intent-to-add, unmerged or submodule entry ({}).", plain(reason)) };
        }
        CleanRefusalKind::Unreadable => {
            return SlowRoute { word: "unproven", detail: reason.to_string(), sentence: format!("{subject} could not be read to prove it unchanged ({}).", plain(reason)) };
        }
        CleanRefusalKind::StagedChange | CleanRefusalKind::FileChanged | CleanRefusalKind::StatStale | CleanRefusalKind::Racy | CleanRefusalKind::Untracked => {}
    }
    let listed: ChangedPaths = match changed_paths(context, root) {
        Ok(found) => found,
        Err(failure) => return status_failed(subject, root, failure.as_str()),
    };
    if let ChangedPaths { count, first: Some(first), .. } = listed {
        let paths: &str = if count == 1 { "path" } else { "paths" };
        return SlowRoute {
            word: "uncommitted-changes",
            detail: reason.to_string(),
            sentence: format!("{subject} has uncommitted changes in {count} {paths} (for example {first}). Commit or stash them, {THEN_QUICK}."),
        };
    }
    if matches!(kind, CleanRefusalKind::Racy | CleanRefusalKind::StatStale) {
        return SlowRoute {
            word: "stale-index",
            detail: reason.to_string(),
            sentence: format!("Git's record of file times in {subject} is out of date. Run `git status` there once, {THEN_QUICK}."),
        };
    }
    return SlowRoute { word: "unproven", detail: reason.to_string(), sentence: format!("{subject} could not be proven unchanged ({}).", plain(reason)) };
}

/// What:
///  The sentence for a source the expensive checks refused, or nothing when no sentence would help.
/// Why:
///   The kind of the refusal chooses between the unclean sentence, which reads Git's status, and the plain statement of an
///  unsupported feature.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainRefusal({ context, identity, refusal });
/// ```
pub fn explain_refusal(context: &GitContext, identity: &WorktreeIdentity, refusal: &Refusal) -> Option<SlowRoute> {
    let root: &Path = match identity {
        WorktreeIdentity::MainWorktree { worktree_root, .. } | WorktreeIdentity::LinkedWorktree { worktree_root, .. } => worktree_root.as_path(),
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => return None,
    };
    return Some(match refusal.kind {
        RefusalKind::Unclean(kind) => explain_unclean(context, root, subject(identity), kind, refusal.reason.as_str()),
        RefusalKind::Unsupported => explain_unsupported(refusal.reason.as_str()),
        RefusalKind::StatusFailed => status_failed(subject(identity), root, refusal.reason.as_str()),
    });
}

/// What:
///  The sentence for a feature of the source that a snapshot does not reproduce.
/// Why:
///   These reasons are already plain statements ("core.sparseCheckout is set"); the sentence adds nothing to them, and names
///  no remedy because the person's choice of feature is not the wrapper's to undo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainUnsupported('core.sparseCheckout is set');
/// ```
pub fn explain_unsupported(reason: &str) -> SlowRoute {
    return SlowRoute { word: "unsupported", detail: reason.to_string(), sentence: format!("{}.", plain(reason)) };
}

/// What:
///  The sentence for a snapshot that started and could not be finished.
/// Why:
///   The add is whole, made by Git's checkout; the sentence says why the quick way ended, in the check's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainFailure('the stage could not be swapped into place');
/// ```
pub fn explain_failure(reason: &str) -> SlowRoute {
    return SlowRoute { word: "snapshot-failed", detail: reason.to_string(), sentence: format!("the snapshot could not be finished ({}), so Git's own checkout made the worktree.", plain(reason)) };
}

/// What:
///  The sentence for the cheap refusals (the source is no subvolume, or the two sides are not on one filesystem), or nothing
///  where the filesystem is not btrfs.
/// Why:
///   On a btrfs source that is no subvolume the person can convert it; on another filesystem nothing makes the add a
///  snapshot, and a line on every add would be noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainCheapRefusal({ identity, plan, reason });
/// ```
pub fn explain_cheap_refusal(identity: &WorktreeIdentity, reason: &str) -> Option<SlowRoute> {
    let root: &Path = match identity {
        WorktreeIdentity::MainWorktree { worktree_root, .. } | WorktreeIdentity::LinkedWorktree { worktree_root, .. } => worktree_root.as_path(),
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => return None,
    };
    if filesystem_kind(root) != Ok(FilesystemKind::Btrfs) {
        return None;
    }
    let subject: &str = subject(identity);
    // The main checkout is migrated with the repository's other worktrees in view; a linked worktree is converted alone.
    let command: &str = if matches!(identity, WorktreeIdentity::MainWorktree { .. }) { "migrate-to-subvolume" } else { "convert-worktree-to-subvolume" };
    if !is_subvolume_root(root).unwrap_or(false) {
        return Some(SlowRoute {
            word: "not-subvolume",
            detail: reason.to_string(),
            sentence: format!("{subject} is not a btrfs subvolume. Run `git monochromatic-git-policy-cli {command}` in it, {THEN_QUICK}."),
        });
    }
    return Some(SlowRoute {
        word: "other-filesystem",
        detail: reason.to_string(),
        sentence: format!("the new worktree is not on the same btrfs filesystem as {subject}. Put it on the same one, {THEN_QUICK}."),
    });
}

/// Explanation controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_slow_route_tests.rs"]
mod tests;
