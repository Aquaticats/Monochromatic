//! What:
//!  The watched one-shot run of `child_watch.rs`
//! for a caller that is itself a task of the runtime:
//! the child's start,
//! its output and its end are awaited,
//! so nothing of this wait blocks the worker that runs it.
//! Why:
//!  The owner's ruling is "Full on tokio.
//! Multi-thread."
//! (`doc/planning/git-policy-cli-full-tokio.md`),
//! and a task that awaits a child must never
//!  enter the runtime from inside it,
//! where waiting for the runtime would never end
//! (`lazy_runtime::run_on_runtime`,
//! `RuntimeError::Nested`).
//! The synchronous face of this run
//!  (`child_watch.rs`,
//! `output_watched_within_for_a_stop`) enters the runtime around the same steps,
//! so it can be called only from outside the runtime;
//!  this file is the same run with the entering taken out.
//! Both faces word the endings through `ending_of_a_watched_step`
//!  (`child_watch_step_bounds_and_errors.rs`),
//! so a task and a synchronous caller cannot report a step differently.
//! The watch,
//! the lock ledger and the service's stop are shared with the synchronous face
//! unchanged:
//! the watch is already a task of the runtime (`ChildWatch::start_under_stop`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = await spawnWatchedOnTheRuntime(command, { waitingFor, ledgerStep, terms, stoppable, bound, aStopWaits });
//! ```

/// The watch of one child,
/// its terms and the wording of its endings.
use crate::child_watch::{ChildWatch, WatchTerms};
/// The bound of a step and the errors of its endings.
use crate::child_watch_step_bounds_and_errors::{NotKnown, Outlived, StepBound, ending_of_a_watched_step, not_known_text, output_of, refused_while_stopping};
/// The captured output of a step that is never ended,
/// in files that outlive the command.
use crate::output_capture::Capture;
/// The child the wrapper adopted and the pidfd its end is awaited through.
use crate::child_awaited_through_a_pidfd::adopt;
/// Whether a task can run on this process's runtime at all,
/// which is the question the rest of this run asks.
use crate::lazy_runtime::tasks_can_run;
/// Starts that ride out a short shortage of processes,
/// without holding the worker.
use crate::start_retry::spawn_retrying_without_blocking;
/// The pipes of a step that is ended at its ceiling,
/// read to their end on the runtime.
use crate::step_output_read_whole_through_pipes::{StepPipes, attach_pipes_awaiting, release_write_ends};
/// The lock files a stopped step leaves.
use crate::stopped_git_locks::LockLedger;
/// The count and the arm of the work the service's shutdown waits for.
use crate::stop_of_the_service_noticed_by_its_children::{StopArm, Unfinished, begin_unfinished_work};
/// The child process types of the one-shot run.
use std::process::{Child, Command, ExitStatus, Output};
/// A span of time,
/// for the ceiling and the left-running wait.
use std::time::Duration;

/// What:
///  The output of the step `child` runs,
/// or the error of a step that was left running and whose result is therefore not known.
///  `captured` is where its output went when it was not read through pipes.
/// Why:
///   A write that cannot be run twice is never ended (owner,
/// 2026-11-04:
/// "Never cut a branch write (Recommended)"):
/// the command stops
///  waiting at its bound plus the short extra wait and says the result is not known,
/// and the step goes on.
///  The wait for the child is a task of the runtime,
/// not a thread of its own as in the synchronous face,
/// so
///  a stop of the command leaves nothing blocked;
/// where no task can run (a runtime with no worker of its own),
/// the wait is made
///  with the runtime's timer and the child is left to run unreaped,
/// which ends the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await Promise.race([child.exited, delay(after + extra).then(() => LEFT_RUNNING)]);
/// if (status === LEFT_RUNNING) throw new NotKnown(notKnownText(waitingFor, after, extra));
/// ```
async fn output_of_a_step_left_running(
    child: Child,
    captured: Option<&Capture>,
    waiting_for: &str,
    after: Duration,
    extra: Duration,
) -> std::io::Result<Output> {
    let limit: Duration = after.saturating_add(extra);
    // `mut` lets the wait take the child to itself.
    let mut owned = adopt(child);
    // The question is whether a task can run now, not how the runtime was first built: a build that missed its deadline can
    //  have arrived since, and its workers take this child. Asking the first build left a step that was deliberately left
    //  running to a drop that never collects it (the fifth review's finding 4).
    let waiting: bool = tasks_can_run();
    let finished: std::io::Result<ExitStatus> = if waiting {
        // The child moves to a task that waits for it and is reaped there, so this command can stop waiting and the child goes on.
        let (sender, receiver) = tokio::sync::oneshot::channel::<std::io::Result<ExitStatus>>();
        let handed_over = crate::lazy_runtime::spawn_on_runtime(async move {
            let status: std::io::Result<ExitStatus> = owned.wait().await;
            // The command may have stopped waiting; a closed channel is that case, and there is nobody to tell.
            let _abandoned = sender.send(status);
        });
        if handed_over.is_none() {
            // Nothing took the child, and the future that held it was dropped, so its drop ended it; that is a defect rather than a state.
            return Err(std::io::Error::other(format!(
                "{waiting_for} was started but the runtime had no task to wait for it: this is a defect of monochromatic-git-policy-cli"
            )));
        }
        match tokio::time::timeout(limit, receiver).await {
            Ok(Ok(status)) => return match status {
                Ok(finished) => Ok(output_of(finished, captured)),
                Err(error) => Err(error),
            },
            Ok(Err(_gone)) => {
                return Err(std::io::Error::other(format!(
                    "{waiting_for} was started and the task waiting for it ended without its end: this is a defect of monochromatic-git-policy-cli"
                )));
            }
            Err(_elapsed) => Err(std::io::Error::new(std::io::ErrorKind::TimedOut, NotKnown(not_known_text(waiting_for, after, extra)))),
        }
    } else {
        match tokio::time::timeout(limit, owned.wait()).await {
            Ok(status) => status,
            // Nobody waits for the child from here on, and it is left running on purpose: it is handed to a reaper, which
            //  collects its end whenever the system lets it go, so a step that outlives this command is never a zombie of it.
            Err(_elapsed) => {
                if let Some(unwaited) = owned.into_std_when_unreaped() {
                    crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon(unwaited);
                }
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, NotKnown(not_known_text(waiting_for, after, extra))));
            }
        }
    };
    // Nobody will read what the step writes from here on when it was left running; that arm returned above.
    return match finished {
        Ok(status) => Ok(output_of(status, captured)),
        Err(error) => Err(error),
    };
}

/// What:
///  `child_watch::output_watched_within_for_a_stop` as the future a task of the runtime awaits:
/// the command is started,
/// announced and watched,
///  and its output read to its end,
/// with every wait a wait of the runtime.
/// Why:
///   A caller that awaits a Git step
/// (`transaction/transaction_git_awaiting.rs`) must not enter the runtime from inside it,
/// and the start,
///  the pipe read and the wait for the child are the three steps that would have done so.
/// Everything else is the synchronous face's:
///  the pipes are made before the child starts,
/// so a refusal starts nothing,
/// and the service's stop,
/// the lock ledger and
///  the quiet window behave the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await spawnWatchedOnTheRuntime(command, { waitingFor, ledgerStep, terms, stoppable, bound, aStopWaits });
/// ```
pub async fn output_watched_within_for_a_stop_awaiting(
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
        StepBound::EndAt(_) => (None, Some(attach_pipes_awaiting(command).await?)),
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
    let spawned: std::io::Result<Child> = spawn_retrying_without_blocking(command).await;
    // The command holds the write ends of the pipes until they are replaced, and the readers see no end of output while it does.
    if pipes.is_some() {
        release_write_ends(command);
    }
    let child: Child = spawned?;
    // A step that a stop lets finish keeps its count and takes no arm, so nothing ends it when the stop is told.
    let arm: Option<StopArm> = if a_stop_lets_it_finish { None } else { unfinished.take_arm() };
    let started: ChildWatch = ChildWatch::start_under_stop(waiting_for, child.id(), terms, locks, arm).kept_as(ledger_step);
    // A step with no ceiling is waited for as long as it lives: the watch starts without one.
    // `mut` lets the watch be closed here rather than dropped, which would wait for its task on this worker.
    let mut watch: ChildWatch = match ceiling {
        Some(limit) => started.within(limit),
        None => started,
    };
    if !stoppable {
        watch.never_stop();
    }
    // Nothing watches this step, so this wait is what ends it at its ceiling; a step with no ceiling is waited for as before.
    let carried: Option<crate::step_output_read_whole_through_pipes::CarriedCeiling> =
        crate::step_output_read_whole_through_pipes::carried_for(ceiling, watch.unwatched().is_none());
    watch.begin();
    // Whether the step was left running, so what it writes from here on is read by nobody.
    let mut left_running: bool = false;
    let output: std::io::Result<Output> = match bound {
        StepBound::LeaveRunning { after, extra } => match output_of_a_step_left_running(child, captured.as_ref(), waiting_for, after, extra).await {
            Ok(finished) => Ok(finished),
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
                left_running = true;
                Err(error)
            }
            Err(error) => Err(error),
        },
        StepBound::EndAt(_) => match pipes {
            Some(open) => open.read_until_the_child_ends_awaiting(child, waiting_for, carried).await,
            None => Err(std::io::Error::other("a step that is ended at its ceiling had no pipes to read: a defect of monochromatic-git-policy-cli")),
        },
        StepBound::Unbounded => {
            let mut waited_for = adopt(child);
            match waited_for.wait().await {
                Ok(status) => Ok(output_of(status, captured.as_ref())),
                Err(error) => Err(error),
            }
        }
    };
    watch.end();
    // Nobody will read what a step that was left running writes from here on, and it is held in memory until the step ends.
    if left_running
        && let Some(files) = captured.as_ref()
    {
        files.discard();
    }
    // The stop ended the step: the caller is told so, whatever the step's own end looked like.
    let outlived: Outlived = Outlived {
        ended_by_the_stop: watch.ended_by_the_stop(),
        ceiling,
        ceiling_hit: watch.ceiling_hit(),
        expired: watch.expired(),
        limit: watch.limit(),
    };
    let ending: Option<std::io::Error> = ending_of_a_watched_step(waiting_for, outlived);
    // A watch dropped on a worker asks its task to end and then waits there; this asks the same and awaits it instead.
    watch.close_awaiting().await;
    return match ending {
        Some(error) => Err(error),
        None => output,
    };
}

/// Watched-run controls for a caller that is a task stay out of the release executable.
#[cfg(test)]
#[path = "child_watch_output_watched_on_the_runtime_tests.rs"]
mod tests;
