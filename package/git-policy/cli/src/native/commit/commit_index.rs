//! What:
//!  The private commit index of one transaction:
//!       its timestamp-preserving copies of the real index,
//!       its initialization for each selection mode,
//!       the intended tree written from it,
//!       its path listings,
//!       and the pathspec-file source.
//! Why:
//!  Git's staging and selection run against a private index and the shadow object store,
//!  so the real index's entries and the object store change only at landing.
//!  The real index's stat data is refreshed earlier,
//!  as native `git commit` refreshes it,
//!  so a stale index costs one refresh and not one on every commit,
//!  and only an index that no longer describes the worktree is refreshed at all.
//!  A copy keeps the real index's timestamps,
//! because Git re-reads a stat entry
//!  whose time is not older than the index file (issue #544).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await initializeCommitIndex({ workspace, gitPath, cwd, mode, pathspecs, baseRevision, ... });
//! ```

/// One required Git start,
/// its request builder and the failure text.
use crate::private_index_request::{CaptureIndexes, indexed_request};
/// Staged additions carried into the explicit-path index,
/// and the pathspec-file reader.
use crate::commit_staged_additions::{
    StagedAddition, carry_additions, explain_ignored_refusal, pathspecs_of_file, staged_additions_selected,
};
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;
/// The one read of the real index every private copy of this commit is written from,
/// kept in a file of its own,
/// with the failure a read or a copy reports.
pub use crate::commit_index_bytes::copy_index_with_timestamps;
use crate::commit_index_bytes::{IndexBytes, copy_index_failure};
/// How an index's bytes may seed an explicit-path commit,
/// and the copy a reset to a tree follows.
use crate::commit_index_copying::{PrivateIndexSeed, seed_of_index_bytes, write_index_for_reset};
/// The note of how each build was made,
/// which the commit's timing record takes.
use crate::commit_index_builds::{IndexBuildKind, note_index_build};
/// The stat-data refresh that keeps the scratch commit inside its ceiling,
/// and when it is asked for.
use crate::commit_index_stat_refresh::{StatRefresh, refresh_stat_data, refresh_stat_data_when_stale};
/// Which paths this commit stages from the worktree,
/// so their recorded stat data cannot cost the scratch commit a read.
use crate::commit_index_stat_staleness::WorktreeStaging;
/// The selection a commit names,
/// and the `git add` arguments it stages with.
use crate::commit_index_selection::selection_arguments;
pub use crate::commit_index_selection::{IndexSelection, prepare_interactive_selection};
/// `OsString` is owned operating-system text.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::Path;

/// Name of the single-consumption stdin snapshot inside the transaction directory.
pub const PATHSPEC_INPUT_NAME: &str = "pathspec.input";

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
///  Refreshes the real index's stat data,
/// copies or builds the private commit index from it,
/// and snapshots the real index at invocation.
/// Why:
///   Index commits start from the real index,
/// so their staged state is the state at invocation;
///  explicit-path commits start from the recorded base,
/// never live `HEAD`,
///  because another landing may have moved it since invocation.
///  The stat data is refreshed first,
/// because a copy carries it into the scratch commit,
/// and that refresh is the one native `git commit` makes and writes back (`commit_index_stat_refresh.rs`).
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
    let real_index_present: bool = real_index.exists();
    if !real_index_present {
        run_git_checked(
            context,
            &indexed_request(cwd, &["read-tree".to_string(), "--empty".to_string()], Some(real_index), None),
        )?;
    }
    // The real index's stat data is refreshed before anything is copied from it, so the copies carry it and the scratch
    // commit does not read and hash the whole work tree inside its three-second step (`commit_index_stat_refresh.rs`);
    // the real index keeps the refresh, so the commit after this one does not pay it again. The refresh asks for Git and
    // its lock only when the index no longer describes the worktree, so an up-to-date index costs neither
    // (`commit_index_stat_staleness.rs`).
    // What:
    //  The paths this commit writes into the private index from the worktree,
    //  whose recorded stat data the scratch commit never reads.
    // Why:
    //  An explicit-path commit stages its named paths with `git add --all`,
    //  and an index commit takes what the index holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const staging = selection.mode === 'explicit-path' ? { named: selection.pathspecs } : { none: true };
    // ```
    let staging: WorktreeStaging<'_> = match selection.mode {
        TransactionMode::ExplicitPath => WorktreeStaging::Named(selection.pathspecs),
        TransactionMode::Index => WorktreeStaging::None,
    };
    // What:
    //  The answer of the refresh step,
    //  which says whether the private index needs it instead.
    // Why:
    //  Only a refresh that was asked for and not kept leaves the private index needing it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const refresh = statRefreshIfStale({ context, cwd, index: realIndex, staging });
    // ```
    let refresh: StatRefresh = refresh_stat_data_when_stale(context, cwd, real_index, staging);
    stage_into_the_private_index(context, cwd, real_index, real_index_present, indexes, selection)?;
    if refresh == StatRefresh::NotKept {
        // The real index could not be written: another program held `index.lock`, or the step was ended at its ceiling.
        // The private index is refreshed here instead, so the scratch commit still finds the work tree's stat data and is
        // not ended at its own ceiling, where a stop would fail the commit.
        refresh_stat_data(context, cwd, indexes.commit_index.as_path());
    }
    return Ok(());
}

/// What:
///  Copies the real index into the private commit index and stages the selection into it,
/// in the order the selection mode needs.
/// Why:
///   The refresh of the real index's stat data has to come before the copies and the refresh of the private index after
/// them,
/// so the copies are one step and its two refreshes are the other (`initialize_commit_index`).
/// `real_index_present` is asked by the caller before it makes an empty index for a repository that has none,
/// because the file exists by the time this runs.
///  The real index is read one time here,
///  and both of the copies below and the seed decision are made from those bytes,
///  so a staging that lands in the real index between this read and the copies reaches neither of them.
///  A copy is written even when the seed is the base alone,
///  because the landing compares the person's index against the state this commit saw.
///  The copy is written from the bytes one read returned,
///  so neither of them can hold a state the real index reached afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const index = await readIndexOnce(realIndex); await index.writeCopy(capturedIndex); await git(['add', '--all', ...selection], { index: commitIndex });
/// ```
fn stage_into_the_private_index(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    real_index_present: bool,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
    // What:
    //  The one read of the real index every private copy of this commit is written from.
    // Why:
    //  A copy that read its own source again cost a second and a third read of the same megabytes, and the copy of a
    //  file this process had just written waited seconds inside the kernel under this host's write pressure
    //  (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const index = await IndexBytes.read(realIndex);
    // ```
    let index: IndexBytes = IndexBytes::read(real_index).map_err(copy_index_failure)?;
    index.write_copy(indexes.captured_index.as_path()).map_err(copy_index_failure)?;
    if selection.mode == TransactionMode::Index {
        index.write_copy(indexes.commit_index.as_path()).map_err(copy_index_failure)?;
        // The commit's index is a copy of the real one, so its entries carry the stat data a real index had to keep.
        note_index_build(IndexBuildKind::First, real_index_present);
        if selection.stage_tracked_changes {
            // Whole-tree update of tracked paths, exactly what `commit -a` stages, captured once at invocation. The
            // pathspec is `:/`, one the wrapper wrote, so every pathspec mode the caller set is removed from this start:
            // literal mode would read it as a file name and stage nothing, and two glob modes at once make Git refuse it
            // and fail the commit.
            let staged: GitRequest = indexed_request(
                cwd,
                &["add", "--update", "--", ":/"],
                Some(indexes.commit_index.as_path()),
                Some(indexes.object_directory.as_path()),
            )
            .own_pathspecs();
            run_git_checked(context, &staged)?;
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
        // slower way, from the base alone. An intent-to-add entry carries no stat data and stops nothing: the copy is
        // made, the marks it holds are cleared, and the reset below writes the base's own entry at each of those paths,
        // replaces a mark whose mode or object name differs and drops one the base does not hold
        // (`write_index_for_reset`).
        let seed: PrivateIndexSeed = seed_of_index_bytes(index.bytes());
        // A private index built without the real index's stat data makes the scratch commit hash every file (233 MB of LFS content
        //  here), so which way it was built goes into the commit's timing record, which is written on every commit.
        note_index_build(IndexBuildKind::First, seed.keeps_stat_data());
        if seed != PrivateIndexSeed::Base {
            write_index_for_reset(
                context,
                cwd,
                &index,
                indexes.commit_index.as_path(),
                indexes.object_directory.as_path(),
                &seed,
                selection.base_revision.len(),
            )?;
        }
        // An index that already holds exactly the base has nothing for `read-tree --reset` to remove and no staged addition
        // to carry, and the copy keeps everything the reset would have kept (the stat data and the extensions), so the Git
        // start and the rewrite of the whole index are saved. Only the in-process readers can say so; a question they leave
        // to Git is answered by the slow way below, never by one more Git start here.
        // An index holding an intent-to-add entry never takes it: the reset has work to do at those paths, and the
        // additions they name have to be carried, or a force-added ignored file the selection names stays out of the
        // commit where native Git records it (`commit_index_copying.rs`).
        if seed == PrivateIndexSeed::Copy && index_equals_base(context, cwd, indexes, selection.base_revision) {
            return add_selected_paths(context, cwd, indexes, selection);
        }
        let mut read_tree: Vec<String> = vec![String::from("read-tree")];
        if seed != PrivateIndexSeed::Base {
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
///  Makes the private index of a retry hold exactly `tree`,
/// starting from a copy of the real index so every entry the tree shares with it keeps its cached stat data.
/// Why:
///   A `read-tree` into a fresh index leaves every entry with no stat data,
/// and the scratch `git commit` then reads and hashes every file of the work tree to refresh it,
/// LFS-tracked files through the clean filter:
/// measured on 2026-11-07 in this repository,
/// 233,618,538 bytes cleaned and 3.6 s with the index of a retry built that way against 0.1 s with the real index's stat data,
/// for a commit of one Markdown file.
/// A retry runs when the branch moved under a commit,
/// which is when other commits land,
/// so the cost fell on the busiest moments and each rebuild gave the branch time to move again.
/// An index that holds an assume-unchanged or skip-worktree mark,
/// or that cannot be read,
/// is not copied,
/// as for the first preparation (`seed_of_index_bytes`),
/// and the tree is read into a fresh index;
/// an index whose only marks are intent-to-add entries is copied as it is,
/// so the commit keeps the real index's stat data,
///  and the paths those marks name are cleared before the reset,
/// as for the first preparation (`write_index_for_reset`):
/// the reset then writes the requested tree's own entry at each of them,
/// so a scratch-commit hook that invalidates the cached tree cannot leave one of them out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const index = await readIndexOnce(realIndex); await git(['read-tree', '--reset', tree], { index: commitIndex });
/// ```
pub fn initialize_retry_index(context: &GitContext, cwd: &Path, real_index: &Path, indexes: &CaptureIndexes, tree: &str) -> Result<(), GitFailure> {
    let mut arguments: Vec<String> = vec![String::from("read-tree")];
    // The real index's stat data is refreshed before the read below, for the reason the first preparation gives
    // (`initialize_commit_index`): a rebuild runs when other commits land, so it is the one that can least afford to read
    // the whole work tree again. A rebuild stages nothing from the worktree, so every entry the copied index holds is
    // one whose recorded stat data the scratch commit would distrust.
    // What:
    //  The answer of the refresh step for a rebuild,
    //  which stages no path from the worktree.
    // Why:
    //  Only a refresh that was asked for and not kept leaves the private index needing one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const refresh = statRefreshIfStale({ context, cwd, index: realIndex, staging: { none: true } });
    // ```
    let refresh: StatRefresh = refresh_stat_data_when_stale(context, cwd, real_index, WorktreeStaging::None);
    // What:
    //  The one read of the real index a rebuild's seed and copy are made from,
    //  or nothing when that index cannot be read.
    // Why:
    //  An index this crate cannot read seeds the private index from the tree alone, as it always has; every other one is
    //  read once, after the refresh, so the marks and the stat data the copy carries describe one state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const index = await IndexBytes.read(realIndex).catch(() => undefined);
    // ```
    let index: Option<IndexBytes> = IndexBytes::read(real_index).ok();
    let seed: PrivateIndexSeed = index.as_ref().map_or(PrivateIndexSeed::Base, |read| { return seed_of_index_bytes(read.bytes()); });
    note_index_build(IndexBuildKind::Retry, seed.keeps_stat_data());
    // The copy is written from the bytes this read returned, so the marks it clears and the stat data it keeps describe
    // the one state the read saw.
    if seed != PrivateIndexSeed::Base && let Some(read) = index.as_ref() {
        write_index_for_reset(
            context,
            cwd,
            read,
            indexes.commit_index.as_path(),
            indexes.object_directory.as_path(),
            &seed,
            tree.len(),
        )?;
        arguments.push(String::from("--reset"));
    }
    arguments.push(tree.to_string());
    private_git(context, cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path(), &arguments)?;
    if refresh == StatRefresh::NotKept {
        // The real index could not be written, or holds no stat data to keep: the private index is refreshed so the scratch
        // commit does not read the whole work tree inside its own ceiling.
        refresh_stat_data(context, cwd, indexes.commit_index.as_path());
    }
    return Ok(());
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
///  Whether the private index holds exactly the tree of the base revision,
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
///  The private index is the one asked and not the captured one it was copied from:
///  a copy the intent-to-add entries were left out of is missing files the base holds,
///  and the reset below is what puts them back into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readerAnswer(['diff-index', '--cached', '--name-only', '-z', '--no-renames', base], privateIndex)?.stdout.length === 0
/// ```
fn index_equals_base(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, base_revision: &str) -> bool {
    // The question is answered in process from the whole captured index and the base tree, which no Git child covers, so the
    // record names it: an explicit-path preparation whose milliseconds went here would otherwise look unexplained.
    let began: std::time::Instant = std::time::Instant::now();
    let equals: bool = reads_itself_as_the_base(context, cwd, indexes, base_revision);
    crate::prepare_work_spans::note_span(crate::prepare_work_spans::INDEX_EQUALS_BASE, began.elapsed());
    return equals;
}

/// What:
///  `index_equals_base` without the note of how long the question took.
/// Why:
///   The note covers the question a reader declined as well as the one it answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readerAnswer(...)?.stdout.length === 0
/// ```
fn reads_itself_as_the_base(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, base_revision: &str) -> bool {
    let request = indexed_request(
        cwd,
        &["diff-index", "--cached", "--name-only", "-z", "--no-renames", base_revision],
        Some(indexes.commit_index.as_path()),
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
    // `--full-name`: the names are handed to `:(top,literal)<path>` specs and to the person, and a commit run
    // from a subdirectory would otherwise be answered with names read from that directory.
    let output: GitOutput = run_git_checked(
        context,
        &indexed_request(cwd, &["ls-files", "--unmerged", "-z", "--full-name"], Some(commit_index), None),
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

/// Private index controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_index_tests.rs"]
mod tests;

/// Cached stat data of the explicit-path private index is checked in its own sidecar.
#[cfg(test)]
#[path = "commit_index_stat_tests.rs"]
mod stat_tests;
