//! What:
//!  The run of a clone rung:
//!  every entry the frozen index lists,
//!  in contiguous slices across the host's budget,
//!  each entry made by `worktree_clone_entry`,
//!  and then the source's own index installed with the stat fields of the files made.
//! Why:
//!  An entry is a few metadata calls and one `FICLONE`,
//!  which the filesystem serializes at its own metadata anyway,
//!  so the run is split into slices whose answers are read in slice order;
//!  a single-threaded run and one slice do the same work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await cloneAll(source, destination, admin, created); // throws a reason
//! ```

use crate::add_timing::{begin_step, end_step, note};
use crate::diagnostic_log::debug;
use crate::fs_beneath::open_root;
use crate::parallel_budget::{HostResources, PARALLEL_BREAK_EVEN_ENTRIES, sliced_workers};
use crate::side_by_side_on_the_runtime::run_all_side_by_side_on_the_runtime;
use crate::subvolume_index::{StatFields, StatLookup};
use crate::subvolume_index_files::install_index_in;
use crate::worktree_clean::{StatEntry, stored_stats};
use crate::worktree_clone_entry::{ClonedEntries, EntriesToClone, Seams, clone_the_entries};
use crate::worktree_clone_paths::{Created, Directories};
use crate::worktree_copy_stage_leaf_copying::PARALLEL_COPY_MAX_WORKERS;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::HashMap;
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Tag used for diagnostics from this module.
const TAG: &str = "cloneTrackedFiles";

/// What:
///  The stat fields the clones recorded,
/// looked up by path for the index the new worktree takes.
/// Why:
///   Each clone recorded the fields of the file it made,
/// and the index install asks for them by the entry's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StatsMadeByTheClones implements StatLookup { statNow(path) { return this.stats.get(path); } }
/// ```
struct StatsMadeByTheClones<'a> {
    /// The fields by path.
    stats: &'a HashMap<Vec<u8>, StatFields>,
}

impl StatLookup for StatsMadeByTheClones<'_> {
    fn stat_now(&mut self, path: &[u8]) -> Option<StatFields> {
        return self.stats.get(path).copied();
    }
}

/// `clone_all` in one turn,
/// with the moments at which a control may move the tree.
pub(crate) fn clone_with(source: &SnapshotSource, directories: &mut Directories, admin: &File, created: &mut Vec<Created>, seams: &mut Seams<'_>) -> Result<(), String> {
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    let root: File = subject_failure(open_root(source.root.as_path()), &"the source's root", "cannot be opened")?;
    let Some(destination) = directories.get(&Vec::new()) else {
        return Err(String::from("the destination is not held"));
    };
    let positions: Vec<usize> = (0..entries.len()).collect();
    let unshared: AtomicBool = AtomicBool::new(false);
    let entries_entered: bool = begin_step("cloneEntries");
    let cloned: ClonedEntries = clone_the_entries(EntriesToClone { entries: entries.as_slice(), positions: positions.as_slice(), source, source_root: &root, destination, seams, failed: &unshared });
    end_step(entries_entered);
    let ClonedEntries { created: made, stats, failure }: ClonedEntries = cloned;
    created.extend(made);
    if let Some(reason) = failure {
        return Err(reason);
    }
    (seams.before_index)();
    let index_entered: bool = begin_step("cloneIndex");
    let installed: Result<String, String> = install_index_in(admin, source.clean.index_bytes.as_slice(), &mut StatsMadeByTheClones { stats: &stats }, source.clean.hash_length);
    end_step(index_entered);
    let report: String = installed?;
    debug_report(report.as_str());
    return Ok(());
}

/// What one slice of the clones works on.
#[derive(Clone)]
struct SliceOfClones {
    /// The frozen entries,
    /// whole.
    entries: Arc<[StatEntry]>,
    /// The entries this slice owns,
    /// as indexes into `entries`.
    positions: Vec<usize>,
    /// What the checks proved about the source.
    source: Arc<SnapshotSource>,
    /// The held root of the source's files.
    source_root: Arc<File>,
    /// The held root the destinations are opened below.
    destination: Arc<File>,
    /// Raised by the first failure,
    /// so the other slices stop at their next entry.
    failed: Arc<AtomicBool>,
}

/// What:
///  Clone one slice as a job of the blocking pool.
/// Why:
///   A slice owns its entries and its directory handles,
/// so its answer is its own and the flag is all it shares
///  with the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneASlice(state) { const cloned = cloneTheEntries(state); if (cloned.failure) state.failed.store(true); return cloned; }
/// ```
fn clone_a_slice(state: SliceOfClones) -> ClonedEntries {
    let mut seams: Seams<'_> = Seams { after_lookup: &mut do_nothing, after_clone: &mut do_nothing, before_index: &mut do_nothing };
    let cloned: ClonedEntries = clone_the_entries(EntriesToClone { entries: state.entries.as_ref(), positions: state.positions.as_slice(), source: state.source.as_ref(), source_root: state.source_root.as_ref(), destination: state.destination.as_ref(), seams: &mut seams, failed: state.failed.as_ref() });
    if cloned.failure.is_some() {
        state.failed.store(true, Ordering::Relaxed);
    }
    return cloned;
}

/// What:
///  `clone_a_slice` for every slice on the process's runtime,
/// in slice order.
/// Why:
///   A job that panicked or that the pool refused answered nothing about its entries,
/// and the caller must hear that
///  as a failure like any other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = await Promise.allSettled(states.map(cloneASlice));
/// ```
async fn clone_the_slices(states: Vec<SliceOfClones>) -> Vec<Result<ClonedEntries, String>> {
    let mut answers: Vec<Result<ClonedEntries, String>> = Vec::new();
    for answer in run_all_side_by_side_on_the_runtime(states, clone_a_slice).await {
        answers.push(match answer {
            Ok(cloned) => Ok(cloned),
            Err(crate::blocking_job::BlockingJobError::Panicked) => Err(String::from("a clone worker panicked")),
            Err(refused) => Err(format!("a clone worker did not run ({refused})")),
        });
    }
    return answers;
}

/// What:
///  How many threads the clones of this many entries use:
/// one below the break-even the copy stage and `clone_tree` were measured at,
/// else the host's
///  budget under the ceiling the filesystem was measured to need.
/// Why:
///   Making files by cloning is the copy stage's own load on the filesystem's metadata locks,
/// so the ceiling
///  measured for that stage is the ceiling here.
/// Both limits are borrowed:
/// they were measured on btrfs reflinked trees of this repository,
/// for copies of whole
///  files,
/// the install and `clone_tree`,
/// and never for this rung,
/// not on XFS,
/// overlayfs or ZFS,
/// and not between the break-even this rung uses and the entry counts those
///  measurements took,
/// which is where a small repository sits;
/// the borrow is stated here,
/// and
///  measuring this rung at a few hundred entries is owed (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
/// step 3's open items).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloneWorkerCount = (entries) => entries < 16 ? 1 : slicedWorkers(entries, 8, host);
/// ```
pub(crate) fn clone_worker_count(entries: usize) -> usize {
    if entries < PARALLEL_BREAK_EVEN_ENTRIES {
        return 1;
    }
    return sliced_workers(entries, PARALLEL_COPY_MAX_WORKERS, &HostResources::read());
}

/// What:
///  Clone the frozen entries in contiguous slices,
/// one job of the blocking pool each,
/// and install the index with the stat
///  fields of every slice.
/// Why:
///   Each entry is a few metadata calls and one `FICLONE`,
/// which the filesystem serializes at its own metadata anyway;
///  contiguous slices keep neighbouring entries in one slice,
/// so neighbouring files share a directory the way they do on one thread,
///  and the answers are read in slice order,
/// so the entries stay in the index's order.
/// The slices clone in one turn when no runtime can be entered here,
/// which is the same work on one thread.
/// The step around the fan-out is entered on this thread,
/// so the record holds the wait for the slices,
/// and a note names the entries,
/// the slices and the workers the wait was split across,
/// because neither is visible in a record that only sums the work per slice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cloneInSlices({ source, entries, destination, admin, created, workers }); // throws a reason
/// ```
pub(crate) fn clone_in_slices(source: &SnapshotSource, entries: Arc<[StatEntry]>, destination: &File, admin: &File, created: &mut Vec<Created>, workers: usize) -> Result<(), String> {
    let source_root: Arc<File> = Arc::new(subject_failure(open_root(source.root.as_path()), &"the source's root", "cannot be opened")?);
    let destination_held: Arc<File> = Arc::new(subject_failure(destination.try_clone(), &"the destination handle", "cannot be kept")?);
    let shared_source: Arc<SnapshotSource> = Arc::new(source.clone());
    let failed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let slice_length: usize = entries.len().div_ceil(workers).max(1);
    let mut states: Vec<SliceOfClones> = Vec::new();
    let mut at: usize = 0;
    for chunk in entries.chunks(slice_length) {
        states.push(SliceOfClones { entries: Arc::clone(&entries), positions: (at..at + chunk.len()).collect(), source: Arc::clone(&shared_source), source_root: Arc::clone(&source_root), destination: Arc::clone(&destination_held), failed: Arc::clone(&failed) });
        at += chunk.len();
    }
    let slices: usize = states.len();
    note("cloneSlices", format!("{} entries in {} slices, up to {} workers", entries.len(), slices, workers).as_str());
    let entries_entered: bool = begin_step("cloneEntries");
    let ran = crate::lazy_runtime::run_on_runtime(clone_the_slices(states));
    end_step(entries_entered);
    let answers: Vec<Result<ClonedEntries, String>> = match ran {
        Ok(answers) => answers,
        Err(refusal) => {
            debug(TAG, format!("the clones run in one turn ({refusal})").as_str());
            let mut directories: Directories = HashMap::new();
            directories.insert(Vec::new(), subject_failure(destination.try_clone(), &"the destination handle", "cannot be kept")?);
            let mut seams: Seams<'_> = Seams { after_lookup: &mut do_nothing, after_clone: &mut do_nothing, before_index: &mut do_nothing };
            return clone_with(source, &mut directories, admin, created, &mut seams);
        }
    };
    let mut stats: HashMap<Vec<u8>, StatFields> = HashMap::new();
    let mut failure: Option<String> = None;
    for answer in answers {
        match answer {
            Ok(cloned) => {
                created.extend(cloned.created);
                stats.extend(cloned.stats);
                if failure.is_none() {
                    failure = cloned.failure;
                }
            }
            Err(reason) => {
                if failure.is_none() {
                    failure = Some(reason);
                }
            }
        }
    }
    if let Some(reason) = failure {
        return Err(reason);
    }
    let index_entered: bool = begin_step("cloneIndex");
    let installed: Result<String, String> = install_index_in(admin, source.clean.index_bytes.as_slice(), &mut StatsMadeByTheClones { stats: &stats }, source.clean.hash_length);
    end_step(index_entered);
    let report: String = installed?;
    debug_report(report.as_str());
    return Ok(());
}

/// The run itself:
/// every tracked file in the slices the host's budget allows,
/// then the index;
/// the caller removes `created` when this fails.
pub(crate) fn clone_all(source: &SnapshotSource, destination: &File, admin: &File, created: &mut Vec<Created>) -> Result<(), String> {
    let Some(entries) = stored_stats(source.clean.index_bytes.as_slice(), source.clean.hash_length) else {
        return Err(String::from("the source's index changed shape"));
    };
    let workers: usize = clone_worker_count(entries.len());
    if workers > 1 {
        return clone_in_slices(source, Arc::from(entries), destination, admin, created, workers);
    }
    let mut directories: Directories = HashMap::new();
    directories.insert(Vec::new(), subject_failure(destination.try_clone(), &"the destination handle", "cannot be kept")?);
    let mut seams: Seams<'_> = Seams { after_lookup: &mut do_nothing, after_clone: &mut do_nothing, before_index: &mut do_nothing };
    return clone_with(source, &mut directories, admin, created, &mut seams);
}

/// Do nothing;
/// the seams of the clones run this unless a control puts work there.
fn do_nothing() {}

/// Log what the index install did.
fn debug_report(report: &str) {
    debug(TAG, report);
}
