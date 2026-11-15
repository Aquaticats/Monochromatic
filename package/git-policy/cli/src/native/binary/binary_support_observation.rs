//! What:
//!  Running a wrapped command with a time bound and reading what it did:
//! its exit status,
//! its output without the wait notices and hints,
//! and the settling of the service work it started.
//! Why:
//!  The reading of a run needs no fixture beyond its directory,
//! so it lives in a file of its own that the helpers that start commands import.
//!  The bound is this wait's own promise and this wait keeps it:
//! it starts the command in a process group of its own,
//! watches the clock itself,
//! and at the bound its command's line states it ends that whole group and reaps the command.
//!  A command that ended by itself comes back with its own status or signal whatever that was,
//! and the sentence a control reads says this wait ended it only when this wait did.
//!  The two pipes are read on threads under one stated grace,
//! so a process that outlived the command cannot hold the answer past the bound,
//! and an answer whose pipe is still held says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const observed = observeRaw(command); // { code, stdout, stderr, signal }
//! ```

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use crate::support_program_names_and_time_bound::TIME_BOUND_SECONDS;

/// The signal `timeout --signal=KILL` re-raises on itself when the bound it was given was hit:
///  it kills
/// the command,
///  then re-raises the same signal on itself.
/// Why it is here:
///  the helpers that run a command under `/usr/bin/timeout` and read its status back need the mark
///  (`native_git_output`, and `binary_resolution_tests` for a forwarding loop);
///  the shared wait no longer reads it, because a command that kills itself with `SIGKILL` shows the same signal,
///  and only this wait's own act can say that the bound ended a command (`observe_with`).
pub const TIMED_OUT_SIGNAL: i32 = 9;

/// What one finished process showed its caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observed {
    /// Exit code,
    ///  absent when a signal ended the process.
    pub code: Option<i32>,
    /// Exact standard output bytes.
    pub stdout: Vec<u8>,
    /// Exact standard error bytes.
    pub stderr: Vec<u8>,
}

/// The bytes without their additive `"hint"` fields.
///  Most binary tests pin the incumbent's event shape,
/// which has no hint;
///  the hint text is checked by tests that use `observe_raw`.
pub fn without_hint_fields(bytes: &[u8]) -> Vec<u8> {
    let marker: &[u8] = b",\"hint\":\"";
    let mut kept: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index: usize = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(marker) {
            index += marker.len();
            while index < bytes.len() && bytes[index] != b'"' {
                index += if bytes[index] == b'\\' { 2 } else { 1 };
            }
            index += 1;
        } else {
            kept.push(bytes[index]);
            index += 1;
        }
    }
    return kept;
}

/// How every wait notice of the wrapper begins (`wait_progress.rs`,
/// `service_index_lock.rs`).
pub const WAIT_NOTICE_PREFIX: &[u8] = b"monochromatic-git-policy-cli: still waiting for ";

/// How the commit service's shorter notice begins,
/// the one a client whose own commit may be on the branch already reads (`service_busy.rs`).
/// The notice goes on with the command the service is inside,
/// after a comma,
/// so the prefix ends where the fixed words do.
pub const SERVICE_INSIDE_WRITE_NOTICE_PREFIX: &[u8] = b"monochromatic-git-policy-cli: this commit is inside a write of the commit service";

/// How the commit service's longer warning begins,
/// the one a client queued behind that write reads after minutes (`service_busy.rs`).
pub const SERVICE_STUCK_WRITE_NOTICE_PREFIX: &[u8] = b"monochromatic-git-policy-cli: the commit service has been inside a write ";

/// What:
///  Whether a line is one of the notices a wait of the wrapper prints.
/// Why:
///   A control that means the wrapper's words and not the host's speed drops the same lines the three
///  notices are written with.
pub fn is_a_wait_notice(line: &[u8]) -> bool {
    for prefix in [WAIT_NOTICE_PREFIX, SERVICE_INSIDE_WRITE_NOTICE_PREFIX, SERVICE_STUCK_WRITE_NOTICE_PREFIX] {
        if line.starts_with(prefix) {
            return true;
        }
    }
    return false;
}

/// What:
///  The bytes without the lines that are notices about a wait.
/// Why:
///  A step that outlasts its notice grace (5 seconds for a query,
/// 10 for a bulk step) prints a notice on standard
///  error however healthy it is,
/// and a loaded host stretches any step past that.
/// The commit service tells a client whose own commit is in its branch write that it is inside it,
/// which the owner
///  asked for (2026-11-04,
/// "Short extra wait":
/// the commit may be on the branch already,
/// so the client is told when the write starts),
/// and a beat falls inside a write a loaded host stretched past a second even when the commit lands,
/// so a control that means the words drops that notice;
/// it is a sign of the host's speed and not of the command's.
/// The longer warning of the same write,
/// which a client queued behind it reads after minutes,
/// goes too.
/// A control that compares the
///  wrapper's diagnostics with native Git's words means the words,
/// not that the host was fast,
/// so it compares what
///  is left.
/// Only a line that is a wait notice goes;
/// every other line stays,
/// and a control that
///  asserts a notice reads the raw bytes.
pub fn without_wait_notices(bytes: &[u8]) -> Vec<u8> {
    return bytes
        .split_inclusive(|byte| { return *byte == b'\n'; })
        .filter(|line| { return !is_a_wait_notice(line); })
        .flatten()
        .copied()
        .collect();
}

/// What:
///  Run a command to completion under the bound its own line states and observe it without its hint fields,
///  feeding `input` to its standard input.
/// Why:
///  The bound, the ending and the pipes are kept by `observe_with`, once for this face and the two below it.
///
/// # Panics
///  As `observe_raw`.
pub fn observe(command: &mut Command, input: &[u8]) -> Observed {
    let raw: Observed = observe_raw(command, input);
    return Observed {
        code: raw.code,
        stdout: without_hint_fields(raw.stdout.as_slice()),
        stderr: without_hint_fields(raw.stderr.as_slice()),
    };
}

/// What a command run under the shared test hang guard (`TIME_BOUND_SECONDS`) asks the wrapper to do,
/// read the way the
///  wrapper reads it.
pub enum SharedBoundMisuse {
    /// A commit the wrapper lands through its transaction.
    CommitTransaction,
    /// A `--help` that makes Git render a manual page through `man` and the groff programs.
    ManualPage,
}

/// What:
///  Whether `command` runs under the shared bound (`timeout --signal=KILL 5 <program> <arguments>`) with
///  arguments that need a longer bound or a stub instead;
/// absent for any other command.
/// Why:
///  The shared hang guard is for single Git commands,
/// and is not a limit on how long the wrapper may take.
///  A landed commit starts 40 Git processes and makes 36
///  sync calls (`binary_budget_tests.rs`),
/// against one process for direct Git,
/// and a manual page starts 9 more
///  (`man`,
/// `preconv`,
/// `tbl`,
/// `nroff`,
/// `col`,
/// `groff`,
/// `troff`,
/// `grotty`),
/// so under host load either one can pass
///  the bound while the policy it is there to test works.
/// This check does not depend on timing,
/// so a misuse fails
///  on every host,
/// loaded or quiet,
/// instead of once a day on a busy one.
pub fn shared_bound_misuse(command: &Command) -> Option<SharedBoundMisuse> {
    let arguments: Vec<OsString> = command.get_args().map(OsStr::to_os_string).collect();
    let [signal, bound, program, wrapped @ ..] = arguments.as_slice() else {
        return None;
    };
    if signal != "--signal=KILL" || bound != TIME_BOUND_SECONDS {
        return None;
    }
    let stripped = git_policy_cli::wrapper_invocation::strip_wrapper_controls(wrapped);
    if stripped.layout.outcome != git_policy_cli::global_arguments::GlobalOutcome::Command {
        return None;
    }
    let region: &[OsString] = git_policy_cli::wrapper_invocation::command_region(&stripped);
    let word: &[u8] = git_policy_cli::wrapper_invocation::command_word(&stripped);
    if word == b"commit"
        && git_policy_cli::command_commit::parse_commit_region(region, &[]).is_ok_and(|facts| { return !facts.dry_run; })
    {
        return Some(SharedBoundMisuse::CommitTransaction);
    }
    let has_manual_stub: bool = Path::new(program)
        .parent()
        .is_some_and(|directory| { return directory.join("man").exists(); });
    let asks_for_help: bool = region
        .iter()
        .take_while(|token| { return *token != "--"; })
        .any(|token| { return token == "--help"; });
    if word != b"monochromatic-git-policy-cli" && word != b"cli-git" && asks_for_help && !has_manual_stub {
        return Some(SharedBoundMisuse::ManualPage);
    }
    return None;
}

/// How long the pipes of a command may take to reach their end once the command is gone
/// before this wait leaves them to whoever holds them.
/// Why 500 ms:
///  everything a gone command wrote sits in the pipe's buffer already,
///  so this window covers the two reader threads being scheduled and no more;
///  a command that ends in a millisecond costs a millisecond,
///  and a pipe a descendant keeps open costs this window once, however long it holds it.
pub const PIPE_GRACE: std::time::Duration = std::time::Duration::from_millis(500);

/// How long a command this wait has ended may take to be reaped.
/// Why five seconds:
///  a `SIGKILL`ed child is reaped at once, so this window only covers the kernel and the scheduler;
///  a command still not reaped in it is a defect of this wait, not a slow host.
const REAPING: std::time::Duration = std::time::Duration::from_secs(5);

/// The bound this wait keeps for a command whose own line names none:
///  the shared backstop, which is also where the supervisor ends a command that never shows it is working.
/// Why:
///  Nothing waits without a bound, a test included,
///  and a line that carries no `/usr/bin/timeout` is still a command a control waits for:
///  a trace wrapper, a namespace probe, a program a control spawns for itself.
///  The backstop is the bound every other command of this harness already runs under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = statedBound(command) ?? SHARED.backstop;
/// ```
const UNSTATED_BOUND: std::time::Duration = super::supervisor::SHARED.backstop;

/// Who ended one command this wait ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Ending {
    /// The command ended by itself,
    /// and its status and output are its own,
    /// whatever that status was: a code of 124 or 137,
    /// or a signal's death.
    Itself,
    /// This wait ended it at the bound, by killing the process group it led.
    AtTheBound,
}

/// What:
///  What one command showed its caller, and who ended it.
/// Why:
///  The ending is this wait's own act and never a reading of the status the command came back with:
///  a command that kills itself with the signal `timeout` re-raises cannot be told from the bound by its status
///  alone.
struct Answer {
    /// Its own ending and what its pipes gave before it went.
    status: std::process::ExitStatus,
    /// Exact standard output bytes.
    stdout: Vec<u8>,
    /// Exact standard error bytes.
    stderr: Vec<u8>,
    /// Who ended the command.
    ending: Ending,
    /// The pipes a process that outlived the command still holds, named for a failure.
    held_open: Vec<&'static str>,
}

impl Answer {
    /// What:
    ///  Panics unless the command ended by itself and nothing of it still holds a pipe.
    /// Why:
    ///  A control may read an answer as the command's own only when it is:
    ///  this wait ending the command at the bound, and a pipe another process may still write to, are the two ways
    ///  it is not, and each is said in the control's own failure instead of being passed off as a status.
    fn refuse_a_second_ending(&self, bound: std::time::Duration, spelled: &str, directory: Option<&Path>) {
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
///  One pipe of a command: the bytes as they arrive, and the news that its reader reached its end.
/// Why:
///  Reading a pipe on this thread waits for the end of the file,
///  and a process that inherited the pipe can postpone that end past every bound;
///  a reader of its own lets the answer keep what has arrived and leave the rest to whoever holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { bytes, ended } = drain(child.stdout); // ended: a message when the pipe reaches its end
/// ```
fn draining<R: std::io::Read + Send + 'static>(mut reader: R) -> (std::sync::Arc<std::sync::Mutex<Vec<u8>>>, std::sync::mpsc::Receiver<()>) {
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
///  The input is written by a thread of its own, so a child that fills its output before it reads all its input is
///  not deadlocked,
///  and the two pipes are read on threads under one grace, so a process that outlived the command cannot hold the
///  answer.
///  `group` is the group this wait made for the command, or `None` for a child it did not start and that leads no
///  group, which the bound then ends alone.
fn wait_under_bound(mut child: Child, group: Option<libc::pid_t>, bound: std::time::Duration, input: &[u8]) -> Answer {
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
///  and a wait that read the bound back from a status could not tell it from a command's own 124, 137 or signal;
///  keeping the same seconds here ends the command at the moment its line already states.
///  A line of the form `timeout ... 0 ...` asks `timeout` for no limit at all, and nothing waits without a bound,
///  so it gets the backstop as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = secondsOf(`timeout --signal=KILL ${line}`) ?? SHARED.backstop;
/// ```
fn stated_bound(command: &Command) -> std::time::Duration {
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
fn spelled_command(command: &Command) -> String {
    let mut words: Vec<String> = vec![command.get_program().to_string_lossy().into_owned()];
    words.extend(command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }));
    return words.join(" ");
}

/// What:
///  Waits for a spawned child with its input and its output moving from the first moment,
/// claims the supervisor's kill of it
///  if there is one,
/// and only then runs `cleanup`.
/// Why:
///   The input is written by a thread of its own,
/// so a child that fills its output before it reads all its input is not
///  deadlocked by the test,
/// and both pipes are drained on threads:
/// a command that writes much is not idle.
///  The kill is claimed before the cleanup because the cleanup can take longer than the grace in which an unclaimed kill
///  aborts the process.
///  This child is not one this module started and leads no process group of its own,
///  so `UNSTATED_BOUND` ends the child alone;
///  the supervisor's controls use this wait for a child they spawn themselves.
///
/// # Panics
///  With the supervisor's message when it ended the command.
///  When a process that outlived the child still holds one of its pipes after `PIPE_GRACE`.
pub fn supervised_wait(child: Child, input: &[u8], cleanup: impl FnOnce()) -> Output {
    let child_id: u32 = child.id();
    let answer: Answer = wait_under_bound(child, None, UNSTATED_BOUND, input);
    // The supervisor ended it:
    // a loop or a deadlock,
    // with the reason and the tasks of the tree in the message.
    if let Some(message) = super::supervisor::take_kill(child_id) {
        panic!("{message}");
    }
    cleanup();
    answer.refuse_a_second_ending(UNSTATED_BOUND, "the command this control started", None);
    return Output { status: answer.status, stdout: answer.stdout, stderr: answer.stderr };
}

/// Run a command to completion with every byte of its output kept,
///  feeding `input` to its standard input.
/// # Panics
///  When the command is a commit transaction or a manual page under the shared bound;
///  see `shared_bound_misuse`.
///  When the supervisor ended the command, with the supervisor's message.
///  When this wait ended the command at the bound, or a process that outlived the command still holds one of its
///  pipes;
///  see `Answer::refuse_a_second_ending`.
pub fn observe_raw(command: &mut Command, input: &[u8]) -> Observed {
    return observe_with(command, input, true);
}

/// What:
///  Run a command to completion and observe it without waiting for the after-commit work the commit service runs.
/// Why:
///  Every observed command waits for that work (`settle_service_work`),
/// because most controls read what the push or the upkeep did after the command returned.
/// A control about what the command itself said,
/// or about the service's first attempt at a step whose record the service tries again at the end of a claim's lease,
/// would wait minutes for tries it does not mean to read,
/// and a lease is 120 seconds (`CLAIM_LEASE`).
///
/// # Panics
///  As `observe_raw`.
pub fn observe_without_waiting_for_the_after_commit_work(command: &mut Command, input: &[u8]) -> Observed {
    return observe_with(command, input, false);
}

/// What:
///  The body both faces above share:
///  one run of a command, and the wait for the after-commit work given or refused.
/// Why:
///  The two faces differ only in that wait,
/// so the bound,
/// the misuse check and the reading live once.
///  The command leads a process group of its own, which is the group the bound ends;
///  `timeout` makes such a group for itself a moment after it starts (`setpgid` in its own process),
///  and this wait makes it before the command runs, so the group it ends is one it made.
///
/// # Panics
///  When the command is a commit transaction or a manual page under the shared bound;
///  see `shared_bound_misuse`.
///  When the supervisor ended the command, with the supervisor's message.
///  When this wait ended the command at the bound, or a pipe the command's output came through is still held;
///  see `Answer::refuse_a_second_ending`.
fn observe_with(command: &mut Command, input: &[u8], waiting_for_the_after_commit_work: bool) -> Observed {
    match shared_bound_misuse(command) {
        Some(SharedBoundMisuse::CommitTransaction) => panic!(
            "a commit under the shared bound: the work of a landed commit happens in the commit service, outside the command's \
             process tree, so the supervisor would see a tree that waits and end it; run it with `run_commit` (the {}-second \
             wall-clock bound of binary_commit_tests.rs)",
            super::commit::COMMIT_BOUND_SECONDS
        ),
        Some(SharedBoundMisuse::ManualPage) => panic!(
            "a `--help` under the shared bound renders a manual page through man and groff (9 process starts) that is Git's \
             work and the host's `man`; put an executable `man` stub in the fixture's bin directory first (`stub_manual_viewer`)"
        ),
        None => {}
    }
    let bound: std::time::Duration = stated_bound(command);
    let spelled: String = spelled_command(command);
    let directory: Option<PathBuf> = command.get_current_dir().map(Path::to_path_buf);
    command.process_group(0);
    let child: Child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("process starts");
    let child_id: u32 = child.id();
    let group: libc::pid_t = libc::pid_t::try_from(child_id).expect("a process ID fits the pid type");
    let answer: Answer = wait_under_bound(child, Some(group), bound, input);
    // The supervisor ended it: a loop or a deadlock, and its message names the reason and the tasks of the tree.
    //  Claimed before the cleanup, which can take longer than the grace in which an unclaimed kill aborts the process.
    if let Some(message) = super::supervisor::take_kill(child_id) {
        panic!("{message}");
    }
    if waiting_for_the_after_commit_work && let Some(at) = directory.as_deref() {
        settle_service_work(at);
    }
    answer.refuse_a_second_ending(bound, spelled.as_str(), directory.as_deref());
    return Observed { code: answer.status.code(), stdout: answer.stdout, stderr: answer.stderr };
}

/// How long a control waits for the after-commit work (the push,
/// the upkeep) the commit service runs once the commit returned.
pub const SERVICE_WORK_PATIENCE: std::time::Duration = std::time::Duration::from_secs(60);

/// The controls of the wait above: the endings a command can come back with, a pipe a descendant holds, and a
/// command that writes more than a pipe holds.
#[cfg(test)]
#[path = "binary_support_observation_tests.rs"]
mod tests;

/// Every state directory of a work directory with the names it holds,
/// for the failure message of a wait that ran out.
pub fn work_listing(work: &Path) -> String {
    return ["pending", "claimed", "recovering", "done", "failed"]
        .iter()
        .map(|state| {
            let mut names: Vec<String> = std::fs::read_dir(work.join(state))
                .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
                .unwrap_or_default();
            names.sort();
            return format!("{state}={names:?}");
        })
        .collect::<Vec<String>>()
        .join(" ");
}

/// What:
///  Waits until the commit service has settled the after-commit work of the repository around `directory`.
/// Why:
///   A commit that landed through the service returns before its push and its upkeep run,
/// so a control that inspects the
///  origin,
/// the packs or the push's failure after the command must first wait for the records to leave `pending`,
///  `claimed` and `recovering`;
/// it waits only where a work directory exists.
///  This wait is `SERVICE_WORK_PATIENCE`, which is shorter than one claim's lease (`CLAIM_LEASE`, 120 seconds):
/// a record whose handler answered with a retry keeps its claim until that lease ends,
/// so a control that waits here cannot see such a record settle.
/// A control that means to read one attempt of such a record reads that instead
/// (`observe_without_waiting_for_the_after_commit_work`).
pub fn settle_service_work(directory: &Path) {
    let Some(git_dir) = directory.ancestors().map(|candidate| { return candidate.join(".git"); }).find(|candidate| { return candidate.is_dir(); }) else {
        return;
    };
    let work: PathBuf = git_dir.join(git_policy_cli::work_item::WORK_DIRECTORY);
    let open = |state: &str| -> usize { return std::fs::read_dir(work.join(state)).map_or(0, Iterator::count); };
    let began: std::time::Instant = std::time::Instant::now();
    while open("pending") + open("claimed") + open("recovering") > 0 {
        assert!(began.elapsed() < SERVICE_WORK_PATIENCE, "the service never settled the after-commit work in {}: {}", work.display(), work_listing(work.as_path()));
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
