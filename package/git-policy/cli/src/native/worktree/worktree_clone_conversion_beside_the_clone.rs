//! What:
//!  The clone conversion fence beside the clone (option B of
//!  `doc/planning/cli-git-clone-rung-conversion-fence.md`):
//! the decline is decided from `git check-attr` before the first copy,
//! the rest of the fence (`git ls-tree`,
//! the  per-path decision) runs beside the copies,
//! and the index is installed from a seed the fence's answer has already prepared.
//! Why:
//!  The decline decides whether the clone rung runs at all,
//! and it needs nothing but the attributes,
//! so deciding it after the copies made a
//!  declined add pay the copy and then Git's whole checkout over it
//! (the 0.644 fixture measured 1.67 to 1.75 of `main`,
//!  where the fence decided before the copy measured 1.06 to 1.09).
//! The rest of the fence needs the blob sizes and costs a similar time to the copy,
//!  so running it beside the copies keeps the add's waiting path short without putting the decline after the copy.
//! The order that keeps every path honest is fixed here:
//!  the attributes are read and the decline decided first,
//! the bytes are copied next,
//! the fence's remaining reads answer beside the copies,
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
//! const held = fenceTheAttributes(source, context); // throws the reason: a decline ends the rung here
//! const beside = startTheRestBesideTheClone(source, context, held);
//! const cloned = cloneTrackedBytes(source, destination); // throws a reason
//! const fence = beside.join(); // throws the reason the rung cannot be proven
//! const seed = seedWithoutStat(source, fence); // the source's index, the written paths' stat zeroed
//! installTheIndex(admin, seed, cloned.stats, new Set(fence.gitWrites), hashLength);
//! // after the tree difference: checkoutThroughGit(admin, destination, context, fence.gitWrites, hashLength)
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::side_by_side_on_the_runtime::AJob;
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::with_zeroed_stat;
use crate::transaction_git::GitContext;
use crate::worktree_clone_conversion_fence::{AttributeFence, Fence, decide_the_paths, fence_the_attributes};
#[cfg(test)]
use crate::worktree_clone_conversion_fence::fence_the_clone;
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
///  Which shape the clone rung's conversion fence takes in this build.
/// Why:
///   The shipped shape reads the attributes and decides the decline before the copies and the rest beside them;
/// option A′
///  decides the whole fence before anything is copied,
/// and it is kept only in a measurement build,
/// because its `git ls-tree` sits on
///  the add's waiting path where the shipped shape runs it beside the copies;
/// options C and D decide a fence before the copies too,
///  and are measurement builds alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneFence = { when: 'beside'; held: AttributeFence } | { when: 'before'; fence: Fence };
/// ```
pub enum CloneFence {
    /// The attributes were read and the decline was decided before the copies;
    /// the rest runs beside them (option B,
    /// shipped).
    Beside(AttributeFence),
    /// The whole fence was decided before anything was copied (option A′,
    /// or option C or D,
    /// measurement only).
    #[cfg(feature = "measurement-routes")]
    BeforeTheClone(Fence),
}

/// What:
///  The fence the clone rung of this build will run:
/// the attributes and the decline first,
/// then whatever this build's routes add.
/// Why:
///   One place names the shapes,
/// so the ladder only carries the answer:
/// a shipped build always
///  answers the beside-the-copies shape,
/// and a measurement build may decide the whole fence here instead
/// (`beforeclone` is option A′;
///  `decline` and `all` are options C and D,
/// which need no read of the source's paths at all).
/// A refusal or a decline is the caller's signal to take Git's own checkout,
///  and it comes out of the attributes half,
/// before the rung's record is claimed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFenceOfThisBuild(source, context): CloneFence; // throws the reason the rung cannot be proven
/// ```
pub fn the_fence_of_this_build(source: &SnapshotSource, context: &GitContext) -> Result<CloneFence, String> {
    #[cfg(feature = "measurement-routes")]
    if let Some(decided) = crate::worktree_clone_conversion_fence_measurement_routes::fence_of_the_measurement_route(source) {
        return Ok(CloneFence::BeforeTheClone(decided?));
    }
    let held: AttributeFence = fence_the_attributes(source, context)?;
    #[cfg(feature = "measurement-routes")]
    if std::env::var("MONOCHROMATIC_CLONE_FENCE").unwrap_or_default() == "beforeclone" {
        return Ok(CloneFence::BeforeTheClone(decide_the_paths(source, context, held)?));
    }
    return Ok(CloneFence::Beside(held));
}

/// What:
///  The fence's remaining job for the clone's join:
/// the attributes the add already read and the function that decides with them.
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
/// const job = fenceBesideTheClone(source, context, held); // [state, decideTheRest]
/// ```
pub fn fence_beside_the_clone(source: &SnapshotSource, context: &GitContext, held: AttributeFence) -> AJob<(SnapshotSource, GitContext, AttributeFence), Result<Fence, String>> {
    return ((source.clone(), context.clone(), held), run_the_fence);
}

/// What:
///  The job itself:
/// the fence's remaining reads over the source the add proved clean.
/// Why:
///   The job owns what it reads,
/// so it takes the source,
/// the Git context and the attributes by value and answers the fence or its reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTheFence([source, context, held]) { return decideThePaths(source, context, held); }
/// ```
fn run_the_fence(state: (SnapshotSource, GitContext, AttributeFence)) -> Result<Fence, String> {
    let began: std::time::Instant = begin_work();
    let outcome: Result<Fence, String> = decide_the_paths(&state.0, &state.1, state.2);
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
///  the add then takes Git's own checkout,
/// which is correct.
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
