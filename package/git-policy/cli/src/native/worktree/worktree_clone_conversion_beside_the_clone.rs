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
use crate::subvolume_index::{StatFields, with_zeroed_stat};
use crate::transaction_git::GitContext;
use crate::worktree_clone_conversion_fence::{Fence, fence_the_clone};
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::{HashMap, HashSet};
use std::thread::JoinHandle;

/// The name the fence's own duration is noted under in the add's record,
/// where it runs on a thread the join waits for.
/// Why:
///   A step is recorded on the thread that runs it,
/// so the fence's time in its parallel window is named as work,
///  beside the window's own step.
const WORK: &str = "fenceBesideTheClone";

/// The thread's name,
/// which a `ps` or a debugger shows while it runs.
const THREAD: &str = "clone-conversion-fence";

/// What:
///  The fence that another thread is deciding,
/// and the reason its thread could not be started.
/// Why:
///   The caller starts the fence before it clones,
/// joins it after the clone whatever the clone's answer was,
///  and never leaves a thread reading the source while Git's own checkout writes the destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FenceBesideTheClone = { handle?: Thread; refusal?: string };
/// ```
pub struct FenceBesideTheClone {
    /// The thread the fence runs on,
    /// while it runs.
    handle: Option<JoinHandle<Result<Fence, String>>>,
    /// Why no thread was started,
    /// when none was.
    refusal: Option<String>,
}

impl FenceBesideTheClone {
    /// What:
    ///  Start the fence's reads on a thread of their own.
    /// Why:
    ///   The reads take no lock and change nothing,
    /// so they may run beside the copy of the same files;
    /// a thread the system refuses is a reason the rung cannot be proven,
    ///  never an add without the fence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const started = startFenceBesideTheClone(source, context);
    /// ```
    pub fn start(source: &SnapshotSource, context: &GitContext) -> FenceBesideTheClone {
        let held: SnapshotSource = source.clone();
        let environment: GitContext = context.clone();
        let spawned = std::thread::Builder::new().name(THREAD.to_string()).spawn(move || {
            let began: std::time::Instant = begin_work();
            let outcome: Result<Fence, String> = fence_the_clone(&held, &environment);
            end_work(WORK, began);
            return outcome;
        });
        return match spawned {
            Ok(handle) => FenceBesideTheClone { handle: Some(handle), refusal: None },
            Err(error) => FenceBesideTheClone { handle: None, refusal: Some(format!("the fence's thread could not be started ({error})")) },
        };
    }

    /// What:
    ///  Wait for the fence and give its answer.
    /// Why:
    ///   The join is what says the thread no longer reads the source,
    /// and a thread that panicked proved nothing about any path,
    /// so its panic is
    ///  the rung's reason rather than a silence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fence = started.join(); // throws the reason the rung cannot be proven
    /// ```
    pub fn join(mut self) -> Result<Fence, String> {
        if let Some(refusal) = self.refusal.take() {
            return Err(refusal);
        }
        let Some(handle) = self.handle.take() else {
            return Err(String::from("the fence's thread was not started"));
        };
        return handle.join().unwrap_or_else(|_| return Err(String::from("the fence's own thread panicked, so nothing is proven about the source's paths")));
    }
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
