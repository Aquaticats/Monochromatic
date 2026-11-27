//! What:
//!  The walk of an index's entries:
//!  one contiguous slice at a time,
//!  on several threads where the index is large enough for them to pay,
//!  and the answer it gives.
//! Why:
//!  Git's own stat pass preloads the same work across threads (`core.preloadIndex`),
//!  and one thread of this walk took 47 to 58 ms on a checkout of 22,161 entries against Git's 10 ms;
//!  over 22,200 files a walk of one thread took 160.12 ms and seven took 30.56 ms.
//!  The paths the walk answers with are the whole of what a refresh has to look at,
//!  so a clean slice costs the same walk the yes-or-no question cost.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stale = walkInSlices(entries, worktreeRoot, restaged, workersFor(entries.length));
//! ```

use crate::commit_index_stat_staleness_left_to_git::{
    is_left_to_git,
    is_restaged,
};
use crate::commit_index_stat_staleness_worktree_file::{
    matches_recorded,
    worktree_file,
};
/// An entry of the whole index,
///  and what one slice of the walk needs.
use crate::index_full::FullEntry;
/// The threads an independent set of items may use,
///  as the neighbouring file-tree jobs derive them.
use crate::parallel_budget::{
    BudgetRequest,
    HostResources,
    SLICE_MEMORY,
    worker_budget,
};
use crate::side_by_side_threads::run_all_side_by_side;
/// `OsStr` borrows operating-system text without copying it,
/// and `OsStrExt` is the trait whose `from_bytes` names one index path.
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
/// `Path` is a borrowed filesystem path;
/// `PathBuf` an owned one.
use std::path::{
    Path,
    PathBuf,
};

/// 32 entries.
/// Why:
///  A stat walk of one entry costs a few microseconds,
/// so a short walk pays less than the threads it would start.
///  Measured on a throwaway repository of 22,200 committed files on this host
/// (8 CPUs,
/// other sessions running,
/// the release build of 2026-11-08,
/// 25 samples of each walk in one run,
/// medians,
/// warm cache):
/// 16 entries took 0.125 ms on one thread and 0.145 ms on seven,
/// so the threads lost there;
/// 32 took 0.218 ms against 0.158 ms,
/// 64 0.391 ms against 0.193 ms,
/// 128 0.786 ms against 0.289 ms,
/// 256 1.651 ms against 0.479 ms,
/// 512 3.223 ms against 0.858 ms,
/// 2,048 13.335 ms against 2.912 ms,
/// and all 22,200 took 160.12 ms against 30.56 ms on the seven threads the host's budget gives.
/// The neighbouring file-tree jobs measured their own break-even at 16 entries
/// (`parallel_budget.rs`,
/// `PARALLEL_BREAK_EVEN_ENTRIES`);
/// a stat walk's entry is cheaper than theirs,
/// so its break-even is higher.
#[cfg(unix)]
pub(crate) const STAT_WALK_BREAK_EVEN_ENTRIES: usize = 32;

/// What:
///  One contiguous slice of the entries the question walks,
/// with what the walk of every entry needs.
/// Why:
///   A worker borrows its slice of the caller's entries without copying them,
/// and the slices together are the one linear walk
///  over the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntrySlice = { entries: FullEntry[]; worktreeRoot: string; restaged: Uint8Array[] };
/// ```
#[cfg(unix)]
pub(crate) struct EntrySlice<'a> {
    /// The entries this walk judges,
    /// in index order.
    pub(crate) entries: &'a [FullEntry],
    /// The worktree's top level,
    /// which index paths are relative to.
    pub(crate) worktree_root: &'a Path,
    /// The top-relative paths the commit stages from the worktree afresh.
    pub(crate) restaged: &'a [Vec<u8>],
}

/// What:
///  What the walk of an index found about the stat data its entries record.
/// Why:
///   The answer is the reason a refresh of the private index is started,
/// or is not,
/// and the paths it names are the ones a refresh has to look at:
/// a refresh that names them costs a question per stale path instead of one per entry of the index
/// (`commit_index_stat_refresh.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StaleStatData = { kind: 'current' } | { kind: 'stale'; paths: Uint8Array[]; entries: number } | { kind: 'unknown' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaleStatData {
    /// Every entry the commit leaves alone records the stat data its file has.
    Current,
    /// These entries,
    /// by their top-relative index path,
    /// record stat data their files no longer have.
    Stale {
        /// The paths the walk could not prove,
        /// in index order.
        paths: Vec<Vec<u8>>,
        /// How many entries the index holds,
        /// which says whether naming the paths is cheaper than naming the index
        /// (`REFRESH_NAMES_ONE_PATH_IN` in `commit_index_stat_refresh.rs`).
        entries: usize,
    },
    /// This reading cannot answer,
    /// so Git's own refresh of the whole index has to.
    Unknown,
}

/// What:
///  Why one entry is one the walk judges,
/// or what keeps it out of the walk altogether.
/// Why:
///   The walk asks about every entry the commit leaves alone,
/// because an entry it skips cannot starve the refresh of a path Git would have needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judged = (entry) => !isLeftToGit(entry) && !isRestaged(restaged, entry.path);
/// ```
#[cfg(unix)]
fn judged(
    entry: &FullEntry,
    restaged: &[Vec<u8>],
) -> bool {
    return !is_left_to_git(entry) && !is_restaged(restaged, entry.path.as_slice());
}

/// What:
///  The paths of `slice` whose entries record stat data their files no longer have.
/// Why:
///   Every entry the walk judges is asked about,
/// so the paths it answers with are the whole of what a refresh has to look at,
///  and the answer for a clean slice costs the same walk the yes-or-no question cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stalePaths = (slice) => slice.entries.filter((entry) => judged(entry) && !matchesRecorded(entry, worktreeFile(join(slice.worktreeRoot, entry.path)) ?? stale)).map((entry) => entry.path);
/// ```
#[cfg(unix)]
pub(crate) fn stale_paths_in_slice(slice: EntrySlice<'_>) -> Vec<Vec<u8>> {
    let mut stale: Vec<Vec<u8>> = Vec::new();
    for entry in slice.entries.iter() {
        // What:
        //  Whether the commit leaves this entry's recorded stat data to this walk at all.
        // Why:
        //  An entry left to Git or staged afresh costs the scratch commit nothing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!judged(entry, slice.restaged)) continue;
        // ```
        if !judged(entry, slice.restaged) {
            continue;
        }
        // What:
        //  The file the entry names, inside the worktree.
        // Why:
        //  Its stat data is the answer to the question for this entry.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const path = join(slice.worktreeRoot, entry.path);
        // ```
        let path: PathBuf = slice
            .worktree_root
            .join(OsStr::from_bytes(entry.path.as_slice()));
        // What:
        //  The stat data that file has now.
        // Why:
        //  A file this cannot read is one Git has to judge.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const file = worktreeFile(path);
        // ```
        let Some(file) = worktree_file(path.as_path()) else {
            stale.push(entry.path.clone());
            continue;
        };
        if !matches_recorded(entry, &file) {
            stale.push(entry.path.clone());
        }
    }
    return stale;
}

/// What:
///  The threads the walk uses for `entries` entries:
/// one below `STAT_WALK_BREAK_EVEN_ENTRIES`,
/// the host's budget above it.
/// Why:
///  Below the break-even measured for this walk,
/// creating the threads costs more than the walk they share,
/// and the budget
///  leaves the CPU margin of the host free and honours a container's quota.
/// No ceiling bounds it:
/// no measurement found the stat calls flattening at a number of threads the
///  budget gives.
///  Each worker copies nothing,
/// so `SLICE_MEMORY`,
/// the conservative per-worker bound of the file-tree jobs,
/// is an upper bound here too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const workers = entries < STAT_WALK_BREAK_EVEN_ENTRIES ? 1 : workerBudget({ items: entries, perItemMemory: SLICE_MEMORY, ceiling: undefined, cap: undefined, host: HostResources.read() });
/// ```
#[cfg(unix)]
pub(crate) fn workers_for(entries: usize) -> usize {
    if entries < STAT_WALK_BREAK_EVEN_ENTRIES {
        return 1;
    }
    return worker_budget(&BudgetRequest {
        items: entries,
        per_item_memory: SLICE_MEMORY,
        ceiling: None,
        cap: None,
        host: &HostResources::read(),
    });
}

/// What:
///  The paths `slice`'s entries hold that need the refresh,
/// the slices walked side by side on `workers` threads,
/// or nothing when a worker could not answer.
/// Why:
///   A worker that panicked judged no entry,
/// and an entry nothing judged is one Git's refresh decides,
/// so the answer is
///  the whole index there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stale = (await Promise.allSettled(slices.map((slice) => walk(slice)))).every(ok) ? merged : undefined;
/// ```
#[cfg(unix)]
pub(crate) fn walk_in_slices(
    slice: EntrySlice<'_>,
    workers: usize,
) -> Option<Vec<Vec<u8>>> {
    let size: usize = slice.entries.len().div_ceil(workers.max(1)).max(1);
    let mut slices: Vec<EntrySlice<'_>> = Vec::new();
    for entries in slice.entries.chunks(size) {
        slices.push(EntrySlice {
            entries,
            worktree_root: slice.worktree_root,
            restaged: slice.restaged,
        });
    }
    let walked: Vec<std::thread::Result<Vec<Vec<u8>>>> =
        run_all_side_by_side(slices, stale_paths_in_slice);
    let mut stale: Vec<Vec<u8>> = Vec::new();
    for outcome in walked {
        match outcome {
            Ok(paths) => stale.extend(paths),
            Err(_) => return None,
        }
    }
    return Some(stale);
}
