//! What:
//! The step of a publish a build made for controls can arm:
//! the instant between its two renames,
//! where a process it ends leaves the version's name absent.
//! Why:
//! A control has to show what a kill at that instant leaves,
//! and a shipped build must hold no way to arm it:
//! the read below is compiled out of a build that is not a test,
//! the shape `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` chose
//! for the git wrapper's own switches.
//! Nothing here is read from the environment at all,
//! so no `Environment=` line and no shell can arm the step:
//! a control arms it for its own thread and for one publish.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-control-plants-these-steps.ts
//! function plantEndingTheProcess(): void;  // tests only
//! function reachedTheInstantBetweenTheTwoRenames(): void;
//! ```

/// Imports the cell a control's plant lives in.
#[cfg(test)]
use std::cell::Cell;

/// What:
/// Whether this thread's next publish ends in the instant between the two renames.
/// Why:
/// A control asks for it before the publish it means,
/// and the thread is where its own publish runs,
/// so two controls in one test binary never arm each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plantEndingTheProcess(): void { PLANTED.set(true); }
/// ```
#[cfg(test)]
pub(crate) fn plant_ending_the_process() {
    ENDS_HERE.with(|cell| {
        cell.set(true);
    });
}

/// What:
/// Forgets a plant.
/// Why:
/// A plant that outlived its control would end the next publish on the same thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearThePlant(): void { PLANTED.set(undefined); }
/// ```
#[cfg(test)]
pub(crate) fn clear_the_plant() {
    ENDS_HERE.with(|cell| {
        cell.set(false);
    });
}

#[cfg(test)]
thread_local! {
    /// Whether this thread's next publish ends in the instant between the two renames.
    static ENDS_HERE: Cell<bool> = const { Cell::new(false) };
}

/// What:
/// Ends the process in the instant between the two renames,
/// when a control armed that.
/// Why:
/// It is the one instant a publish has,
/// and a control shows what a kill in it leaves behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reachedTheInstantBetweenTheTwoRenames(): void { if (PLANTED.get()) process.kill(process.pid, 'SIGKILL'); }
/// ```
#[cfg(test)]
pub(super) fn reached_the_instant_between_the_two_renames() {
    if ENDS_HERE.with(|cell| cell.get()) {
        end_this_process_as_a_kill_does();
    }
}

/// The same instant in a shipped build:
/// nothing ends the process there.
#[cfg(not(test))]
pub(super) fn reached_the_instant_between_the_two_renames() {}

/// What:
/// Ends this process with `SIGKILL`,
/// the signal no handler can catch and after which no cleanup runs.
/// Why:
/// The control shows what a killed install leaves,
/// so nothing of the installer may run after the instant:
/// no destructor and no cleanup of the directories it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(process.pid, 'SIGKILL');
/// ```
#[cfg(all(test, unix))]
fn end_this_process_as_a_kill_does() -> ! {
    // The wrapper is safe, and nothing follows a `SIGKILL` delivered to this process.
    rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::KILL).ok();
    // A signal the system refused to deliver leaves this process alive, so it ends the other way.
    std::process::abort()
}

/// What:
/// Ends this process at once on a platform without `SIGKILL`.
/// Why:
/// A control that cannot use that signal still must not run any cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.abort();
/// ```
#[cfg(all(test, not(unix)))]
fn end_this_process_as_a_kill_does() -> ! {
    std::process::abort()
}
