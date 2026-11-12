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

use crate::conflict_sides::SavedCopy;
use crate::rule_abort_operation::{AbortOperation, operation_name};
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
use std::path::{Path, PathBuf};

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
pub fn abort_command_text(operation: AbortOperation) -> &'static str {
    return match operation {
        AbortOperation::Merge => "git merge --abort",
        AbortOperation::Rebase => "git rebase --abort",
        AbortOperation::Pick => "git cherry-pick --abort or git revert --abort",
        AbortOperation::Am => "git am --abort",
    };
}

/// What:
///  The directory this invocation's copies went into,
///  or nothing when no copy has a place.
/// Why:
///   The notice shows one directory and the names below it,
///  instead of repeating a long absolute path once per file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copiesDirectory(copies: SavedCopy[]): PathBuf | undefined;
/// ```
fn copies_directory(copies: &[SavedCopy]) -> Option<PathBuf> {
    for copy in copies {
        let Some(place) = copy.saved_to.as_deref() else {
            continue;
        };
        return place.parent().map(Path::to_path_buf);
    }
    return None;
}

/// What:
///  The notice that names the saved copies before the abort runs.
///  `&[SavedCopy]` borrows the copies the save step made.
/// Why:
///   The person learns where their work went,
///  and that the operation's own state (its conflict markers and the staged merge result) is not among it,
///  because that state is what the abort exists to let go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function savedBeforeAbortNotice(operation: AbortOperation, copies: SavedCopy[]): string;
/// ```
pub fn saved_before_abort_notice(operation: AbortOperation, copies: &[SavedCopy]) -> String {
    let command: &str = abort_command_text(operation);
    let word: &str = operation_name(operation);
    let mut text: String = format!(
        "monochromatic-git-policy-cli: this worktree holds changes besides the {word} in progress, and {command} discards them; the wrapper saved them \
         first, so nothing you wrote is lost.\n"
    );
    if let Some(directory) = copies_directory(copies) {
        text.push_str(format!("  copies are under {}\n", directory.display()).as_str());
    }
    let mut named: usize = 0;
    let mut more: usize = 0;
    for copy in copies {
        if named < SHARED_PATHS_NAMED {
            match &copy.saved_to {
                Some(place) => text.push_str(format!("  {} => {}\n", copy.path, place.display()).as_str()),
                None => text.push_str(
                    format!("  {} was deleted in the worktree, so there was nothing to copy\n", copy.path).as_str(),
                ),
            }
            named += 1;
            continue;
        }
        more += 1;
    }
    if more > 0 {
        text.push_str(format!("  and {more} more in the same directory\n").as_str());
    }
    text.push_str(format!("  {command} runs now and restores the state before the {word}.\n").as_str());
    return text;
}

/// What:
///  The text of a rejected abort whose copy could not be written.
///  `&str` borrows the path and the reason the save step named.
/// Why:
///   The command would discard work that is not on disk anywhere else,
///  so it is refused and the person is told what to fix;
///  saving the file by hand and passing the hatch stays the way through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copy_failed_text(operation: AbortOperation, subject: string, reason: string): string;
/// ```
pub fn copy_failed_text(operation: AbortOperation, subject: &str, reason: &str) -> String {
    let command: &str = abort_command_text(operation);
    let word: &str = operation_name(operation);
    return format!(
        "monochromatic-git-policy-cli: the {word} in progress holds changes besides its own, and your copy of them could not be saved ({subject}: \
         {reason}), so {command} is rejected and nothing was changed. Free the space or fix the permissions of the Git directory and run the \
         command again, or save the files yourself and pass `--no-enforce-worktree` directly after the subcommand."
    );
}

/// The notices and the refused-save text.
#[cfg(test)]
#[path = "rule_abort_messages_tests.rs"]
mod tests;
