//! What:
//!  The one-shot run of a watched step:
//! a command started with its output read whole through pipes,
//!  announced by a watch,
//! and ended by it at the step's ceiling or when it shows no sign of work.
//! Why:
//!  `child_watch.rs` holds the watch itself and what a wait on a child marks;
//! the run of a whole step uses it
//!  and none of the watch's own parts,
//! so it sits in a file of its own and `child_watch.rs` re-exports its face.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = await spawnWatched(command, { waitingFor, ledgerStep, terms, stoppable, bound, aStopLetsItFinish });
//! ```

use crate::child_class::ChildClass;
use crate::child_watch::ChildWatch;
use crate::child_watch_clock_marks_and_stopping::WatchTerms;
use crate::child_watch_running_task::a_waiter_thread_is_refused_now;
use crate::child_watch_step_bounds_and_errors::{NotKnown, Outlived, StepBound, ending_of_a_watched_step, not_known_text, output_of, refused_while_stopping};
use crate::output_capture::Capture;
use crate::start_retry::RetryStart;
use crate::step_output_read_whole_through_pipes::{StepPipes, attach_pipes, release_write_ends};
use crate::stop_of_the_service_noticed_by_its_children::{StopArm, Unfinished, begin_unfinished_work};
use crate::stopped_git_locks::LockLedger;
use crate::wait_ceilings::OBJECT_STEP_CEILING;
use std::process::{Child, Command, Output};
use std::time::Duration;

/// What:
///  How the object reader's wait names itself in notices and in the timeout sentence.
/// Why:
///   The reader is one long-lived `cat-file --batch` whose single request can name a large blob,
/// so it runs under the
///  bulk class on purpose (a `cat-file` that answers one query through `run_git` is a quick query step),
/// and its
///  subject says so like every other step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `git cat-file --batch, ${labelOf(BULK)}`
/// ```
pub fn reader_subject() -> String {
    return format!("git cat-file --batch, {}", ChildClass::Object.label());
}

/// What:
///  Run a command to its end and collect its output,
/// announcing the wait and ending the child when it shows no sign of work
///  for the window.
///  The command's input must be set by the caller;
///  its output streams are replaced by pipes.
/// Why:
///   `Command::output` blocks until the child ends and prints nothing meanwhile.
///  A Git step that stalls under load or behind a filter must say so,
/// and must not hold a commit for good;
/// a step that is slow
///  and works is waited for however long it takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, terms }); // throws on a stall
/// ```
pub fn output_watched(command: &mut Command, waiting_for: &str, terms: WatchTerms) -> std::io::Result<Output> {
    return output_watched_stopping(command, waiting_for, waiting_for, terms, true);
}

/// What:
///  `output_watched`,
/// where `stoppable` says whether a child that shows no sign of work for the window is ended,
/// and
///  `ledger_step` names the step under which its longest quiet gap is kept.
/// Why:
///   A step that is not safe to run twice is announced at the usual interval and waited for as long as it lives;
/// the caller
///  starts a stoppable step again when this returns the `TimedOut` error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, ledgerStep, terms, stoppable }); // throws on a stall
/// ```
pub fn output_watched_stopping(
    command: &mut Command,
    waiting_for: &str,
    ledger_step: &str,
    terms: WatchTerms,
    stoppable: bool,
) -> std::io::Result<Output> {
    return output_watched_within(command, waiting_for, ledger_step, terms, stoppable, StepBound::EndAt(OBJECT_STEP_CEILING));
}

/// What:
///  `output_watched_stopping` where the step is ended at `ceiling` whatever it shows,
/// or never ended for its length when there is
///  none.
/// Why:
///   The owner's rule is that nothing waits for ever;
/// the ceiling is what ends a step that keeps working and one that may not be
///  ended for being quiet.
/// A step with no ceiling is a write in place that is not safe to run twice (owner,
/// 2026-11-04:
/// "Never cut
///  a branch write (Recommended)";
/// `transaction_git.rs`,
/// `step_bound`):
/// ending it in the middle of its write could cost the
///  person's data,
/// so only the person ends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, ledgerStep, terms, stoppable, ceilingMs }); // throws on a stall or the ceiling
/// ```
pub fn output_watched_within(
    command: &mut Command,
    waiting_for: &str,
    ledger_step: &str,
    terms: WatchTerms,
    stoppable: bool,
    bound: StepBound,
) -> std::io::Result<Output> {
    return output_watched_within_for_a_stop(command, waiting_for, ledger_step, terms, stoppable, bound, false);
}

/// What:
///  `output_watched_within` where the service's stop may be told to let the step finish.
/// Why:
///   The owner's pick of 2026-11-08 ("B:
/// let a branch write finish (Recommended)"):
/// a stop does not end the branch write.
/// Its count stays,
/// so the shutdown waits for it up to its own bound and the service ends after it;
/// what is still running then is left to the manager's final kill (`KillMode=mixed` in the service unit),
/// and the batch's open intent and the commit on its scratch branch are what recovery reads.
/// Every other step is ended by the stop as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatched(command, { waitingFor, ledgerStep, terms, stoppable, bound, aStopWaits });
/// ```
pub fn output_watched_within_for_a_stop(
    command: &mut Command,
    waiting_for: &str,
    ledger_step: &str,
    terms: WatchTerms,
    stoppable: bool,
    bound: StepBound,
    a_stop_lets_it_finish: bool,
) -> std::io::Result<Output> {
    let ceiling: Option<Duration> = match bound {
        StepBound::EndAt(limit) => Some(limit),
        StepBound::LeaveRunning { .. } | StepBound::Unbounded => None,
    };
    // A step that is never ended writes into files that outlive this command, not into pipes this command holds: when the command
    // stops waiting and exits, the step must find its output open, or it dies of SIGPIPE at its next write, which is a cut. A step
    // that is ended at its ceiling is read to its end by this command through pipes on the runtime, every byte of it; the pipes are
    // made before the child starts, so a refusal starts nothing.
    let (captured, pipes): (Option<Capture>, Option<StepPipes>) = match bound {
        StepBound::EndAt(_) => (None, Some(attach_pipes(command)?)),
        StepBound::LeaveRunning { .. } | StepBound::Unbounded => (Some(Capture::new()?), None),
    };
    if let Some(files) = captured.as_ref() {
        let (out, err) = files.stdio()?;
        command.stdout(out).stderr(err);
    }
    // The service's shutdown waits for this step and ends it when the service is stopped. A landing that is already unwinding after a stop
    //  may start the steps that undo what the stop cut (the side ref that keeps its commit); every other step begun after the stop is refused.
    let mut unfinished: Unfinished = match begin_unfinished_work(crate::service_busy_step_record_on_this_thread::on_landing_thread(), waiting_for) {
        Ok(counted) => counted,
        Err(refusal) => return Err(refused_while_stopping(&refusal)),
    };
    // Taken before the child starts, so a lock born earlier is never taken for the child's.
    let locks: LockLedger = LockLedger::before_start(command);
    let spawned: std::io::Result<Child> = command.spawn_retrying();
    // The command holds the write ends of the pipes until they are replaced, and the readers see no end of output while it does.
    if pipes.is_some() {
        release_write_ends(command);
    }
    let child: Child = spawned?;
    // A step that a stop lets finish keeps its count and takes no arm, so nothing ends it when the stop is told.
    let arm: Option<StopArm> = if a_stop_lets_it_finish { None } else { unfinished.take_arm() };
    let started: ChildWatch = ChildWatch::start_under_stop(waiting_for, child.id(), terms, locks, arm).kept_as(ledger_step);
    // A step with no ceiling is waited for as long as it lives: the watch starts without one.
    let watch: ChildWatch = match ceiling {
        Some(limit) => started.within(limit),
        None => started,
    };
    if !stoppable {
        watch.never_stop();
    }
    // A step the command stops waiting for and leaves running says so in every notice: the notice names that bound, not a wait
    //  that has none (`wait_progress.rs`, `child_wait_line`).
    if let StepBound::LeaveRunning { after, extra } = bound {
        watch.stops_waiting_after(after.saturating_add(extra));
    }
    // Whatever the watch could not do, the wait for the output carries the step's own ceiling: a watch that cannot run, and one
    // that may announce a wait but must not end it, both leave this wait as the only thing that ends the step at its ceiling
    // (the third review's finding 2). A watch whose pin proves the process is the one that ends the step, and this wait is then the
    // backstop behind it, later by the ending such a watch needs. A step with no ceiling is waited for as before.
    let carried: Option<crate::step_output_read_whole_through_pipes::CarriedCeiling> =
        crate::step_output_read_whole_through_pipes::carried_for(ceiling, watch.what_watches_the_step());
    watch.begin();
    let output: std::io::Result<Output> = match bound {
        StepBound::LeaveRunning { after, extra } => {
            // The child moves to a thread that waits for it and collects its output, so the command can stop waiting and the child
            // goes on; the thread reaps it when it ends.
            let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::ExitStatus>>();
            // The child is handed over through a channel rather than moved into the closure, so a refused start leaves it here
            //  (`a_waiter_thread_is_refused_now`, and a system at its thread limit below).
            let (hand_over, taken) = std::sync::mpsc::channel::<Child>();
            let waiter = match a_waiter_thread_is_refused_now() {
                Some(refusal) => {
                    // Nothing was given the receiver, so it is dropped here: the channel closes, and the child comes back to this
                    //  command below to be reaped rather than left in a queue nobody reads.
                    drop(taken);
                    Err(refusal)
                }
                None => std::thread::Builder::new().name(String::from("cli-git-left-running-step")).spawn(move || {
                    // A refused start drops this receiver with the closure, and the child is handed back to the caller below.
                    let Ok(mut leaving) = taken.recv() else { return };
                    // The command may have stopped waiting; a closed channel is that case, and there is nobody to tell.
                    let _abandoned = sender.send(leaving.wait());
                }),
            };
            // A child dropped with a refused start would run on unreaped and stay a zombie of this process once it ended, so it is
            //  kept by whoever can still collect it: the thread above, or a reaper of its own where no thread could start.
            if let Err(std::sync::mpsc::SendError(unwaited)) = hand_over.send(child) {
                crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon(unwaited);
            }
            if let Err(error) = waiter {
                watch.end();
                return Err(std::io::Error::other(format!("{waiting_for} was started but {}", crate::scoped_start::refused_thread_sentence("wait for it", &error))));
            }
            match receiver.recv_timeout(after.saturating_add(extra)) {
                Ok(finished) => match finished {
                    Ok(status) => Ok(output_of(status, captured.as_ref())),
                    Err(error) => Err(error),
                },
                Err(_timeout_or_gone) => {
                    watch.end();
                    // Nobody will read what the step writes from here on, and it is held in memory until the step ends.
                    if let Some(files) = captured.as_ref() {
                        files.discard();
                    }
                    return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, NotKnown(not_known_text(waiting_for, after, extra))));
                }
            }
        }
        StepBound::EndAt(_) => match pipes {
            Some(open) => open.read_until_the_child_ends(child, waiting_for, carried),
            None => Err(std::io::Error::other("a step that is ended at its ceiling had no pipes to read: a defect of monochromatic-git-policy-cli")),
        },
        StepBound::Unbounded => {
            let mut waited_for: Child = child;
            match waited_for.wait() {
                Ok(status) => Ok(output_of(status, captured.as_ref())),
                Err(error) => Err(error),
            }
        }
    };
    watch.end();
    // The stop ended the step: the caller is told so, whatever the step's own end looked like.
    let outlived: Outlived = Outlived {
        ended_by_the_stop: watch.ended_by_the_stop(),
        ceiling,
        ceiling_hit: watch.ceiling_hit(),
        the_watch_could_end_it: watch.could_end_the_step(),
        expired: watch.expired(),
        limit: watch.limit(),
    };
    return match ending_of_a_watched_step(waiting_for, outlived) {
        Some(error) => Err(error),
        None => output,
    };
}
