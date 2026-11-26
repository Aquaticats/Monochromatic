//! What:
//!  The children this command stopped waiting for,
//!  gathered by a thread of their own,
//!  and what happens when the system refuses that thread.
//! Why:
//!  A child in an uninterruptible call (a stalled disk,
//! a stopped network file system) ends only when the
//!  kernel lets it go,
//! which can be long after the ceiling that ended the wait for it.
//! The command must not
//!  wait for that,
//! and a process nobody waits for stays a zombie while this process lives,
//! so the child is handed
//!  to a thread that collects it.
//!  Two handles name such a child,
//! and both come to this file:
//! the `Child` the standard library made (`reap_soon`),
//!  and the number of a child this process forked itself
//! (`reap_soon_pid`), which is every handle a raw `fork` leaves behind
//!  (`open_file_of_a_descriptor_in_a_child.rs`,
//! and the account companion's own fork).
//!  The wait for such a child may never end,
//! so no job of the blocking pool can carry it:
//! that pool's threads are bounded,
//!  and a job that never returns takes one of them for as long as this process lives.
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
//! reapSoonPid(pid);         // the same bargain for a child this process forked itself
//! drainTheRegistry();       // a reaper that started takes what a refused start left behind
//! ```

use std::process::Child;
use std::sync::{
    Mutex,
    MutexGuard,
};

/// What:
///  `enum Nowait` is one child this command stopped waiting for,
///  with the handle its starter held it by.
/// Why:
///  A `Child` and a number are reaped the same way,
///  so one list holds both and one reaper drains it,
/// and the two cannot disagree about what a refused start keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Nowait = { kind: 'child'; child: Child } | { kind: 'pid'; pid: number }
/// ```
enum Nowait {
    /// A child the standard library started,
    /// which its starter no longer holds.
    Child(Child),
    /// A child this process forked itself,
    /// which its starter holds only as a number.
    Pid(libc::pid_t),
}

/// The children whose reaper could not be started,
/// kept until a reaper runs.
static WAITING_FOR_A_REAPER: Mutex<Vec<Nowait>> = Mutex::new(Vec::new());

/// The name of the thread that collects a child this command stopped waiting for.
pub const REAPER_THREAD_NAME: &str = "cli-git-stuck-child-reaper";

/// What:
///  `fn collect_the_number(pid: libc::pid_t)` waits for one child the starter holds only as a number,
///  however long the kernel takes.
/// Why:
///  It is a number reaper's whole work,
/// and a number is a safe address here because
///  the child is this process's own and unreaped,
///  so the kernel cannot have given it to anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitpid(pid, 0);
/// ```
fn collect_the_number(pid: libc::pid_t) {
    let mut status: libc::c_int = 0;
    // SAFETY: the number is this process's own unreaped child, and `status` is a live variable of the right type.
    let _collected: libc::pid_t = unsafe { libc::waitpid(pid, &raw mut status, 0) };
}

impl Nowait {
    /// What:
    ///  Wait for the child however long the kernel takes,
    ///  and let it go.
    /// Why:
    ///  It is the reaper's whole work,
    ///  and it runs on a thread that has nothing else to do.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// kind === 'child' ? child.wait() : waitpid(pid, 0);
    /// ```
    fn collect(self) {
        match self {
            Nowait::Child(mut child) => {
                let _reaped = child.wait();
            }
            Nowait::Pid(pid) => collect_the_number(pid),
        }
    }
}

/// The list of children waiting for a reaper.
fn waiting_for_a_reaper() -> Result<
    MutexGuard<'static, Vec<Nowait>>,
    std::sync::PoisonError<MutexGuard<'static, Vec<Nowait>>>,
> {
    return WAITING_FOR_A_REAPER.lock();
}

/// Keeps a child for the next reaper.
fn keep(nowait: Nowait) {
    match waiting_for_a_reaper() {
        Ok(mut kept) => kept.push(nowait),
        // A poisoned list still holds its children, and nothing here may panic on the way out of a stop.
        Err(poisoned) => poisoned.into_inner().push(nowait),
    }
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
    let kept: Vec<Nowait> = match waiting_for_a_reaper() {
        Ok(mut waiting) => std::mem::take(&mut *waiting),
        Err(poisoned) => std::mem::take(&mut *poisoned.into_inner()),
    };
    for nowait in kept {
        nowait.collect();
    }
}

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
    let started = std::thread::Builder::new()
        .name(String::from(REAPER_THREAD_NAME))
        .spawn(move || {
            // This reaper collects the child it was given, and then whatever a refused start left behind.
            if let Ok(mut first) = taken.recv() {
                let _reaped = first.wait();
            }
            drain_the_registry();
        });
    if started.is_err() {
        // No thread could be given the child, so the child that the closure would have owned is still here, in this process's
        //  hands: it waits for a reaper that the next call may start.
        keep(Nowait::Child(child));
        return;
    }
    if let Err(std::sync::mpsc::SendError(returned)) = hand_over.send(child) {
        // The reaper ended before it was given the child; the child is this process's again.
        keep(Nowait::Child(returned));
    }
}

/// What:
///  Hands a child this process forked itself,
///  held only as its number,
///  to a thread that waits for it,
///  or keeps it for the next reaper when no thread can start.
/// Why:
///  It is the bargain `reap_soon` makes,
///  for the handle a raw `fork` leaves:
/// the number is this process's own unreaped child,
///  so waiting for it collects that child and nothing else.
///  A refused start loses nothing here,
///  unlike the child `reap_soon` hands over through a channel:
/// the number is `Copy`,
///  so a closure that could not start holds only a copy of it and this call keeps the number it still has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reapSoonPid(pid);
/// ```
pub fn reap_soon_pid(pid: libc::pid_t) {
    let started = std::thread::Builder::new()
        .name(String::from(REAPER_THREAD_NAME))
        .spawn(move || {
            collect_the_number(pid);
            drain_the_registry();
        });
    if started.is_err() {
        keep(Nowait::Pid(pid));
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
