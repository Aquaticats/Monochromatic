//! What:
//!  The clone conversion fence beside the clone (option B of
//!  `doc/planning/cli-git-clone-rung-conversion-fence.md`):
//! the fence's reads (`git check-attr`,
//! the decline rule,
//! `git ls-tree` and the decision) run on a thread of their own while the clone copies the tracked files,
//! and the
//!  index is installed from a seed the fence's answer has already prepared.
//! Why:
//!  The fence's two Git runs and its decision cost about 0.10 of `main` on this repository,
//! and the clone costs a similar time beside them,
//! so running them one after the other puts the reads on the add's waiting path for no reason:
//!  the fence reads the source and the clone copies it,
//! and neither changes what the other reads.
//! The order that keeps every path honest is fixed here:
//!  the bytes are copied first,
//! the fence answers next,
//! the index is installed with the stat data of every path Git still has to write zeroed
//! (`subvolume_index::with_zeroed_stat`),
//!  and only then are those paths written by Git.
//! Nothing between the copy and Git's write records a written path clean:
//!  no index exists before the fence answers,
//!  and the index that exists after it holds no stat data for them,
//!  so a `git status` in that window reads the bytes and finds them different instead of trusting them,
//!  and a process killed there leaves an index that cannot vouch for the paths Git had not written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const beside = startFenceBesideTheClone(source, context);
//! const cloned = cloneTrackedBytes(source, destination); // throws a reason
//! const fence = beside.join(); // throws the reason the rung cannot be proven
//! const seed = seedWithoutStat(source, fence); // the source's index, the written paths' stat zeroed
//! installTheIndex(admin, seed, cloned.stats, new Set(fence.gitWrites), hashLength);
//! // after the tree difference: checkoutThroughGit(admin, destination, context, fence.gitWrites, hashLength)
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::side_by_side_on_the_runtime::AJob;
use crate::subvolume_index::{StatFields, with_zeroed_stat};
use crate::transaction_git::GitContext;
use crate::worktree_clone_conversion_fence::{Fence, fence_the_clone};
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::{HashMap, HashSet};

/// The name the fence's own duration is noted under in the add's record,
/// where it runs on a thread the join waits for.
/// Why:
///   A step is recorded on the thread that runs it,
/// so the fence's time in its parallel window is named as work,
///  beside the window's own step.
const WORK: &str = "fenceBesideTheClone";

/// What:
///  The fence's job for the clone's join:
/// what it reads and the function that reads it.
/// Why:
///   The work beside the copies is a job of the process's blocking pool
/// (`side_by_side_on_the_runtime.rs`),
/// not a thread of its own:
/// the add's own thread waits once for the copies and the fence together,
///  and a pool that refuses a thread runs this job in turn before the copies answer.
/// The job records its own duration as work,
/// because a timing step belongs to the thread that runs it and this one runs on a worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const job = fenceBesideTheClone(source, context); // [state, runTheFence]
/// ```
pub fn fence_beside_the_clone(source: &SnapshotSource, context: &GitContext) -> AJob<(SnapshotSource, GitContext), Result<Fence, String>> {
    return ((source.clone(), context.clone()), run_the_fence);
}

/// What:
///  The job itself:
/// the fence's reads over the source the add proved clean.
/// Why:
///   The job owns what it reads,
/// so it takes the source and the Git context by value and answers the fence or its reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTheFence([source, context]) { return fenceTheClone(source, context); }
/// ```
fn run_the_fence(state: (SnapshotSource, GitContext)) -> Result<Fence, String> {
    let began: std::time::Instant = begin_work();
    let outcome: Result<Fence, String> = fence_the_clone(&state.0, &state.1);
    end_work(WORK, began);
    return outcome;
}

/// What:
///  The source's index,
/// with the stat data of every path the fence leaves to Git set to zero,
/// as the seed the destination's index is installed from.
/// Why:
///   The source's index bytes are the ones the clean check proved,
/// so the object names and modes are kept;
/// only the stat data of the written paths goes,
/// because those paths' bytes on disk are not Git's checkout's yet,
///  and an entry whose stat data matches is exactly what makes Git skip a file.
/// A seed the reader cannot walk is the rung's reason:
///  the add then takes Git's own checkout, which is correct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seed = seedWithoutStat(source, fence); // throws the reason the rung cannot be proven
/// ```
pub fn seed_without_stat(source: &SnapshotSource, fence: &Fence) -> Result<Vec<u8>, String> {
    let written: HashSet<Vec<u8>> = fence.git_writes.iter().cloned().collect();
    return with_zeroed_stat(source.clean.index_bytes.as_slice(), &written, source.clean.hash_length).ok_or_else(|| {
        return String::from("the source's index has a layout the reader does not know, so the paths Git must write cannot be installed without stat data");
    });
}

/// What:
///  Install the destination's index for one fence beside the clone:
/// the source's index as a seed with the paths Git must write stripped of their stat data,
///  and the stat fields the copies recorded for the rest.
/// Why:
///   This is the one place the order is decided after both sides answered,
///  so a control can read the index it publishes and see that no path Git still has to write is vouched for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// installBesideTheClone(admin, source, fence, cloned.stats); // throws the reason the rung cannot be proven
/// ```
pub fn install_beside_the_clone(admin: &std::fs::File, source: &SnapshotSource, fence: &Fence, stats: &HashMap<Vec<u8>, StatFields>) -> Result<(), String> {
    let seed: Vec<u8> = seed_without_stat(source, fence)?;
    let written: HashSet<Vec<u8>> = fence.git_writes.iter().cloned().collect();
    crate::worktree_clone::install_the_index(admin, seed.as_slice(), stats, &written, source.clean.hash_length)?;
    return Ok(());
}

/// Fence-beside-the-clone controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_conversion_beside_the_clone_tests.rs"]
mod tests;
