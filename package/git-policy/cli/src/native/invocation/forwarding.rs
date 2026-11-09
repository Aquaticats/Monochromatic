//! What:
//!  Start real Git with the caller's exact arguments,
//!  streams and environment.
//! Why:
//!  The wrapper adds policy around Git;
//!  it must never change what Git receives
//!      or what the caller observes from Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // spawn(gitPath, args, { stdio: 'inherit', env }) and exit with the child's status.
//! ```

/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments are forwarded byte for byte;
///  they are never decoded or re-quoted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.argv entries, but without any UTF-8 decoding.
/// ```
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;
/// What:
///  The marker every real-Git start carries,
/// and the two ways a start can be made.
/// Why:
///   A monochromatic-git-policy-cli executable that is started as "Git" finds the marker and stops instead of starting one more wrapper.
use crate::started_as_git::{StartMode, mark_real_git_start};
/// Starts that ride out a short shortage of processes.
#[cfg(unix)]
use crate::start_retry::exec_retrying;
/// Starts that ride out a short shortage of processes.
use crate::start_retry::RetryStart;
/// `Command` builds a child process from a program and an argument list,
///  with no shell.
use std::process::Command;

/// What:
///  How a Git child ended.
///       `i32` is a signed 32-bit integer,
///  the type of exit codes and signal numbers
///       (siblings `u8`,
///  `i64`).
/// Why:
///   A child killed by a signal has no exit code;
///  callers need both cases to
///       report Git's result faithfully.
///  `i32` is what the operating system returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildOutcome = { exited: number } | { signaled: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildOutcome {
    /// The child called `exit` with this code.
    Exited(i32),
    /// The child was terminated by this signal number.
    Signaled(i32),
}

/// What:
///  Build the child command:
///  program,
///  unchanged arguments,
///  added variables.
///       `&[OsString]` borrows the argument list;
///  `&[(OsString, OsString)]` borrows
///       name/value pairs.
/// Why:
///   One builder guarantees every real-Git start,
///  forwarded or queried,
///  uses an
///       argument array (no shell string) and inherits the caller's environment plus
///       only the overlay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitCommand(realGit: string, args: string[], overlay: [string, string][]): Command;
/// ```
pub fn git_command(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> Command {
    // `mut` allows the builder to be configured step by step.
    let mut command: Command = Command::new(real_git);
    // No hook runs through the wrapper, and the options follow the caller's globals; see `hooks_off`.
    crate::hooks_off::add_arguments(&mut command, arguments);
    // `for (name, value) in overlay` destructures each borrowed pair.
    for (name, value) in overlay {
        command.env(name, value);
    }
    // Every real-Git start says it is one, so a monochromatic-git-policy-cli executable that arrives as "Git" stops (`started_as_git.rs`).
    mark_real_git_start(&mut command, StartMode::Child);
    // The chain record names the file this start runs as Git, so a refusal a launcher leaves behind can name it
    //  (`chain_starts_record.rs`).
    crate::chain_starts_record::remember_chosen(real_git.as_os_str());
    return command;
}

/// The base a POSIX shell and Git 2.56.0 `run-command.c` add to a signal number.
pub const SIGNAL_EXIT_BASE: i32 = 128;

/// What:
///  The signal the control of `end_as_git_ended` blocks and expects a death by,
/// which is the one a supervisor or a person sends to stop a command.
/// Why:
///  A control cannot name a signal the wrapper's own start is likely to block,
/// and this one shows the case:
/// the signal is blocked,
/// so raising
///  it leaves it pending unless the block is lifted first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIGNAL_TO_END_BY = 15; // SIGTERM
/// ```
pub const SIGNAL_TO_END_BY: i32 = libc::SIGTERM;

/// What:
///  Translate a child's end into this process's exit code.
/// Why:
///   Git 2.56.0 `run-command.c` reports a child killed by signal N as `128 + N`,
///       which is also what a POSIX shell reports;
///  an exit code passes through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exitCode(outcome: ChildOutcome): number;
/// ```
pub fn exit_code(outcome: ChildOutcome) -> i32 {
    // What: `match` picks by variant and binds the number inside it.
    // Why:  The two cases are computed differently and both must be handled.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return 'exited' in outcome ? outcome.exited : 128 + outcome.signaled;
    // ```
    match outcome {
        ChildOutcome::Exited(code) => return code,
        ChildOutcome::Signaled(signal) => return SIGNAL_EXIT_BASE + signal,
    }
}

/// What:
///  Read a finished child's status into a `ChildOutcome`.
///       `#[cfg(unix)]` on the inner block compiles it only on Unix-like systems.
/// Why:
///   Unix reports either an exit code or a terminating signal,
///  never both.
///  Other
///       systems report only an exit code.
///  One function serves both,
///  so the part every
///       system shares is the part every test run exercises;
///  a status that is neither
///       cannot be reported faithfully and counts as a general failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outcomeOf(status: { code: number | null; signal: number | null }): ChildOutcome;
/// ```
pub(crate) fn outcome_of(status: std::process::ExitStatus) -> ChildOutcome {
    // `if let Some(code) = ...` runs only when the child exited normally.
    if let Some(code) = status.code() {
        return ChildOutcome::Exited(code);
    }
    #[cfg(unix)]
    {
        // The trait adds `.signal()` to exit statuses on Unix.
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return ChildOutcome::Signaled(signal);
        }
    }
    return ChildOutcome::Exited(1);
}

/// What:
///  Run real Git as a child with inherited stdin,
///  stdout and stderr,
///  and wait.
///       `std::io::Result<T>` is `Result<T, std::io::Error>`.
/// Why:
///   Callers that must do work after Git returns (post-commit policies,
///  status
///       notes) need the wrapper process to survive Git.
///  The child shares the
///       caller's terminal and streams directly;
///  nothing is buffered or copied.
///       This path installs no signal handlers:
///  a signal sent to the wrapper alone
///       does not reach Git,
///  exactly as with the TypeScript wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runRealGit(realGit, args, overlay): Promise<ChildOutcome>;
/// ```
pub fn run_real_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> std::io::Result<ChildOutcome> {
    return run_real_git_naming(real_git, arguments, overlay, &mut |_pid: u32| {});
}

/// What:
///  `run_real_git`, reporting the child's process number as soon as it has one.
/// Why:
///   A caller whose record has to name the Git it started
/// cannot wait for the child to end to learn its number:
/// the
///  record is what a later command reads while this one is gone and its Git still runs.
/// The child may be reported
///  only while it is this process's own unreaped child,
/// which is exactly here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await spawn(realGit, args, { onSpawn: (pid) => onStarted(pid) });
/// ```
pub fn run_real_git_naming(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    on_started: &mut dyn FnMut(u32),
) -> std::io::Result<ChildOutcome> {
    // What: `.status()` starts the child with inherited streams and waits for it;
    //       a trailing `?` returns a start failure to our caller.
    // Why:  Inherited streams are the default for `.status()`, so Git reads the
    //       caller's stdin and writes the caller's stdout and stderr itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const status = await spawn(realGit, args, { stdio: 'inherit' });
    // ```
    let mut child: std::process::Child = git_command(real_git, arguments, overlay).spawn_retrying()?;
    on_started(child.id());
    let status: std::process::ExitStatus = child.wait()?;
    // `Ok(...)` is the success variant carrying how the child ended.
    return Ok(outcome_of(status));
}

/// What:
/// What:
///  How often a run with a ceiling looks at its child before the ceiling.
/// Why:
///   `Child::try_wait` is the whole wait,
/// so the loop that reads the clock needs an interval;
/// one look every twentieth of a second
///  costs nothing beside a checkout and ends the child close to its ceiling.
const BOUNDED_LOOK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(20);

/// What:
///  How long Git is given to exit and be collected after it was killed before the wrapper leaves without it.
/// Why:
///   `SIGKILL` cannot be ignored,
/// so Git still running after this grace is inside a system call the kernel cannot interrupt
///  (a hung filesystem,
/// a dead NFS or FUSE server);
/// the ceiling exists for exactly that case,
/// and waiting for
///  Git to leave it would wait for the thing that never answers (owner:
/// "nothing should ever wait forever or
///  indefinitely").
/// A killed Git normally exits in microseconds,
/// so this grace is hundreds of thousands of times that,
/// and only a
///  process the kernel cannot stop uses it up.
const STOP_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

#[cfg(test)]
thread_local! {
    /// What:
    ///  What a control's runs report about Git after the kill,
    /// instead of asking the operating system.
    /// Why:
    ///   A grace of zero does not force an uncollected child:
    /// a child that exits before the next look is
    ///  collected,
    /// so a control that demanded being unable to collect one failed under a valid scheduling order and
    ///  without any defect.
    /// The control states the outcome and the production path is unchanged.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// let stopOutcome: 'collected' | 'left' | undefined;
    /// ```
    pub(crate) static STOP_OUTCOME_FOR_THIS_THREAD: std::cell::Cell<Option<StopOutcome>> = const { std::cell::Cell::new(None) };
}

/// What:
///  What became of Git when the ceiling ended it.
/// Why:
///   The report says what is known and nothing more,
/// and only two things are known:
/// Git was collected,
/// or it did not
///  exit inside the grace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StopOutcome = 'collected' | 'left';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StopOutcome {
    /// Git exited and was collected.
    Collected,
    /// Git was killed but had not exited when the grace ran out.
    Left,
}

/// What:
///  Run real Git as a child with inherited streams and wait no longer than `ceiling`,
/// ending Git when the ceiling is reached.
/// Why:
///   The wrapper's own Git steps are not the person's command and must not wait forever (owner:
/// "nothing should ever
///  wait forever or indefinitely"):
/// a step that runs the checkout's conversion filters can be held by a smudge filter,
/// a
///  Git LFS process or a filesystem that never answers.
/// Git keeps the process group and the terminal this wrapper has,
/// exactly as a Git the person starts themselves:
/// a
///  conversion filter that reads `/dev/tty` is then a foreground reader and not one the line discipline stops
///  with `SIGTTIN`,
/// and a `Ctrl-C` reaches Git as it always does.
/// The cost is the one native Git
///  has:
/// the filters and helpers Git started are not signalled when Git is killed,
/// so one of them may still be
///  running after this function returns,
/// and the error says so instead of claiming they ended.
///  The reap of Git itself is bounded by `STOP_GRACE`:
/// a Git the kernel cannot stop is left,
/// and the error says
///  that too,
/// because waiting for it is waiting for the call the ceiling exists to end.
///  The person's own forwarded command is never run this way:
/// native Git waits as long as its own work
///  takes,
/// and the wrapper adds no ceiling to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = spawn(gitPath, args, { stdio: 'inherit' });
/// const deadline = Date.now() + ceilingMs;
/// while (Date.now() < deadline) { const status = tryWait(child); if (status) return status; await delay(20); }
/// process.kill(child.pid, 'SIGKILL');
/// const collected = await collectWithin(child, STOP_GRACE);
/// throw new Error(collected ? `it ran longer than ${ceiling}s and Git ended` : `it ran longer than ${ceiling}s and Git has not exited`);
/// ```
pub fn run_real_git_until(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    ceiling: std::time::Duration,
) -> std::io::Result<ChildOutcome> {
    let mut command: Command = git_command(real_git, arguments, overlay);
    let mut child: std::process::Child = command.spawn_retrying()?;
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(outcome_of(status)),
            Ok(None) if began.elapsed() >= ceiling => break,
            Ok(None) => std::thread::sleep(BOUNDED_LOOK_INTERVAL),
            Err(error) => {
                stop_git(&mut child);
                return Err(error);
            }
        }
    }
    let outcome: StopOutcome = stop_git(&mut child);
    // The report says only what is known: Git ended, or it has not exited, and a helper Git started may still run, which is
    //  the one thing native Git also leaves behind when it is killed. The helper holds the streams it inherited, so a caller
    //  that waits for this command's standard error to close waits for that helper and not for this command.
    let ending: String = match outcome {
        StopOutcome::Collected => String::from("and Git ended"),
        StopOutcome::Left => format!("and Git was told to stop {STOP_GRACE:?} ago but has not exited, so it is inside a call the kernel cannot interrupt and this command did not wait for it"),
    };
    return Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!(
            "it ran longer than {ceiling:?} {ending}; a filter or helper Git started may still be running and holds the streams it inherited, so a caller reading this command's standard error to its end waits until that helper closes it, something it waited for never answered, and a lock it held may still stand (`index.lock` in the worktree's Git directory)"
        ),
    ));
}

/// What:
///  Kill Git and collect it,
/// waiting no longer than the stop grace,
/// and report what became of it.
/// Why:
///   Only Git is signalled.
/// Its filters and helpers stay in this process group and this terminal,
/// so a filter
///  that reads `/dev/tty` runs in the foreground as it does under native Git,
/// and a helper may outlive the kill,
/// which
///  the caller's message says.
/// A plain `wait` here would wait for Git to leave the call that the ceiling interrupted,
/// which is the
///  wait this whole path exists to avoid;
/// a Git left unreaped is a zombie for as long as this process lives,
/// which is
///  cheaper than a command that never returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(child.pid, 'SIGKILL'); return await collectWithin(child, STOP_GRACE);
/// ```
fn stop_git(child: &mut std::process::Child) -> StopOutcome {
    #[cfg(unix)]
    {
        if let Ok(pid) = libc::pid_t::try_from(child.id()) {
            // SAFETY: `kill` takes a process id and a signal number and touches no memory; the id is the child this function
            // was given, and a positive number names that process and not its group.
            let _stopped: libc::c_int = unsafe { libc::kill(pid, libc::SIGKILL) };
        }
    }
    #[cfg(test)]
    if let Some(reported) = STOP_OUTCOME_FOR_THIS_THREAD.with(std::cell::Cell::get) {
        return reported;
    }
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => return StopOutcome::Collected,
            Ok(None) if began.elapsed() >= STOP_GRACE => return StopOutcome::Left,
            Ok(None) => std::thread::sleep(BOUNDED_LOOK_INTERVAL),
            Err(_unwaitable) => return StopOutcome::Left,
        }
    }
}

/// What:
///  End this process by the signal Git was ended by,
/// as replacing the process with Git does,
/// and answer the code a shell
///  reports for that death when the signal cannot end it here.
/// Why:
///   The plain forward replaces this process with Git,
/// so the caller's `wait` sees `WIFSIGNALED` with Git's
///  signal and a shell reports 128 + the signal.
/// A route that keeps a process around Git,
/// the copy route's child or a coordinated run,
/// has to end the same way:
/// restoring the signal's default action and raising it makes the kernel report
///  exactly what Git's own death reported.
/// Rust's start makes `SIGPIPE` ignored,
/// and an ignored signal is discarded,
/// so the
///  default action is restored first.
/// The return is the fallback for a signal this process blocks or cannot receive,
///  where ending is impossible but the status still has to be one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // signal(SIG, SIG_DFL); raise(SIG); return 128 + SIG;  // only reached when SIG cannot end this process
/// ```
pub fn end_as_git_ended(signal: i32) -> i32 {
    #[cfg(unix)]
    {
        // SAFETY: `sigemptyset` and `sigaddset` write only this local set, `pthread_sigmask` reads it, `signal` and
        //  `raise` take one number each, and none of them touches other memory of this process; every call is checked
        //  only by what this process then observes, which is the death the caller asked for.
        unsafe {
            libc::signal(signal, libc::SIG_DFL);
            // What: The block this process may hold on the signal is lifted before it is raised.
            // Why:  A blocked signal stays pending and `raise` returns, so the wrapper left the command as a normal exit
            //       with `128 + the signal` where native Git's caller sees `WIFSIGNALED`; a signal a parent blocked before
            //       the wrapper ran, or one this process's own start left blocked, is removed here
            //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 5 of the
            //       round-3 review). `pthread_sigmask` changes the calling thread's mask, which is the thread `raise`
            //       sends to.
            let mut blocked: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut blocked);
            libc::sigaddset(&mut blocked, signal);
            let _unblocked = libc::pthread_sigmask(libc::SIG_UNBLOCK, &blocked, std::ptr::null_mut());
            libc::raise(signal);
        }
    }
    return SIGNAL_EXIT_BASE + signal;}

/// What:
///  Become real Git;
///  returns only when Git could not be started.
///       The returned `std::io::Error` says why the operating system refused.
///       `#[cfg(unix)]` and `#[cfg(not(unix))]` each compile one of the two inner blocks.
/// Why:
///   On Unix this process is replaced by Git,
///  which is exact forwarding:
///  same process
///       ID,
///  same streams,
///  same signals,
///  same exit status,
///  and no wrapper process left
///       between the caller and Git.
///  Windows cannot replace a process image;
///  the nearest
///       faithful behaviour there is to wait for Git and exit with its status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Unix has no Node equivalent: process.execve(realGit, args, env) replaces the process image.
/// // Elsewhere: process.exit(exitCode(await runRealGit(realGit, args, overlay)));
/// ```
pub fn replace_process_with_real_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> std::io::Error {
    #[cfg(unix)]
    {
        let mut command: Command = git_command(real_git, arguments, overlay);
        // The program replaces this very process, so the marker says "replace" instead of the builder's "child".
        mark_real_git_start(&mut command, StartMode::Replace);
        // `exec_retrying` replaces this process like `.exec()` and tries again when the start is refused.
        return exec_retrying(&mut command);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // `match` unpacks the run: success exits this process with Git's code.
        match run_real_git(real_git, arguments, overlay) {
            Ok(outcome) => std::process::exit(exit_code(outcome)),
            Err(error) => return error,
        }
    }
}

/// Real-Git process controls stay out of the release executable.
#[cfg(test)]
#[path = "forwarding_tests.rs"]
mod tests;
