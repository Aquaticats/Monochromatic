//! What:
//!  The bound one observed command is given,
//!  who ended it,
//!  and the two pipes its output came through.
//! Why:
//!  The bound is this wait's own promise and this wait keeps it:
//!  it reads the bound the command's own line states,
//!  ends the whole process group the command leads at that bound,
//!  reaps the command,
//!  reads the two pipes on threads under one grace,
//!  and says which of those things ended the command.
//!  The bound counts from the command's own start,
//!  read from the kernel's record of it,
//!  so no work of this parent after the spawn can move the deadline;
//!  that start is also where the `timeout` in the line counts its own,
//!  so this wait's deadline is the earlier of the two,
//!  and a command the line's own `timeout` ends at that bound is still the bound's doing.
//!  A wait that reads the exit rather than reaching that deadline names the same ending:
//!  a death by `SIGKILL` in a line of this harness's bounded form is the bound's once that line's own bound has been reached by this wait's clock,
//!  because the line's `timeout` sends `SIGKILL` to the group it leads and dies with it,
//!  while a command's own `SIGKILL` read before its bound stays its own.
//!  The status alone cannot say which of the two sent the signal,
//!  so a command's own `SIGKILL` that this wait reads at or after its line's bound is read as the bound's,
//!  and the start's truncation to whole clock ticks widens that by less than one tick.
//!  A signal goes out only through a handle that still names the command:
//!  a pinned process for the command itself,
//!  and the process group only while the command,
//!  held unreaped,
//!  still pins the group's number.
//!  The faces a control calls live next door in `binary_support_observation.rs`,
//!  which reaches these items as `crate::support_observation_bound`.
//!  The wait marks its command in the supervisor's registry for as long as it runs
//!  (`supervisor::waiting_for`),
//!  so a kill nobody claims is aborted on only when no thread of the test is occupied with that command,
//!  never because this host took long to schedule the thread that claims it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = lineBoundOf(command); // { kept, stated }
//! const answer = waitUnderBound(child, group, line, input); // { status, stdout, stderr, ending, heldOpen }
//! ```

use crate::supervisor::{
    Occupied,
    take_kill,
    waiting_for,
};
use crate::supervisor_pin::{
    Delivered,
    Pin,
};
use crate::supervisor_proc::{
    stat_of,
    tick,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::io::Write;
use std::os::unix::process::ExitStatusExt as _;
use std::process::{
    Child,
    Command,
};

/// How long a command this wait has ended may take to be reaped.
/// Why five seconds:
///  a `SIGKILL`ed child is reaped at once,
///  so this window only covers the kernel and the scheduler;
///  a command still not reaped in it is a defect of this wait,
///  not a slow host.
pub const REAPING: std::time::Duration = std::time::Duration::from_secs(5);

/// What:
///  What this wait answers with:
///  the two bounds a command's own line gives it,
///  who ended it,
///  and the sentence a refused answer prints,
///  in a file of their own because this one is over the per-file line budget.
/// Why:
///  A caller that named one of them through this module goes on working,
///  and the file that holds them describes the answer rather than the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './binary_support_observation_bound_what_the_wait_answers_with.rs';
/// ```
pub use crate::support_observation_bound_what_the_wait_answers_with::*;

/// What:
///  One pipe of a command:
///  the bytes as they arrive,
///  and the news that its reader reached its end.
/// Why:
///  Reading a pipe on this thread waits for the end of the file,
///  and a process that inherited the pipe can postpone that end past every bound;
///  a reader of its own lets the answer keep what has arrived and leave the rest to whoever holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { bytes, ended } = drain(child.stdout); // ended: a message when the pipe reaches its end
/// ```
pub fn draining<R: std::io::Read + Send + 'static>(
    mut reader: R
) -> (
    std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    std::sync::mpsc::Receiver<()>,
) {
    let kept: std::sync::Arc<std::sync::Mutex<Vec<u8>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let sink: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::clone(&kept);
    let _reader: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .spawn(move || {
            let mut buffer: [u8; 8192] = [0; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => sink
                        .lock()
                        .expect("the bytes of a command's pipe")
                        .extend_from_slice(&buffer[..read]),
                    Err(_) => break,
                }
            }
            let _ended = sender.send(());
        })
        .expect("the reader of a command's pipe starts");
    return (kept, receiver);
}

/// What:
///  Wait for an already started child under `bound`,
///  drain its two pipes,
///  and answer what it showed and who ended it.
/// Why:
///  The bound is this wait's own:
///  it counts from the child's own start (`remaining_of`),
///  it waits on the child with that as its deadline,
///  and at the bound it ends the process group the command leads and reaps the command,
///  so the ending is decided by this wait's act and never by a status another program's bound happens to share.
///  The child is never reaped before the decision:
///  the waiter learns of the exit with `WNOWAIT`,
///  so the number a signal has to name is still the child's when the signal goes out,
///  and a command that ended by itself is signalled by nothing.
///  The input is written by a thread of its own,
///  so a child that fills its output before it reads all its input is not deadlocked,
///  and the two pipes are read on threads under one grace,
///  so a process that outlived the command cannot hold the answer.
///  `group` is the group this wait made for the command,
///  or `None` for a child it did not start and that leads no group,
///  which the bound then ends alone.
///  `line` is the pair one reading of the command's own line gives (`line_bound_of`):
///  the bound this wait keeps and the bound that line states,
///  and the stated bound is what tells the death that line's `timeout` deals there from a command's own `SIGKILL`,
///  because the status alone cannot say which of the two sent the signal (`ending_of_an_exit_read_first`).
pub fn wait_under_bound(
    mut child: Child,
    group: Option<libc::pid_t>,
    line: LineBound,
    input: &[u8],
) -> Answer {
    let id: u32 = child.id();
    // What: the mark saying a wait of this process is inside for this command, held until this function returns or unwinds.
    // Why:  the monitor aborts the process for a kill no wait will claim, and this wait is the one that claims the kill of
    //       the command it was given, so the monitor must know it is running however long this host takes to schedule it.
    let _waiting: Occupied = waiting_for(id);
    let pid: libc::pid_t = libc::pid_t::try_from(id).expect("a process ID fits the pid type");
    let started: u64 = stat_of(pid)
        .expect("a command this wait holds is readable until it is reaped")
        .started;
    let remaining: std::time::Duration = remaining_of(started, line.kept);
    let (standard_output, output_end) =
        draining(child.stdout.take().expect("piped standard output"));
    let (standard_error, error_end) = draining(child.stderr.take().expect("piped standard error"));
    if let Some(mut standard_input) = child.stdin.take() {
        let owned: Vec<u8> = input.to_vec();
        // A child that exits without reading its input closes the pipe; that is not a control's failure.
        // The writer is not joined: a process that holds the other end of that pipe could hold this wait,
        //  and the thread ends with this process.
        let _writer: std::thread::JoinHandle<()> = std::thread::Builder::new()
            .spawn(move || {
                let _ = standard_input.write_all(owned.as_slice());
            })
            .expect("the writer of a command's input starts");
    }
    // The wait blocks on the child instead of looking at the clock,
    //  so a command that ends in a millisecond costs a millisecond;
    //  the waiter reports the exit without reaping the child,
    //  because the number a signal has to name must still be the child's when it goes out.
    let (exit_sender, exit_receiver) = std::sync::mpsc::channel::<std::io::Result<()>>();
    let waiter: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .spawn(move || {
            let _ = exit_sender.send(wait_for_the_exit(pid));
        })
        .expect("the waiter of a command starts");
    let (ending, supervisor_message, status): (Ending, Option<String>, std::process::ExitStatus) =
        match exit_receiver.recv_timeout(remaining) {
            Ok(Ok(())) => {
                // The exit reached this wait before its deadline ran out,
                //  which is not the same as the command ending by itself:
                //  the line's own `timeout` can be what ended it (`ending_of_an_exit_read_first`).
                // The claim comes before the reap for the same reason as the other arm's.
                let claimed: Option<String> = take_kill(id);
                let status: std::process::ExitStatus = reaped(&mut child);
                (
                    ending_of_an_exit_read_first(line, started, status),
                    claimed,
                    status,
                )
            }
            Ok(Err(error)) => panic!("the ending of the command {id} cannot be read: {error}"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if exited_without_reaping(pid) {
                    // It ended in the instant the bound came due, so nothing is signalled.
                    // A death by the signal the line's own `timeout` sends is the bound's doing:
                    //  the two timers keep one value and this wait's deadline is the earlier one,
                    //  so a `SIGKILL` that arrives here is the bound's, never a command's own death long before it.
                    let claimed: Option<String> = take_kill(id);
                    let status: std::process::ExitStatus = reaped(&mut child);
                    let by_the_bound: Ending = if status.signal() == Some(libc::SIGKILL) {
                        Ending::AtTheBound
                    } else {
                        Ending::Itself
                    };
                    (by_the_bound, claimed, status)
                } else {
                    let claimed: Option<String> = take_kill(id);
                    signal_the_child_and_its_group(pid, started, group);
                    // A `SIGKILL`ed command is a zombie within this window,
                    //  so a command still not ended in it is a defect of this wait,
                    //  not a slow host;
                    //  the waiter reports the exit without reaping, and the reap below is then immediate.
                    let ended: std::io::Result<()> = exit_receiver.recv_timeout(REAPING).unwrap_or_else(|_| {
                        panic!("the command {id} was ended at its bound of {:?} and not reaped within {REAPING:?}", line.kept)
                    });
                    ended.unwrap_or_else(|error| {
                        panic!("the ending of the command {id} cannot be read: {error}")
                    });
                    (Ending::AtTheBound, claimed, reaped(&mut child))
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                panic!("the waiter of the command {id} ended without its ending")
            }
        };
    let _joined = waiter.join();
    // What the pipes gave, under one grace for both, and never a wait past it.
    let grace_ends: std::time::Instant = std::time::Instant::now() + PIPE_GRACE;
    let output_ended: bool = output_end
        .recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now()))
        .is_ok();
    let error_ended: bool = error_end
        .recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now()))
        .is_ok();
    let mut held_open: Vec<&'static str> = Vec::new();
    if !output_ended {
        held_open.push("standard output");
    }
    if !error_ended {
        held_open.push("standard error");
    }
    return Answer {
        line,
        status,
        stdout: standard_output
            .lock()
            .expect("the bytes of a command's standard output")
            .clone(),
        stderr: standard_error
            .lock()
            .expect("the bytes of a command's standard error")
            .clone(),
        ending,
        held_open,
        supervisor_message,
    };
}

/// What:
///  Who ended a command whose exit this wait read from its waiter,
///  which is the arm where its own deadline did not decide.
/// Why:
///  A command's ending is its own unless this wait can name what made it,
///  and one thing can:
///  a line of this harness's bounded form puts a `timeout` of the same bound in front of the program,
///  that `timeout` sends `SIGKILL` to the whole group it leads and dies with it when its own timer fires,
///  and `SIGKILL` cannot be ignored,
///  so the command this wait holds comes back as a death by `SIGKILL` with no code.
///  The exit can reach this wait before its deadline runs out,
///  because this wait's deadline and the line's timer keep one value,
///  the line's own bound is reached at the same moment,
///  and a wait that is read late still reads a message already in hand.
///  A command that kills itself with `SIGKILL` before its line's bound is its own,
///  and a line that is not this harness's bounded form carries no timer of the bound at all,
///  so both stay `Ending::Itself`.
///  The status alone cannot say which of the two sent the signal,
///  so a command's own `SIGKILL` that this wait reads at or after its line's bound is read as the bound's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (line.stated === undefined) return Ending.Itself; // no timer of the bound is in this line
/// if (status.signal !== SIGKILL) return Ending.Itself; // only the line's `timeout` dies by the signal it sends
/// if (remainingOf(started, line.stated) > 0) return Ending.Itself; // the line's bound was not reached yet
/// return Ending.AtTheBound;
/// ```
fn ending_of_an_exit_read_first(
    line: LineBound,
    started: u64,
    status: std::process::ExitStatus,
) -> Ending {
    // `undefined` here is `None`: a line that states no bound of its own carries no timer that could have ended it.
    let Some(line_bound) = line.stated else {
        return Ending::Itself;
    };
    // Only a death by `SIGKILL` can be the line's own `timeout`, which dies by the signal it sends to its group.
    if status.signal() != Some(libc::SIGKILL) {
        return Ending::Itself;
    }
    // The line's own bound has been reached by this wait's own clock,
    //  which counts from the command's start as the line's timer does (`remaining_of`).
    if !remaining_of(started, line_bound).is_zero() {
        return Ending::Itself;
    }
    // Only a `SIGKILL` at or after the bound the line states is the bound's doing.
    return Ending::AtTheBound;
}

/// What:
///  How much of `bound` is left,
///  counted from the command's own start.
/// Why:
///  A control's bound is a bound on the command,
///  so it counts from the instant the command began,
///  which the kernel recorded when it created the process:
///  no work of this parent after the spawn,
///  and no delay of a loaded host in reaching the wait,
///  moves the deadline,
///  and a wait given a child another program started still counts the bound as its control stated it.
///  The start is recorded in whole clock ticks,
///  so the count is short by less than one tick and the command is never ended after its bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = birthTicks * tick + bound - bootNow(); // never longer than the bound
/// ```
fn remaining_of(
    started: u64,
    bound: std::time::Duration,
) -> std::time::Duration {
    let period_ns: u128 = tick().as_nanos();
    let birth_ns: u128 = u128::from(started).saturating_mul(period_ns);
    let deadline_ns: u128 = birth_ns.saturating_add(bound.as_nanos());
    let left_ns: u128 = deadline_ns.saturating_sub(boot_now_ns());
    return std::time::Duration::from_nanos(u64::try_from(left_ns).unwrap_or(u64::MAX));
}

/// The boot clock now,
///  in nanoseconds:
///  the clock a process's start is recorded in.
fn boot_now_ns() -> u128 {
    let mut now: libc::timespec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `clock_gettime` writes the one `timespec` it is given and reads no memory of ours.
    let asked: libc::c_int = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &raw mut now) };
    assert_eq!(asked, 0, "the boot clock answers");
    let seconds: u128 = u128::try_from(now.tv_sec).expect("a boot time is not before the boot");
    let nanoseconds: u128 =
        u128::try_from(now.tv_nsec).expect("a part of a second is not negative");
    return seconds
        .saturating_mul(1_000_000_000)
        .saturating_add(nanoseconds);
}

/// What:
///  Wait until the child exits,
///  without reaping it.
/// Why:
///  A reaped process frees its number,
///  and another process of the account can be given it at once;
///  `WNOWAIT` leaves the child a zombie,
///  so its number and its group's number still name it while this wait may still have to signal them.
fn wait_for_the_exit(pid: libc::pid_t) -> std::io::Result<()> {
    let mut ended: libc::siginfo_t = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: `waitid` writes the one `siginfo_t` it is given and reads no memory of ours;
        //  `WNOWAIT` says the child is not reaped.
        let answered: libc::c_int = unsafe {
            libc::waitid(
                libc::P_PID,
                libc::id_t::try_from(pid).expect("a pid fits the id type"),
                &raw mut ended,
                libc::WEXITED | libc::WNOWAIT,
            )
        };
        if answered == 0 {
            return Ok(());
        }
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

/// Whether the child has exited,
/// without reaping it.
fn exited_without_reaping(pid: libc::pid_t) -> bool {
    let mut ended: libc::siginfo_t = unsafe { std::mem::zeroed() };
    // SAFETY: as `wait_for_the_exit`; `WNOHANG` makes the look answer at once,
    //  and `WNOWAIT` keeps the child unreaped either way.
    let answered: libc::c_int = unsafe {
        libc::waitid(
            libc::P_PID,
            libc::id_t::try_from(pid).expect("a pid fits the id type"),
            &raw mut ended,
            libc::WEXITED | libc::WNOWAIT | libc::WNOHANG,
        )
    };
    return answered == 0 && unsafe { ended.si_pid() } != 0;
}

/// Reap the child this wait holds and answer what it came to.
fn reaped(child: &mut Child) -> std::process::ExitStatus {
    return child
        .wait()
        .expect("a command this wait holds unreaped is reaped at once");
}

/// What:
///  Signal the command at its bound,
///  through handles that still name it.
/// Why:
///  A process number is reused once its process is reaped,
///  so a signal sent to a number saved a moment ago can end another process of the account;
///  the command is a zombie here and is not reaped until after these signals,
///  so its own number and its group's number are still its own,
///  and the pin is checked again before the signal goes out.
fn signal_the_child_and_its_group(
    pid: libc::pid_t,
    started: u64,
    group: Option<libc::pid_t>,
) {
    let pin: Pin = Pin::open(pid, started)
        .expect("a command this wait holds unreaped is still the process it started");
    let _delivered: Delivered = pin.signal(libc::SIGKILL);
    if let Some(leader) = group {
        // SAFETY: `kill` reads the two numbers it is given and touches no memory of ours;
        //  the leader of this group is unreaped,
        //  so no other process can have been given the group's number yet.
        let _signalled: libc::c_int = unsafe { libc::kill(-leader, libc::SIGKILL) };
    }
}

/// What:
///  The seconds the harness's bounded form puts in front of a program,
///  `/usr/bin/timeout --signal=KILL <seconds>`,
///  or nothing for a line that is not that form or asks `timeout` for no limit.
/// Why:
///  One place reads the form,
///  so the bound a line states and the program it starts cannot disagree.
fn in_front_of_a_bounded_program(command: &Command) -> Option<u64> {
    let program: String = command.get_program().to_string_lossy().into_owned();
    if program.rsplit('/').next() != Some("timeout") {
        return None;
    }
    let arguments: Vec<OsString> = command.get_args().map(OsStr::to_os_string).collect();
    let [signal, seconds, _rest @ ..] = arguments.as_slice() else {
        return None;
    };
    if signal != "--signal=KILL" {
        return None;
    }
    let Ok(whole_seconds) = seconds.to_string_lossy().parse::<u64>() else {
        return None;
    };
    if whole_seconds == 0 {
        return None;
    }
    return Some(whole_seconds);
}

/// What:
///  The two bounds one command's own line gives this wait,
///  read in one place:
///  the bound this wait keeps,
///  and the bound the line states,
///  when that line is this harness's bounded form,
///  `/usr/bin/timeout --signal=KILL <seconds>`;
///  the shared backstop and nothing stated for any other line.
/// Why:
///  Every command of this harness carries its bound in its own line,
///  and the wait keeps the same seconds as its own bound and as the bound the line states,
///  so it ends the command at the moment its line already states,
///  and it names the ending the line's own `timeout` deals at that moment (`ending_of_an_exit_read_first`).
///  One reading answers both,
///  so no caller pairs a bound with a stated bound its line does not have,
///  and a wait that read the bound back from a status could not tell it from a command's own 124,
/// 137 or signal.
///  A line of the form `timeout ... 0 ...` asks `timeout` for no limit at all,
///  and nothing waits without a bound,
///  so that line states nothing here and its caller keeps the backstop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seconds = inFrontOfABoundedProgram(command); // undefined: a line that states no bound, or no limit
/// const line = { kept: seconds ?? sharedBackstop, stated: seconds };
/// ```
pub fn line_bound_of(command: &Command) -> LineBound {
    // `.map` converts the seconds the line names, and `undefined` in TS is the `None` of a line that names none.
    let stated: Option<std::time::Duration> =
        in_front_of_a_bounded_program(command).map(std::time::Duration::from_secs);
    // `??` in TS: a line that states no bound runs under the shared backstop.
    let kept: std::time::Duration = stated.unwrap_or(UNSTATED_BOUND);
    return LineBound {
        kept,
        stated,
    };
}

/// The command line as a person reads it:
///  the program and then each argument,
///  separated by spaces.
pub fn spelled_command(command: &Command) -> String {
    let mut words: Vec<String> = vec![command.get_program().to_string_lossy().into_owned()];
    words.extend(command.get_args().map(|argument| {
        return argument.to_string_lossy().into_owned();
    }));
    return words.join(" ");
}
