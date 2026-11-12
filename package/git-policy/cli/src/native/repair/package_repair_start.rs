//! What:
//!  The start of one package-manager program for a worktree repair:
//! run off the runtime's workers and bounded,
//!  and the child such a start must never let run behind the run's own answer.
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
//!  Bounding the wait alone is not enough:
//! the finding that ends a start the run gave up on already told the person that the tool did not start and named the
//!  command to run by hand,
//! so nothing of the tool may have run.
//! A tool that reaches its own files while the run gives up has already opened,
//!  truncated or replaced them,
//! and no kill that lands after its first write restores what it wrote.
//!  The child therefore reports itself before its own `exec` and waits there for the run to let it go,
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

/// The job of the blocking pool a start runs on.
use crate::blocking_job::run_blocking_job;
/// The descriptor a stop names a process by,
/// and the signal it sends through it.
use crate::member_signalled_through_its_pidfd::{Delivered, pidfd_of, signal_through};
/// The start that rides out a refused start of a child.
use crate::start_retry::RetryStart;
/// The thread that collects a child no wait of the run will reach.
use crate::stuck_children_reaped_when_the_kernel_lets_go::reap_soon;
use std::io::Write as _;
use std::os::fd::{AsRawFd as _, OwnedFd, RawFd};
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt as _;
use tokio::net::unix::pipe::Receiver;

/// The one byte the run writes when the child may go on to its own `exec`.
const THE_RUN_LETS_THE_CHILD_GO: u8 = b'g';

/// How many bytes the child writes to say which process it is:
/// its own number,
/// in a form both sides of the fork read the same way.
const REPORTED_BYTES: usize = size_of::<i32>();

/// The status a child leaves with when the run did not let it go:
/// nothing of the tool has run at that point.
const EXITED_BEFORE_ITS_EXEC: i32 = 1;

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
        return Self { child: Some(child) };
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
        return self.child.take().expect("a child not yet taken is taken once, and this value stops it otherwise");
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
pub(crate) async fn start_within(ceiling: Duration, mut command: Command) -> ToolStart {
    let began: Instant = Instant::now();
    // A control holds the child before it reports itself, or after the run lets it go, which are the two places a start
    // can be made to take time; the run that ships holds neither.
    #[cfg(test)]
    let held_before_the_report: Option<Duration> = HOLD_BEFORE_EXEC.with(std::cell::Cell::get);
    #[cfg(not(test))]
    let held_before_the_report: Option<Duration> = None;
    #[cfg(test)]
    let held_after_the_let_go: Option<Duration> = HOLD_AFTER_THE_LET_GO.with(std::cell::Cell::get);
    #[cfg(not(test))]
    let held_after_the_let_go: Option<Duration> = None;
    let (mut gate, the_childs_start): (TheStartGate, TheChildsStart) =
        match TheStartGate::made(TheStartsHolds { before_the_report: held_before_the_report, after_the_let_go: held_after_the_let_go }) {
            Ok(made) => made,
            Err(error) => return ToolStart::Refused(format!("no place to watch the start ({error})")),
        };
    the_childs_start.arms(&mut command);
    // The job is no part of the runtime's workers, and the value it returns is the child: a child nothing takes is kept
    // by `AChildNotYetTaken` until the run's answer, and stopped when that answer is given without it.
    let dispatched = run_blocking_job(move || -> std::io::Result<AChildNotYetTaken> {
        let started: Child = command.spawn_retrying()?;
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
    let left_of_the_ceiling: Duration = ceiling.saturating_sub(began.elapsed());
    return match tokio::time::timeout(left_of_the_ceiling, job).await {
        Ok(Ok(Ok(taken))) => ToolStart::Running(taken),
        Ok(Ok(Err(error))) => ToolStart::Refused(error.to_string()),
        Ok(Err(refusal)) => ToolStart::Refused(format!("the pool could not make its start ({refusal})")),
        Err(_past_the_ceiling) => {
            // The child was let go and its own `exec` did not answer: it is stopped through the descriptor the kernel made
            // for it at its report, and the job that started it keeps the child it is still waiting for.
            gate.stop_the_reported_child();
            ToolStart::LetGoWithoutAnAnswer
        }
    };
}

/// What:
///  The two pipes a start is decided through,
/// and the child that reported itself on the first of them.
/// Why:
///  `Command::spawn` returns only after the child's `exec`,
/// so without this the run learns nothing of the child until the tool could
///  already have run.
/// The child reports its own number through one pipe and waits at the other,
/// so the run decides whether the tool ever
///  runs,
/// and a tool the run gave up on never ran at all.
///  The run's ends are the only copies its two pipes have on its side,
/// so closing them is what ends a child that waited:
/// every end the
///  child holds is closed in it by the kernel at its own `exec`,
/// and otherwise when this gate is dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = StartGate.make(); gate.arm(command); if (!await gate.theChildWithin(ms)) return 'no answer'; gate.letItGo();
/// ```
struct TheStartGate {
    /// What the child reports its own number on.
    report: Receiver,
    /// What the run writes the one byte to that lets the child go on to its own `exec`.
    release: std::io::PipeWriter,
    /// The child that reported itself:
    /// its own number,
    /// and the descriptor the kernel made for it at that moment.
    reported: Option<TheReportedChild>,
}

/// The child that reported itself at the gate.
struct TheReportedChild {
    /// The number the child wrote.
    number: i32,
    /// The descriptor that names that process for good,
    /// when the kernel made one.
    descriptor: Option<OwnedFd>,
}

impl TheStartGate {
    /// What:
    ///  Make the gate,
    ///  and the ends the child keeps of it,
    /// or say why the system refused one.
    /// Why:
    ///   The gate is made before the fork,
    /// so a failure here starts nothing at all,
    ///  and the read end is registered with the reactor the wait for the child runs on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const [reportReader, reportWriter] = pipe(); const [releaseReader, releaseWriter] = pipe();
    /// ```
    fn made(holds: TheStartsHolds) -> std::io::Result<(TheStartGate, TheChildsStart)> {
        let (report, report_writer): (std::io::PipeReader, std::io::PipeWriter) = std::io::pipe()?;
        let (release_reader, release): (std::io::PipeReader, std::io::PipeWriter) = std::io::pipe()?;
        // The ends a child keeps are its own, and the ends the run keeps are closed in it, so the child's wait for the
        // release sees the end of its pipe when this gate is dropped.
        let the_runs_ends: [RawFd; 2] = [report.as_raw_fd(), release.as_raw_fd()];
        let watching: Receiver = Receiver::from_owned_fd(OwnedFd::from(report))?;
        let gate: TheStartGate = TheStartGate { report: watching, release, reported: None };
        let the_childs_start: TheChildsStart = TheChildsStart { report: report_writer, release: release_reader, the_runs_ends, holds };
        return Ok((gate, the_childs_start));
    }

    /// What:
    ///  The child's own number,
    /// awaited no longer than `ceiling`,
    ///  with the descriptor that names that process for good.
    /// Why:
    ///   A report that does not come is the start that gave no answer,
    /// and it is what keeps the tool from ever running.
    /// The descriptor is made here,
    ///  at the report,
    /// while this process is still the child's parent and its number cannot have been reused,
    /// so a stop of a child that
    ///  never comes up reaches that child and never a process that took its number.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const reported = await withTimeout(ceilingMs, readExactly(gate.report, 4)); gate.reported = { pid: decode(reported) };
    /// ```
    async fn the_child_within(&mut self, ceiling: Duration) -> bool {
        let mut reported: [u8; REPORTED_BYTES] = [0; REPORTED_BYTES];
        // A read that leaves bytes of the report unread is the start that never said which process it is, which is no answer.
        let Ok(Ok(REPORTED_BYTES..)) = tokio::time::timeout(ceiling, self.report.read_exact(&mut reported)).await else {
            return false;
        };
        let number: i32 = i32::from_le_bytes(reported);
        let descriptor: Option<OwnedFd> = pidfd_of(number);
        self.reported = Some(TheReportedChild { number, descriptor });
        return true;
    }

    /// What:
    ///  Let the child go on to its own `exec`.
    /// Why:
    ///   The start was answered in time,
    /// so the tool is this run's to supervise from here,
    ///  and the one byte is what its own `exec` waited for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// gate.release.write('g');
    /// ```
    fn let_the_child_go(&mut self) -> std::io::Result<()> {
        return self.release.write_all(&[THE_RUN_LETS_THE_CHILD_GO]);
    }

    /// What:
    ///  Stop the child that reported itself, and every helper it started.
    /// Why:
    ///   A child that was let go and whose own `exec` did not answer is the one no wait of the run reached,
    /// so neither it nor anything it started may run any further.
    ///  The child leads the group it was started in,
    /// so the group's stop ends it and its helpers together,
    /// and the number is safe here
    ///  because the child is still this process's own unreaped one,
    /// which the kernel does not hand to another process until it is reaped.
    /// The
    ///  descriptor made at its report names that same child for good,
    /// so the leader is signalled through it too wherever the kernel made one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stopGroup(reportedChild.pid); signalThrough(reportedChild.descriptor, 'SIGKILL');
    /// ```
    fn stop_the_reported_child(&mut self) {
        let Some(reported) = self.reported.take() else {
            return;
        };
        // The child leads the group it was started in, so the group's stop ends it and every helper it started in the
        // moment before its own `exec` was asked for. This is the one place a group is named by a number, and the number
        // is this process's own unreaped child here, which the kernel does not hand to another process until it is reaped.
        if let Ok(leader) = u32::try_from(reported.number) {
            stop_group(leader);
        }
        if let Some(descriptor) = reported.descriptor.as_ref() {
            let _delivered: Delivered = signal_through(descriptor, libc::SIGKILL);
        }
    }
}

/// What:
///  The two places a control can hold a start,
/// both of which the run that ships leaves empty.
/// Why:
///   A start that takes time is the defect this file bounds,
/// and only a hold inside the child, or on the thread that waits for it, makes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheStartsHolds = { beforeTheReport?: number; afterTheLetGo?: number };
/// ```
struct TheStartsHolds {
    /// How long the child waits before it reports itself.
    before_the_report: Option<Duration>,
    /// How long the child waits after the run let it go and before its own `exec`.
    after_the_let_go: Option<Duration>,
}

/// What:
///  The child's own part of a start:
/// the ends it holds of the gate,
///  the ends the run holds that it closes in itself,
///  and the number it reports.
/// Why:
///   Only the child can say that it exists and wait before its own `exec`,
/// and it must say it with calls a forked child of a threaded process may make:
/// `write`, `read`, `close`,
///  `getpid`, `nanosleep` and `_exit`, and nothing that allocates or locks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // There is no TS counterpart of the forked child's own hook: it is a few syscalls before `exec`.
/// child.preExec(() => { report(getpid()); await oneByte(release); });
/// ```
struct TheChildsStart {
    /// The end the child reports its own number through.
    report: std::io::PipeWriter,
    /// The end the child waits at until the run lets it go.
    release: std::io::PipeReader,
    /// The ends the run keeps,
    /// closed in the child so the child's own wait sees the end of its pipe.
    the_runs_ends: [RawFd; 2],
    /// The holds a control armed.
    holds: TheStartsHolds,
}

impl TheChildsStart {
    /// Installs the child's own part of the start on `command`.
    fn arms(self, command: &mut Command) {
        use std::os::unix::process::CommandExt as _;
        let childs_start: TheChildsStart = self;
        // SAFETY: the hook reports this child's own number and waits for one byte, with calls that only touch the child's
        // own descriptors and its own process: `write`, `read`, `close`, `getpid`, `nanosleep` and `_exit` are all
        // async-signal-safe, which is every call a forked child of a threaded parent may make.
        unsafe {
            command.pre_exec(move || {
                childs_start.part_of_the_start();
                return Ok(());
            });
        }
    }

    /// What:
    ///  Report this child's own number and wait for the run's one byte,
    /// or leave before the tool's own `exec`.
    /// Why:
    ///   The child that waits here is a tool that has not run,
    /// so the run may still answer that nothing of it ran,
    ///  and a child that leaves here leaves nothing behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // reported === 4 || _exit(1); const go = read(release); go === 'g' || _exit(1);
    /// ```
    fn part_of_the_start(&self) {
        pause_for(self.holds.before_the_report);
        // SAFETY: `getpid` reads this process's own number and touches no memory.
        let number: i32 = unsafe { libc::getpid() };
        let reported: [u8; REPORTED_BYTES] = number.to_le_bytes();
        for end in self.the_runs_ends {
            // SAFETY: `close` only releases a descriptor of this forked child's own table.
            let _closed: i32 = unsafe { libc::close(end) };
        }
        // SAFETY: `write` hands four bytes of this child's own stack to a pipe it inherited, and touches no other memory.
        let written: isize = unsafe { libc::write(self.report.as_raw_fd(), reported.as_ptr().cast(), REPORTED_BYTES) };
        if written != REPORTED_BYTES as isize {
            // The run stopped waiting for this start, so nothing of the tool may run. The standard library sets `SIGPIPE`
            // back to its default before this hook (`sys/process/unix/unix.rs`, the block above the closures), so a pipe
            // with no reader ends this child by that signal rather than through this branch; both are before its `exec`.
            // SAFETY: `_exit` ends this forked child and returns to nothing.
            unsafe { libc::_exit(EXITED_BEFORE_ITS_EXEC) };
        }
        let mut released: u8 = 0;
        // SAFETY: `read` fills one byte of this child's own stack from a pipe it inherited.
        let read: isize = unsafe { libc::read(self.release.as_raw_fd(), (&raw mut released).cast(), 1) };
        if read != 1 || released != THE_RUN_LETS_THE_CHILD_GO {
            // SAFETY: `_exit` ends this forked child and returns to nothing.
            unsafe { libc::_exit(EXITED_BEFORE_ITS_EXEC) };
        }
        pause_for(self.holds.after_the_let_go);
    }
}

/// What:
///  Pause the child for `held`, when a control armed a hold.
/// Why:
///   A start that takes time is the one thing no control can make from outside the child,
/// and this is the only call that spends it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (held) nanosleep(held);
/// ```
fn pause_for(held: Option<Duration>) {
    let Some(how_long) = held else {
        return;
    };
    let pause: libc::timespec = libc::timespec { tv_sec: libc::time_t::try_from(how_long.as_secs()).unwrap_or(0), tv_nsec: how_long.subsec_nanos() as libc::c_long };
    // SAFETY: `nanosleep` only pauses this forked child and touches no memory of the parent.
    let _paused: i32 = unsafe { libc::nanosleep(&pause, std::ptr::null_mut()) };
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
