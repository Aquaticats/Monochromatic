//! What:
//!  The children this command stopped waiting for,
//!  gathered by a thread of their own,
//!  and what happens when the system refuses that thread.
//! Why:
//!  A child in an uninterruptible call (a stalled disk, a stopped network file system) ends only when the
//!  kernel lets it go,
//! which can be long after the ceiling that ended the wait for it.
//! The command must not
//!  wait for that,
//! and a process nobody waits for stays a zombie while this process lives,
//! so the child is handed
//!  to a thread that collects it.
//!  A refused thread must not lose the child:
//! the standard library's `spawn` consumes the closure,
//! so a child moved
//!  into a refused start would be dropped and never reaped,
//! and the panic of a bare `std::thread::spawn` would leave it the same way.
//! The child is therefore
//!  always held by this module:
//! either by the reaper that was given it,
//! or by the list of children waiting for a reaper,
//!  which the next reaper that starts drains.
//!  The list is the smallest unit a lock may cover,
//! and it is held only while a child is put in or taken out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! reapSoon(child);          // the child is collected when the kernel lets it go, or kept for the next reaper
//! drainTheRegistry();       // a reaper that started takes what a refused start left behind
//! ```

use std::process::Child;
use std::sync::{Mutex, MutexGuard};

/// The children whose reaper could not be started,
/// kept until a reaper runs.
static WAITING_FOR_A_REAPER: Mutex<Vec<Child>> = Mutex::new(Vec::new());

/// The name of the thread that collects a child this command stopped waiting for.
pub const REAPER_THREAD_NAME: &str = "cli-git-query-reaper";

/// What:
///  Hands `child` to a thread that waits for it,
/// or keeps it for the next reaper when no thread can start.
/// Why:
///  The child is this command's own and unreaped,
/// so a thread that waits for it collects its end whenever
///  the kernel releases it;
/// the command itself goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reapSoon(child);
/// ```
pub fn reap_soon(child: Child) {
    let (hand_over, taken) = std::sync::mpsc::channel::<Child>();
    let started = std::thread::Builder::new().name(String::from(REAPER_THREAD_NAME)).spawn(move || {
        // This reaper collects the child it was given, and then whatever a refused start left behind.
        if let Ok(mut first) = taken.recv() {
            let _reaped = first.wait();
        }
        drain_the_registry();
    });
    if started.is_err() {
        // No thread could be given the child, so the child that the closure would have owned is still here, in this process's
        //  hands: it waits for a reaper that the next call may start.
        keep(child);
        return;
    }
    if let Err(std::sync::mpsc::SendError(returned)) = hand_over.send(child) {
        // The reaper ended before it was given the child; the child is this process's again.
        keep(returned);
    }
}

/// Keeps a child for the next reaper.
fn keep(child: Child) {
    match waiting_for_a_reaper() {
        Ok(mut kept) => kept.push(child),
        // A poisoned list still holds its children, and nothing here may panic on the way out of a stop.
        Err(poisoned) => poisoned.into_inner().push(child),
    }
}

/// The list of children waiting for a reaper.
fn waiting_for_a_reaper() -> Result<MutexGuard<'static, Vec<Child>>, std::sync::PoisonError<MutexGuard<'static, Vec<Child>>>> {
    return WAITING_FOR_A_REAPER.lock();
}

/// What:
///  Waits for every child a refused reaper start left behind.
/// Why:
///  A reaper that starts is the only place that can collect them,
/// and a child kept for ever is a zombie for as long as this
///  process lives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const child of registry.splice(0)) await child.exited;
/// ```
pub fn drain_the_registry() {
    let kept: Vec<Child> = match waiting_for_a_reaper() {
        Ok(mut waiting) => std::mem::take(&mut *waiting),
        Err(poisoned) => std::mem::take(&mut *poisoned.into_inner()),
    };
    for mut child in kept {
        let _reaped = child.wait();
    }
}

/// How many children wait for a reaper,
/// for the controls of a refused start.
#[cfg(test)]
pub fn waiting_count() -> usize {
    return match waiting_for_a_reaper() {
        Ok(waiting) => waiting.len(),
        Err(poisoned) => poisoned.into_inner().len(),
    };
}

/// Controls for a refused reaper start stay out of the release executable.
#[cfg(test)]
#[path = "stuck_children_reaped_when_the_kernel_lets_go_tests.rs"]
mod tests;
