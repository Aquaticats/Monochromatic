//! What:
//!  Which items of one batch of the service's work run at once,
//! how many of them,
//! and what running them at once does with their records.
//! Why:
//!  One repository's work directory holds the records of every worktree of that repository
//! (a new worktree's prewarm and relink,
//! a removed one's subvolume and trash),
//! and the drain ran each kind's items one after another,
//! so a queue of twenty two records of one repository drained one at a time
//! while the owner's rule is "Shouldn't the service run anything that can be ran in parallel in parallel?"
//! (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`,
//! the queue of 22 jobs that delayed a push of `main`).
//!  Two items of one worktree may stand in each other's way:
//! a prewarm fills the Cargo build directories while a relink writes the package state of the same worktree,
//! a trash empties the directories a snapshot left out of the same worktree,
//! and a removal's subvolume delete takes the whole worktree away.
//! The trash's own emptying removes the trash's parent directory when nothing else is in it
//! (`worktree_trash_dispose.rs`),
//! so two trashes of one worktree would race over it.
//!  Two items of different worktrees share nothing but the disk and the process's blocking places,
//! so they run beside each other,
//! one item of each worktree per wave,
//! within the width this file measures and the host allows.
//! Nothing here locks:
//! the claims stay in the drain's own set and are settled by the task that took them,
//! and every wave's answers come back in the order the items were claimed
//! (`foundation/side_by_side_on_the_current_task.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const wave of wavesOfWorktrees(items, width)) await Promise.all(wave.map(run)); for (const item of wave) settle(item);
//! ```

use crate::landing::service_work_handlers_replanning_and_acceptance::{Verdict, WorkHandlers};
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::service_claim_renewal::HeldClaims;
use crate::service_work_settling_with_words::settle_with_the_words;
use crate::side_by_side_on_the_current_task::run_all_side_by_side_on_the_current_task;
use crate::work_item::{Claimed, WorkItem};
use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::pin::Pin;

/// The most items of one batch of slow work that run at once in one work directory:
/// 4.
/// Measured on `2026-11-11T22:20:00-05:00` on this host (28 threads as `nproc` reports them,
/// the load average 26 to 44 while the arms ran,
/// so other sessions were using it),
/// in Sartre's own run folder under `${HOME}/temp/agent`,
/// on the btrfs volume this repository lives on:
/// the removal of 16 directory trees of 200 files each,
/// every removal one job of the process's blocking pool,
/// with the pool held to the service's own four places (`SERVICE_WORKER_CAP`) as the service holds it,
/// and one removal per item at a time.
/// One item at once drained the queue in 3.68 s and 3.60 s,
/// two items at once in 2.47 s and 2.39 s,
/// four in 1.93 s and 1.88 s,
/// six in 1.91 s and 1.94 s,
/// and eight in 1.90 s and 1.93 s.
/// The same spacing appears without the three arms' spread:
/// the time to drain the queue stops falling after four items,
/// which is the number of places the service's blocking pool has and the number of its runtime's workers
/// (`service/service_main.rs`,
/// `SERVICE_WORKER_CAP`).
/// More than four items at once buys nothing and only shortens the turn each of them gets before the work a person's
/// push needs,
/// so the ceiling is four.
/// A share cannot be measured as a quantity on a host of another size,
/// so the figure is capped inside `worker_budget` by the CPUs and the memory this host has,
/// and the ceiling is the disk's own measured answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SLOW_ITEMS_CEILING = 4;
/// ```
pub(crate) const SLOW_ITEMS_CEILING: usize = 4;

/// The bytes one slow item of a work directory holds at its peak:
/// the bytes the drain of one repository holds at its peak,
/// which a unit's own measured peak bounds
/// (`MEASURED_REPOSITORY_PEAK` in `landing/service_round.rs`,
/// 130,482,176 bytes rounded up to a power of two).
/// Why:
///   A prewarm's Cargo child and a trash's listing are inside the service's own control group when they run,
/// so the peaked figure of a service unit is an upper bound of one of its items as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SLOW_ITEM_MEMORY = REPOSITORY_MEMORY;
/// ```
const SLOW_ITEM_MEMORY: u64 = crate::service_round::REPOSITORY_MEMORY;

/// What:
///  How many slow items of one work directory may run at once:
/// the host's CPUs less a margin,
/// the memory free over what one item holds,
/// and `SLOW_ITEMS_CEILING`,
/// never fewer than one.
/// Why:
///   The least of them, so a small host or a full control group narrows the width and a large one does not widen it
/// past what the disk was measured to take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const width = workerBudget({ items, perItemMemory: SLOW_ITEM_MEMORY, ceiling: SLOW_ITEMS_CEILING, cap: undefined, host });
/// ```
pub(crate) fn slow_items_side_by_side(items: usize, host: &HostResources) -> usize {
    return worker_budget(&BudgetRequest { items, per_item_memory: SLOW_ITEM_MEMORY, ceiling: Some(SLOW_ITEMS_CEILING), cap: None, host });
}

/// What:
///  How many pushes of different branches of one work directory may run at once:
/// the host's CPUs less a margin,
/// the memory free over what one push holds,
/// and the number of branches due.
/// Why:
///   A push is a child process of the service and a wait on the network,
/// not a job of the blocking pool, so the disk's own ceiling does not bound it;
/// the memory figure is the one a repository's drain is bounded by,
/// because the push is part of that drain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const width = workerBudget({ items: branches, perItemMemory: REPOSITORY_MEMORY, ceiling: undefined, cap: undefined, host });
/// ```
pub(crate) fn pushes_side_by_side(branches: usize, host: &HostResources) -> usize {
    return worker_budget(&BudgetRequest { items: branches, per_item_memory: SLOW_ITEM_MEMORY, ceiling: None, cap: None, host });
}

/// What:
///  The waves items run in:
/// in each wave at most `width` items,
/// no two of them naming the same key,
/// and an item of one key never before the item of that key that was claimed first.
/// Why:
///   The keys are what two items may stand in each other's way over: a worktree's path,
/// or a branch of a repository.
/// A wave is what the caller runs beside each other and settles before the next,
/// so a key's items keep the order they were claimed in and no two of them meet.
/// The turn moves by one key each wave, so a long queue of one worktree does not keep another worktree's single
/// item waiting for the whole of it unless the width is one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waves = wavesOf(keys, width); // one item per key per wave, in the order the keys were first named
/// ```
pub(crate) fn waves_of<K: Clone + Eq + Hash>(keys: &[K], width: usize) -> Vec<Vec<usize>> {
    let mut queues: Vec<Vec<usize>> = Vec::new();
    let mut queue_of: HashMap<K, usize> = HashMap::new();
    for (index, key) in keys.iter().enumerate() {
        let queue: usize = match queue_of.get(key) {
            Some(found) => *found,
            None => {
                queue_of.insert(key.clone(), queues.len());
                queues.push(Vec::new());
                queues.len() - 1
            }
        };
        queues[queue].push(index);
    }
    let mut taken: Vec<usize> = std::iter::repeat_n(0, queues.len()).collect();
    let mut waves: Vec<Vec<usize>> = Vec::new();
    let mut turn: usize = 0;
    loop {
        let mut wave: Vec<usize> = Vec::new();
        for step in 0..queues.len() {
            if wave.len() >= width {
                break;
            }
            let queue: usize = (turn + step) % queues.len();
            if let Some(item) = queues[queue].get(taken[queue]) {
                wave.push(*item);
                taken[queue] += 1;
            }
        }
        if wave.is_empty() {
            return waves;
        }
        turn = (turn + 1) % queues.len();
        waves.push(wave);
    }
}

/// What:
///  Run one batch of items of one kind beside each other,
/// one item of each worktree per wave,
/// and settle every one of them with the words its kind leaves.
/// Why:
///   The kinds all answer the four verdicts and are all settled the same way
/// (`service_work_settling_with_words.rs`),
/// so the account of running them is one function and each kind names only its handler and its words.
/// The claims stay in the caller's set and come back with it,
/// so nothing here locks the set or names a claim.
/// A set the system refuses a job for ends the batch,
/// as it ends the drain (`service_work_drain_pending_claims.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const wave of wavesOf(worktrees, width)) { const verdicts = await Promise.all(wave.map(ask)); for (const [item, verdict] of zip(wave, verdicts)) held = await settleWithTheWords(held, workDir, item, verdict, kind); }
/// ```
pub(crate) async fn run_and_settle_side_by_side<H, F>(mut held: HeldClaims, work_dir: &Path, handlers: &H, items: Vec<Claimed>, width: usize, kind: &str, ask: F) -> Option<HeldClaims>
where
    H: WorkHandlers + Sync + ?Sized,
    F: for<'lent> Fn(&'lent H, &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>>,
{
    let worktrees: Vec<PathBuf> = items.iter().map(|claimed| { return claimed.item.repository.clone(); }).collect();
    for wave in waves_of(worktrees.as_slice(), width) {
        let mut running: Vec<Pin<Box<dyn Future<Output = Verdict> + Send + '_>>> = Vec::with_capacity(wave.len());
        for index in &wave {
            running.push(ask(handlers, &items[*index].item));
        }
        let verdicts: Vec<Verdict> = run_all_side_by_side_on_the_current_task(running).await;
        for (index, verdict) in wave.iter().zip(verdicts) {
            let claimed: Claimed = items[*index].clone();
            held = settle_with_the_words(held, work_dir, claimed, verdict, kind).await?;
        }
    }
    return Some(held);
}

/// Controls for the items of one work directory run beside each other stay out of the release executable.
#[cfg(test)]
#[path = "service_work_items_side_by_side_tests.rs"]
mod tests;
