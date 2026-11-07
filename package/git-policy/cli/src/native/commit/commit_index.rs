//! What:
//!  The private commit index of one transaction:
//!       its timestamp-preserving copies of the real index,
//!       its initialization for each selection mode,
//!       the intended tree written from it,
//!       its path listings,
//!       the pathspec-file source,
//! and
//!       interactive and patch selection run against it.
//! Why:
//!  Git's staging and selection run against a private index and the shadow object store,
//!  so the real index and object store change only at landing.
//!  A copy keeps the real index's timestamps,
//! because Git re-reads a stat entry
//!  whose time is not older than the index file (`index-file-timestamps.ts`,
//! issue #544).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await initializeCommitIndex({ workspace, gitPath, cwd, mode, pathspecs, baseRevision, ... });
//! ```

/// One required Git start,
/// its request builder and the failure text.
use crate::private_index_request::{CaptureIndexes, indexed_request};
use crate::start_retry::RetryStart;
/// Staged additions carried into the explicit-path index,
/// and the pathspec-file reader.
use crate::commit_staged_additions::{
    StagedAddition, carry_additions, explain_ignored_refusal, pathspecs_of_file, staged_additions_selected,
};
use crate::transaction_git::{GitContext, GitFailure, GitOutput, run_git_checked};
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;
/// The index copy steps,
/// kept in a file of their own.
pub use crate::commit_index_copying::copy_index_with_timestamps;
use crate::commit_index_copying::{copy_index_failure, seeds_private_index};
/// Real-Git command builder shared with the forwarded commands.
use crate::forwarding::git_command;
/// `OsString` is owned operating-system text.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::Path;

/// Name of the single-consumption stdin snapshot inside the transaction directory.
pub const PATHSPEC_INPUT_NAME: &str = "pathspec.input";

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
///  Whether one byte is the NUL byte that ends each record of a `-z` listing.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNulByte = (byte: number) => byte === 0;
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether one byte is the tab that separates the stage columns from the path in `git ls-files -u`.
///  `&u8` borrows the byte,
/// as `position` hands it over.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTabByte = (byte: number) => byte === '\t'.charCodeAt(0);
/// ```
fn is_tab_byte(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What:
///  Copies or builds the private commit index and snapshots the real index at invocation.
/// Why:
///   Index commits start from the real index,
/// so their staged state is the state at invocation;
///  explicit-path commits start from the recorded base,
/// never live `HEAD`,
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
    // A repository nothing was ever staged in has no index file, which Git reads as an empty index.
    if !real_index.exists() {
        run_git_checked(
            context,
            &indexed_request(cwd, &["read-tree".to_string(), "--empty".to_string()], Some(real_index), None),
        )?;
    }
    copy_index_with_timestamps(real_index, indexes.captured_index.as_path()).map_err(copy_index_failure)?;
    if selection.mode == TransactionMode::Index {
        copy_index_with_timestamps(indexes.captured_index.as_path(), indexes.commit_index.as_path())
            .map_err(copy_index_failure)?;
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
        // Starts from a copy of the real index and resets it to the base, as Git's own partial commit does, so every
        // entry the base shares with the real index keeps its cached stat data. A `read-tree` into a fresh index
        // leaves every entry without stat data, and the shadow `git commit` then reads and hashes every file of the
        // worktree to refresh it (measured: 2.9 s against 6 ms on a 20 000-file checkout,
        // `doc/planning/cli-git-commit-timeouts.md`).
        // The copy also keeps assume-unchanged and skip-worktree marks, and `git add --all` then skips or refuses a
        // marked file the commit named, so an index with such a mark (or one this crate cannot read) is built the
        // slower way, from the base alone.
        let seeded: bool = seeds_private_index(indexes.captured_index.as_path());
        if seeded {
            copy_index_with_timestamps(indexes.captured_index.as_path(), indexes.commit_index.as_path())
                .map_err(copy_index_failure)?;
        }
        // An index that already holds exactly the base has nothing for `read-tree --reset` to remove and no staged addition
        // to carry, and the copy keeps everything the reset would have kept (the stat data and the extensions), so the Git
        // start and the rewrite of the whole index are saved. Only the in-process readers can say so; a question they leave
        // to Git is answered by the slow way below, never by one more Git start here.
        if seeded && index_equals_base(context, cwd, indexes, selection.base_revision) {
            return add_selected_paths(context, cwd, indexes, selection);
        }
        let mut read_tree: Vec<String> = vec![String::from("read-tree")];
        if seeded {
            read_tree.push(String::from("--reset"));
        }
        read_tree.push(selection.base_revision.to_string());
        private_git(
            context,
            cwd,
            indexes.commit_index.as_path(),
            indexes.object_directory.as_path(),
            &read_tree,
        )?;
        carry_selected_staged_additions(context, cwd, indexes, selection)?;
    }
    return add_selected_paths(context, cwd, indexes, selection);
}

/// What:
///  Stages the named paths into the private index,
/// as Git's partial commit does:
/// what the worktree holds at them,
/// deletions
///  included.
/// Why:
///   Both ways of making the private index (reset to the base,
/// or the real index kept as it is) end here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['add', '--all', ...selectionArguments], { index: commitIndex })
/// ```
fn add_selected_paths(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
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
///  Whether the captured index holds exactly the tree of the base revision,
/// answered without starting Git.
/// Why:
///   The in-process reader of `diff-index --cached` lists the paths whose entry differs from the tree;
/// it declines an index
///  it does not fully understand (unmerged,
/// intent-to-add,
/// sparse,
/// split) and a tree it cannot read,
/// and a declined question
///  is "not known to be equal",
/// which takes the slow way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readerAnswer(['diff-index', '--cached', '--name-only', '-z', '--no-renames', base])?.stdout.length === 0
/// ```
fn index_equals_base(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, base_revision: &str) -> bool {
    let request = indexed_request(
        cwd,
        &["diff-index", "--cached", "--name-only", "-z", "--no-renames", base_revision],
        Some(indexes.captured_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    return match crate::repo_read_hook::try_request(context, &request) {
        Some(answer) => answer.code == Some(0) && answer.stdout.is_empty(),
        None => false,
    };
}

/// What:
///  Makes the selected paths that the real index stages as additions tracked in the private index.
/// Why:
///   Native `git commit -- <path>` takes such a path's worktree content without asking an ignore rule,
///  and the private index,
/// which starts from the base,
/// would otherwise apply the rule (issue #1807).
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
    let text: &str = match std::str::from_utf8(output.stdout.as_slice()) {
        Ok(decoded) => decoded,
        Err(_) => return Err(GitFailure(String::from("Git returned non-UTF-8 output."))),
    };
    let oid: &str = text.trim();
    if oid.is_empty() {
        return Err(GitFailure(String::from("Git returned empty private tree identity.")));
    }
    return Ok(String::from(oid));
}

/// What:
///  Lists the repository paths with unmerged stages,
/// once each,
/// in first-seen order.
/// Why:
///   A conflicted path cannot be committed by automatic fixes,
/// so preparation refuses it.
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
    for record in output.stdout.split(is_nul_byte) {
        let Some(tab) = record.iter().position(is_tab_byte) else {
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
///   The base is the recorded one,
/// so a landing that moved `HEAD` since invocation changes nothing here.
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
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for path in output.stdout.split(is_nul_byte) {
        if !path.is_empty() {
            paths.push(path.to_vec());
        }
    }
    return Ok(paths);
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
    let mut command = git_command(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice());
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

/// Private index controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_index_tests.rs"]
mod tests;

/// Cached stat data of the explicit-path private index is checked in its own sidecar.
#[cfg(test)]
#[path = "commit_index_stat_tests.rs"]
mod stat_tests;
