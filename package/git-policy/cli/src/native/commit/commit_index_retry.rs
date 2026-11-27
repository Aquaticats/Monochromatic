//! What:
//!  The second attempt of an initialization whose branch moved under the commit:
//!  the private index is rebuilt to hold exactly the tree the retry commits.
//! Why:
//!  A `read-tree` into a fresh index leaves every entry with no stat data,
//!  and the scratch `git commit` then reads and hashes every file of the work tree to refresh it,
//!  LFS-tracked files through the clean filter:
//!  measured on 2026-11-07 in this repository,
//!  233,618,538 bytes cleaned and 3.6 s for a commit of one Markdown file.
//!  A retry runs when other commits land,
//!  so that cost fell on the busiest moments and each rebuild gave the branch time to move again.
//!  A copy of the real index keeps the stat data and the marks the tree shares with it,
//!  and an index with an assume-unchanged or skip-worktree mark,
//!  or one this crate cannot read,
//!  is seeded from the tree alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const index = await readIndexOnce(realIndex);
//! await git(['read-tree', '--reset', tree], { index: commitIndex });
//! ```

/// The note of how each build was made,
/// which the commit's timing record takes.
use crate::commit_index_builds::{
    IndexBuildKind,
    note_index_build,
};
/// The bytes of the index the retry's seed and copy are made from.
use crate::commit_index_bytes::IndexBytes;
/// How an index's bytes may seed an explicit-path commit,
/// and the copy a reset to a tree follows.
use crate::commit_index_copying::{
    PrivateIndexSeed,
    seed_of_index_bytes,
    write_index_for_reset,
};
/// The one Git start of a private-index step.
use crate::commit_index_private_git::private_git;
/// The stat-data refresh that keeps the scratch commit inside its ceiling,
/// and when it is asked for.
use crate::commit_index_stat_refresh::{
    StatRefresh,
    refresh_stat_data_when_stale,
};
/// Which paths this retry stages from the worktree,
/// which is none of them.
use crate::commit_index_stat_staleness::WorktreeStaging;
/// The private index and object store a commit's preparation selects.
use crate::private_index_request::CaptureIndexes;
/// The transaction Git wrapper:
///  the context a step runs in,
///  and the failure it reports.
use crate::transaction_git::{
    GitContext,
    GitFailure,
};
/// `Path` is a borrowed filesystem path.
use std::path::Path;
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
pub fn initialize_retry_index(
    context: &GitContext,
    cwd: &Path,
    real_index: &Path,
    indexes: &CaptureIndexes,
    tree: &str,
) -> Result<(), GitFailure> {
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
    let seed: PrivateIndexSeed = index
        .as_ref()
        .map_or(PrivateIndexSeed::Base, seed_of_the_read_index);
    note_index_build(IndexBuildKind::Retry, seed.keeps_stat_data());
    // The copy is written from the bytes this read returned, so the marks it clears and the stat data it keeps describe
    // the one state the read saw.
    if seed != PrivateIndexSeed::Base
        && let Some(read) = index.as_ref()
    {
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
    private_git(
        context,
        cwd,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
        &arguments,
    )?;
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
    let _outcome: StatRefresh = refresh_stat_data_when_stale(
        context,
        cwd,
        indexes.commit_index.as_path(),
        WorktreeStaging::None,
    );
    return Ok(());
}
