//! What:
//! The two steps of a publish that a build made for controls can arm:
//! the exchange answered as a file system without `RENAME_EXCHANGE` answers it,
//! and the process ended in the instant between the two renames of the fallback.
//! Why:
//! A control has to show both arms of the replace step
//! and the instant only one of them has,
//! and a shipped build must hold no way to arm either:
//! every read below is compiled out of a build that is not a test,
//! the shape `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` chose
//! for the git wrapper's own switches.
//! The reason is the one recorded there:
//! a switch that can end the process or refuse a publish turns a working install into a failing one,
//! and a build a release ships holds no such switch.
//! Nothing here is read from the environment at all,
//! so no `Environment=` line and no shell can arm a step:
//! a control arms one for its own thread and for one publish.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-control-plants-these-steps.ts
//! type Plant = { refuseTheExchange: boolean; killBetweenTheTwoRenames: boolean };
//! function plantTheseSteps(plant: Plant): void;  // tests only
//! function exchangeIsRefused(): boolean;         // always false in a shipped build
//! function reachedTheInstantBetweenTheTwoRenames(): void;
//! ```

/// Imports the cell a control's plant lives in.
#[cfg(test)]
use std::cell::Cell;

/// What:
/// The steps of one publish a control arms.
/// Why:
/// The exchange refusal picks the arm a publish takes,
/// and the kill names the instant only that arm has,
/// so a control asks for both at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plant = { refuseTheExchange: boolean; killBetweenTheTwoRenames: boolean };
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Plant {
    /// Whether the exchange answers as a file system without `RENAME_EXCHANGE` answers it.
    pub(crate) refuse_the_exchange: bool,
    /// Whether the process ends in the instant between the two renames.
    pub(crate) kill_between_the_two_renames: bool,
}

#[cfg(test)]
thread_local! {
    /// The steps this thread's next publish takes.
    static PLANTED: Cell<Option<Plant>> = const { Cell::new(None) };
}

/// What:
/// Arms `plant` for the next publish of this thread.
/// Why:
/// A test asks for the arm it means and nothing else,
/// and the thread is where its own publish runs,
/// so two controls in one test binary never arm each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plantTheseSteps(plant: Plant): void
/// ```
#[cfg(test)]
pub(crate) fn plant_these_steps(plant: Plant) {
    PLANTED.with(|cell| {
        cell.set(Some(plant));
    });
}

/// What:
/// Forgets a plant,
/// so a publish made after a control's own takes the shipped steps.
/// Why:
/// A plant that outlived its control would decide another control's publish on the same thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearThePlant(): void
/// ```
#[cfg(test)]
pub(crate) fn clear_the_plant() {
    PLANTED.with(|cell| {
        cell.set(None);
    });
}

/// What:
/// Whether the exchange answers as a file system without `RENAME_EXCHANGE` answers it,
/// which is what a control's plant says.
/// Why:
/// A control needs the two-rename arm on a machine whose file systems all exchange the two names,
/// and this is the only way to ask for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exchangeIsRefused(): boolean { return plantedSteps()?.refuseTheExchange === true; }
/// ```
#[cfg(test)]
pub(super) fn exchange_is_refused() -> bool {
    let planted: Option<Plant> = PLANTED.with(|cell| return cell.get());
    return planted.is_some_and(|plant| return plant.refuse_the_exchange);
}

/// The answer a shipped build gives:
/// the exchange is attempted,
/// because nothing can ask it not to be.
#[cfg(not(test))]
pub(super) fn exchange_is_refused() -> bool {
    return false;
}

/// What:
/// Ends the process in the instant between the two renames,
/// when a control armed that.
/// Why:
/// It is the one instant a publish that cannot exchange the two names has,
/// and a control shows what a kill in it leaves behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reachedTheInstantBetweenTheTwoRenames(): void { if (planted?.killBetweenTheTwoRenames) process.kill(process.pid, 'SIGKILL'); }
/// ```
#[cfg(test)]
pub(super) fn reached_the_instant_between_the_two_renames() {
    let planted: Option<Plant> = PLANTED.with(|cell| return cell.get());
    if planted.is_some_and(|plant| return plant.kill_between_the_two_renames) {
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
    // A signal that the system refused to deliver leaves this process alive, so it ends the other way.
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
