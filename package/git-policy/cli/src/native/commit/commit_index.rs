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
//!  The stat data of the index this commit reads is refreshed in the private copy,
//!  as native `git commit` refreshes the index it commits,
//!  so the scratch commit does not read and hash the whole worktree inside its own step,
//!  and only a copy that no longer describes the worktree is refreshed at all.
//!  The real index is never written here:
//!  a commit prepares without the real index's lock
//!  (`doc/decision/cli-git-concurrent-commits.md`),
//!  and the landing records the worktree's stat data in the index it installs
//!  (`service_land_follow.rs`),
//!  once per batch instead of once per client.
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
    StagedAddition, carry_additions, pathspecs_of_file, staged_additions_selected,
};
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;
/// The one read of the real index every private copy of this commit is written from,
/// kept in a file of its own,
/// with the failure a read or a copy reports.
pub use crate::commit_index_bytes::copy_index_with_timestamps;
use crate::commit_index_bytes::{IndexBytes, copy_index_failure};
/// Why the private staging of the selection ended,
/// so a pathspec the caller named and Git refused is reported as the caller's input error.
use crate::commit_selection_staging::SelectionStagingFailure;
/// How an index's bytes may seed an explicit-path commit,
/// and the copy a reset to a tree follows.
use crate::commit_index_copying::{PrivateIndexSeed, seed_of_index_bytes, write_index_for_reset};
/// The note of how each build was made,
/// which the commit's timing record takes.
use crate::commit_index_builds::{IndexBuildKind, note_index_build};
/// The stat-data refresh that keeps the scratch commit inside its ceiling,
/// and when it is asked for.
use crate::commit_index_stat_refresh::{StatRefresh, refresh_stat_data_when_stale};
/// Which paths this commit stages from the worktree,
/// so their recorded stat data cannot cost the scratch commit a read.
use crate::commit_index_stat_staleness::WorktreeStaging;
/// What the real index's marks hide from Git's own staging,
/// which this commit's staging must not take either.
use crate::index_flags::{IndexFlags, read_index_flags};
/// A path's bytes as an argument,
/// for a name that is not text.
use crate::path_bytes::os_from_path_bytes;
pub use crate::commit_index_selection::{IndexSelection, prepare_interactive_selection};
/// `OsStr`/`OsString` are borrowed/owned operating-system text.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

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
        &indexed_request(context, cwd, arguments, Some(index), Some(object_directory)),
    );
}

/// The one start that writes an empty index,
/// for a repository whose index file does not exist yet.
/// It writes a private copy and never the person's index.
const EMPTY_INDEX: [&str; 2] = ["read-tree", "--empty"];

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
///  Reads the real index once,
/// writes and builds the private commit index from those bytes,
/// and snapshots the real index at invocation.
/// Why:
///   Index commits start from the real index,
/// so their staged state is the state at invocation;
///  explicit-path commits start from the recorded base,
/// never live `HEAD`,
///  because another landing may have moved it since invocation.
///  The private copy's stat data is refreshed last,
/// because that copy is the index the scratch commit reads,
/// and that refresh is the one native `git commit` makes and writes back (`commit_index_stat_refresh.rs`).
///  The real index is read here and never written:
/// the read takes no lock,
/// so a preparation takes no `index.lock` and a batch of overlapping commits does not serialize on one.
///  The landing records the worktree's stat data in the index it installs (`service_land_follow.rs`),
/// once per batch instead of once per client.
///  A selection Git refuses for a pathspec that matches nothing ends here with that refusal,
/// which the caller reports as the caller's own input error (issue 6283);
/// every other failure is Git's and stays an engine failure.
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
) -> Result<(), SelectionStagingFailure> {
    // What:
    //  The index this preparation's copies are written from.
    // Why:
    //  A repository nothing was ever staged in has no index file, which Git reads as an empty index,
    //  and that empty index is written beside the private copies and never as the person's index:
    //  writing it there created the real index before any landing and took the real index's lock,
    //  so a commit in such a repository failed while another command held the lock,
    //  and two commits in it failed each other.
    //  Every commit prepares without the real index lock (`doc/decision/cli-git-concurrent-commits.md`,
    //  "Parallel preparation").
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = realIndexPresent ? realIndex : emptyIndexBesideTheCopies();
    // ```
    let real_index_present: bool = real_index.exists();
    let source: PathBuf = if real_index_present {
        real_index.to_path_buf()
    } else {
        private_git(context, cwd, indexes.captured_index.as_path(), indexes.object_directory.as_path(), &EMPTY_INDEX)?;
        indexes.captured_index.clone()
    };
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
    stage_into_the_private_index(context, cwd, source.as_path(), real_index_present, indexes, selection)?;
    // What:
    //  The refresh of the private commit index's stat data,
    //  which is the index the scratch commit reads.
    // Why:
    //  A copy that no longer describes the worktree makes the scratch `git commit` read and hash every file it names,
    //  and that read is what the step's ceiling ends, where a stop fails the commit
    //  (`commit_index_stat_refresh.rs`).
    //  The file is the private one, so the refresh takes that file's own lock and never the real index's:
    //  a commit prepares without the real index lock,
    //  and the landing records the worktree's stat data in the index it installs, once per batch
    //  (`service_land_follow.rs`).
    //  The question costs no Git start and no lock when the copy already describes the worktree
    //  (`commit_index_stat_staleness.rs`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const refresh = statRefreshIfStale({ context, cwd, index: commitIndex, staging });
    // ```
    let _outcome: StatRefresh = refresh_stat_data_when_stale(context, cwd, indexes.commit_index.as_path(), staging);
    return Ok(());
}

/// What:
///  Copies the index this preparation read into the private commit index and stages the selection into it,
/// in the order the selection mode needs.
/// Why:
///   The copies and the staging are one step,
/// and the refresh of the private index is the other one,
/// after it
/// (`initialize_commit_index`).
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
    source: &Path,
    real_index_present: bool,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
) -> Result<(), SelectionStagingFailure> {
    // What:
    //  The one read of the source index every private copy of this commit is written from.
    // Why:
    //  A copy that read its own source again cost a second and a third read of the same megabytes, and the copy of a
    //  file this process had just written waited seconds inside the kernel under this host's write pressure
    //  (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
    //  The source is the person's index, or the empty index this preparation wrote beside the copies for a repository
    //  that has none yet.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const index = await IndexBytes.read(source);
    // ```
    let index: IndexBytes = IndexBytes::read(source).map_err(copy_index_failure)?;
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
                context,
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
            crate::commit_selection_staging::add_selected_paths(context, cwd, indexes, selection)?;
            return leave_the_marked_paths_as_they_were(context, cwd, indexes, selection, index.bytes());
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
    crate::commit_selection_staging::add_selected_paths(context, cwd, indexes, selection)?;
    return leave_the_marked_paths_as_they_were(context, cwd, indexes, selection, index.bytes());
}

/// What:
///  Puts back the entry of every path the real index's marks hide from Git's own staging.
/// Why:
///   Measured against Git 2.56.0,
/// `git commit -am x` over an edited skip-worktree file records no change for it,
///  `git commit -m x -- <a skip-worktree path>` records nothing at all (exit 1,
/// "no changes added to commit"),
/// and `-a` leaves an assume-unchanged entry alone too,
///  because Git's own `add_files_to_cache` skips a marked entry.
///  The wrapper's private index is seeded from the base and carries no mark,
///  so its staging took those files and the commit recorded bytes Git would not;
///  this puts each of them back to the entry the base holds,
/// which is the entry Git leaves in place.
///  A path the person staged inside a marked entry keeps that change in the real index,
/// which the landing leaves as it is;
/// `-a` beside such a change is the one shape this leaves to the wrapper's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hidden = readIndexFlags(realIndex).marked; await git(['--literal-pathspecs', 'reset', '-q', base, '--', ...hidden]);
/// ```
fn leave_the_marked_paths_as_they_were(
    context: &GitContext,
    cwd: &Path,
    indexes: &CaptureIndexes,
    selection: &IndexSelection,
    real_index: &[u8],
) -> Result<(), SelectionStagingFailure> {
    let IndexFlags::Marked(marks) = read_index_flags(real_index) else { return Ok(()) };
    let mut hidden: Vec<Vec<u8>> = marks.skip_worktree;
    if selection.stage_tracked_changes {
        hidden.extend(marks.assume_unchanged);
    }
    if hidden.is_empty() {
        return Ok(());
    }
    let mut arguments: Vec<OsString> = vec![
        OsString::from("--literal-pathspecs"),
        OsString::from("reset"),
        OsString::from("--quiet"),
        OsString::from(selection.base_revision),
        OsString::from("--"),
    ];
    for path in hidden {
        arguments.push(os_from_path_bytes(path.as_slice()));
    }
    private_git(context, cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path(), arguments.as_slice())?;
    return Ok(());
}

/// What:
///  Which way an index one read returned may seed a rebuild's private index.
/// Why:
///   `Option::map_or` takes a named function where a closure that captures nothing stood,
///  and the answer is the seed decision asked of the bytes that read returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seedOfTheReadIndex = (read: IndexBytes) => seedOfIndexBytes(read.bytes());
/// ```
fn seed_of_the_read_index(read: &IndexBytes) -> PrivateIndexSeed {
    return seed_of_index_bytes(read.bytes());
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
    // What:
    //  The one read of the real index a rebuild's seed and copy are made from,
    //  or nothing when that index cannot be read.
    // Why:
    //  An index this crate cannot read seeds the private index from the tree alone, as it always has; every other one is
    //  read once, so the marks and the stat data the copy carries describe one state.
    //  The read takes no lock of the real index's and refreshes nothing in it:
    //  a commit prepares without that lock
    //  (`doc/decision/cli-git-concurrent-commits.md`),
    //  and the landing records the worktree's stat data in the index it installs (`service_land_follow.rs`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const index = await IndexBytes.read(realIndex).catch(() => undefined);
    // ```
    let index: Option<IndexBytes> = IndexBytes::read(real_index).ok();
    let seed: PrivateIndexSeed = index.as_ref().map_or(PrivateIndexSeed::Base, seed_of_the_read_index);
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
    // What:
    //  The refresh of the rebuilt private index's stat data.
    // Why:
    //  A rebuild runs when other commits land, so it is the one that can least afford to read the whole worktree inside
    //  the scratch step's ceiling, and every entry the copy holds that the tree shares keeps the real index's stat data
    //  (`initialize_commit_index`). A rebuild stages nothing from the worktree, so no entry is exempt from the question.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const refresh = statRefreshIfStale({ context, cwd, index: commitIndex, staging: { none: true } });
    // ```
    let _outcome: StatRefresh = refresh_stat_data_when_stale(context, cwd, indexes.commit_index.as_path(), WorktreeStaging::None);
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
        context,
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
        &indexed_request(context, cwd, &["ls-files", "--unmerged", "-z", "--full-name"], Some(commit_index), None),
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
            context,
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
