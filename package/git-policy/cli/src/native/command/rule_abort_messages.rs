//! What:
//!  The plain-language texts of the abort carve-out:
//!  which command is letting the operation's own state go,
//!  where the person's own changes were saved,
//!  and what is said when a copy could not be written.
//! Why:
//!  The person is told before Git runs:
//!  a copy nobody is told about is a copy they will not find,
//!  and a command refused because its copy failed must say which file and why.
//!  The wording names the command and the paths,
//!  never the wrapper's internals.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! abortCommandText('merge') === 'git merge --abort';
//! savedBeforeAbortNotice('merge', copies) // 'monochromatic-git-policy-cli: saved ...'
//! ```

use crate::conflict_side_source::FailureKind;
use crate::conflict_sides::SavedCopy;
use crate::rule_abort_operation::{AbortOperation, OperationEnd, operation_name};
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
use std::path::Path;

/// What:
///  The command an operation's abort is spelled with.
/// Why:
///   The notice names the command that is about to run,
///  in the spelling the person typed (the bare form),
///  so a reader can match the sentence to what they ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function abortCommandText(operation: AbortOperation): string;
/// ```
pub fn abort_command_text(operation: AbortOperation, ends: OperationEnd) -> &'static str {
    return match (operation, ends) {
        (AbortOperation::Merge, _) => "git merge --abort",
        (AbortOperation::Rebase, OperationEnd::Abort) => "git rebase --abort",
        (AbortOperation::Pick, OperationEnd::Abort) => "git cherry-pick --abort or git revert --abort",
        (AbortOperation::Am, OperationEnd::Abort) => "git am --abort",
        (AbortOperation::Rebase, OperationEnd::Skip) => "git rebase --skip",
        (AbortOperation::Pick, OperationEnd::Skip) => "git cherry-pick --skip or git revert --skip",
        (AbortOperation::Am, OperationEnd::Skip) => "git am --skip",
    };
}

/// What:
///  The sentence that says what the command does once the copies are on disk.
///  `OperationEnd` says whether it ends the operation or goes on with the next one.
/// Why:
///   An abort restores the state before the operation,
///  while a skip drops the commit it stopped at and continues;
///  a notice that said the wrong one would tell the person the wrong thing about their branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatTheCommandDoes(command: string, ends: OperationEnd, word: string): string;
/// ```
fn what_the_command_does(command: &str, ends: OperationEnd, word: &str) -> String {
    return match ends {
        // The command is described as standing behind the comparison the notice just named, never as already running:
        //  a source that keeps changing stops it there, and the words must still be true then.
        OperationEnd::Abort => format!("{command} restores the state before the {word} once that comparison is done."),
        OperationEnd::Skip => format!("{command} goes on with the next commit or patch once that comparison is done, without the one it stopped at."),
    };
}

/// What:
///  The notice that names the saved copies before the abort runs.
///  `&[SavedCopy]` borrows the copies the save step made.
/// Why:
///   The person learns where their work went,
///  and that the operation's own state (its conflict markers and the staged merge result) is not among it,
///  because that state is what the abort exists to let go.
///  The command and the operation are named apart,
///  because `git am --abort` can end a rebase that applies patches,
///  and a notice that said "before the am" would be false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function savedBeforeTheCommandNotice({ operation, ends, inProgress, copies }): string;
/// ```
pub fn saved_before_the_command_notice(
    operation: AbortOperation,
    ends: OperationEnd,
    in_progress: AbortOperation,
    copies: &[SavedCopy],
    directory: &Path,
) -> String {
    let command: &str = abort_command_text(operation, ends);
    let word: &str = operation_name(in_progress);
    let mut text: String = format!(
        "monochromatic-git-policy-cli: this worktree holds changes besides the {word} in progress, and {command} discards them; the wrapper saved them \
         first, so nothing you wrote before that is lost.\n"
    );
    text.push_str(format!("  copies are under {}\n", directory.display()).as_str());
    let mut named: usize = 0;
    let mut more: usize = 0;
    for copy in copies {
        if named < SHARED_PATHS_NAMED {
            match &copy.saved_to {
                Some(place) => text.push_str(format!("  {} => {}\n", copy.path, place.display()).as_str()),
                None => text.push_str(format!("  {} was not in the worktree, so there was nothing to copy\n", copy.path).as_str()),
            }
            named += 1;
            continue;
        }
        more += 1;
    }
    if more > 0 {
        text.push_str(format!("  and {more} more in the same directory\n").as_str());
    }
    // The window the wrapper cannot close, in plain words: the comparison happens once, so a write after it is not saved.
    text.push_str(
        "  before Git starts, the wrapper compares each of these files with its copy once more and saves again whatever changed, up to three \
         times; a write that lands after that last comparison is not saved, because only the program writing the file can decide against what Git \
         is about to do.\n",
    );
    text.push_str(format!("  {}\n", what_the_command_does(command, ends, word)).as_str());
    return text;
}

/// What:
///  The text of a rejected abort whose copy could not be written.
///  `&str` borrows the path and the reason the save step named.
/// Why:
///   The command would discard work that is not on disk anywhere else,
///  so it is refused and the person is told what to fix;
///  saving the file by hand and passing the hatch stays the way through.
///  The operation in progress is named apart from the command,
///  because one abort form can end another operation's state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyFailedText(operation: AbortOperation, ends: OperationEnd, inProgress: AbortOperation, failure: SaveFailure): string;
/// ```
pub fn copy_failed_text(
    operation: AbortOperation,
    ends: OperationEnd,
    in_progress: AbortOperation,
    subject: &str,
    reason: &str,
    kind: FailureKind,
) -> String {
    let command: &str = abort_command_text(operation, ends);
    let word: &str = operation_name(in_progress);
    // The advice fits what went wrong: a file that changed under the copy needs quiet, not free space.
    let advice: &str = match kind {
        FailureKind::Changed => {
            "Run the command again once nothing is writing to that file (an editor, a build or a log may be), or save the files yourself and \
             pass `--no-enforce-worktree` directly after the subcommand."
        }
        FailureKind::Io | FailureKind::Ancestor => {
            "Free the space or fix the permissions of the Git directory and run the command again, or save the files yourself and pass \
             `--no-enforce-worktree` directly after the subcommand."
        }
    };
    return format!(
        "monochromatic-git-policy-cli: the {word} in progress holds changes besides its own, and your copy of them could not be saved ({subject}: \
         {reason}), so {command} is rejected and nothing was changed. {advice}"
    );
}

/// What:
///  The paths a sentence names, the first few and how many more there are.
///  `&[String]` borrows the paths Git printed.
/// Why:
///   A sentence that named a thousand paths would bury its remedy,
///  and one that named none would leave the person guessing which file stands in the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function named_paths(paths: string[]): string;
/// ```
fn named_paths(paths: &[String]) -> String {
    let mut listed: String = paths.iter().take(SHARED_PATHS_NAMED).cloned().collect::<Vec<String>>().join(", ");
    if paths.len() > SHARED_PATHS_NAMED {
        listed.push_str(format!(", and {} more", paths.len() - SHARED_PATHS_NAMED).as_str());
    }
    return listed;
}

/// What:
///  The text of a rejected abort whose restored tree has a path the worktree holds as an untracked file.
/// Why:
///   Git refuses that reset and names the files,
///  so nothing is lost and nothing is saved;
///  a notice that said the abort runs,
///  and a copy of a file Git never touches,
///  would both describe something that did not happen.
///  The person is told which files stand in the way,
///  that Git will not restore the tree while they are there,
///  and the ways forward that lose nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function untrackedFileText(operation: AbortOperation, ends: OperationEnd, inProgress: AbortOperation, paths: string[]): string;
/// ```
pub fn untracked_file_text(operation: AbortOperation, ends: OperationEnd, in_progress: AbortOperation, paths: &[String]) -> String {
    let command: &str = abort_command_text(operation, ends);
    let word: &str = operation_name(in_progress);
    let listed: String = named_paths(paths);
    let (holding, tracks, there, them): (&str, &str, &str, &str) = if paths.len() == 1 {
        ("the untracked file", "it", "it is", "it")
    } else {
        ("untracked files", "them", "they are", "them")
    };
    let whose: &str = if paths.len() == 1 { "its" } else { "their" };
    return format!(
        "monochromatic-git-policy-cli: {command} is rejected because the worktree holds {holding} {listed}, and the {word} restores a tree that tracks \
         {tracks}: Git will not restore that tree while {there} there, so the {word} in progress still waits, and nothing was saved or changed. Move \
         {them} out of the way (mv <path> <somewhere else>), or delete {them} if you no longer need {them}, and run {command} again. To keep \
         {whose} content, copy {them} somewhere else first."
    );
}

/// The notices and the refused-save text.
#[cfg(test)]
#[path = "rule_abort_messages_tests.rs"]
mod tests;
