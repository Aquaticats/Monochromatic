//! What:
//!  The private commit index of one transaction:
//!       its timestamp-preserving copies of the real index,
//!       its initialization for each selection mode,
//!       the intended tree written from it,
//!       its path listings,
//!       the pathspec-file source, and
//!       interactive and patch selection run against it.
//! Why:
//!  Git's staging and selection run against a private index and the shadow object store,
//!  so the real index and object store change only at landing.
//!  A copy keeps the real index's timestamps, because Git re-reads a stat entry
//!  whose time is not older than the index file (`index-file-timestamps.ts`, issue #544).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await initializeCommitIndex({ workspace, gitPath, cwd, mode, pathspecs, baseRevision, ... });
//! ```

/// One required Git start, its request builder and the failure text.
use super::capture_order::{CaptureIndexes, indexed_request};
use super::start_retry::RetryStart;
/// Staged additions carried into the explicit-path index, and the pathspec-file reader.
use super::commit_staged_additions::{
    StagedAddition, carry_additions, explain_ignored_refusal, pathspecs_of_file, staged_additions_selected,
};
use super::transaction_git::{GitContext, GitFailure, GitOutput, run_git_checked};
/// Commit selection mode of the transaction.
use super::transaction_journal::TransactionMode;
/// Exclusive private file writes for the pathspec snapshot.
use super::private_storage::write_private_file;
/// Real-Git command builder shared with the forwarded commands.
use super::forwarding::git_command;
/// `Read` consumes the standard input snapshot; `FileTimes` carries the copied timestamps.
use std::io::Read;
use std::fs::FileTimes;
/// `OsString` is owned operating-system text.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Name of the single-consumption stdin snapshot inside the transaction directory.
pub const PATHSPEC_INPUT_NAME: &str = "pathspec.input";

/// What:
///  Copies an index and gives the copy the source's access and modification times.
/// Why:
///   A copy with fresh times would make Git trust stat data captured in the same second
///  as a later same-size edit (issue #544).
///  A missing source is an error, as the incumbent's `copyFile` reports it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyIndexFile({ sourcePath, destinationPath });
/// ```
pub fn copy_index_with_timestamps(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::copy(source, destination)?;
    let metadata: std::fs::Metadata = std::fs::metadata(source)?;
    let times: FileTimes = FileTimes::new()
        .set_accessed(metadata.accessed()?)
        .set_modified(metadata.modified()?);
    return std::fs::File::options()
        .write(true)
        .open(destination)?
        .set_times(times);
}

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
    /// Materialized pathspec file, when the command names one.
    pub pathspec_file: Option<&'selection Path>,
    /// Whether the pathspec file separates records with NUL.
    pub pathspec_file_nul: bool,
    /// Whether an index commit stages the selected worktree content into its copy.
    pub stage_into_index: bool,
    /// Whether an index commit stages every tracked change, as `commit -a` does.
    pub stage_tracked_changes: bool,
    /// Recorded base commit, or the empty tree for an unborn base.
    pub base_revision: &'selection str,
}

/// What:
///  The selection arguments of `git add`:
///  the pathspecs after `--`, or the pathspec-file controls.
/// Why:
///   Staging reads the same selection the policies were given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// selectionArguments({ pathspecs, pathspecFile, pathspecFileNul });
/// ```
fn selection_arguments(selection: &IndexSelection) -> Vec<OsString> {
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
///  Runs one Git command against the private index and the shadow object store,
///  requiring success.
/// Why:
///   Every private-index step uses the same index and object directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, indexPath, objectDirectory, args });
/// ```
fn private_git<S: AsRef<OsStr>>(
    context: &GitContext,
    cwd: &Path,
    index: &Path,
    object_directory: &Path,
    arguments: &[S],
) -> Result<GitOutput, GitFailure> {
    return run_git_checked(
        context,
        &indexed_request(cwd, arguments, Some(index), Some(object_directory)),
    );
}

/// What:
///  Copies or builds the private commit index and snapshots the real index at invocation.
/// Why:
///   Index commits start from the real index, so their staged state is the state at invocation;
///  explicit-path commits start from the recorded base, never live `HEAD`,
///  because another landing may have moved it since invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await initializeCommitIndex({ workspace, gitPath, cwd, mode, pathspecs, baseRevision, ... });
/// ```
pub fn initialize_commit_index(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
    let copy_failure = |error: std::io::Error| { return GitFailure(format!("copying the index failed: {error}")); };
    // A repository nothing was ever staged in has no index file, which Git reads as an empty index.
    if !real_index.exists() {
        run_git_checked(
            context,
            &indexed_request(cwd, &["read-tree".to_string(), "--empty".to_string()], Some(real_index), None),
        )?;
    }
    copy_index_with_timestamps(real_index, indexes.captured_index.as_path()).map_err(copy_failure)?;
    if selection.mode == TransactionMode::Index {
        copy_index_with_timestamps(indexes.captured_index.as_path(), indexes.commit_index.as_path())
            .map_err(copy_failure)?;
        if selection.stage_tracked_changes {
            // Whole-tree update of tracked paths, exactly what `commit -a` stages, captured once at invocation.
            private_git(
                context,
                cwd,
                indexes.commit_index.as_path(),
                indexes.object_directory.as_path(),
                &["add".to_string(), "--update".to_string(), "--".to_string(), ":/".to_string()],
            )?;
        }
        if !selection.stage_into_index {
            return Ok(());
        }
    } else {
        private_git(
            context,
            cwd,
            indexes.commit_index.as_path(),
            indexes.object_directory.as_path(),
            &["read-tree".to_string(), selection.base_revision.to_string()],
        )?;
        carry_selected_staged_additions(context, cwd, indexes, selection)?;
    }
    let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--all")];
    arguments.extend(selection_arguments(selection));
    private_git(
        context,
        cwd,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
        &arguments,
    )
    .map_err(explain_ignored_refusal)?;
    return Ok(());
}

/// What:
///  Makes the selected paths that the real index stages as additions tracked in the private index.
/// Why:
///   Native `git commit -- <path>` takes such a path's worktree content without asking an ignore rule,
///  and the private index, which starts from the base, would otherwise apply the rule (issue #1807).
///  A pathspec file that this wrapper cannot read leaves the selection to Git alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await carrySelectedStagedAdditions({ context, cwd, indexes, selection });
/// ```
fn carry_selected_staged_additions(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
    let mut pathspecs: Vec<OsString> = selection.pathspecs.to_vec();
    if let Some(file) = selection.pathspec_file {
        // A file that cannot be read now fails in `git add` with Git's own message.
        let Ok(bytes) = std::fs::read(file) else {
            return Ok(());
        };
        let Some(listed) = pathspecs_of_file(bytes.as_slice(), selection.pathspec_file_nul) else {
            return Ok(());
        };
        pathspecs.extend(listed);
    }
    let additions: Vec<StagedAddition> = staged_additions_selected(
        context,
        cwd,
        indexes.captured_index.as_path(),
        selection.base_revision,
        pathspecs.as_slice(),
    )?;
    return carry_additions(
        context,
        cwd,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
        additions.as_slice(),
    );
}

/// What:
///  Writes the intended tree from the private index into the shadow object store.
/// Why:
///   The tree is the exact content the commit will carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePrivateTree({ workspace, gitPath, cwd });
/// ```
pub fn write_private_tree(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
    object_directory: &Path,
) -> Result<String, GitFailure> {
    let output: GitOutput = private_git(
        context,
        cwd,
        commit_index,
        object_directory,
        &[String::from("write-tree")],
    )?;
    let text: &str = std::str::from_utf8(output.stdout.as_slice())
        .map_err(|_| { return GitFailure(String::from("Git returned non-UTF-8 output.")); })?;
    let oid: &str = text.trim();
    if oid.is_empty() {
        return Err(GitFailure(String::from("Git returned empty private tree identity.")));
    }
    return Ok(String::from(oid));
}

/// What:
///  Lists the repository paths with unmerged stages, once each, in first-seen order.
/// Why:
///   A conflicted path cannot be committed by automatic fixes, so preparation refuses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listUnmergedIndexPaths({ gitPath, cwd, indexPath });
/// ```
pub fn list_unmerged_index_paths(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
) -> Result<Vec<Vec<u8>>, GitFailure> {
    let output: GitOutput = run_git_checked(
        context,
        &indexed_request(cwd, &["ls-files", "--unmerged", "-z"], Some(commit_index), None),
    )?;
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for record in output.stdout.split(|byte| { return *byte == 0; }) {
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else {
            continue;
        };
        let path: Vec<u8> = record[tab + 1..].to_vec();
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    return Ok(paths);
}

/// What:
///  Lists the paths the private index changes against the recorded base.
/// Why:
///   The base is the recorded one, so a landing that moved `HEAD` since invocation changes nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await listChangedIndexPaths({ gitPath, cwd, indexPath, baseRevision, objectDirectory });
/// ```
pub fn list_changed_index_paths(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
    base_revision: &str,
    object_directory: &Path,
) -> Result<Vec<Vec<u8>>, GitFailure> {
    let output: GitOutput = run_git_checked(
        context,
        &indexed_request(
            cwd,
            &["diff", "--cached", "--name-only", "-z", base_revision],
            Some(commit_index),
            Some(object_directory),
        ),
    )?;
    return Ok(output
        .stdout
        .split(|byte| { return *byte == 0; })
        .filter(|path| { return !path.is_empty(); })
        .map(<[u8]>::to_vec)
        .collect());
}

/// What:
///  Resolves a pathspec-file source to the file the private index reads.
/// Why:
///   A regular source is read where it is named, relative to the invocation directory,
///  and standard input is copied once into the transaction, so no second read sees different bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await materializePathspecFile({ workspace, effectiveCwd, source });
/// ```
pub fn materialize_pathspec_file(
    directory: &Path,
    effective_cwd: &Path,
    source: Option<&str>,
    standard_input: &mut dyn Read,
) -> std::io::Result<Option<PathBuf>> {
    let Some(name) = source else {
        return Ok(None);
    };
    if name != "-" {
        let path: PathBuf = PathBuf::from(name);
        return Ok(Some(if path.is_absolute() {
            path
        } else {
            effective_cwd.join(path)
        }));
    }
    let mut bytes: Vec<u8> = Vec::new();
    standard_input.read_to_end(&mut bytes)?;
    let path: PathBuf = directory.join(PATHSPEC_INPUT_NAME);
    write_private_file(path.as_path(), bytes.as_slice())?;
    return Ok(Some(path));
}

/// What:
///  Runs `git add --patch` or `git add --interactive` against the private index with inherited stdio.
/// Why:
///   The user's terminal answers the selection, and the choice lands only in the private copy.
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
    let shown: String = arguments
        .iter()
        .map(|argument| { return argument.to_string_lossy().into_owned(); })
        .collect::<Vec<String>>()
        .join(" ");
    let mut command = git_command(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice());
    command
        .current_dir(cwd)
        .env("GIT_INDEX_FILE", commit_index)
        .env("GIT_OBJECT_DIRECTORY", object_directory);
    let status: std::process::ExitStatus = command
        .status_retrying()
        .map_err(|error| { return GitFailure(format!("git {shown} could not start: {error}")); })?;
    if !status.success() {
        return Err(GitFailure(format!(
            "git {shown} exited {}",
            status.code().map_or_else(|| { return String::from("by signal"); }, |code| { return code.to_string(); })
        )));
    }
    return Ok(());
}

/// Private index controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_index_tests.rs"]
mod tests;
