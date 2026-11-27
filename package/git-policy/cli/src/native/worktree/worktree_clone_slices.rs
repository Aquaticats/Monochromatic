//! What:
//!  The slices of the clone rung:
//! contiguous runs of the frozen entries,
//! one job of the blocking pool each,
//!  and the worker count the host's budget allows.
//! Why:
//!  Each entry is a few metadata calls and one `FICLONE`,
//! which the filesystem serializes at its own metadata anyway;
//! contiguous slices keep
//!  neighbouring entries in one slice,
//! so neighbouring files share a directory the way they do on one thread,
//! and the answers are read
//!  in slice order,
//! so the entries stay in the index's order.
//! The fan-out lives here so `worktree_clone.rs` stays under the line limit and the
//!  run that needs no second thread is in `worktree_clone_entries.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { stats, beside } = cloneInSlices({ source, entries, destination, created, workers, beside });
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
    note,
};
use crate::diagnostic_log::debug;
use crate::fs_beneath::open_root;
use crate::parallel_budget::{
    HostResources,
    PARALLEL_BREAK_EVEN_ENTRIES,
    sliced_workers,
};
use crate::side_by_side_on_the_runtime::{
    AJob,
    run_all_side_by_side_on_the_runtime,
};
use crate::subvolume_index::StatFields;
use crate::worktree_clean::StatEntry;
use crate::worktree_clone_entries::{
    ClonedEntries,
    EntriesToClone,
    clone_the_entries,
    clone_with_entries,
};
use crate::worktree_clone_entry::{
    Seams,
    do_nothing,
};
use crate::worktree_clone_paths::{
    Created,
    Directories,
};
use crate::worktree_copy_stage_leaf_copying::PARALLEL_COPY_MAX_WORKERS;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::SnapshotSource;
use std::collections::HashMap;
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};

/// Tag used for diagnostics from this module.
const TAG: &str = "cloneSlices";

/// What:
///  Whether a control asked the process's runtime to refuse the slices,
/// so the one-turn arm runs.
/// Why:
///   That arm runs only where no runtime can be entered,
/// which no argument of an add can ask for;
///  a control of the arm then has no shape to reach it
/// (the release pass over alpha.45's finding H2).
/// The flag is a control's route alone:
///  the release build has no such route,
///  so no shipped add can choose the arm.
#[cfg(test)]
pub(crate) static FORCED_ONE_TURN_ARM: AtomicBool = AtomicBool::new(false);

/// What:
///  A host whose processors and memory bound no clone:
/// eight threads for every worker the copy ceiling allows,
/// and no memory figure,
/// so a budget asked about it answers that ceiling and never the machine the control runs on.
/// Why:
///  A control that asked for the worker count of the live host asserted on whatever the machine had at that moment:
/// the threads `HostResources::read` answers are the smaller of the affinity mask and the control group's CPU quota,
/// so a run in a scope capped below two processors answered one worker and the control's own precondition failed
///  beside a fixture that was right (issue 6537).
/// Eight threads per allowed worker keeps the eighth the CPU margin takes back above the ceiling,
///  so a raised ceiling still leaves room for every worker it allows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_HOST_THAT_BOUNDS_NO_WORKER = { parallelism: PARALLEL_COPY_MAX_WORKERS * 8, memoryAvailable: undefined };
/// ```
#[cfg(test)]
pub(crate) const A_HOST_THAT_BOUNDS_NO_WORKER: HostResources = HostResources {
    parallelism: PARALLEL_COPY_MAX_WORKERS * 8,
    memory_available: None,
};

/// What:
///  Whether the slices are refused, which a control may ask for and no shipped add can.
/// Why:
///   The arm's shape is the caller's answer and not the runtime's here,
/// so a control names why the arm ran rather than reaching into the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = () => process.env.NODE_ENV === 'test' && forcedOneTurn;
/// ```
fn the_slices_are_refused_by_a_control() -> bool {
    #[cfg(test)]
    return FORCED_ONE_TURN_ARM.load(Ordering::SeqCst);
    #[cfg(not(test))]
    return false;
}

/// What one slice of the clones works on.
#[derive(Clone)]
pub(crate) struct SliceOfClones {
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
    let mut seams: Seams<'_> = Seams {
        after_lookup: &mut do_nothing,
        after_clone: &mut do_nothing,
        before_index: &mut do_nothing,
    };
    let cloned: ClonedEntries = clone_the_entries(EntriesToClone {
        entries: state.entries.as_ref(),
        positions: state.positions.as_slice(),
        source: state.source.as_ref(),
        source_root: state.source_root.as_ref(),
        destination: state.destination.as_ref(),
        seams: &mut seams,
        failed: state.failed.as_ref(),
    });
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
            Err(crate::blocking_job::BlockingJobError::Panicked) => {
                Err(String::from("a clone worker panicked"))
            }
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
/// The host is an input rather than a reading of this process,
/// so a caller states the processors and the memory its own budget is derived from,
/// and a control asserts on a host it names instead of on the machine the run is on (issue 6537).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloneWorkerCount = (entries, host) => entries < 16 ? 1 : slicedWorkers(entries, 8, host);
/// ```
pub(crate) fn clone_worker_count(
    entries: usize,
    host: &HostResources,
) -> usize {
    if entries < PARALLEL_BREAK_EVEN_ENTRIES {
        return 1;
    }
    return sliced_workers(entries, PARALLEL_COPY_MAX_WORKERS, host);
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
/// cloneInSlices({ source, entries, destination, created, workers }); // throws a reason
/// ```
pub(crate) fn clone_in_slices<S, T>(
    source: &SnapshotSource,
    entries: Arc<[StatEntry]>,
    destination: &File,
    created: &mut Vec<Created>,
    workers: usize,
    mut beside: Option<AJob<S, T>>,
) -> Result<(HashMap<Vec<u8>, StatFields>, Option<T>), String>
where
    S: Clone + Send + 'static,
    T: Send + 'static,
{
    let source_root: Arc<File> = Arc::new(subject_failure(
        open_root(source.root.as_path()),
        &"the source's root",
        "cannot be opened",
    )?);
    let destination_held: Arc<File> = Arc::new(subject_failure(
        destination.try_clone(),
        &"the destination handle",
        "cannot be kept",
    )?);
    let shared_source: Arc<SnapshotSource> = Arc::new(source.clone());
    let failed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let slice_length: usize = entries.len().div_ceil(workers).max(1);
    let mut states: Vec<SliceOfClones> = Vec::new();
    let mut at: usize = 0;
    for chunk in entries.chunks(slice_length) {
        states.push(SliceOfClones {
            entries: Arc::clone(&entries),
            positions: (at..at + chunk.len()).collect(),
            source: Arc::clone(&shared_source),
            source_root: Arc::clone(&source_root),
            destination: Arc::clone(&destination_held),
            failed: Arc::clone(&failed),
        });
        at += chunk.len();
    }
    let slices: usize = states.len();
    note(
        "cloneSlices",
        format!(
            "{} entries in {} slices, up to {} workers",
            entries.len(),
            slices,
            workers
        )
        .as_str(),
    );
    // What:
    //  The copies start as one task of the process's runtime, and this thread leaves the runtime before the work that
    //  must run outside it.
    // Why:
    //  The synchronous Git face enters the runtime around the pipes it reads, so it is refused from inside it
    //  (`RuntimeError::Nested`); the fence's Git runs are that face. The task awaits the slices' jobs and does no
    //  blocking work itself, and the wait for it is taken on this thread once the work beside it has answered.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const started = runtime.spawn(cloneTheSlices(states));
    // const beside = besideJob ? besideJob() : undefined;
    // const answers = await runtime.wait(started);
    // ```
    let started: Result<
        tokio::task::JoinHandle<Vec<Result<ClonedEntries, String>>>,
        crate::lazy_runtime::RuntimeError,
    > = match the_slices_are_refused_by_a_control() {
        true => Err(crate::lazy_runtime::RuntimeError::Nested),
        false => crate::lazy_runtime::run_on_runtime(async move {
            return tokio::spawn(clone_the_slices(states));
        }),
    };
    let beside_answer: Option<T> = beside.take().map(|(state, work)| return work(state));
    let entries_entered: bool = begin_step("cloneEntries");
    let ran = match started {
        Ok(handle) => crate::lazy_runtime::run_on_runtime(handle)
            .map_err(|refusal| return refusal.to_string())
            .and_then(|joined| {
                return joined.map_err(|error| {
                    return format!("a clone task ended without an answer ({error})");
                });
            }),
        Err(refusal) => Err(refusal.to_string()),
    };
    end_step(entries_entered);
    let answers: Vec<Result<ClonedEntries, String>> = match ran {
        Ok(answers) => answers,
        Err(refusal) => {
            debug(
                TAG,
                format!("the clones run in one turn ({refusal})").as_str(),
            );
            let mut directories: Directories = HashMap::new();
            directories.insert(
                Vec::new(),
                subject_failure(
                    destination.try_clone(),
                    &"the destination handle",
                    "cannot be kept",
                )?,
            );
            let mut seams: Seams<'_> = Seams {
                after_lookup: &mut do_nothing,
                after_clone: &mut do_nothing,
                before_index: &mut do_nothing,
            };
            // What:
            //  The one-turn arm clones the entries `clone_in_slices` was handed, not a fresh read of the source's
            //  index.
            // Why:
            //  The caller chose the entry set:
            //  a partial clone of a source with changes hands over the paths its proof settled, so a fresh read of the
            //  index would clone every index path instead, staged additions included, and a staged addition is in no
            //  tree (the release pass over alpha.45's finding H2).
            let stats: HashMap<Vec<u8>, StatFields> = clone_with_entries(
                source,
                entries.as_ref(),
                &mut directories,
                created,
                &mut seams,
            )?;
            return Ok((stats, beside_answer));
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
    return Ok((stats, beside_answer));
}
