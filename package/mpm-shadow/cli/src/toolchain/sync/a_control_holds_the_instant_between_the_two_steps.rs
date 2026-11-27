//! What:
//! The one instant of a recovery's newest-only rule,
//! between the two steps that rule takes:
//! pointing the tool's `current` link at the newest finished version,
//! and removing the version directories older than it.
//! Why:
//! A control has to stand in that instant,
//! with a start of the tool made there,
//! and it has to end a run there,
//! to show that a start in the window runs the newest finished version and that a run killed in it leaves a link that
//! resolves;
//! neither can be shown from the end state of a run that finished,
//! because the window is two file system calls wide.
//! A shipped build holds no way to stand there:
//! every read below is compiled out of a build that is not a test,
//! the shape `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` chose,
//! as the publish's own plant does (`archive_staging`,
//! `a_control_plants_these_steps`).
//! Nothing here is read from the environment at all,
//! so no `Environment=` line and no shell can plant it:
//! a control plants one for its own thread and one run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a-control-holds-the-instant-between-the-two-steps.ts
//! type Hold = { acts?: (scope: Scope) => void; killAtTheInstant: boolean };
//! function holdingTheNextInstant(hold: Hold): HoldGuard;  // tests only
//! function clearTheHold(): void;                           // tests only
//! function reachedTheInstantBetweenTheTwoSteps(scope: Scope): void;  // nothing in a shipped build
//! ```

/// Imports the cell a control's hold lives in.
#[cfg(test)]
use std::cell::Cell;

/// Imports the scope of the tree the run installs for,
/// which a control's own stand in the instant starts the tool of.
use super::super::scope::Scope;

/// What:
/// The two things a control may ask of this instant:
/// what another program does while this run stands in it,
/// and whether this process ends there as a kill ends it.
/// Why:
/// A start made in the window is one control,
/// and a run killed in the window is another,
/// and each names exactly one of the two;
/// a shipped build never builds this value,
/// so the type only says what a control may ask for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Hold = { acts?: (scope: Scope) => void; killAtTheInstant: boolean };
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Hold {
    /// What another program does while this run stands at the instant,
    /// which is where a control starts the tool through its launcher.
    pub(crate) acts: Option<fn(&Scope)>,
    /// Whether this process ends at the instant,
    /// which is what a kill there leaves.
    pub(crate) kill_at_the_instant: bool,
}

#[cfg(test)]
thread_local! {
    /// The hold this thread's next newest-only rule takes.
    static HELD: Cell<Option<Hold>> = const { Cell::new(None) };
}

/// What:
/// Forgets a hold,
/// so a newest-only rule run after a control's own meets the shipped steps.
/// Why:
/// A hold that outlived its control would decide another control's run on the same thread,
/// and the kill one would end this whole test binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearTheHold(): void
/// ```
#[cfg(test)]
fn clear_the_hold() {
    HELD.with(|cell| {
        cell.set(None);
    });
}

/// What:
/// The release of a hold,
/// taken by the guard `holding_the_next_instant` answers.
/// Why:
/// A control's failing assertion raises a panic,
/// which skips the line that would forget the hold,
/// and a kill left armed would end the next run on this thread with a signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HoldGuard { [Symbol.dispose]() { clearTheHold(); } }
/// ```
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct HoldGuard;

/// The release of a hold.
#[cfg(test)]
impl Drop for HoldGuard {
    fn drop(&mut self) {
        clear_the_hold();
    }
}

/// What:
/// Plants `hold` for the next newest-only rule of this thread,
/// and answers the guard that forgets it when the control's body leaves,
/// however it leaves.
/// Why:
/// A control asks for the instant it means and nothing else,
/// the thread is where its own run goes through that rule,
/// and the guard means a failed assertion cannot arm a kill for another test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdingTheNextInstant(hold: Hold): HoldGuard {
///   HOOKS[threadId] = hold;
///   return new HoldGuard();
/// }
/// ```
#[cfg(test)]
pub(crate) fn holding_the_next_instant(hold: Hold) -> HoldGuard {
    HELD.with(|cell| {
        cell.set(Some(hold));
    });
    return HoldGuard;
}

/// What:
/// The hold this thread's next newest-only rule takes,
/// or nothing when no control planted any.
/// Why:
/// The step below asks this one cell,
/// so a control plants once and the run it starts reads the same hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plantedHold(): Hold | undefined
/// ```
#[cfg(test)]
fn planted_hold() -> Option<Hold> {
    return HELD.with(|cell| return cell.get());
}

/// What:
/// What a control planted at the instant between the two steps:
/// runs what it named,
/// then ends this process when it asked for a kill.
/// Why:
/// The window a start can be made in is the same window a kill can leave `current` naming a version that is gone,
/// and a control picks the instant exactly,
/// so what it shows never depends on how long removing a version takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reachedTheInstantBetweenTheTwoSteps(scope: Scope): void {
///   const held = plantedHold();
///   held?.acts?.(scope);
///   if (held?.killAtTheInstant) process.kill(process.pid, 'SIGKILL');
/// }
/// ```
#[cfg(test)]
pub(super) fn reached_the_instant_between_the_two_steps(scope: &Scope) {
    let held: Option<Hold> = planted_hold();
    if let Some(acts) = held.and_then(|hold| return hold.acts) {
        acts(scope);
    }
    if held.is_some_and(|hold| return hold.kill_at_the_instant) {
        end_this_process_as_a_kill_does();
    }
}

/// The same instant in a shipped build:
/// nothing else acts there and nothing ends the process.
#[cfg(not(test))]
pub(super) fn reached_the_instant_between_the_two_steps(_scope: &Scope) {}

/// What:
/// Ends this process with `SIGKILL`,
/// the signal no handler can catch and after which no cleanup runs.
/// Why:
/// The control shows what a run killed in the window leaves,
/// so nothing of that run may happen after the instant:
/// no destructor and no repair of the link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(process.pid, 'SIGKILL');
/// ```
#[cfg(all(test, unix))]
fn end_this_process_as_a_kill_does() -> ! {
    // What: `kill_process` sends `Signal::KILL` to the process number `getpid` answers, this process's own number.
    // Why: only this process ends, and no other process of the machine is signalled.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // process.kill(process.pid, 'SIGKILL');
    // ```
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
