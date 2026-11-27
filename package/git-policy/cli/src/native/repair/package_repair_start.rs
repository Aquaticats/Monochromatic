//! What:
//!  The bounded start of one package-manager program for a worktree repair,
//! run off the runtime's workers,
//! and the stop of a child that appears after the bound.
//! Why:
//!  `Command::spawn` returns only after the child's `exec`,
//! and an `exec` can wait in the kernel with no end,
//! so a start that waited for it would hold the add while no bound runs.
//! The child therefore reports itself before its own `exec` and waits there for the run to let it go,
//! and a run that gives up closes that gate:
//! the tool never runs,
//!  whatever moment the kernel finally lets its `exec` through.
//! The wait for a started tool is `package_repair_command.rs`'s;
//! this file is the step before it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const start = await startWithin(ceilingMs, command); // { child } | { error } | 'no answer' | 'stopped while starting'
//! ```
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_start_gate.rs`:
//!   the gate the child waits at,
//!   the report that names it,
//!   and the stop of the group it leads.

/// Import the job of the blocking pool a start runs on.
use crate::blocking_job::run_blocking_job;
/// Import the gate the child waits at,
/// the report that names it,
/// the hold a control arms,
/// and the stop of the group it leads.
pub(crate) use crate::package_repair_start_gate::stop_group;
pub(crate) use crate::package_repair_start_gate::{
    TheChildsStart,
    TheStartGate,
    TheStartsHolds,
};
/// Import the start that rides out a refused start of a child.
use crate::start_retry::RetryStart;
/// Import the thread that collects a child no wait of the run will reach.
use crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon;
/// Import the child process handle the standard library starts,
/// and the command it is started from.
use std::process::{
    Child,
    Command,
};
/// Import the spans of time a bound and a hold are written as.
use std::time::{
    Duration,
    Instant,
};
/// What:
///  How the start of a tool ended.
/// Why:
///   A start that answers,
/// one the system refused,
/// one that gave no answer,
///  and one whose child was let go and then did not come up are four different things to the person,
///  and only the third promises that nothing of the tool ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolStart = { kind: 'running'; child: Child } | { kind: 'refused'; error: string } | { kind: 'no-answer' } | { kind: 'let-go' };
/// ```
pub(crate) enum ToolStart {
    /// The tool started,
    /// and the run supervises it from here.
    Running(AChildNotYetTaken),
    /// The start could not be made,
    /// and no child of it runs.
    Refused(String),
    /// No child of it reported itself within the ceiling,
    /// so nothing of the tool ran and nothing of it can:
    /// the gate is closed when this answer is given,
    /// and a child forked after that leaves before its own `exec`.
    NoAnswer,
    /// The child was let go and its own `exec` gave no answer within the ceiling;
    /// it was stopped,
    ///  and whatever it wrote before that stands.
    LetGoWithoutAnAnswer,
}

/// What:
///  A child the wrapper started and no wait of the run has taken yet.
/// Why:
///   A start that hands its child through a channel can lose it:
/// the value is dropped when the run stops waiting,
///  and a dropped `Child` neither kills nor reaps the process,
/// so the tool would keep rewriting the new worktree with no owner and no bound.
///  Whatever way this value is dropped, the group the child leads is stopped and the child itself is collected by a
///  reaper, so no path out of a start leaves a package tool running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AChildNotYetTaken { [Symbol.dispose]() { kill(-pid, 'SIGKILL'); reapSoon(pid); } taken() { return this.child; } }
/// ```
pub(crate) struct AChildNotYetTaken {
    /// The child,
    /// until the run takes it.
    child: Option<Child>,
}

impl AChildNotYetTaken {
    /// What:
    ///  Keep `child` until the run that started it takes it,
    ///  or stop it with the group it leads when nothing does.
    /// Why:
    ///   The value that arrives from the start is what stands between a running package tool and a run that stopped
    ///  waiting for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const notYetTaken = new AChildNotYetTaken(child);
    /// ```
    pub(crate) fn new(child: Child) -> Self {
        return Self {
            child: Some(child),
        };
    }

    /// What:
    ///  The child, for the wait that supervises it from here.
    /// Why:
    ///   Once the run holds the child it owns ending and reaping it,
    ///  so this value has nothing left to stop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const child = notYetTaken.taken();
    /// ```
    pub(crate) fn taken(mut self) -> Child {
        return self
            .child
            .take()
            .expect("a child not yet taken is taken once, and this value stops it otherwise");
    }
}

impl Drop for AChildNotYetTaken {
    /// Stops the whole group the child leads, then hands the child to a reaper,
    /// so a run that gave up on a start leaves nothing of the tool running.
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            stop_group(child.id());
            reap_soon(child);
        }
    }
}

/// What:
///  Start the tool on the blocking pool,
/// wait for the child to report itself no longer than `ceiling`,
/// let it go,
///  and wait for its own `exec` no longer than what is left of that ceiling.
/// Why:
///   The start waits for the child's own `exec`,
/// which can wait in the kernel with no end,
/// so the ceiling ends the wait for
///  an answer whether or not that `exec` ever comes.
///  The fork runs on a job of the blocking pool,
/// so a start that does not answer holds no worker of the
///  runtime:
/// the task beside it still runs the timer that ends this wait.
///  The child cannot run before this run lets it go,
/// so an answer that says nothing of the tool ran is true
///  even when the kernel only reaches the child's `exec` after the ceiling,
/// and the child a later `exec` would have produced is stopped through the
///  descriptor the kernel made for it at its report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const start = await startWithin(ceilingMs, command);
/// ```
pub(crate) async fn start_within(
    ceiling: Duration,
    mut command: Command,
) -> ToolStart {
    let began: Instant = Instant::now();
    // A control holds the child before it reports itself, or after the run lets it go, which are the two places a start
    // can be made to take time; the run that ships holds neither. A control can also name the file its fixture's tool
    // writes first, which is where the run's own answer waits for that write; the run that ships names none.
    #[cfg(test)]
    let held_before_the_report: Option<Duration> = HOLD_BEFORE_EXEC.with(std::cell::Cell::get);
    #[cfg(not(test))]
    let held_before_the_report: Option<Duration> = None;
    #[cfg(test)]
    let held_after_the_let_go: Option<Duration> = HOLD_AFTER_THE_LET_GO.with(std::cell::Cell::get);
    #[cfg(not(test))]
    let held_after_the_let_go: Option<Duration> = None;
    let (mut gate, the_childs_start): (TheStartGate, TheChildsStart) =
        match TheStartGate::made(TheStartsHolds {
            before_the_report: held_before_the_report,
            after_the_let_go: held_after_the_let_go,
        }) {
            Ok(made) => made,
            Err(error) => {
                return ToolStart::Refused(format!("no place to watch the start ({error})"));
            }
        };
    the_childs_start.arms(&mut command);
    // The job is no part of the runtime's workers, and the value it returns is the child: a child nothing takes is kept
    // by `AChildNotYetTaken` until the run's answer, and stopped when that answer is given without it.
    // A control holds the job after a successful start, which is the interval the audit of 0.6.1-alpha.12 forced: the
    // child exists and the run is still waiting for the value that owns it.
    #[cfg(test)]
    let held_after_the_start: Option<Duration> = HOLD_AFTER_THE_START.with(std::cell::Cell::get);
    #[cfg(not(test))]
    let held_after_the_start: Option<Duration> = None;
    let dispatched = run_blocking_job(move || -> std::io::Result<AChildNotYetTaken> {
        let started: Child = command.spawn_retrying()?;
        if let Some(held) = held_after_the_start {
            std::thread::sleep(held);
        }
        return Ok(AChildNotYetTaken::new(started));
    });
    let mut job = std::pin::pin!(dispatched);
    // The child's own report is the start's answer, and it comes before the tool could run at all. At most one report
    // reaches the gate: a retried start is one the system refused before any child existed, so the hook that reports has
    // run no more than once.
    let reported: bool = tokio::select! {
        // The job ended before a child reported itself: the pool refused a thread, the system refused the fork, or the
        // child left before its own `exec` and nothing of the tool ran.
        ended = &mut job => {
            return match ended {
                Ok(Ok(never_taken)) => {
                    drop(never_taken);
                    ToolStart::Refused(String::from("the start ended before the child reported itself"))
                }
                Ok(Err(error)) => ToolStart::Refused(error.to_string()),
                Err(refusal) => ToolStart::Refused(format!("the pool could not make its start ({refusal})")),
            };
        }
        reported = gate.the_child_within(ceiling) => reported,
    };
    if !reported {
        // Nothing of the tool ran, and nothing of it will: the gate is closed when this function returns, and a child
        // forked after that leaves before its own `exec`.
        return ToolStart::NoAnswer;
    }
    if let Err(error) = gate.let_the_child_go() {
        // The child is gone, so its end of the gate is closed: nothing of the tool can run.
        return ToolStart::Refused(format!("the child could not be let go ({error})"));
    }
    // What: A control names the file its fixture's tool writes its own process number into first, and the run does not
    //       give up on the child before that number is there.
    // Why:  The tool's own `exec` and its first write take time, and a control that reads that write to see what the tool
    //       did has no way to make them fast: on a loaded host the write lost a race against the bound that ends the
    //       start, and the control failed on evidence the run never gave it. Waiting here forces the scene such a control
    //       names (a child let go whose start outlasts the bound) in every order the fixture's own start takes.
    //       The run that ships names no file.
    #[cfg(test)]
    if let Some(number_file) = WAIT_FOR_THE_TOOLS_NUMBER.with(|seam| return seam.borrow().clone()) {
        wait_for_the_number_in_before_giving_up(number_file).await;
    }
    let left_of_the_ceiling: Duration = ceiling.saturating_sub(began.elapsed());
    return match tokio::time::timeout(left_of_the_ceiling, job).await {
        Ok(Ok(Ok(taken))) => ToolStart::Running(taken),
        Ok(Ok(Err(error))) => ToolStart::Refused(error.to_string()),
        Ok(Err(refusal)) => {
            ToolStart::Refused(format!("the pool could not make its start ({refusal})"))
        }
        Err(_past_the_ceiling) => {
            // The child was let go and its own `exec` did not answer: it is stopped through the descriptor the kernel made
            // for it at its report, and the job that started it keeps the child it is still waiting for.
            gate.stop_the_reported_child();
            ToolStart::LetGoWithoutAnAnswer
        }
    };
}

#[cfg(test)]
thread_local! {
    /// A control holds the child before it reports itself for this long,
    /// which makes a start give no answer at all:
    /// nothing outside the child can pause it between the
    ///  fork and its own report.
    /// The seam is read on the thread that enters the runtime,
    /// so a control sets it there.
    pub(crate) static HOLD_BEFORE_EXEC: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
thread_local! {
    /// A control holds the child after the run let it go and before its own `exec` for this long,
    /// which makes a start whose own `exec` does not answer.
    pub(crate) static HOLD_AFTER_THE_LET_GO: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
thread_local! {
    /// A control holds the job that started the child for this long after a successful start,
    /// which is the interval between the start and the run's answer,
    /// and the one the audit of
    ///  0.6.1-alpha.12 forced.
    pub(crate) static HOLD_AFTER_THE_START: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
thread_local! {
    /// The file a control's fixture tool writes its own process number into first,
    /// which the run does not give up on the child before:
    /// the tool's own first write then never races the bound that ends its start.
    /// The seam is read on the thread that enters the runtime,
    /// so a control sets it there,
    /// and the run that ships names no file.
    pub(crate) static WAIT_FOR_THE_TOOLS_NUMBER: std::cell::RefCell<Option<std::path::PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// How long a control's fixture has to write the number the run waits for:
/// long enough for a loaded host,
/// and short enough that a fixture that never writes still answers.
#[cfg(test)]
const THE_TOOLS_FIRST_WRITE_WAIT: Duration = Duration::from_secs(10);

/// The pause between looks at the file a control's fixture writes its number into,
/// on the runtime's own clock,
/// so no worker of it is held.
#[cfg(test)]
const THE_TOOLS_FIRST_WRITE_PAUSE: Duration = Duration::from_millis(2);

/// What:
///  Hold the run's own answer until a control's fixture has written its process number into `number_file`,
///  or until this wait's own bound.
/// Why:
///   A control forces its scene by naming the file its fixture's tool writes first:
/// the run does not give up on the child before that write landed,
///  so the tool's own start never loses a race against the bound.
///  A tool that never writes leaves this wait at its own bound,
/// and the control's own read of the number then says what happened.
///  Each look is a job of the blocking pool,
/// so neither the read nor a slow disk holds a worker of the runtime,
///  and a pool that refused a thread leaves the look queued with its answer unknown,
/// so the look is asked for again rather than skipped:
/// a skipped look would put the tool's first write back in a race with the bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const until = Date.now() + 10_000; Date.now() < until; await sleep(2)) {
///   if ((await runBlockingJob(() => numberIn(pidFile))) !== undefined) return;
/// }
/// ```
#[cfg(test)]
async fn wait_for_the_number_in_before_giving_up(number_file: std::path::PathBuf) {
    let waiting_since: Instant = Instant::now();
    while waiting_since.elapsed() < THE_TOOLS_FIRST_WRITE_WAIT {
        let looking_at: std::path::PathBuf = number_file.clone();
        let looked =
            run_blocking_job(move || return the_number_written_to(looking_at.as_path())).await;
        if let Ok(Some(_number)) = looked {
            return;
        }
        // The file is not there yet, or a refused thread left the look unanswered: either way the look is asked for again.
        tokio::time::sleep(THE_TOOLS_FIRST_WRITE_PAUSE).await;
    }
    return;
}

/// What:
///  The process number a control's fixture wrote to `path`,
///  when the file holds one whole number.
/// Why:
///   A stop that landed between the write of a fixture's own open and its number would leave an empty file,
/// so only a number that parses whole ends a control's wait:
/// a prefix of one would name a process the control never started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whole = Number.parseInt(readFileSync(path, 'utf8'), 10); return Number.isNaN(whole) ? undefined : whole;
/// ```
#[cfg(test)]
fn the_number_written_to(path: &std::path::Path) -> Option<i32> {
    let text: String = std::fs::read_to_string(path).ok()?;
    return text.trim().parse::<i32>().ok();
}
