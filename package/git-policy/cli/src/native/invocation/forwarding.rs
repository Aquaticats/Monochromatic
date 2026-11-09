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
    return command;
}

/// The base a POSIX shell and Git 2.56.0 `run-command.c` add to a signal number.
pub const SIGNAL_EXIT_BASE: i32 = 128;

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
    // What: `.status()` starts the child with inherited streams and waits for it;
    //       a trailing `?` returns a start failure to our caller.
    // Why:  Inherited streams are the default for `.status()`, so Git reads the
    //       caller's stdin and writes the caller's stdout and stderr itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const status = await spawn(realGit, args, { stdio: 'inherit' });
    // ```
    let status: std::process::ExitStatus = git_command(real_git, arguments, overlay).status_retrying()?;
    // `Ok(...)` is the success variant carrying how the child ended.
    return Ok(outcome_of(status));
}

/// What:
///  How often a run with a ceiling looks at its child before the ceiling.
/// Why:
///   `Child::try_wait` is the whole wait,
/// so the loop that reads the clock needs an interval;
/// one look every twentieth of a second
///  costs nothing beside a checkout and ends the child close to its ceiling.
const BOUNDED_LOOK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(20);

/// What:
///  How long a child that was just killed is given to exit and be collected before the wrapper leaves without it.
/// Why:
///   `SIGKILL` cannot be ignored,
/// so a child still running after this grace is inside a system call the kernel cannot
///  interrupt (a hung filesystem,
/// a dead NFS or FUSE server);
/// the ceiling exists for exactly that case,
/// and waiting for the
///  child to leave it would wait for the thing that never answers (owner:
/// "nothing should ever wait forever or
///  indefinitely").
/// A killed child normally exits in microseconds,
/// so this grace is hundreds of thousands of times that,
/// and only a child the kernel cannot stop uses it up.
const STOP_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

#[cfg(test)]
thread_local! {
    /// What:
    ///  The grace for the runs of the current thread;
    /// a control shortens it so a child that cannot be collected ends the control
    ///  at once instead of at the shipped bound.
    /// Why:
    ///   The grace is a shipped resource limit,
    /// and a control that proved the leaving path by a real hung filesystem would need a
    ///  hung filesystem.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// let stopGrace = STOP_GRACE;
    /// ```
    pub(crate) static STOP_GRACE_FOR_THIS_THREAD: std::cell::Cell<Option<std::time::Duration>> = const { std::cell::Cell::new(None) };
}

/// The stop grace the current thread's runs use.
fn stop_grace() -> std::time::Duration {
    #[cfg(test)]
    if let Some(shortened) = STOP_GRACE_FOR_THIS_THREAD.with(std::cell::Cell::get) {
        return shortened;
    }
    return STOP_GRACE;
}

/// What:
///  Run real Git as a child with inherited streams and wait no longer than `ceiling`,
/// ending the child and every process
///  it started when the ceiling is reached.
/// Why:
///   The wrapper's own Git steps are not the person's command and must not wait forever (owner:
/// "nothing should ever
///  wait forever or indefinitely"):
/// a step that runs the checkout's conversion filters can be held by a smudge filter,
/// a
///  Git LFS process or a filesystem that never answers.
/// The child is its own process group,
/// so a stop takes the
///  filters it started with it and nothing beside it,
/// and the leader is then reaped so it leaves no zombie.
///  The reap is bounded by `STOP_GRACE`:
/// a child the kernel cannot stop is left,
/// and the error says so,
/// because
///  waiting for it is waiting for the call the ceiling exists to end.
///  The person's own forwarded command is never run this way:
/// native Git waits as long as its own work
///  takes,
/// and the wrapper adds no ceiling to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = spawn(gitPath, args, { stdio: 'inherit', detached: true });
/// const deadline = Date.now() + ceilingMs;
/// while (Date.now() < deadline) { const status = tryWait(child); if (status) return status; await delay(20); }
/// process.kill(-child.pid, 'SIGKILL');
/// const collected = await collectWithin(child, STOP_GRACE);
/// throw new Error(collected ? `it ran longer than ${ceiling}s and was ended` : `it ran longer than ${ceiling}s and has not exited`);
/// ```
pub fn run_real_git_until(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    ceiling: std::time::Duration,
) -> std::io::Result<ChildOutcome> {
    let mut command: Command = git_command(real_git, arguments, overlay);
    // The group is this child's own, so a stop reaches every filter and helper Git started and nothing else.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child: std::process::Child = command.spawn_retrying()?;
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(outcome_of(status)),
            Ok(None) if began.elapsed() >= ceiling => break,
            Ok(None) => std::thread::sleep(BOUNDED_LOOK_INTERVAL),
            Err(error) => {
                stop_child_group(&mut child);
                return Err(error);
            }
        }
    }
    let collected: bool = stop_child_group(&mut child);
    // A child that did not exit inside the grace is still running: the message says what was seen instead of claiming it ended.
    let ending: String = if collected {
        String::from("and was ended, with the filters and helpers it started")
    } else {
        format!("and was told to stop {:?} ago, but it has not exited: it is inside a call the kernel cannot interrupt, so it may still write, and this command did not wait for it", stop_grace())
    };
    return Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!(
            "it ran longer than {ceiling:?} {ending}; something it waited for never answered, and a lock it held may still stand (`index.lock` in the worktree's Git directory)"
        ),
    ));
}

/// What:
///  End the child's whole process group and collect the leader,
/// waiting no longer than the stop grace,
///  and report whether the leader was collected.
/// Why:
///   Killing only the leader would leave a smudge filter or a Git LFS process writing into the worktree
///  beside a cleanup that removes what it writes.
/// A plain `wait` here would wait for the child to leave the call that the ceiling
///  interrupted,
/// which is the wait this whole path exists to avoid;
/// a leader left unreaped is a zombie for as
///  long as this process lives,
/// which is cheaper than a command that never returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(-child.pid, 'SIGKILL'); return await collectWithin(child, STOP_GRACE);
/// ```
fn stop_child_group(child: &mut std::process::Child) -> bool {
    #[cfg(unix)]
    {
        if let Ok(leader) = libc::pid_t::try_from(child.id()) {
            // SAFETY: `kill` takes a process-group id and a signal number and touches no memory; the group was made for this
            // child by `process_group(0)` above.
            let _stopped: libc::c_int = unsafe { libc::kill(-leader, libc::SIGKILL) };
        }
    }
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => return true,
            Ok(None) if began.elapsed() >= stop_grace() => return false,
            Ok(None) => std::thread::sleep(BOUNDED_LOOK_INTERVAL),
            Err(_unwaitable) => return false,
        }
    }
}

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
