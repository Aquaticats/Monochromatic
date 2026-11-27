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

/// The one read of the real index every private copy of this commit is written from,
/// kept in a file of its own,
/// with the failure a read or a copy reports.
pub use crate::commit_index_bytes::copy_index_with_timestamps;
/// The tree and the two path listings of the private index,
/// kept in a file of their own,
/// re-exported where every caller of them already looks.
pub use crate::commit_index_listings::{
    list_changed_index_paths,
    list_unmerged_index_paths,
    write_private_tree,
};
/// The one start every private-index step makes,
/// and the empty index a repository with no index file starts from.
use crate::commit_index_private_git::{
    EMPTY_INDEX,
    private_git,
};
/// The retry that rebuilds the private index to hold one tree,
/// kept in a file of its own,
/// re-exported where every caller of it already looks.
pub use crate::commit_index_retry::initialize_retry_index;
pub use crate::commit_index_selection::{
    IndexSelection,
    prepare_interactive_selection,
};
/// The staging of one selection into the private commit index,
/// kept in a file of its own.
use crate::commit_index_staging::stage_into_the_private_index;
/// The stat-data refresh that keeps the scratch commit inside its ceiling,
/// and when it is asked for.
use crate::commit_index_stat_refresh::{
    StatRefresh,
    refresh_stat_data_when_stale,
};
/// Which paths this commit stages from the worktree,
/// so their recorded stat data cannot cost the scratch commit a read.
use crate::commit_index_stat_staleness::WorktreeStaging;
/// Why the private staging of the selection ended,
/// so a pathspec the caller named and Git refused is reported as the caller's input error.
use crate::commit_selection_staging::SelectionStagingFailure;
/// The private index and object store a commit's preparation selects.
use crate::private_index_request::CaptureIndexes;
/// The transaction Git wrapper:
///  the context a step runs in.
use crate::transaction_git::GitContext;
/// Commit selection mode of the transaction.
use crate::transaction_journal::TransactionMode;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// Name of the single-consumption stdin snapshot inside the transaction directory.
pub const PATHSPEC_INPUT_NAME: &str = "pathspec.input";

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
///  `worktree_root` is the top Git answered for the invocation,
///  which is the worktree the selection's relative pathspecs are read from,
///  whichever way the caller named it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await initializeCommitIndex({ workspace, gitPath, cwd, mode, pathspecs, baseRevision, ... });
/// ```
pub fn initialize_commit_index(
    context: &GitContext,
    cwd: &Path,
    worktree_root: &Path,
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
        private_git(
            context,
            cwd,
            indexes.captured_index.as_path(),
            indexes.object_directory.as_path(),
            EMPTY_INDEX,
        )?;
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
    stage_into_the_private_index(
        context,
        cwd,
        worktree_root,
        source.as_path(),
        real_index_present,
        indexes,
        selection,
    )?;
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
    let _outcome: StatRefresh =
        refresh_stat_data_when_stale(context, cwd, indexes.commit_index.as_path(), staging);
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
