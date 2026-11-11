//! What:
//!  The start of one package-manager program for a worktree repair:
//! run off the runtime's workers and bounded,
//!  and the child a late start must not leave running.
//! Why:
//!  `Command::spawn` returns only after the child's own `exec`
//!  (the standard library reads the pipe `exec` closes),
//!  and that `exec` can wait in the kernel with no end:
//! a program on a mount whose server stopped answering,
//!  a disk that stalls.
//! The owner's rule is "nothing should ever wait forever or indefinitely",
//! and the quiet window,
//!  the ceiling and the notice the run keeps are read from a clock it has not reached while that wait stands,
//!  so the start is the step that must be bounded itself.
//! The wait for a started tool is `package_repair_command.rs`'s;
//! this file is the step before it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const start = await startWithin(ceilingMs, command); // { child } | { error } | 'no answer'
//! ```

/// The job of the blocking pool a start runs on.
use crate::blocking_job::run_blocking_job;
/// The start that rides out a refused start of a child.
use crate::start_retry::RetryStart;
use std::process::{Child, Command};
use std::time::Duration;

/// What:
///  How the start of a tool ended.
/// Why:
///   A start that answers,
/// one the system refused,
/// and one that gave no answer are three different things to the person,
///  and only the last is the one no wait of the run could see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolStart = { kind: 'running'; child: Child } | { kind: 'refused'; error: string } | { kind: 'no-answer' };
/// ```
pub(crate) enum ToolStart {
    /// The tool started.
    Running(Child),
    /// The start could not be made,
    /// and no child of it runs.
    Refused(String),
    /// The start gave no answer within its ceiling;
    /// a child it starts later is stopped before it reaches the worktree.
    NoAnswer,
}

/// What:
///  Start the tool on the blocking pool,
/// and wait for the start no longer than `ceiling`.
/// Why:
///   The start waits for the child's own `exec`,
/// which can wait in the kernel with no end,
/// so the ceiling ends the wait for
///  the answer whether or not that `exec` ever comes.
///  The fork runs on a job of the blocking pool,
/// so a start that does not answer holds no worker of the
///  runtime:
/// the task beside it still runs the timer that ends this wait.
///  A child that appears after the ceiling is stopped (see `hand_over_or_stop`),
/// because the run has already
///  told the person that the tool did not start,
/// and nothing may rewrite the new worktree behind that answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const start = await startWithin(ceilingMs, command);
/// ```
pub(crate) async fn start_within(ceiling: Duration, mut command: Command) -> ToolStart {
    let (hand_over, answer) = tokio::sync::oneshot::channel::<Result<Child, std::io::Error>>();
    // The job is no part of this wait: the pool runs it to its own end, and a child it starts after the ceiling
    // finds the receiver gone and is stopped by `hand_over_or_stop`.
    let job = run_blocking_job(move || {
        match command.spawn_retrying() {
            Ok(started) => hand_over_or_stop(started, hand_over),
            Err(error) => {
                let _told = hand_over.send(Err(error));
            }
        }
    });
    match tokio::time::timeout(ceiling, job).await {
        // The system refused the thread the fork needed. The pool had already queued the job, so the closure may still
        // run there later; a child it starts then finds no receiver and is stopped.
        Ok(Err(refusal)) => return ToolStart::Refused(format!("no thread could start it ({refusal})")),
        Ok(Ok(())) => {}
        Err(_past_the_ceiling) => return ToolStart::NoAnswer,
    }
    return match answer.await {
        Ok(Ok(started)) => ToolStart::Running(started),
        Ok(Err(error)) => ToolStart::Refused(error.to_string()),
        // The job ended without handing a child over, which only a panic of it does.
        Err(_nothing_handed_over) => ToolStart::NoAnswer,
    };
}

/// What:
///  Hand a started child to the run that asked for it,
/// and stop it at once when that run has stopped waiting.
/// Why:
///   A child that appears after the ceiling would run in the new worktree with nobody reading it,
///   and the run has already said that the tool did not start.
///  Its whole process group is stopped,
/// so the helpers it starts end with
///  it,
/// and the leader goes to a reaper the command outlives,
/// because a child the kernel will not let go ends only when it does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverOrStop(child, handOver); // the run that waited is gone: kill(-pid) then reapSoon(child)
/// ```
fn hand_over_or_stop(started: Child, hand_over: tokio::sync::oneshot::Sender<Result<Child, std::io::Error>>) {
    // A send the receiver never takes gives the value back: the child this run must not leave running.
    let refused: Result<Child, std::io::Error> = match hand_over.send(Ok(started)) {
        Ok(()) => return,
        Err(answer) => answer,
    };
    let Ok(late) = refused else { return };
    stop_group(late.id());
    crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon(late);
}

/// What:
///  Stop the child's whole process group.
/// Why:
///   pnpm and Cargo start helpers;
/// stopping only the leader would leave them running against the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(-child.pid, 'SIGKILL');
/// ```
pub(crate) fn stop_group(leader_id: u32) {
    let Ok(leader) = libc::pid_t::try_from(leader_id) else { return };
    // SAFETY: `kill` takes a process-group id and a signal number and touches no memory; the group was made by this process.
    let _stopped = unsafe { libc::kill(-leader, libc::SIGKILL) };
}

#[cfg(test)]
thread_local! {
    /// A control holds the child before its own `exec` for this long,
    /// which is the only way a control makes a start itself take time:
    /// nothing outside
    ///  the child can pause it between the fork and the `exec`.
    /// The seam is read where the command is built,
    /// so a control sets it on the thread that enters the runtime.
    pub(crate) static HOLD_BEFORE_EXEC: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

/// What:
///  Install a control's hold before the child's own `exec`.
/// Why:
///   A start that does not answer is the defect this file bounds,
/// and a control cannot make one without pausing the child
///  between the fork and the `exec`,
/// which only the child itself can do.
///  Nothing is installed unless a control armed the seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdTheStartBeforeItsExec(command); // the seam is set by the control that needs a slow start
/// ```
#[cfg(test)]
pub(crate) fn hold_the_start_before_its_exec(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    let Some(held) = HOLD_BEFORE_EXEC.with(std::cell::Cell::get) else { return };
    // SAFETY: the hook only pauses the child before its own `exec` and touches nothing of this process, and
    // `nanosleep` is async-signal-safe, which is all a forked child of a threaded parent may call.
    unsafe {
        command.pre_exec(move || {
            let pause: libc::timespec = libc::timespec { tv_sec: libc::time_t::try_from(held.as_secs()).unwrap_or(0), tv_nsec: held.subsec_nanos() as libc::c_long };
            let _paused = libc::nanosleep(&pause, std::ptr::null_mut());
            return Ok(());
        });
    }
}
