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
//!  A wave settles each item as that item's work ends,
//! not after the slowest of the wave,
//! so a finished item's record does not wait for a sibling that takes minutes;
//! and an item whose handler stopped unexpectedly is settled as failed with the words the panic left,
//! so its siblings finish and are settled (`a_panic_that_ends_one_item.rs`).
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
use crate::side_by_side_on_the_current_task::{AnAnswer, AnswersAsTheyCome};
use crate::work_item::{Claimed, WorkItem};
use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::pin::Pin;

/// The most items of one batch of slow work that run at once in one work directory:
/// 4,
/// which is the number of places the service's blocking pool has and the number of its runtime's workers
/// (`service/service_main.rs`,
/// `SERVICE_WORKER_CAP`).
/// Measured on `2026-11-11T16:45:00-05:00` on this host,
/// under its own load (average 68 to 100,
/// so other sessions were working it hard),
/// with the service's caps in force and the same binary asked for each width,
/// alternating,
/// eight worktrees' trashes of one entry of 8,000 files each:
/// one item at once drained them in 1,599 ms and 1,389 ms,
/// two at once in 9,935 ms and 948 ms,
/// four at once in 1,129 ms and 1,090 ms,
/// and eight at once in 1,916 ms and 853 ms.
/// The spread of one width from round to round covers the differences between widths,
/// so a wider wave bought nothing measurable on this host:
/// the removals run at I/O priority class `idle`,
/// and the drive,
/// not the number of items,
/// was the limit.
/// What the width does buy is visible in the controls and not in a wall time:
/// two worktrees' trashes are inside their handler at the same moment
/// (`landing/service_work_items_side_by_side_tests.rs`),
/// which the drain of the release before this one never did.
/// Beyond the process's own places a wider wave only adds tasks that wait for one,
/// so the ceiling is that number,
/// and it is bounded further by the CPUs and the memory of a host of another size
/// (`worker_budget`).
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
///   The least of them,
/// so a small host or a full control group narrows the width and a large one does not widen it
/// past what the disk was measured to take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const width = workerBudget({ items, perItemMemory: SLOW_ITEM_MEMORY, ceiling: SLOW_ITEMS_CEILING, cap: undefined, host });
/// ```
pub(crate) fn slow_items_side_by_side(items: usize, host: &HostResources) -> usize {
    #[cfg(test)]
    {
        let asked: usize = WIDTH_FOR_A_CONTROL.load(std::sync::atomic::Ordering::SeqCst);
        if asked > 0 {
            return items.min(asked).max(1);
        }
    }
    return worker_budget(&BudgetRequest { items, per_item_memory: SLOW_ITEM_MEMORY, ceiling: Some(SLOW_ITEMS_CEILING), cap: None, host });
}

/// What:
///  The width a control stated for the next batches of this process,
///  or nothing when it stated none.
/// Why:
///  A measurement of what running a batch beside each other is worth has to ask both widths of one binary on one host,
/// alternating,
/// because the spread of the same width from run to run is what the difference is read against.
/// The release executable keeps nothing of it.
#[cfg(test)]
static WIDTH_FOR_A_CONTROL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// State the width of the batches that follow,
/// for a measurement of one binary against itself.
#[cfg(test)]
pub(crate) fn hold_the_width_to(width: usize) {
    WIDTH_FOR_A_CONTROL.store(width, std::sync::atomic::Ordering::SeqCst);
}

/// State that the batches that follow take the width their host allows.
#[cfg(test)]
pub(crate) fn take_the_hosts_width() {
    WIDTH_FOR_A_CONTROL.store(0, std::sync::atomic::Ordering::SeqCst);
}

/// What:
///  How many pushes of different branches of one work directory may run at once:
/// the host's CPUs less a margin,
/// the memory free over what one push holds,
/// and the number of branches due.
/// Why:
///   A push is a child process of the service and a wait on the network,
/// not a job of the blocking pool,
/// so the disk's own ceiling does not bound it;
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
///   The keys are what two items may stand in each other's way over:
/// a worktree's path,
/// or a branch of a repository.
/// A wave is what the caller runs beside each other and settles before the next,
/// so a key's items keep the order they were claimed in and no two of them meet.
/// The turn moves by one key each wave,
/// so a long queue of one worktree does not keep another worktree's single
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

/// The worktree an item names,
/// which is what two items of one batch may stand in each other's way over.
fn worktree_of(claimed: &Claimed) -> PathBuf {
    return claimed.item.repository.clone();
}

/// What:
///  Run one batch of items of one kind beside each other,
/// one item of each worktree per wave,
///  settling each item with the words its kind leaves as that item's work ends.
/// Why:
///   The kinds all answer the four verdicts and are all settled the same way
/// (`service_work_settling_with_words.rs`),
/// so the account of running them is one function and each kind names only its handler and its words.
/// An item is settled when its work ends and not when the slowest of its wave does,
/// so a finished item's record does not wait for a sibling that takes minutes.
/// An item whose handler stopped unexpectedly is settled as failed with the words the panic left,
///  and its siblings finish and are settled.
/// The claims stay in the caller's set and come back with it,
/// so nothing here locks the set or names a claim.
/// A set the system refuses a job for ends the batch,
/// as it ends the drain (`service_work_drain_pending_claims.rs`),
///  and the items still inside their work are waited for and settled by nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const wave of wavesOf(worktrees, width)) { for await (const [at, answer] of AnswersAsTheyCome.starting(wave.map(ask))) held = await settleWithTheWords(held, workDir, items[wave[at]], answer, kind); }
/// ```
pub(crate) async fn run_and_settle_side_by_side<H, F>(held: HeldClaims, work_dir: &Path, handlers: &H, items: Vec<Claimed>, width: usize, kind: &str, ask: F) -> Option<HeldClaims>
where
    H: WorkHandlers + Sync + ?Sized,
    F: for<'lent> Fn(&'lent H, &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>>,
{
    let mut every_item_may_start = || true;
    let mut nothing_left_for_a_later_round: Vec<Claimed> = Vec::new();
    return run_and_settle_what_may_start(
        held,
        work_dir,
        handlers,
        items,
        width,
        kind,
        ask,
        &mut every_item_may_start,
        &mut nothing_left_for_a_later_round,
    )
    .await;
}

/// What:
///  Run one batch of housekeeping items the round's bound answers for,
/// one item of each worktree per wave,
///  asking the bound as each item starts rather than once for the batch,
///  settling each item as its work ends,
///  and handing the items the bound did not let start back for a later round.
/// Why:
///   A step runs its items as one batch of waves,
/// and a bound decided once for the batch started every item of any size:
/// a relink, a review or a snapshot that arrived during a long batch of trashes was claimed only when the whole batch was over
/// (`housekeeping_of_one_round.rs`).
/// The bound is asked here as each item is put into a wave,
/// so the items of one wave still start together and the rest keep their turn for a later round.
/// An item the bound refused has not started, so it is not claimed by this round's work:
/// it goes back to `pending` with the others the caller names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const wave of wavesOf(worktrees, width)) { for (const at of wave) { if (!mayStart()) { left.push(items[at]); continue; } starting.push(at); running.push(ask(items[at])); } ... }
/// ```
pub(crate) async fn run_and_settle_what_the_round_may_start<H, F>(
    held: HeldClaims,
    work_dir: &Path,
    handlers: &H,
    items: Vec<Claimed>,
    width: usize,
    kind: &str,
    ask: F,
    may_start: &mut (dyn FnMut() -> bool + Send),
    left_for_a_later_round: &mut Vec<Claimed>,
) -> Option<HeldClaims>
where
    H: WorkHandlers + Sync + ?Sized,
    F: for<'lent> Fn(&'lent H, &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>>,
{
    return run_and_settle_what_may_start(
        held,
        work_dir,
        handlers,
        items,
        width,
        kind,
        ask,
        may_start,
        left_for_a_later_round,
    )
    .await;
}

/// What:
///  `run_and_settle_side_by_side` and `run_and_settle_what_the_round_may_start` with everything the two differ in given.
/// Why:
///   The waves, the starting of each item and the settling of each answer are one account,
///  so a kind that is bounded like housekeeping and a kind that is not cannot drain differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const wave of wavesOf(worktrees, width)) { const starting = []; for (const at of wave) { if (!mayStart()) { left.push(items[at]); continue; } starting.push(at); running.push(ask(items[at])); } for await (const [at, answer] of AnswersAsTheyCome.starting(running)) held = await settleWithTheWords(held, workDir, items[starting[at]], answer, kind); }
/// ```
async fn run_and_settle_what_may_start<H, F>(
    mut held: HeldClaims,
    work_dir: &Path,
    handlers: &H,
    items: Vec<Claimed>,
    width: usize,
    kind: &str,
    ask: F,
    may_start: &mut (dyn FnMut() -> bool + Send),
    left_for_a_later_round: &mut Vec<Claimed>,
) -> Option<HeldClaims>
where
    H: WorkHandlers + Sync + ?Sized,
    F: for<'lent> Fn(&'lent H, &'lent WorkItem) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'lent>>,
{
    let worktrees: Vec<PathBuf> = items.iter().map(worktree_of).collect();
    for wave in waves_of(worktrees.as_slice(), width) {
        let mut running: Vec<Pin<Box<dyn Future<Output = Verdict> + Send + '_>>> = Vec::with_capacity(wave.len());
        let mut starting: Vec<usize> = Vec::with_capacity(wave.len());
        for index in &wave {
            // The bound is asked as each item is put in flight, so a wave still starts its items together and the items after
            //  it keep their turn for a later round.
            if !may_start() {
                left_for_a_later_round.push(items[*index].clone());
                continue;
            }
            starting.push(*index);
            running.push(ask(handlers, &items[*index].item));
        }
        let mut answers: AnswersAsTheyCome<'_, Verdict> = AnswersAsTheyCome::starting(running);
        while let Some((at, answer)) = answers.the_next_answer().await {
            let verdict: Verdict = match answer {
                AnAnswer::Answered(answer) => answer,
                // One item's handler stopped unexpectedly: that item fails with what happened, and the rest of its wave goes on.
                AnAnswer::Stopped(said) => Verdict::Failed(said),
            };
            held = match settle_with_the_words(held, work_dir, items[starting[at]].clone(), verdict, kind).await {
                Some(back) => back,
                None => {
                    // The set went with a job the system refused a thread for, so nothing here can settle what is left:
                    //  the items still inside their work are waited for, and their records keep their lease.
                    answers.the_rest().await;
                    return None;
                }
            };
        }
    }
    return Some(held);
}

/// Controls for the items of one work directory run beside each other stay out of the release executable.
#[cfg(test)]
#[path = "service_work_items_side_by_side_tests.rs"]
mod tests;
