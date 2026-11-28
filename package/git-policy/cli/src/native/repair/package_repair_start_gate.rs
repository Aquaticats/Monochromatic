//! What:
//!  The gate a started tool's child waits at before its own `exec`:
//! the two pipes the start is decided through,
//! the child that reports its own number,
//! the byte that lets it go,
//! the stop of the group it leads,
//! and the pause a control spends inside it.
//! Why:
//!  `Command::spawn` returns only after the child's `exec`,
//! so without a gate the run learns nothing of the child until the tool could already have run;
//! the child reports its number through one pipe and waits at the other,
//! so the run decides whether the tool ever runs,
//! and a tool the run gave up on never ran at all.
//! The descriptor the kernel makes at that report names the child for good,
//! so a stop reaches it and never a process that took its number.
//! Nothing here decides whether a start is waited for:
//! that is `package_repair_start.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gate = StartGate.make(); gate.arm(command); if (!await gate.theChildWithin(ms)) return 'no answer'; gate.letItGo();
//! const stopGroup = (leader: number) => process.kill(-leader, 'SIGKILL');
//! ```

/// Import the descriptor that names one process for good,
/// and the signal a stopped child is ended with.
use crate::member_signalled_through_its_pidfd::{
    Delivered,
    pidfd_of,
    signal_through,
};
/// Import the write that hands one byte to the child's end of the gate.
use std::io::Write as _;
/// Import the owned descriptor a pipe end and a process handle are held by.
use std::os::fd::{
    AsRawFd as _,
    OwnedFd,
    RawFd,
};
/// Import the standard-library command the child's own part is installed on.
use std::process::Command;
/// Import the span of time a control's hold is written as.
use std::time::Duration;
/// Import the pipe reader the child's report is awaited on.
use tokio::io::AsyncReadExt as _;
/// Import the reactor's own pipe end.
use tokio::net::unix::pipe::Receiver;
/// The one byte the run writes when the child may go on to its own `exec`.
const THE_RUN_LETS_THE_CHILD_GO: u8 = b'g';

/// The one byte the run writes when the child will not be let go,
/// so a child still waiting leaves before its own `exec` at once.
const THE_RUN_GIVES_UP: u8 = b'x';

/// How many bytes the child writes to say which process it is:
/// its own number,
/// in a form both sides of the fork read the same way.
const REPORTED_BYTES: usize = size_of::<i32>();

/// The status a child leaves with when the run did not let it go:
/// nothing of the tool has run at that point.
const EXITED_BEFORE_ITS_EXEC: i32 = 1;

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
///  The run's ends are its two pipes' only copies on its side,
/// so a byte this gate writes says whether the child may go,
///  and closing them is what ends a child that waited:
/// every end the child holds is closed in it by the kernel at its own
///  `exec`,
///  and otherwise when this gate is dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = StartGate.make(); gate.arm(command); if (!await gate.theChildWithin(ms)) return 'no answer'; gate.letItGo();
/// ```
pub(crate) struct TheStartGate {
    /// What the child reports its own number on.
    report: Receiver,
    /// What the run writes the one byte to that lets the child go on to its own `exec`.
    release: std::io::PipeWriter,
    /// The child that reported itself:
    /// its own number,
    /// and the descriptor the kernel made for it at that moment.
    reported: Option<TheReportedChild>,
}

impl Drop for TheStartGate {
    /// What:
    ///  Say that the child will not be let go,
    ///  and then close the run's ends of the gate.
    /// Why:
    ///  The end of the gate is what ends a waiting child,
    /// and a copy of its write end that a fork of another start inherited delays that end until that process reaches
    ///  its own `exec`;
    /// one byte the child reads as anything but the word to go ends it at once,
    /// and a child that was already let go read
    ///  its byte before this one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { this.release.write('x'); }
    /// ```
    fn drop(&mut self) {
        let _told: std::io::Result<()> = self.release.write_all(&[THE_RUN_GIVES_UP]);
    }
}

/// The child that reported itself at the gate.
pub(crate) struct TheReportedChild {
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
    pub(crate) fn made(holds: TheStartsHolds) -> std::io::Result<(TheStartGate, TheChildsStart)> {
        let (report, report_writer): (std::io::PipeReader, std::io::PipeWriter) = std::io::pipe()?;
        let (release_reader, release): (std::io::PipeReader, std::io::PipeWriter) =
            std::io::pipe()?;
        // The ends a child keeps are its own, and the ends the run keeps are closed in it, so the child's wait for the
        // release sees the end of its pipe when this gate is dropped.
        let the_runs_ends: [RawFd; 2] = [report.as_raw_fd(), release.as_raw_fd()];
        let watching: Receiver = Receiver::from_owned_fd(OwnedFd::from(report))?;
        let gate: TheStartGate = TheStartGate {
            report: watching,
            release,
            reported: None,
        };
        let the_childs_start: TheChildsStart = TheChildsStart {
            report: report_writer,
            release: release_reader,
            the_runs_ends,
            holds,
        };
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
    pub(crate) async fn the_child_within(
        &mut self,
        ceiling: Duration,
    ) -> bool {
        let mut reported: [u8; REPORTED_BYTES] = [0; REPORTED_BYTES];
        // A read that leaves bytes of the report unread is the start that never said which process it is, which is no answer.
        let Ok(Ok(REPORTED_BYTES..)) =
            tokio::time::timeout(ceiling, self.report.read_exact(&mut reported)).await
        else {
            return false;
        };
        let number: i32 = i32::from_le_bytes(reported);
        let descriptor: Option<OwnedFd> = pidfd_of(number);
        self.reported = Some(TheReportedChild {
            number,
            descriptor,
        });
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
    pub(crate) fn let_the_child_go(&mut self) -> std::io::Result<()> {
        return self.release.write_all(&[THE_RUN_LETS_THE_CHILD_GO]);
    }

    /// What:
    ///  Stop the child that reported itself,
    ///  and every helper it started.
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
    pub(crate) fn stop_the_reported_child(&mut self) {
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
/// and only a hold inside the child,
///  or on the thread that waits for it,
///  makes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheStartsHolds = { beforeTheReport?: number; afterTheLetGo?: number };
/// ```
pub(crate) struct TheStartsHolds {
    /// How long the child waits before it reports itself.
    pub(crate) before_the_report: Option<Duration>,
    /// How long the child waits after the run let it go and before its own `exec`.
    pub(crate) after_the_let_go: Option<Duration>,
}

/// What:
///  The child's own part of a start:
/// the ends it holds of the gate,
///  the ends the run holds that it closes in itself,
///  and the number it reports.
/// Why:
///   Only the child can say that it exists and wait before its own `exec`,
/// and it must say it with calls a forked child of a threaded process may make:
/// `write`,
///  `read`,
///  `close`,
///  `getpid`,
///  `nanosleep` and `_exit`,
///  and nothing that allocates or locks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // There is no TS counterpart of the forked child's own hook: it is a few syscalls before `exec`.
/// child.preExec(() => { report(getpid()); await oneByte(release); });
/// ```
pub(crate) struct TheChildsStart {
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
    pub(crate) fn arms(
        self,
        command: &mut Command,
    ) {
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
        let written: isize = unsafe {
            libc::write(
                self.report.as_raw_fd(),
                reported.as_ptr().cast(),
                REPORTED_BYTES,
            )
        };
        if written != REPORTED_BYTES as isize {
            // The run stopped waiting for this start, so nothing of the tool may run. The standard library sets `SIGPIPE`
            // back to its default before this hook (`sys/process/unix/unix.rs`, the block above the closures), so a pipe
            // with no reader ends this child by that signal rather than through this branch; both are before its `exec`.
            // SAFETY: `_exit` ends this forked child and returns to nothing.
            unsafe { libc::_exit(EXITED_BEFORE_ITS_EXEC) };
        }
        let mut released: u8 = 0;
        // SAFETY: `read` fills one byte of this child's own stack from a pipe it inherited.
        let read: isize =
            unsafe { libc::read(self.release.as_raw_fd(), (&raw mut released).cast(), 1) };
        if read != 1 || released != THE_RUN_LETS_THE_CHILD_GO {
            // SAFETY: `_exit` ends this forked child and returns to nothing.
            unsafe { libc::_exit(EXITED_BEFORE_ITS_EXEC) };
        }
        pause_for(self.holds.after_the_let_go);
    }
}

/// What:
///  Pause the child for `held`,
///  when a control armed a hold.
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
    let pause: libc::timespec = libc::timespec {
        tv_sec: libc::time_t::try_from(how_long.as_secs()).unwrap_or(0),
        tv_nsec: how_long.subsec_nanos() as libc::c_long,
    };
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
    let Ok(leader) = libc::pid_t::try_from(leader_id) else {
        return;
    };
    // SAFETY: `kill` takes a process-group id and a signal number and touches no memory; the group was made by this process.
    let _stopped = unsafe { libc::kill(-leader, libc::SIGKILL) };
}
