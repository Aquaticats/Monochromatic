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

use super::client_journal::record_entry;
use super::diagnostic_log::debug;
use super::fs_kind::{FilesystemKind, filesystem_kind, is_subvolume_root};
use super::journald_native::Priority;
use super::transaction_git::{GitContext, GitRequest, run_git};
use super::worktree_add_plan::AddPlan;
use super::worktree_identity::WorktreeIdentity;
use super::worktree_snapshot_source::{Refusal, RefusalKind};
use std::io::Write;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSlowRoute";

/// The start of the line on standard error and of the journal message; the cause and the remedy follow.
pub const LEAD: &str = "cli-git: this worktree was not made as a snapshot, so it took longer, because ";

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
///  The text with every control character replaced by `?`.
/// Why:
///   A path in a sentence on a terminal or in the journal must not carry an escape sequence or start a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.replace(/\p{Cc}/gu, '?')
/// ```
fn plain(text: &str) -> String {
    return text.chars().map(|character: char| { return if character.is_control() { '?' } else { character }; }).collect();
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
///  How many paths `git status` lists as changed or untracked in the source, and the first of them.
/// Why:
///   Only Git's own status tells a real change from a stale record of file times, and the person is told the number and one
///  name to look for. The read takes no optional lock, so it never disturbs the checkout a person works in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { count, first } = await changedPaths(context, root); // undefined when Git cannot say
/// ```
fn changed_paths(context: &GitContext, root: &Path) -> Option<(usize, String)> {
    let request: GitRequest = GitRequest {
        cwd: root.to_path_buf(),
        arguments: ["--no-optional-locks", "status", "--porcelain=v1", "-z", "--untracked-files=normal", "--no-renames"].map(Into::into).to_vec(),
        without_prefix: true,
        ..GitRequest::default()
    };
    let output = match run_git(context, &request) {
        Ok(found) if found.code == Some(0) => found,
        Ok(found) => {
            debug(TAG, format!("Git could not list the changes ({:?}: {})", found.code, String::from_utf8_lossy(found.stderr.as_slice()).trim()).as_str());
            return None;
        }
        Err(error) => {
            debug(TAG, format!("Git could not list the changes ({error})").as_str());
            return None;
        }
    };
    let entries: Vec<&[u8]> = output.stdout.split(|byte: &u8| { return *byte == 0; }).filter(|entry: &&[u8]| { return !entry.is_empty(); }).collect();
    // Each entry is two status letters, a space, then the path.
    let first: &[u8] = entries.first()?;
    return Some((entries.len(), plain(String::from_utf8_lossy(first.get(3..).unwrap_or(first)).as_ref())));
}

/// What:
///  Whether a check's reason says a Git command is running or an operation is unfinished, and the name of the file that says
///  so.
/// Why:
///   `check_clean` reports `<name> exists in <directory>` for `index.lock`, `MERGE_HEAD` and the other marks of an
///  operation in progress; the controls of this module run each against a real source, so a change of that wording fails there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reason.split(' exists in ')[0] // when the reason has that shape
/// ```
fn operation_mark(reason: &str) -> Option<&str> {
    return reason.split_once(" exists in ").map(|(name, _directory): (&str, &str)| { return name; });
}

/// Whether a check's reason says the index cannot yet vouch for a file, which `git status` repairs.
fn index_cannot_vouch(reason: &str) -> bool {
    return reason.contains("is as new as the index") || reason.contains("differs from its index entry");
}

/// What:
///  The cause and the sentence for an unclean source.
/// Why:
///   A running command and an unfinished operation are told as such; otherwise Git's own status decides between a real change
///  and a record of file times that is out of date.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainUnclean({ context, root, subject, reason });
/// ```
fn explain_unclean(context: &GitContext, root: &Path, subject: &str, reason: &str) -> SlowRoute {
    if let Some(name) = operation_mark(reason) {
        let (word, sentence): (&'static str, String) = if name == "index.lock" {
            ("git-running", format!("another Git command is running in {subject} (its index.lock exists). Add again when it has finished, {THEN_QUICK}."))
        } else {
            ("operation-unfinished", format!("{subject} is in the middle of an operation ({} exists in its Git directory). Finish or abort it, {THEN_QUICK}.", plain(name)))
        };
        return SlowRoute { word, detail: reason.to_string(), sentence };
    }
    if let Some((count, first)) = changed_paths(context, root) {
        let paths: &str = if count == 1 { "path" } else { "paths" };
        return SlowRoute {
            word: "uncommitted-changes",
            detail: reason.to_string(),
            sentence: format!("{subject} has uncommitted changes in {count} {paths} (for example {first}). Commit or stash them, {THEN_QUICK}."),
        };
    }
    if index_cannot_vouch(reason) {
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
        RefusalKind::Unclean => explain_unclean(context, root, subject(identity), refusal.reason.as_str()),
        RefusalKind::Unsupported => explain_unsupported(refusal.reason.as_str()),
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
            sentence: format!("{subject} is not a btrfs subvolume. Run `git cli-git {command}` in it, {THEN_QUICK}."),
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
