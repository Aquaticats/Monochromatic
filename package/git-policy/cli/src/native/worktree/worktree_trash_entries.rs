//! What:
//!  The removal of the entries a snapshot's leave-out moved into its trash,
//! side by side.
//! Why:
//!  The entries are separate directories that share only the held trash handle,
//! each opened by its own name and removed only
//!  when it is the recorded object,
//! so they are independent items:
//! a reason for one entry never stops the others.
//! They run
//!  within the budget the host allows (`parallel_budget.rs`),
//! which inside the commit service's control group is the two
//!  threads and the memory that unit is given,
//! and the reasons come back in entry order (`parallel_runner.rs`).
//! Each removal is a job of the process's blocking pool,
//! awaited by a task of its runtime
//!  (`parallel_runner_on_the_runtime_awaiting.rs`),
//! so the process's one bound on blocking jobs covers it and no
//!  thread of a runtime's own is held while a directory is emptied.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await removeEntries({ inside, trash, manifest, host, remove }); // one reason per entry that stays
//! ```

use crate::blocking_jobs_within_the_budget::{places_a_round_may_take, run_blocking_job_within_the_budget};
use crate::lazy_runtime::{RuntimeError, run_on_runtime};
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::parallel_runner::{Event, OrderedEvents};
use crate::parallel_runner_on_the_runtime::LinesToTheConsumer;
use crate::parallel_runner_on_the_runtime_awaiting::{OrderedAwaitingWork, OrderedRunOfAwaitingOwned, run_ordered_awaiting_on_the_runtime};
use crate::worktree_trash_manifest::{TrashManifest, entry_name};
use crate::worktree_trash_remove::Removal;
use std::ffi::OsStr;
use std::fs::File;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

/// The peak resident growth of removing one flat directory of a million names (`remove_tree_beneath`,
/// names of 42 characters,
///  a fresh process,
/// a throwaway directory on btrfs,
/// one run):
/// 97,320 KiB,
/// which is 99,655,680 bytes.
/// Runs of 100,000,
/// 200,000
///  (twice) and 400,000 names grew by 131,
/// 113,
/// 113 and 104 bytes a name,
/// so the figure per name falls as the count rises and
///  the one-million run is the case the estimate is for.
const MEASURED_MILLION_NAME_PEAK: u64 = 99_655_680;

/// The bytes the removal of one trash entry holds at its peak:
/// `MEASURED_MILLION_NAME_PEAK` rounded up to a power of two.
/// The
///  listing of a directory is held whole while its entries are removed.
/// The coordinator's pick under the owner's delegation,
///  open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Parallelism caps and memory estimates").
const ENTRY_MEMORY: u64 = MEASURED_MILLION_NAME_PEAK.next_power_of_two();

/// The removal of one entry beneath the trash handle,
/// replaceable so a control can watch or delay it.
///  It is shared rather than lent because a job of the blocking pool must own what it reads,
/// and it must be `Send` so the job that runs it may travel to another thread.
pub type RemoveOne = dyn Fn(&Path, &File, &OsStr, Option<crate::fs_beneath::ObjectId>) -> Result<crate::worktree_trash_remove::Removal, String> + Send + Sync;

/// What:
///  What the removal of a trash's entries is told.
/// Why:
///   One value names the inputs:
/// where the trash is,
/// its manifest,
/// the host the budget is derived from,
/// and the removal of one entry.
///  Every field is owned so the value can outlive the call:
/// the items of the run are jobs of the blocking pool,
/// which must be `'static`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryRemoval = { inside: string; trash: Dir; manifest: TrashManifest; host: HostResources; remove: RemoveOne };
/// ```
pub struct EntryRemoval {
    /// The path of the trash directory,
    /// for messages.
    pub inside: PathBuf,
    /// The trash directory,
    /// held open;
    /// every entry is opened by name below it.
    pub trash: Arc<File>,
    /// What the leave-out recorded.
    pub manifest: Arc<TrashManifest>,
    /// The CPUs and memory the budget is derived from.
    pub host: HostResources,
    /// The removal of one entry.
    pub remove: Arc<RemoveOne>,
}

/// What:
///  The removal of one trash entry,
/// run by a task of the awaited ordered runner.
/// Why:
///   The trash handle,
/// the manifest and the removal are what the removal reads;
/// the runner shares them with its tasks,
///  and each job takes its own share of them before it goes to the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RemoveEachEntry implements OrderedAwaitingWork<number, Result<Removal>> { async run(slot, index) { return await runBlockingJobWithinTheBudget(() => remove(inside, trash, entryName(index), manifest.entries[index])); } }
/// ```
struct RemoveEachEntry {
    /// The path of the trash directory,
    /// for messages.
    inside: PathBuf,
    /// The trash directory,
    /// held open;
    /// every entry is opened by name below it.
    trash: Arc<File>,
    /// What the leave-out recorded.
    manifest: Arc<TrashManifest>,
    /// The removal of one entry.
    remove: Arc<RemoveOne>,
}

/// What:
///  Remove the entry at `entry` of the trash `inside`,
/// found by the name and identity the manifest recorded,
/// and at the lowest
///  I/O priority Linux offers.
/// Why:
///   One body removes one entry,
/// so the job of the pool,
/// the awaited face and the in-turn fallback cannot remove
///  different things or report a different reason.
/// The reason names the entry,
/// because the list keeps no index.
/// The
///  priority is lowered here,
/// on the thread that removes,
/// rather than on the whole service,
/// which pushes a branch and reads the
///  repository on its other threads meanwhile;
/// every `unlink` of a trash starts a btrfs transaction,
/// and hundreds of
///  thousands of them made ten concurrent adds spend more than ten minutes each on 2026-11-10
///  (`doc/planning/cli-git-worktree-add-cost-of-its-trash.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = withIdleIoPriority(() => remove(inside, trash, entryName(entry), manifest.entries[entry]));
/// ```
fn remove_one(inside: &Path, trash: &File, manifest: &TrashManifest, remove: &RemoveOne, entry: usize) -> Result<Removal, String> {
    // The reason the removal gives is the person's, word for word: the entry's name is in it already, and a prefix would say it
    //  twice.
    return crate::idle_io_priority::with_idle_io_priority(|| return remove(inside, trash, OsStr::new(entry_name(entry).as_str()), Some(manifest.entries[entry])));
}

impl OrderedAwaitingWork<usize, Result<Removal, String>> for RemoveEachEntry {
    fn run<'a>(&'a self, _index: usize, index: &'a usize, _say: LinesToTheConsumer<Result<Removal, String>>) -> Pin<Box<dyn Future<Output = Result<Removal, String>> + Send + 'a>> {
        return Box::pin(async move {
            let inside: PathBuf = self.inside.clone();
            let trash: Arc<File> = Arc::clone(&self.trash);
            let manifest: Arc<TrashManifest> = Arc::clone(&self.manifest);
            let remove: Arc<RemoveOne> = Arc::clone(&self.remove);
            let entry: usize = *index;
            let removal = run_blocking_job_within_the_budget(move || {
                return remove_one(inside.as_path(), trash.as_ref(), manifest.as_ref(), remove.as_ref(), entry);
            })
            .await;
            return match removal {
                Ok(outcome) => outcome,
                // Which entry this was is what makes the reason usable, since the list keeps no index.
                Err(refused) => Err(format!("{}: {refused}", entry_name(entry))),
            };
        });
    }
}

/// What:
///  Keeps the reason of each entry that stays,
/// in entry order.
/// Why:
///   The runner reports on the calling task only and in input order,
/// so the list needs no lock and keeps entry order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class KeepReasonsOfEntriesThatStay implements OrderedEvents<Result<Removal>> { left: string[] }
/// ```
struct KeepReasonsOfEntriesThatStay<'a> {
    /// The reasons so far.
    left: &'a mut Vec<String>,
}

impl OrderedEvents<Result<Removal, String>> for KeepReasonsOfEntriesThatStay<'_> {
    fn on_event(&mut self, event: Event<'_, Result<Removal, String>>) {
        if let Event::End(_slot, Err(reason)) = event {
            self.left.push(reason);
        }
    }
}

/// What:
///  Remove every entry the manifest records,
/// side by side,
/// and return the reason each entry that stays gives,
/// in entry order.
/// Why:
///   The entries are separate directories moved into the trash,
/// each removed only when it is the recorded object,
/// and share no
///  state but the held trash handle,
/// which opens each by its own name.
/// A reason for one never stops the others.
/// The callers of this face are synchronous (the copy path of a worktree and the service's
/// handler),
///  so it enters the process's runtime once and waits there;
/// from inside the runtime it refuses,
///  and `remove_entries_awaiting` is the face such a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await removeEntries({ inside, trash, manifest, host, remove }); // one reason per entry that stays
/// ```
pub fn remove_entries(request: &EntryRemoval) -> Vec<String> {
    return match run_on_runtime(remove_entries_awaiting(request)) {
        Ok(left) => left,
        // A caller that is already inside the runtime cannot enter it, and its work is not lost: the entries are removed here,
        //  one after another, which is the work the pool would have done.
        Err(RuntimeError::Nested) => remove_entries_in_turn(request),
        // No entry was removed, and the runtime's own reason is why they all stay.
        Err(refusal) => vec![refusal.to_string()],
    };
}

/// What:
///  Remove every entry the manifest records on the calling thread,
/// one after another,
/// and return the reason each entry that stays gives,
/// in entry order.
/// Why:
///   This is the face of a caller that is already inside the runtime,
/// where entering it is refused and no
///  job of the blocking pool is available either (a job of the pool is inside the runtime too):
/// a handler that knows
///  nothing of the runtime,
/// written against the synchronous face,
/// still empties its trash.
/// The removals are the pool's own,
/// so a trash emptied here holds exactly what a trash emptied there holds;
/// what
///  differs is only that they run one after another rather than side by side,
/// which the caller that takes this path pays for and the
///  drain of the service never does (it asks `empty_trash_awaiting`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of manifest.entries.keys()) { const reason = removeOne(...); if (reason) left.push(reason) }
/// ```
fn remove_entries_in_turn(request: &EntryRemoval) -> Vec<String> {
    let mut left: Vec<String> = Vec::new();
    for entry in 0..request.manifest.entries.len() {
        if let Err(reason) = remove_one(request.inside.as_path(), request.trash.as_ref(), request.manifest.as_ref(), request.remove.as_ref(), entry) {
            left.push(reason);
        }
    }
    return left;
}

/// What:
///  `remove_entries` on the runtime:
/// every entry is a task of the awaited ordered runner,
/// and each removal is one job of the blocking pool inside its task.
/// Why:
///   A caller that is itself asynchronous (the service's work loop when it lands on the runtime)
/// cannot enter the runtime,
///  so it awaits this face;
/// the lines and the reasons are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeEntriesAwaiting(request: EntryRemoval): Promise<string[]>;
/// ```
pub async fn remove_entries_awaiting(request: &EntryRemoval) -> Vec<String> {
    let indices: Arc<[usize]> = Arc::from((0..request.manifest.entries.len()).collect::<Vec<usize>>());
    // One lane per entry, so no entry waits for another.
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    for index in indices.iter() {
        lanes.push(vec![*index]);
    }
    // The entries of one trash are swept for the service's after-commit round, which runs beside the connections that answer
    //  clients: one place of the pool stays free for those, so a client's own storage read (which instance its commit belongs
    //  to, the note of a beat) never waits for a trash, however many files the snapshot left out
    //  (`places_a_round_may_take`).
    let workers: usize = worker_budget(&BudgetRequest { items: indices.len(), per_item_memory: ENTRY_MEMORY, ceiling: None, cap: None, host: &request.host })
        .min(places_a_round_may_take());
    let mut left: Vec<String> = Vec::new();
    let removed = run_ordered_awaiting_on_the_runtime(OrderedRunOfAwaitingOwned {
        items: indices,
        lanes: Arc::from(lanes),
        workers,
        work: Arc::new(RemoveEachEntry { inside: request.inside.clone(), trash: Arc::clone(&request.trash), manifest: Arc::clone(&request.manifest), remove: Arc::clone(&request.remove) }),
        on_event: &mut KeepReasonsOfEntriesThatStay { left: &mut left },
    })
    .await;
    if let Err(stopped) = removed {
        // The entries the run did not reach stay, so what ended the run is a reason among the others.
        left.push(format!("the removal of the other entries stopped: {stopped}"));
    }
    return left;
}

/// The controls for the entries of a trash removed side by side stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_entries_tests.rs"]
mod tests;
