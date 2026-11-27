//! What:
//!  The staging of one selection into the private commit index:
//!  the copy of the index this preparation read,
//!  the reset to the base an explicit-path commit needs,
//!  and the `git add` that stages the selected paths.
//! Why:
//!  The copies and the staging are one step,
//!  and the refresh of the private index is the other one (`initialize_commit_index`).
//!  The real index is read one time here and both copies are written from those bytes,
//!  so a staging that lands in the real index between the read and the copies reaches neither of them;
//!  a copy keeps the real index's stat data,
//!  so the scratch commit does not read and hash the whole worktree inside its own ceiling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const index = await readIndexOnce(realIndex);
//! await index.writeCopy(capturedIndex);
//! await git(['add', '--all', ...selection], { index: commitIndex });
//! ```

/// The note of how each build was made,
/// which the commit's timing record takes.
use crate::commit_index_builds::{
    IndexBuildKind,
    note_index_build,
};
/// The bytes of the index this preparation read,
///  and the failure a read or a copy reports.
use crate::commit_index_bytes::{
    IndexBytes,
    copy_index_failure,
};
/// The two questions of the private index this step asks before its Git starts.
use crate::commit_index_checks::{
    carry_selected_staged_additions,
    index_equals_base,
};
/// How an index's bytes may seed an explicit-path commit,
/// and the copy a reset to a tree follows.
use crate::commit_index_copying::{
    PrivateIndexSeed,
    seed_of_index_bytes,
    write_index_for_reset,
};
/// The one Git start of a private-index step.
use crate::commit_index_private_git::private_git;
/// The selection a commit names:
///  its mode,
///  the paths it stages and the pathspec file they may come from.
use crate::commit_index_selection::IndexSelection;
/// Why the private staging of the selection ended,
/// so a pathspec the caller named and Git refused is reported as the caller's input error.
use crate::commit_selection_staging::SelectionStagingFailure;
/// The private index and object store a commit's preparation selects,
///  and the request builder of its Git starts.
use crate::private_index_request::{
    CaptureIndexes,
    indexed_request,
};
/// The transaction Git wrapper:
///  the context a step runs in,
///  the request it is asked with,
///  and the start that requires success.
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git_checked,
};
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
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
///  `worktree_root` is the top Git answered for the invocation,
///  which is the worktree the staging's relative pathspecs are read from
/// (`commit_selection_staging.rs`,
/// `pathspec_base`).
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
pub(crate) fn stage_into_the_private_index(
    context: &GitContext,
    cwd: &Path,
    worktree_root: &Path,
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
    index
        .write_copy(indexes.captured_index.as_path())
        .map_err(copy_index_failure)?;
    if selection.mode == TransactionMode::Index {
        index
            .write_copy(indexes.commit_index.as_path())
            .map_err(copy_index_failure)?;
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
        if seed == PrivateIndexSeed::Copy
            && index_equals_base(context, cwd, indexes, selection.base_revision)
        {
            return crate::commit_selection_staging::add_selected_paths(
                context,
                cwd,
                worktree_root,
                indexes,
                selection,
            );
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
    return crate::commit_selection_staging::add_selected_paths(
        context,
        cwd,
        worktree_root,
        indexes,
        selection,
    );
}
