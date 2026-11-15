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
//!  The faces a control calls live next door in `binary_support_observation.rs`,
//!  which reaches these items as `crate::support_observation_bound`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = waitUnderBound(child, group, bound, input); // { status, stdout, stderr, ending, heldOpen }
//! ```

use std::io::Write;
use std::path::Path;
use std::process::{Child, Command};

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
    /// This wait ended it at the bound,
    /// by killing the process group it led.
    AtTheBound,
}

/// What:
///  What one command showed its caller,
///  and who ended it.
/// Why:
///  The ending is this wait's own act and never a reading of the status the command came back with:
///  a command that kills itself with the signal `timeout` re-raises cannot be told from the bound by its status
///  alone.
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
                panic!("this wait ended the command at its {bound:?} wall-clock bound: {spelled}{location}{also}");
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
///  it waits on the child with the bound as its deadline,
///  and at the bound it ends the process group the command leads and reaps the command,
///  so the ending is decided by this wait's act and never by a status another program's bound happens to share.
///  The input is written by a thread of its own,
///  so a child that fills its output before it reads all its input is not deadlocked,
///  and the two pipes are read on threads under one grace,
///  so a process that outlived the command cannot hold the answer.
///  `group` is the group this wait made for the command,
///  or `None` for a child it did not start and that leads no group,
///  which the bound then ends alone.
pub fn wait_under_bound(mut child: Child, group: Option<libc::pid_t>, bound: std::time::Duration, input: &[u8]) -> Answer {
    let id: u32 = child.id();
    let pid: libc::pid_t = libc::pid_t::try_from(id).expect("a process ID fits the pid type");
    let (standard_output, output_end) = draining(child.stdout.take().expect("piped standard output"));
    let (standard_error, error_end) = draining(child.stderr.take().expect("piped standard error"));
    if let Some(mut standard_input) = child.stdin.take() {
        let owned: Vec<u8> = input.to_vec();
        // A child that exits without reading its input closes the pipe; that is not a control's failure.
        // The writer is not joined: a process that holds the other end of that pipe could hold this wait, and the
        //  thread ends with this process.
        let _writer: std::thread::JoinHandle<()> = std::thread::Builder::new()
            .spawn(move || { let _ = standard_input.write_all(owned.as_slice()); })
            .expect("the writer of a command's input starts");
    }
    // The wait blocks on the child instead of looking at the clock, so a command that ends in a millisecond costs a
    // millisecond; the bound is the wait's own deadline, not a poll's.
    let (status_sender, status_receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::ExitStatus>>();
    let waiter: std::thread::JoinHandle<()> = std::thread::Builder::new()
        .spawn(move || { let _ = status_sender.send(child.wait()); })
        .expect("the waiter of a command starts");
    let (status, ending): (std::process::ExitStatus, Ending) = match status_receiver.recv_timeout(bound) {
        Ok(Ok(status)) => (status, Ending::Itself),
        Ok(Err(error)) => panic!("the ending of the command {id} cannot be read: {error}"),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            // SAFETY: `kill` reads the two numbers it is given and touches no memory of ours.
            let signalled: libc::c_int = unsafe {
                match group {
                    Some(leader) => libc::kill(-leader, libc::SIGKILL),
                    None => libc::kill(pid, libc::SIGKILL),
                }
            };
            let reaped: std::io::Result<std::process::ExitStatus> = status_receiver
                .recv_timeout(REAPING)
                .unwrap_or_else(|_| panic!("the command {id} was ended at its bound of {bound:?} and not reaped within {REAPING:?}"));
            let status: std::process::ExitStatus = reaped.unwrap_or_else(|error| panic!("the ending of the command {id} cannot be read: {error}"));
            // This wait's own act decides this and never the status the command came back with;
            // a group that was already gone was ended by the command itself, not by this call.
            (status, if signalled == 0 { Ending::AtTheBound } else { Ending::Itself })
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => panic!("the waiter of the command {id} ended without its status"),
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
    };
}

/// What:
///  The bound this wait keeps for `command`:
///  the seconds the `timeout --signal=KILL <seconds>` at the front of its line names,
///  or `UNSTATED_BOUND` when its line names none.
/// Why:
///  Every command of this harness carries its bound in its own line,
///  and a wait that read the bound back from a status could not tell it from a command's own 124,
///  137 or signal;
///  keeping the same seconds here ends the command at the moment its line already states.
///  A line of the form `timeout ... 0 ...` asks `timeout` for no limit at all,
///  and nothing waits without a bound,
///  so it gets the backstop as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = secondsOf(`timeout --signal=KILL ${line}`) ?? SHARED.backstop;
/// ```
pub fn stated_bound(command: &Command) -> std::time::Duration {
    let program: String = command.get_program().to_string_lossy().into_owned();
    if program.rsplit('/').next() != Some("timeout") {
        return UNSTATED_BOUND;
    }
    let mut arguments = command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); });
    let (Some(signal), Some(seconds)) = (arguments.next(), arguments.next()) else {
        return UNSTATED_BOUND;
    };
    if signal != "--signal=KILL" {
        return UNSTATED_BOUND;
    }
    let Ok(whole_seconds) = seconds.parse::<u64>() else { return UNSTATED_BOUND };
    if whole_seconds == 0 {
        return UNSTATED_BOUND;
    }
    return std::time::Duration::from_secs(whole_seconds);
}

/// The command line as a person reads it:
///  the program and then each argument,
///  separated by spaces.
pub fn spelled_command(command: &Command) -> String {
    let mut words: Vec<String> = vec![command.get_program().to_string_lossy().into_owned()];
    words.extend(command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }));
    return words.join(" ");
}
