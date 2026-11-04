//! What:
//!  The removal of the entries a snapshot's leave-out moved into its trash, side by side.
//! Why:
//!  The entries are separate directories that share only the held trash handle, each opened by its own name and removed only
//!  when it is the recorded object, so they are independent items: a reason for one entry never stops the others. They run
//!  within the budget the host allows (`parallel_budget.rs`), which inside the commit service's control group is the two
//!  threads and the memory that unit is given, and the reasons come back in entry order (`parallel_runner.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await removeEntries({ inside, trash, manifest, host, remove }); // one reason per entry that stays
//! ```

use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::parallel_runner::{Event, OrderedRun, run_ordered};
use crate::worktree_trash_manifest::{TrashManifest, entry_name};
use crate::worktree_trash_remove::Removal;
use std::fs::File;
use std::path::Path;

/// The peak resident growth of removing one flat directory of a million names (`remove_tree_beneath`, names of 42 characters,
///  a fresh process, a throwaway directory on btrfs, one run): 97,320 KiB, which is 99,655,680 bytes. Runs of 100,000, 200,000
///  (twice) and 400,000 names grew by 131, 113, 113 and 104 bytes a name, so the figure per name falls as the count rises and
///  the one-million run is the case the estimate is for.
const MEASURED_MILLION_NAME_PEAK: u64 = 99_655_680;

/// The bytes the removal of one trash entry holds at its peak: `MEASURED_MILLION_NAME_PEAK` rounded up to a power of two. The
///  listing of a directory is held whole while its entries are removed. The coordinator's pick under the owner's delegation,
///  open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`, "Parallelism caps and memory estimates").
const ENTRY_MEMORY: u64 = MEASURED_MILLION_NAME_PEAK.next_power_of_two();

/// The removal of one entry beneath the trash handle, replaceable so a control can watch or delay it.
pub type RemoveOne<'a> = dyn Fn(&Path, &File, &std::ffi::OsStr, Option<crate::fs_beneath::ObjectId>) -> Result<crate::worktree_trash_remove::Removal, String> + Sync + 'a;

/// What:
///  What the removal of a trash's entries is told.
/// Why:
///   One value names the inputs: where the trash is, its manifest, the host the budget is derived from, and the removal of one entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryRemoval = { inside: string; trash: Dir; manifest: TrashManifest; host: HostResources; remove: RemoveOne };
/// ```
pub struct EntryRemoval<'a> {
    /// The path of the trash directory, for messages.
    pub inside: &'a Path,
    /// The trash directory, held open; every entry is opened by name below it.
    pub trash: &'a File,
    /// What the leave-out recorded.
    pub manifest: &'a TrashManifest,
    /// The CPUs and memory the budget is derived from.
    pub host: &'a HostResources,
    /// The removal of one entry.
    pub remove: &'a RemoveOne<'a>,
}

/// What:
///  Remove every entry the manifest records, side by side, and return the reason each entry that stays gives, in entry order.
/// Why:
///   The entries are separate directories moved into the trash, each removed only when it is the recorded object, and share no
///  state but the held trash handle, which opens each by its own name. A reason for one never stops the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await removeEntries({ inside, trash, manifest, host, remove }); // one reason per entry that stays
/// ```
pub fn remove_entries(request: &EntryRemoval<'_>) -> Vec<String> {
    let EntryRemoval { inside, trash, manifest, host, remove } = *request;
    let indices: Vec<usize> = (0..manifest.entries.len()).collect();
    let lanes: Vec<Vec<usize>> = indices.iter().map(|index: &usize| { return vec![*index]; }).collect();
    let workers: usize = worker_budget(&BudgetRequest { items: indices.len(), per_item_memory: ENTRY_MEMORY, ceiling: None, cap: None, host });
    let mut left: Vec<String> = Vec::new();
    run_ordered(OrderedRun {
        items: indices.as_slice(),
        lanes: lanes.as_slice(),
        workers,
        work: &|_slot: usize, index: &usize, _say: &mut dyn FnMut(&str)| { return remove(inside, trash, std::ffi::OsStr::new(entry_name(*index).as_str()), Some(manifest.entries[*index])); },
        on_event: &mut |event: Event<'_, Result<Removal, String>>| {
            if let Event::End(_slot, Err(reason)) = event {
                left.push(reason);
            }
        },
    });
    return left;
}

/// The controls for the entries of a trash removed side by side stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_entries_tests.rs"]
mod tests;
