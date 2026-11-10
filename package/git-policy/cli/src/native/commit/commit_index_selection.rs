//! What:
//!  The selection a commit names:
//!  the choices that shape its private index,
//!  the `git add` arguments of that selection,
//!  and the interactive and patch selection that runs against the private index.
//! Why:
//!  The staging steps and the selection user interface read the same selection,
//!  and `commit_index.rs` initializes the index and stays under the file-size limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await prepareInteractiveSelection({ selection, patch });
//! ```

/// One failure of a Git step.
use crate::transaction_git::{GitContext, GitFailure};
/// A Git start the system refused,
/// which is started again instead of failing.
use crate::start_retry::RetryStart;
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;
/// Real-Git command builder shared with the forwarded commands.
use crate::forwarding::git_command;
/// Who a real-Git start runs for:
/// the wrapper's own work,
/// not the person's command.
use crate::hooks_off::StartOwner;
/// `OsString` is owned operating-system text.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The choices that shape one private index.
/// Why:
///   The incumbent's `initializeCommitIndex` takes these as one object,
///  and the selection modes differ only in these values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// initializeCommitIndex({ mode, pathspecs, pathspecFile, pathspecFileNul, stageIntoIndex, stageTrackedChanges, baseRevision });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct IndexSelection<'selection> {
    /// Commit selection mode.
    pub mode: TransactionMode,
    /// Positional pathspecs of the command,
    ///  exactly as the command line gave them.
    pub pathspecs: &'selection [OsString],
    /// Materialized pathspec file,
    /// when the command names one.
    pub pathspec_file: Option<&'selection Path>,
    /// Whether the pathspec file separates records with NUL.
    pub pathspec_file_nul: bool,
    /// Whether an index commit stages the selected worktree content into its copy.
    pub stage_into_index: bool,
    /// Whether an index commit stages every tracked change,
    /// as `commit -a` does.
    pub stage_tracked_changes: bool,
    /// Recorded base commit,
    /// or the empty tree for an unborn base.
    pub base_revision: &'selection str,
}

/// What:
///  The selection arguments of `git add`:
///  the pathspecs after `--`,
/// or the pathspec-file controls.
/// Why:
///   Staging reads the same selection the policies were given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// selectionArguments({ pathspecs, pathspecFile, pathspecFileNul });
/// ```
pub(crate) fn selection_arguments(selection: &IndexSelection) -> Vec<OsString> {
    return match selection.pathspec_file {
        None => {
            let mut arguments: Vec<OsString> = vec![OsString::from("--")];
            arguments.extend(selection.pathspecs.iter().cloned());
            arguments
        }
        Some(file) => {
            let mut option: OsString = OsString::from("--pathspec-from-file=");
            option.push(file.as_os_str());
            let mut arguments: Vec<OsString> = vec![option];
            if selection.pathspec_file_nul {
                arguments.push(OsString::from("--pathspec-file-nul"));
            }
            arguments
        }
    };
}

/// What:
///  Runs `git add --patch` or `git add --interactive` against the private index with inherited stdio.
/// Why:
///   The user's terminal answers the selection,
/// and the choice lands only in the private copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await prepareInteractiveSelection({ workspace, gitPath, cwd, patch, pathspecs, pathspecFile, pathspecFileNul });
/// ```
pub fn prepare_interactive_selection(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
    object_directory: &Path,
    patch: bool,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
    let mode: &str = if patch { "--patch" } else { "--interactive" };
    let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from(mode)];
    arguments.extend(selection_arguments(selection));
    // The message shows the command as text; the command itself keeps the exact bytes.
    let mut shown_words: Vec<String> = Vec::new();
    for argument in &arguments {
        shown_words.push(argument.to_string_lossy().into_owned());
    }
    let shown: String = shown_words.join(" ");
    let mut command = git_command(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice(), StartOwner::Wrapper);
    command
        .current_dir(cwd)
        .env("GIT_INDEX_FILE", commit_index)
        .env("GIT_OBJECT_DIRECTORY", object_directory);
    let status: std::process::ExitStatus = match command.status_retrying() {
        Ok(finished) => finished,
        Err(error) => return Err(GitFailure(format!("git {shown} could not start: {error}"))),
    };
    if !status.success() {
        return Err(GitFailure(format!(
            "git {shown} exited {}",
            match status.code() {
                Some(code) => code.to_string(),
                None => String::from("by signal"),
            }
        )));
    }
    return Ok(());
}
