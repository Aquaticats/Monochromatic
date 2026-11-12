//! What:
//!  The line an add prints while it is still running,
//! naming the step it is in and how long it has been there.
//! Why:
//!  An add of this checkout takes seconds on an idle host and tens of seconds on a loaded one:
//! `git status` beside it,
//! a
//!  build,
//! and other agents' work all write to the same disk.
//! Five concurrent adds measured 2026-11-11 took 29.5 to 35.6 seconds each with nothing said,
//! and one step of
//!  the bookkeeping alone waited 11.2 seconds inside one `fsync`.
//! A silent wait cannot be told from a hang,
//! and the owner's rule is that an add still
//!  waiting after a few seconds says what it waits for
//! ("60s is fine for adding worktrees",
//! 2026-11-03).
//! So the step the add is in is published here as it is entered and left,
//! and a thread of the add's own prints it
//!  once the wait is long enough,
//! then again at the cadence,
//! until the add ends.
//! The thread never ends the add and never changes what it does:
//! the line goes to the standard error,
//! one write
//!  per line,
//! and a write that never returns ends with the process like any other output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = beginAddStandingNotice();   // a thread, stopped by the value going out of scope
//! aStepBegan('gitAdd'); aStepEnded();        // published by the add's own timing records
//! ```

use crate::wait_ceilings::{ADD_STANDING_NOTICE_REPEAT, add_standing_notice_after};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// The name the notice's thread is given,
/// so a stack of a stuck add names what it is.
const NOTICE_THREAD_NAME: &str = "cli-git-add-standing-notice";

/// What:
///  What the add of this thread is doing now:
/// the step it is in,
/// and when the whole add began.
/// Why:
///   The steps are recorded on the thread that runs the add,
/// and the notice is printed on a thread of its own,
/// which reads nothing of the other's stack:
/// so the one fact it needs is published here,
/// the innermost step's name and the add's start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let standing: { step?: string; began: number } | undefined;
/// ```
#[derive(Clone, Debug)]
struct Standing {
    /// The innermost step the add has entered and not left,
    /// or nothing between two steps.
    step: Option<String>,
    /// When the add this step belongs to began.
    began: Instant,
}

/// The state the add's own thread writes and the notice thread reads,
/// and the one add a process runs.
/// Why:
///   One process runs one add's steps at a time,
/// and a second add in the same process would publish its own steps over this one's:
/// the notice names the add whose step is there.
static STANDING: Mutex<Option<Standing>> = Mutex::new(None);

/// What:
///  The notice the running add prints,
/// and the answer its thread reads to know when the add ended.
/// Why:
///   Dropping it stops the notice,
/// so the add's own end needs no call of its own and a panic stops it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StandingNotice { stop(): void } // the thread ends within one tick
/// ```
pub struct StandingNotice {
    /// Set when the add ended,
    /// so the notice's thread ends at its next look.
    ended: Arc<AtomicBool>,
    /// The notice's thread,
    /// kept so that it is named here and never joined:
    /// the add never waits for it,
    /// because a stream nobody drains would wait for ever
    /// ("nothing should ever wait forever or indefinitely").
    thread: Option<JoinHandle<()>>,
}

impl Drop for StandingNotice {
    /// Ends the notice without waiting for it.
    fn drop(&mut self) {
        self.ended.store(true, Ordering::SeqCst);
        // The handle is dropped, not joined: the thread ends at its next look, and an add never waits for a line.
        let Some(thread) = self.thread.take() else { return };
        drop(thread);
    }
}

/// What:
///  Tell a notice thread what the add does now,
/// or nothing between two steps.
/// Why:
///   Called by the add's own steps as they are entered and left,
/// so the notice never has to ask what a step means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aStepBegan('gitAdd') // or aStepEnded(undefined) for a step that left
/// ```
pub fn a_step_began(step: Option<&str>, began: Instant) {
    let Ok(mut standing) = STANDING.lock() else {
        crate::diagnostic_log::debug("addStandingNotice", "the step of the add could not be published for the notice");
        return;
    };
    *standing = Some(Standing { step: step.map(str::to_string), began });
}

/// What:
///  Say that the add whose steps were published here ended,
/// so no line names a step that is not running.
/// Why:
///   The steps of an add are published as they are entered and left,
/// and the last one is left by a command that then reports;
/// a notice that arrived after the add's answer would name work that is done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theAddEnded(); // no step is published any more
/// ```
pub fn the_add_ended() {
    let Ok(mut standing) = STANDING.lock() else {
        return;
    };
    *standing = None;
}

/// What:
///  Start the notice for the add whose steps are published here,
/// printing to the standard error.
/// Why:
///   The add's own entry point starts it once,
/// so every step of the add is covered by the one thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = beginAddStandingNotice();
/// ```
pub fn begin() -> StandingNotice {
    return begin_for(false);
}

/// What:
///  Start the notice for an add,
/// which says nothing when the person asked for `--quiet`.
/// Why:
///   `--quiet` is the person asking for no lines,
/// and every other line the add writes respects it,
/// so the notice keeps its thread and prints nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = beginFor({ quiet: plan.quiet });
/// ```
pub fn begin_for(quiet: bool) -> StandingNotice {
    return begin_quiet_after(quiet, Box::new(write_to_standard_error), add_standing_notice_after());
}

/// What:
///  Start the notice for an add whose quietness,
/// sink and wait are the caller's.
/// Why:
///   A control reads the line of a quiet add and of a loud one in one process,
/// with a wait of one tick instead of the shipped seconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = beginQuietAfter({ quiet, announce, wait });
/// ```
pub(crate) fn begin_quiet_after(quiet: bool, announce: Box<dyn FnMut(String) + Send>, wait: Duration) -> StandingNotice {
    return begin_after(announcement(quiet, announce), wait);
}

/// What:
///  What the notice does with a line,
/// which is to write it unless the person asked for no lines.
/// Why:
///   The thread takes one answer for its whole life,
/// so the quietness is read once and never again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const announcement = (quiet, announce) => quiet ? sayNothing : announce;
/// ```
fn announcement(quiet: bool, announce: Box<dyn FnMut(String) + Send>) -> Box<dyn FnMut(String) + Send> {
    if quiet {
        return Box::new(say_nothing);
    }
    return announce;
}

/// What:
///  Do nothing with a line,
/// for an add whose person asked for no lines.
/// Why:
///   The notice's thread prints through this answer,
/// so a quiet add is one answer rather than a second loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sayNothing = (line) => undefined;
/// ```
fn say_nothing(_line: String) {}

/// What:
///  Write one notice line to the standard error,
///  to the descriptor itself rather than through the handle the whole process shares.
/// Why:
///   The add's own entry point hands this function to the notice,
/// so the line and its failure are one place and no closure carries anything.
/// `std::io::stderr()` is one handle behind one lock for the whole process,
/// and a write through it holds that
///  lock for as long as the write takes:
/// a notice whose reader stopped draining would hold the stream the add
///  writes its own reports to,
/// and the add would wait behind its own notice
/// (`standard_error_descriptor.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeToStandardError = (line) => fs.writeSync(2, line + '\n');
/// ```
fn write_to_standard_error(line: String) {
    // Terminal output error: a notice is advisory and must never fail the add it describes.
    let mut stream: crate::standard_error_descriptor::StandardErrorDescriptor = crate::standard_error_descriptor::StandardErrorDescriptor;
    let _written = writeln!(stream, "{line}");
}

/// What:
///  Start the notice with a sink of its own and the shipped wait,
/// for the add's entry point and for a control that reads the line.
/// Why:
///   The line is advisory output on a stream the caller owns,
/// and the thread that prints it must not hold the add's own stream,
/// which the add writes its reports to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = beginWith(sink, ADD_STANDING_NOTICE_AFTER);
/// ```
pub fn begin_with(announce: Box<dyn FnMut(String) + Send>) -> StandingNotice {
    return begin_after(announce, add_standing_notice_after());
}

/// What:
///  Start the notice with a sink and a wait of the caller's choosing.
/// Why:
///   A control waits one tick instead of five seconds,
/// and the add's entry point takes the shipped wait or the one the environment names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = beginAfter(sink, wait);
/// ```
pub fn begin_after(announce: Box<dyn FnMut(String) + Send>, wait: Duration) -> StandingNotice {
    let ended: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let stopping: Arc<AtomicBool> = Arc::clone(&ended);
    let mut writing: Box<dyn FnMut(String) + Send> = announce;
    let thread: Option<JoinHandle<()>> = std::thread::Builder::new()
        .name(String::from(NOTICE_THREAD_NAME))
        .spawn(move || {
            return notice_until_stopped(&mut writing, wait, &stopping);
        })
        .ok();
    return StandingNotice { ended, thread };
}

/// What:
///  Print the notice once the wait is over,
/// then at the cadence,
/// until the process is told the add ended.
/// Why:
///   Nothing here ends the add:
/// the loop stops at the flag the value sets when it goes out of scope,
/// and each line says the step the add is in now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (!stopping.load()) { sleep(tick); const waited = began.elapsed(); if (waited >= wait && due) announce(line(step, waited)); }
/// ```
fn notice_until_stopped(announce: &mut Box<dyn FnMut(String) + Send>, wait: Duration, stopping: &Arc<AtomicBool>) {
    // The wait and the cadence are the caller's, and a wait below the tick would print at every look.
    let tick: Duration = Duration::from_millis(100);
    let mut announced: Option<Duration> = None;
    loop {
        if stopping.load(Ordering::SeqCst) {
            return;
        }
        std::thread::sleep(tick);
        let Some((step, began)) = inside_an_add() else { continue };
        let waited: Duration = began.elapsed();
        if waited < wait {
            continue;
        }
        if let Some(at) = announced
            && waited.saturating_sub(at) < ADD_STANDING_NOTICE_REPEAT
        {
            continue;
        }
        announced = Some(waited);
        announce(notice_line(step.as_deref(), waited));
    }
}

/// What:
///  The step the add is in and when it began,
/// or nothing when no add is recorded.
/// Why:
///   A process that runs no add prints no notice,
/// and a notice thread that finds none ends its look without a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inside = STANDING.tryLock() ? standing : undefined;
/// ```
fn inside_an_add() -> Option<(Option<String>, Instant)> {
    let standing = STANDING.try_lock().ok()?;
    let found: &Standing = standing.as_ref()?;
    return Some((found.step.clone(), found.began));
}

/// What:
///  The line a waiting add prints:
/// how long it has run,
/// the step it is in,
/// and that nothing here ends it.
/// Why:
///   The person learns what the seconds are spent on and that they are spent,
/// without the names of the ladder's steps,
/// which say nothing to someone who has not read this crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: \`git worktree add\` is still running after ${seconds} seconds; it is ${step}. Nothing here waits for another program to finish, so a step that lasts is working, not stuck, and its own failure names what failed.`
/// ```
pub fn notice_line(step: Option<&str>, waited: Duration) -> String {
    let doing: &str = step_sentence(step);
    return format!(
        "monochromatic-git-policy-cli: `git worktree add` is still running after {} seconds; {doing}. Nothing here waits for another command to finish, so a step that lasts is working rather than stuck, and a step that fails names what failed.",
        waited.as_secs()
    );
}

/// What:
///  The step of the add a person reads,
/// in the words of what it does.
/// Why:
///   The step names are the timing record's,
/// written for someone reading that record,
/// so each is said here as the work it is;
/// a step this release knows nothing about is said as the add's own work rather than left unnamed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentenceOf = (step) => ({ gitAdd: "Git's own `git worktree add` is checking out the files", ... })[step] ?? 'the add is working in a step the timing record names'
/// ```
fn step_sentence(step: Option<&str>) -> &'static str {
    let Some(name) = step else {
        return "it is between two of its steps";
    };
    return match name {
        "recover" => "it is settling what interrupted commands left in this repository",
        "chooseRung" | "refusals" | "checkSource" | "claimStage" => "it is deciding how to make the worktree",
        "gitAdd" | "git worktree" => "it is in Git's own `git worktree add`",
        "bindRegistration" | "recordBinding" => "it is recording which directories Git made for the worktree",
        "prepareTarget" | "snapshotAhead" | "snapshotAheadProof" => "it is asking for a snapshot of the main checkout",
        "snapshotAheadWait" => "it is waiting for the commit service to finish a snapshot it already started",
        "takeSnapshot" | "snapshotIoctl" | "walkSnapshot" | "readOwnIndex" | "prune" | "removeStubs" | "replacePointer" | "installIndexFile" => "it is taking the snapshot of the main checkout",
        "applyDifference" | "restoreDirty" => "it is bringing the snapshot to the commit the worktree is at",
        "exchange" => "it is swapping the finished snapshot into place",
        "settleDisplaced" => "it is clearing the directory Git made once the snapshot is in place",
        "markComplete" | "markCompleteFile" | "markCompleteDirectory" | "releaseClaim" => "it is recording that the worktree is complete",
        "copyIgnored" => "it is copying the ignored files into the new worktree",
        "handOverTrash" => "it is handing the files the snapshot set aside to the commit service",
        "repair" => "it is repairing the new worktree's package-manager state",
        _ => "the add is working in a step its timing record names",
    };
}

/// The notice's words and its thread.
#[cfg(test)]
#[path = "add_standing_notice_tests.rs"]
mod tests;
