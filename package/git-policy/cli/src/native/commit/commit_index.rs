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
use crate::private_index_request::{CaptureIndexes, caller_pathspec_request, indexed_request};
/// Staged additions carried into the explicit-path index,
/// and the pathspec-file reader.
use crate::commit_staged_additions::{
    StagedAddition, carry_additions, explain_ignored_refusal, pathspecs_of_file, staged_additions_selected,
};
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;
/// The index copy steps,
/// kept in a file of their own.
pub use crate::commit_index_copying::copy_index_with_timestamps;
use crate::commit_index_copying::{PrivateIndexSeed, copy_index_failure, copy_index_for_reset, private_index_seed};
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyIndexFile(realIndex, commitIndex); await git(['add', '--all', ...selection], { index: commitIndex });
/// ```
fn stage_into_the_private_index(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    real_index_present: bool,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
    copy_index_with_timestamps(real_index, indexes.captured_index.as_path()).map_err(copy_index_failure)?;
    if selection.mode == TransactionMode::Index {
        copy_index_with_timestamps(indexes.captured_index.as_path(), indexes.commit_index.as_path())
            .map_err(copy_index_failure)?;
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
        // (`copy_index_for_reset`).
        let seed: PrivateIndexSeed = private_index_seed(indexes.captured_index.as_path());
        // A private index built without the real index's stat data makes the scratch commit hash every file (233 MB of LFS content
        //  here), so which way it was built goes into the commit's timing record, which is written on every commit.
        note_index_build(IndexBuildKind::First, seed.keeps_stat_data());
        if seed != PrivateIndexSeed::Base {
            copy_index_for_reset(
                context,
                cwd,
                indexes.captured_index.as_path(),
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
/// as for the first preparation (`private_index_seed`),
/// and the tree is read into a fresh index;
/// an index whose only marks are intent-to-add entries is copied as it is,
/// so the commit keeps the real index's stat data,
///  and the paths those marks name are cleared before the reset,
/// as for the first preparation (`copy_index_for_reset`):
/// the reset then writes the requested tree's own entry at each of them,
/// so a scratch-commit hook that invalidates the cached tree cannot leave one of them out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyIndexFile(realIndex, commitIndex); await git(['read-tree', '--reset', tree], { index: commitIndex });
/// ```
pub fn initialize_retry_index(context: &GitContext, cwd: &Path, real_index: &Path, indexes: &CaptureIndexes, tree: &str) -> Result<(), GitFailure> {
    let mut arguments: Vec<String> = vec![String::from("read-tree")];
    let seed: PrivateIndexSeed = if real_index.exists() { private_index_seed(real_index) } else { PrivateIndexSeed::Base };
    note_index_build(IndexBuildKind::Retry, seed.keeps_stat_data());
    // The real index's stat data is refreshed before the copy below, for the reason the first preparation gives
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
    if seed != PrivateIndexSeed::Base {
        copy_index_for_reset(
            context,
            cwd,
            real_index,
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
/// await git(['add', '--', ...exactFiles], { index: commitIndex }); await git(['add', '--update', '--', ...patterns], { index: commitIndex })
/// ```
fn add_selected_paths(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), GitFailure> {
    match selection_items(selection) {
        Selected::Items(items) => return stage_items(context, cwd, indexes, items.as_slice()),
        // An empty selection stages nothing, and the scratch commit then refuses as native Git refuses the command.
        Selected::Empty => return Ok(()),
        // A pathspec file too large to name on one command line keeps its own form, and only the tracked paths it matches
        // are staged, so no file nothing staged reaches the commit.
        Selected::File => {
            let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--update")];
            arguments.extend(selection_arguments(selection));
            return private_git_with_modes(context, cwd, indexes, arguments.as_slice()).map_err(explain_ignored_refusal);
        }
        // A pathspec file this reader cannot take is Git's to read, and every path it names is staged as before.
        Selected::Unreadable => {
            let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--all")];
            arguments.extend(selection_arguments(selection));
            return private_git_with_modes(context, cwd, indexes, arguments.as_slice()).map_err(explain_ignored_refusal);
        }
    }
}

/// What:
///  Stages the selection's elements into the private index:
/// an element that names one file as it is,
/// and a directory,
/// a pattern or pathspec magic as the tracked paths it matches.
/// Why:
///  Native Git commits the tracked and already-staged paths a directory or a pattern matches,
/// and never a file nothing staged,
///  while a `git add --all` over the same pathspec staged every untracked file it matched:
/// a commit of `.`,
///  `*` or a directory then published scratch files nobody staged.
///  An exact path the worktree holds is the one shape step 3 of `SPEC.md` "Explicit-path commit" takes from the worktree:
///  a path an ignore rule matches is still refused by Git with the hint the remedy explains,
/// and a file nothing staged is committed where native Git refuses the pathspec (the wrapper's deliberate difference).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['add', '--', ...exactFiles]); await git(['add', '--update', '--', ...patterns]);
/// ```
fn stage_items(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, items: &[OsString]) -> Result<(), GitFailure> {
    let literal: bool = crate::conflict_side_modes::glob_characters_are_literal(crate::conflict_side_modes::modes_in_effect(
        context.global_prefix.as_slice(),
        context.overlay.as_slice(),
    ).as_slice());
    let mut named_files: Vec<&OsString> = Vec::new();
    let mut patterns: Vec<&OsString> = Vec::new();
    for item in items {
        if names_one_file(cwd, item.as_os_str(), literal) {
            named_files.push(item);
        } else {
            patterns.push(item);
        }
    }
    if !named_files.is_empty() {
        let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--")];
        arguments.extend(named_files.into_iter().cloned());
        private_git_with_modes(context, cwd, indexes, arguments.as_slice()).map_err(explain_ignored_refusal)?;
    }
    // A directory or a pattern stages the tracked paths it matches (`--update`), never a file nothing staged. A pathspec
    // that matches no entry of the private index fails here as native Git fails it.
    if !patterns.is_empty() {
        let mut arguments: Vec<OsString> = vec![OsString::from("add"), OsString::from("--update"), OsString::from("--")];
        arguments.extend(patterns.into_iter().cloned());
        private_git_with_modes(context, cwd, indexes, arguments.as_slice()).map_err(explain_ignored_refusal)?;
    }
    return Ok(());
}

/// What:
///  What one selection gives this staging step:
/// its elements when they can be read,
/// the pathspec file itself when it is too large to name,
/// which is what an unreadable one leaves to Git,
/// and nothing for an empty selection.
/// Why:
///   Every shape stages differently,
/// and a reader that does not take a file must not change what Git does with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Selected = { kind: 'items', items: string[] } | { kind: 'file' } | { kind: 'unreadable' };
/// ```
enum Selected {
    /// The selection's elements,
    /// from the command line or from a pathspec file.
    Items(Vec<OsString>),
    /// A pathspec file with no element.
    Empty,
    /// A pathspec file too large to name on one command line.
    File,
    /// A pathspec file this reader cannot take.
    Unreadable,
}

/// The most bytes of pathspec elements this wrapper puts on one command line instead of naming the pathspec file.
///  Exempt from a citation under `LQM` (class: reader fallback):
/// above it the file is passed as a file with `--update`,
/// so the commit still takes the tracked paths and only a file nothing staged stays out;
/// one argument may not exceed 128 KiB (`MAX_ARG_STRLEN`) and a whole Linux command line stays under 2 MiB (`ARG_MAX`),
/// so this bound leaves room for the rest of the command.
const ITEMS_BYTES_LIMIT: u64 = 64 * 1024;

/// What:
///  The elements one selection names,
/// and which form the staging step takes for them.
/// Why:
///   Telling an exact file name from a directory or a pattern needs the elements,
/// and a file's elements are the same selection a command line carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selected = pathspecFile ? readFile(pathspecFile).length <= LIMIT ? items(pathspecsOfFile(...)) : { kind: 'file' } : items(pathspecs);
/// ```
fn selection_items(selection: &IndexSelection) -> Selected {
    let Some(file) = selection.pathspec_file else {
        if selection.pathspecs.is_empty() {
            return Selected::Empty;
        }
        return Selected::Items(selection.pathspecs.to_vec());
    };
    let Ok(size) = std::fs::metadata(file).map(|held| { return held.len(); }) else { return Selected::Unreadable };
    if size > ITEMS_BYTES_LIMIT {
        return Selected::File;
    }
    let Ok(bytes) = std::fs::read(file) else { return Selected::Unreadable };
    let Some(items) = pathspecs_of_file(bytes.as_slice(), selection.pathspec_file_nul) else { return Selected::Unreadable };
    if items.is_empty() {
        return Selected::Empty;
    }
    return Selected::Items(items);
}

/// What:
///  Whether one pathspec token names one file of the worktree rather than a directory or a pattern.
/// Why:
///   An exact name is the only shape step 3 of `SPEC.md` "Explicit-path commit" selects from the worktree,
/// and a directory or a pattern must take what native Git takes.
///  Pathspec magic selects a set and never one name,
/// and a glob character is a pattern character only in a mode that reads it as one:
///  `--literal-pathspecs` and `--noglob-pathspecs` make `st*r.txt` the file of that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesOneFile = (cwd, token, literal) => !token.startsWith(':') && (literal || !/[*?[]/.test(token)) && lstat(join(cwd, token)).isFile();
/// ```
fn names_one_file(cwd: &Path, token: &OsStr, literal: bool) -> bool {
    let bytes: &[u8] = token.as_encoded_bytes();
    if bytes.first() == Some(&b':') {
        return false;
    }
    // A backslash is Git's escape and not a pattern character here: `a\\b.txt` names the file of that name when one is
    // there, and a token whose literal pathname is not there is left to Git as the pattern it may be.
    if !literal && bytes.iter().any(|byte| { return matches!(*byte, b'*' | b'?' | b'['); }) {
        return false;
    }
    let Ok(held) = std::fs::symlink_metadata(cwd.join(token)) else { return false };
    let kind = held.file_type();
    return kind.is_file() || kind.is_symlink();
}

/// What:
///  `private_git` with the caller's own pathspec-mode options reaching Git.
/// Why:
///   The staging pathspecs are the caller's,
/// so the mode the caller wrote as a global option must reach them exactly as the mode they wrote as a variable does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runGit(callerPathspecRequest(context, cwd, arguments, commitIndex, objectDirectory))
/// ```
fn private_git_with_modes(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, arguments: &[OsString]) -> Result<(), GitFailure> {
    let request: GitRequest = caller_pathspec_request(
        context,
        cwd,
        arguments,
        Some(indexes.commit_index.as_path()),
        Some(indexes.object_directory.as_path()),
    );
    run_git_checked(context, &request)?;
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
