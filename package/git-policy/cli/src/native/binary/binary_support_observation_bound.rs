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
//!  a death by `SIGKILL` in a line of this harness's bounded form is the bound's once that line's own bound has been reached,
//!  because the line's `timeout` sends `SIGKILL` to the group it leads and dies with it,
//!  while a command's own `SIGKILL` before its bound stays its own.
//!  A signal goes out only through a handle that still names the command:
//!  a pinned process for the command itself,
//!  and the process group only while the command,
//!  held unreaped,
//!  still pins the group's number.
//!  The faces a control calls live next door in `binary_support_observation.rs`,
//!  which reaches these items as `crate::support_observation_bound`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = waitUnderBound(child, group, bound, input); // { status, stdout, stderr, ending, heldOpen }
//! ```

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::process::ExitStatusExt as _;
use std::path::Path;
use std::process::{Child, Command};
use crate::supervisor::take_kill;
use crate::supervisor_pin::{Delivered, Pin};
use crate::supervisor_proc::{stat_of, tick};

/// How long the pipes of a command may take to reach their end once the command is gone
/// before this wait leaves them to whoever holds them.
/// Why 500 ms:
///  everything a gone command wrote sits in the pipe's buffer already,
///  so this window covers the two reader threads being scheduled and no more;
///  a command that ends in a millisecond costs a millisecond,
///  and a pipe a descendant keeps open costs this window once,
///  however long it holds it.
pub const PIPE_GRACE: std::time::Duration = std::time::Duration::from_millis(500);

/// How long a command this wait has ended may take to be reaped.
/// Why five seconds:
///  a `SIGKILL`ed child is reaped at once,
///  so this window only covers the kernel and the scheduler;
///  a command still not reaped in it is a defect of this wait,
///  not a slow host.
pub const REAPING: std::time::Duration = std::time::Duration::from_secs(5);

/// The bound this wait keeps for a command whose own line names none:
///  the shared backstop,
///  which is also where the supervisor ends a command that never shows it is working.
/// Why:
///  Nothing waits without a bound,
///  a test included,
///  and a line that carries no `/usr/bin/timeout` is still a command a control waits for:
///  a trace wrapper,
///  a namespace probe,
///  a program a control spawns for itself.
///  The backstop is the bound every other command of this harness already runs under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = statedBound(command) ?? SHARED.backstop;
/// ```
pub const UNSTATED_BOUND: std::time::Duration = super::supervisor::SHARED.backstop;

/// Who ended one command this wait ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ending {
    /// The command ended by itself,
    /// and its status and output are its own,
    /// whatever that status was:
    /// a code of 124 or 137,
    /// or a signal's death.
    Itself,
    /// The bound ended it,
    /// at the bound its own line states:
    ///  by this wait's kill of the process group the command led,
    ///  or by the command's own `timeout`,
    ///  which keeps the same value and starts no earlier than the command.
    AtTheBound,
}

/// What:
///  What one command showed its caller,
///  and who ended it.
/// Why:
///  The ending is this wait's own act and never a reading of the status the command came back with:
///  a command that kills itself with `SIGKILL` before its bound is its own,
///  and only a death at the bound is the bound's.
pub struct Answer {
    /// Its own ending and what its pipes gave before it went.
    pub status: std::process::ExitStatus,
    /// Exact standard output bytes.
    pub stdout: Vec<u8>,
    /// Exact standard error bytes.
    pub stderr: Vec<u8>,
    /// Who ended the command.
    pub ending: Ending,
    /// The pipes a process that outlived the command still holds,
    /// named for a failure.
    pub held_open: Vec<&'static str>,
    /// The supervisor's message when it ended the command,
    /// claimed while the command was still unreaped so the number it names was still the command's.
    pub supervisor_message: Option<String>,
}

impl Answer {
    /// What:
    ///  Panics unless the command ended by itself and nothing of it still holds a pipe.
    /// Why:
    ///  A control may read an answer as the command's own only when it is:
    ///  this wait ending the command at the bound,
    ///  and a pipe another process may still write to,
    ///  are the two ways it is not,
    ///  and each is said in the control's own failure instead of being passed off as a status.
    pub fn refuse_a_second_ending(&self, bound: std::time::Duration, spelled: &str, directory: Option<&Path>) {
        let location: String = directory.map(|at| { return format!(" in {}", at.display()); }).unwrap_or_default();
        let held_sentence: Option<String> = match self.held_open.as_slice() {
            [] => None,
            pipes => Some(format!(
                "a process that outlived it still holds its {} {:?} after the command went, so what it wrote there may be short",
                pipes.join(" and "),
                PIPE_GRACE
            )),
        };
        match (self.ending, held_sentence) {
            (Ending::Itself, None) => {}
            (Ending::Itself, Some(named)) => panic!("{spelled}{location} ended by itself, and {named}"),
            (Ending::AtTheBound, named) => {
                let also: String = named.map(|sentence| { return format!(", and {sentence}"); }).unwrap_or_default();
                panic!("the command was ended at its {bound:?} wall-clock bound: {spelled}{location}{also}");
            }
        }
    }
}

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
pub fn draining<R: std::io::Read + Send + 'static>(mut reader: R) -> (std::sync::Arc<std::sync::Mutex<Vec<u8>>>, std::sync::mpsc::Receiver<()>) {
    let kept: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let sink: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::clone(&kept);
    let _reader: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .spawn(move || {
            let mut buffer: [u8; 8192] = [0; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => sink.lock().expect("the bytes of a command's pipe").extend_from_slice(&buffer[..read]),
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
///  `stated` is the bound the command's own line states when that line is this harness's bounded form,
///  which is what tells the death that line's `timeout` deals at that bound from a command's own `SIGKILL`:
///  the status alone cannot say which of the two sent the signal (`ending_of_an_exit_read_first`).
pub fn wait_under_bound(mut child: Child, group: Option<libc::pid_t>, bound: std::time::Duration, stated: Option<std::time::Duration>, input: &[u8]) -> Answer {
    let id: u32 = child.id();
    let pid: libc::pid_t = libc::pid_t::try_from(id).expect("a process ID fits the pid type");
    let started: u64 = stat_of(pid).expect("a command this wait holds is readable until it is reaped").started;
    let remaining: std::time::Duration = remaining_of(started, bound);
    let (standard_output, output_end) = draining(child.stdout.take().expect("piped standard output"));
    let (standard_error, error_end) = draining(child.stderr.take().expect("piped standard error"));
    if let Some(mut standard_input) = child.stdin.take() {
        let owned: Vec<u8> = input.to_vec();
        // A child that exits without reading its input closes the pipe; that is not a control's failure.
        // The writer is not joined: a process that holds the other end of that pipe could hold this wait,
        //  and the thread ends with this process.
        let _writer: std::thread::JoinHandle<()> = std::thread::Builder::new()
            .spawn(move || { let _ = standard_input.write_all(owned.as_slice()); })
            .expect("the writer of a command's input starts");
    }
    // The wait blocks on the child instead of looking at the clock,
    //  so a command that ends in a millisecond costs a millisecond;
    //  the waiter reports the exit without reaping the child,
    //  because the number a signal has to name must still be the child's when it goes out.
    let (exit_sender, exit_receiver) = std::sync::mpsc::channel::<std::io::Result<()>>();
    let waiter: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .spawn(move || { let _ = exit_sender.send(wait_for_the_exit(pid)); })
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
                (ending_of_an_exit_read_first(stated, started, status), claimed, status)
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
                    let by_the_bound: Ending = if status.signal() == Some(libc::SIGKILL) { Ending::AtTheBound } else { Ending::Itself };
                    (by_the_bound, claimed, status)
                } else {
                    let claimed: Option<String> = take_kill(id);
                    signal_the_child_and_its_group(pid, started, group);
                    // A `SIGKILL`ed command is a zombie within this window,
                    //  so a command still not ended in it is a defect of this wait,
                    //  not a slow host;
                    //  the waiter reports the exit without reaping, and the reap below is then immediate.
                    let ended: std::io::Result<()> = exit_receiver.recv_timeout(REAPING).unwrap_or_else(|_| {
                        panic!("the command {id} was ended at its bound of {bound:?} and not reaped within {REAPING:?}")
                    });
                    ended.unwrap_or_else(|error| panic!("the ending of the command {id} cannot be read: {error}"));
                    (Ending::AtTheBound, claimed, reaped(&mut child))
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => panic!("the waiter of the command {id} ended without its ending"),
        };
    let _joined = waiter.join();
    // What the pipes gave, under one grace for both, and never a wait past it.
    let grace_ends: std::time::Instant = std::time::Instant::now() + PIPE_GRACE;
    let output_ended: bool = output_end.recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now())).is_ok();
    let error_ended: bool = error_end.recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now())).is_ok();
    let mut held_open: Vec<&'static str> = Vec::new();
    if !output_ended {
        held_open.push("standard output");
    }
    if !error_ended {
        held_open.push("standard error");
    }
    return Answer {
        status,
        stdout: standard_output.lock().expect("the bytes of a command's standard output").clone(),
        stderr: standard_error.lock().expect("the bytes of a command's standard error").clone(),
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
fn ending_of_an_exit_read_first(stated: Option<std::time::Duration>, started: u64, status: std::process::ExitStatus) -> Ending {
    let Some(line_bound) = stated else { return Ending::Itself; };
    if status.signal() != Some(libc::SIGKILL) {
        return Ending::Itself;
    }
    // The line's own bound has been reached by this wait's own clock,
    //  which counts from the command's start as the line's timer does (`remaining_of`).
    if !remaining_of(started, line_bound).is_zero() {
        return Ending::Itself;
    }
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
fn remaining_of(started: u64, bound: std::time::Duration) -> std::time::Duration {
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
    let mut now: libc::timespec = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `clock_gettime` writes the one `timespec` it is given and reads no memory of ours.
    let asked: libc::c_int = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &raw mut now) };
    assert_eq!(asked, 0, "the boot clock answers");
    let seconds: u128 = u128::try_from(now.tv_sec).expect("a boot time is not before the boot");
    let nanoseconds: u128 = u128::try_from(now.tv_nsec).expect("a part of a second is not negative");
    return seconds.saturating_mul(1_000_000_000).saturating_add(nanoseconds);
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
        let answered: libc::c_int = unsafe { libc::waitid(libc::P_PID, libc::id_t::try_from(pid).expect("a pid fits the id type"), &raw mut ended, libc::WEXITED | libc::WNOWAIT) };
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
    let answered: libc::c_int = unsafe { libc::waitid(libc::P_PID, libc::id_t::try_from(pid).expect("a pid fits the id type"), &raw mut ended, libc::WEXITED | libc::WNOWAIT | libc::WNOHANG) };
    return answered == 0 && unsafe { ended.si_pid() } != 0;
}

/// Reap the child this wait holds and answer what it came to.
fn reaped(child: &mut Child) -> std::process::ExitStatus {
    return child.wait().expect("a command this wait holds unreaped is reaped at once");
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
fn signal_the_child_and_its_group(pid: libc::pid_t, started: u64, group: Option<libc::pid_t>) {
    let pin: Pin = Pin::open(pid, started).expect("a command this wait holds unreaped is still the process it started");
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
///  The bound the command's own line states,
///  when that line is this harness's bounded form,
///  `/usr/bin/timeout --signal=KILL <seconds>`;
///  nothing for a line that is not that form or asks `timeout` for no limit.
/// Why:
///  Every command of this harness carries its bound in its own line,
///  and a wait that read the bound back from a status could not tell it from a command's own 124,
///  137 or signal;
///  the wait keeps the same seconds as its own bound and as the bound the line states,
///  so it ends the command at the moment its line already states,
///  and it names the ending the line's own `timeout` deals at that moment (`ending_of_an_exit_read_first`).
///  A line of the form `timeout ... 0 ...` asks `timeout` for no limit at all,
///  and nothing waits without a bound,
///  so that line answers nothing here and its caller keeps the backstop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stated = inFrontOfABoundedProgram(command) ? seconds : undefined; // undefined: the caller's backstop
/// ```
pub fn stated_line_bound(command: &Command) -> Option<std::time::Duration> {
    let seconds: u64 = in_front_of_a_bounded_program(command)?;
    return Some(std::time::Duration::from_secs(seconds));
}

/// The command line as a person reads it:
///  the program and then each argument,
///  separated by spaces.
pub fn spelled_command(command: &Command) -> String {
    let mut words: Vec<String> = vec![command.get_program().to_string_lossy().into_owned()];
    words.extend(command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }));
    return words.join(" ");
}
