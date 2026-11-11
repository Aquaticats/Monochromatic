//! What:
//!  How one scratch entry is removed,
//!  bounded so a device that stalls cannot hold a command.
//! Why:
//!  A recursive removal cannot be interrupted once it starts,
//!  and the commit timing record holds one `unlinkat` of a shaped tree at 2.5 s
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
//!  A command that waits for one therefore waits for the device,
//!  which the owner's "commits vs native git should not take 5s longer" rules against,
//!  so every removal here runs on a thread of its own,
//!  stops being waited for at `SCRATCH_REMOVAL_CEILING`,
//!  and what it spent and what became of it go to the command's timing record.
//!  A caller that collects the trees of killed commands uses the same ceiling
//!  (`landing/service_scratch_abandonment.rs`).
//!  The threads such a removal leaves behind are bounded:
//!  every removal that has not finished holds one of the process's places for blocking work,
//!  and a removal that finds none leaves its entry to a later sweep instead of waiting.
//!  The place of one removal comes back when its work ends and before its caller is told what became of it,
//!  so a caller that has just swept can sweep again and find the places its own removals took.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removal = removeWithin(path, 30_000); // 'finished' | 'unfinished' | { failed }
//! ```

/// Debug lines for the removals that do not finish.
use crate::diagnostic_log::debug;
/// The recursive removal of a directory tree.
use crate::private_storage::remove_tree;
/// The two costs a command pays outside its phases,
/// one of which is removing a scratch tree.
use crate::time_spent_outside_the_phases::note_scratch_removal;
/// A path,
/// a span of time and a moment.
use std::path::{Path, PathBuf};
/// The counter of the removals that are still running,
/// shared by the threads of one process.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// The tag of this module's diagnostics.
const TAG: &str = "monochromatic-git-policy-cli";

/// The ceiling on removing one scratch entry,
/// which is where a recursive removal could otherwise wait for as long as the device stalls.
///  Measured:
/// the commit timing record of 2026-11-09 holds one `unlinkat` of a scratch tree at 2.5 s in a shaped fixture
///  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
/// "What the steps wait on"),
///  and a shaped tree measured on this host on 2026-11-09 holds 9,081,951 bytes in a handful of entries,
///  whose whole removal took 34 `unlinkat` calls and 1.5 ms of system time under `strace`.
///  Thirty seconds is twelve times the slowest single call the record holds,
///  and a removal that reaches it is reported rather than waited for.
///  It is never a limit on a removal that has to finish before the person can go on:
///  the commit's own scratch tree is handed to the service or left to the sweep instead.
pub const SCRATCH_REMOVAL_CEILING: Duration = Duration::from_secs(30);

/// What a removal of one scratch entry came to.
#[derive(Debug)]
pub enum Removal {
    /// The entry is gone.
    Finished,
    /// The removal was still running when its ceiling passed,
    /// and the entry is partly or wholly there;
    /// the next sweep tries again.
    Unfinished,
    /// The removal was refused,
    /// with the refusal that says why.
    Failed(std::io::Error),
}

impl Removal {
    /// Whether the entry is gone.
    pub fn finished(&self) -> bool {
        return matches!(self, Self::Finished);
    }
}

/// How many removals are running without having finished.
///
///  Every one of them holds a thread that the process cannot end,
///  so the number is held to the places the process has for blocking work
///  (`blocking_jobs_within_the_budget.rs`):
///  a removal is blocking work nobody waits for,
///  and a stalled removal may never give its thread back.
static UNFINISHED_REMOVALS: AtomicUsize = AtomicUsize::new(0);

/// What:
///  The place one removal that outlasted its ceiling took,
///  given back when the removal's own work ends and before the caller is told what became of it.
/// Why:
///   The caller stops waiting at the ceiling and its budget slot goes back,
///  so without this place the threads of stalled removals would gather with nothing bounding them
///  until the process cannot start a thread at all.
struct ThePlaceOfARunningRemoval(());

impl Drop for ThePlaceOfARunningRemoval {
    /// Gives the place back,
    /// whatever the removal came to.
    fn drop(&mut self) {
        let _given_back: usize = UNFINISHED_REMOVALS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// What:
///  Takes a place for one more removal that has not finished,
///  or answers nothing when every place is held.
/// Why:
///   A caller that cannot have a place does not wait for one:
///  it leaves the entry to a later sweep and goes on,
///  so work that stalls can never make a command wait.
///  The bound is the process's budget for blocking work,
///  the same places its Git children and its other blocking jobs draw on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = takeAPlace(); // undefined when every place is held
/// ```
fn take_a_place() -> Option<ThePlaceOfARunningRemoval> {
    let cap: usize = crate::blocking_jobs_within_the_budget::budgeted_blocking_jobs().max(1);
    loop {
        let running: usize = UNFINISHED_REMOVALS.load(Ordering::SeqCst);
        if running >= cap {
            return None;
        }
        if UNFINISHED_REMOVALS.compare_exchange(running, running + 1, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            return Some(ThePlaceOfARunningRemoval(()));
        }
    }
}

/// What:
///  How many removals are running without having finished,
///  which is at most the places the process has for them.
/// Why:
///   A control needs the figure to see a place come back,
///  and a diagnostic that says a removal found no place names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// print(`unfinished removals: ${unfinishedRemovals()}`);
/// ```
pub fn unfinished_removals() -> usize {
    return UNFINISHED_REMOVALS.load(Ordering::SeqCst);
}

/// What:
///  Removes one scratch entry,
///  a directory with its contents or a single name,
///  and returns what became of it at `ceiling`.
/// Why:
///   A recursive removal cannot be interrupted once it starts,
///  so it runs on a thread of its own and the caller stops waiting at the ceiling.
///  A thread still working when the process ends takes its part of the removal with it,
///  which leaves a partly removed entry for the next sweep,
///  never a wrong one.
///  The time and the outcome go to the command's timing record,
///  so a machine where removals stall says so where a reading looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = removeWithin(path, ceiling); // 'finished' | 'unfinished' | { failed }
/// ```
pub fn remove_within(path: &Path, ceiling: Duration) -> Removal {
    let began: Instant = Instant::now();
    let target: PathBuf = path.to_path_buf();
    // A removal that outlasted its ceiling keeps its thread, so a place is taken before one is started, and a caller that finds
    //  none leaves the entry to a later sweep instead of waiting for one.
    let Some(place) = take_a_place() else {
        let running: usize = unfinished_removals();
        debug(
            TAG,
            format!("the scratch entry {} is left to a later sweep: {running} removals that have not finished hold every place", path.display()).as_str(),
        );
        note_scratch_removal(Duration::ZERO, false);
        return Removal::Unfinished;
    };
    // What the thread ended with; a thread that outlives the ceiling sends into a channel nobody reads, which is no error.
    let (ended, waited) = std::sync::mpsc::channel::<std::io::Result<()>>();
    let started = std::thread::Builder::new().name(String::from("scratch-removal")).spawn(move || {
        // The place belongs to the removal's own work, so a removal its caller stopped waiting for keeps it until that work ends.
        let outcome: std::io::Result<()> = remove_entry(&target);
        // It goes back before the answer is sent, so a caller told a removal finished finds its place free: a sweep that
        //  followed one whose removals had just been reported would otherwise find every place held by that sweep's own
        //  work and start nothing (measured on 2026-11-11: 0 of 2 entries removed in the gate's image, `left: 0, right: 2`
        //  in a gate of 2026-11-10).
        drop(place);
        let _sent = ended.send(outcome);
    });
    let removal: Removal = match started {
        Err(error) => Removal::Failed(error),
        Ok(_thread) => match waited.recv_timeout(ceiling) {
            Ok(Ok(())) => Removal::Finished,
            Ok(Err(error)) => Removal::Failed(error),
            Err(_late) => Removal::Unfinished,
        },
    };
    let took: Duration = began.elapsed();
    if let Removal::Failed(error) = &removal {
        debug(TAG, format!("could not remove the scratch entry {}: {error}", path.display()).as_str());
    }
    if let Removal::Unfinished = &removal {
        debug(TAG, format!("the scratch entry {} was still being removed after {ceiling:?}", path.display()).as_str());
    }
    note_scratch_removal(took, removal.finished());
    return removal;
}

/// One scratch entry removed:
/// a real directory with its contents,
/// or anything else as the one name it is,
/// so a link is removed rather than followed.
fn remove_entry(path: &Path) -> std::io::Result<()> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    return if metadata.is_dir() { remove_tree(path) } else { std::fs::remove_file(path) };
}

/// The removal is controlled in the test module.
#[cfg(test)]
#[path = "scratch_entry_removal_tests.rs"]
mod tests;
