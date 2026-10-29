//! What:
//!  Run the auto-push's real `git push` with a deadline,
//!  and derive the wait limits the push coordination uses.
//! Why:
//!  A push whose connection stalls would otherwise hold the branch's push lock forever,
//!  and every later commit on that branch would wait behind it.
//!  The push is stopped when its deadline passes,
//!  the commit stays saved locally,
//!  and the failure is recorded like any other failed push.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { exitCode, output } = await nanoSpawn(gitPath, pushArgs, { cwd, signal: AbortSignal.timeout(deadlineMs) });
//! ```

/// The attempt record a finished or stopped push produces.
use super::auto_push_single_flight::{PushAttempt, SingleFlightError};
/// The diagnostic log of the wrapper.
use super::diagnostic_log::debug;
/// Starts that ride out a short shortage of processes.
/// The anonymous output file of the push, made in the per-user scratch directory.
use super::scratch_location::scratch_file;
use super::start_retry::RetryStart;
/// Owned OS strings of the child environment.
use std::ffi::{OsStr, OsString};
/// `File` is the shared anonymous output file; `Read`, `Seek` and `SeekFrom` read it back from its start.
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
/// `Child` is the running push; `Command` describes it; `Stdio` connects its streams.
use std::process::{Child, Command, ExitStatus, Stdio};
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// Environment variable that sets the push deadline in whole seconds.
pub const PUSH_DEADLINE_VARIABLE: &str = "CLI_GIT_PUSH_DEADLINE_SECONDS";

/// Environment variable that sets how long to wait for another invocation's push lock, in whole seconds.
pub const LOCK_WAIT_VARIABLE: &str = "CLI_GIT_PUSH_LOCK_WAIT_SECONDS";

/// How long a push may run when the variable is absent or unusable.
pub const DEFAULT_PUSH_DEADLINE: Duration = Duration::from_secs(600);

/// How long a stopped push gets to end after the polite stop signal before it is killed.
pub const STOP_GRACE: Duration = Duration::from_secs(5);

/// Extra time a waiter gives the lock holder beyond the holder's whole push,
///  so a holder that runs to its own deadline is outwaited.
pub const LOCK_WAIT_MARGIN: Duration = Duration::from_secs(60);

/// The most output bytes kept from one push;
///  a push that writes more is stopped.
pub const MAX_PUSH_OUTPUT: u64 = 4 * 1024 * 1024;

/// Exit code recorded for a push that this wrapper stopped,
///  the convention of `timeout(1)`.
pub const STOPPED_EXIT_CODE: i64 = 124;

/// Lowest transfer speed in bytes per second Git tolerates on an HTTP transfer.
pub const LOW_SPEED_LIMIT: &str = "1000";

/// Seconds the transfer may stay below the lowest speed before Git aborts it.
pub const LOW_SPEED_TIME: &str = "60";

/// Pause between two looks at the running push.
const POLL_PAUSE: Duration = Duration::from_millis(10);

/// What:
///  Read a positive whole number of seconds from one environment value.
/// Why:
///   A zero, negative, fractional, empty or non-text value is no setting at all,
///  never "no limit",
///  so a typo cannot remove a bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seconds = Number(value); return Number.isInteger(seconds) && seconds > 0 ? seconds * 1000 : undefined;
/// ```
pub fn positive_seconds(name: &str, value: &OsStr) -> Option<Duration> {
    let parsed: Option<u64> = match value.to_str() {
        Some(text) => text.trim().parse::<u64>().ok(),
        None => None,
    };
    return match parsed {
        Some(count) if count > 0 => Some(Duration::from_secs(count)),
        Some(_) | None => {
            debug(
                TAG,
                &format!("{name} is not a positive whole number; ignoring it"),
            );
            None
        }
    };
}

/// What:
///  Read a positive whole number of seconds from the child environment.
/// Why:
///   The last of repeated entries wins,
///  as in an environment block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveSeconds(env[name])
/// ```
fn seconds_variable(environment: &[(OsString, OsString)], name: &str) -> Option<Duration> {
    let mut found: Option<&OsString> = None;
    for (key, value) in environment {
        if key.to_str() == Some(name) {
            found = Some(value);
        }
    }
    let value: &OsString = found?;
    return positive_seconds(name, value.as_os_str());
}

/// What:
///  The push deadline: the environment's value, or the default.
/// Why:
///   A person may need a longer deadline on a slow link or a shorter one on a flaky one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadlineMs = secondsOf(env.CLI_GIT_PUSH_DEADLINE_SECONDS) ?? DEFAULT_MS;
/// ```
pub fn push_deadline(environment: &[(OsString, OsString)]) -> Duration {
    return seconds_variable(environment, PUSH_DEADLINE_VARIABLE).unwrap_or(DEFAULT_PUSH_DEADLINE);
}

/// What:
///  How long a process waits for another invocation's push lock:
///  the environment's value, or the push deadline plus the stop grace plus a margin.
/// Why:
///   The holder may run its whole push deadline and then needs the stop grace to end,
///  so the default waiter outlasts a holder that is merely slow before it reports the lock as stuck.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lockWaitMs = secondsOf(env.CLI_GIT_PUSH_LOCK_WAIT_SECONDS) ?? pushDeadlineMs + STOP_GRACE_MS + LOCK_WAIT_MARGIN_MS;
/// ```
pub fn lock_wait_limit(environment: &[(OsString, OsString)]) -> Duration {
    if let Some(limit) = seconds_variable(environment, LOCK_WAIT_VARIABLE) {
        return limit;
    }
    return push_deadline(environment)
        .saturating_add(STOP_GRACE)
        .saturating_add(LOCK_WAIT_MARGIN);
}

/// What:
///  Whether the environment already sets a name.
/// Why:
///   The low-speed defaults must not override a person's own setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name in env
/// ```
fn sets(environment: &[(OsString, OsString)], name: &str) -> bool {
    for (key, _) in environment {
        if key.to_str() == Some(name) {
            return true;
        }
    }
    return false;
}

/// What:
///  Add Git's low-speed abort to a push command unless the environment sets it.
/// Why:
///   Git ends an HTTP transfer that stays under the limit for the time,
///  which also ends its helper processes;
///  the push deadline is only the backstop for transports this does not cover.
///  Git lets these variables override `http.lowSpeedLimit` and `http.lowSpeedTime` from its configuration,
///  so only a person's environment variables are kept, not their configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// env.GIT_HTTP_LOW_SPEED_LIMIT ??= '1000'; env.GIT_HTTP_LOW_SPEED_TIME ??= '60';
/// ```
pub fn apply_low_speed_defaults(command: &mut Command, environment: &[(OsString, OsString)]) {
    if !sets(environment, "GIT_HTTP_LOW_SPEED_LIMIT") {
        command.env("GIT_HTTP_LOW_SPEED_LIMIT", LOW_SPEED_LIMIT);
    }
    if !sets(environment, "GIT_HTTP_LOW_SPEED_TIME") {
        command.env("GIT_HTTP_LOW_SPEED_TIME", LOW_SPEED_TIME);
    }
}

/// Remove exactly one trailing `\n` or `\r\n`, as the captured output of the incumbent does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// output.endsWith('\n') ? output.slice(0, output.endsWith('\r\n') ? -2 : -1) : output
/// ```
pub fn strip_one_line_break(text: &str) -> &str {
    let without_lf = match text.strip_suffix('\n') {
        Some(stripped) => stripped,
        None => return text,
    };
    return without_lf.strip_suffix('\r').unwrap_or(without_lf);
}

/// Ask a running push to end,
///  so Git can stop the helper processes it started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.kill('SIGTERM');
/// ```
#[cfg(unix)]
fn ask_to_stop(child: &Child) {
    let Ok(pid) = libc::pid_t::try_from(child.id()) else {
        return;
    };
    // SAFETY: `kill` sends one signal to the PID of a child this module started and has not reaped;
    // a failure is ignored because the child may have ended.
    unsafe { libc::kill(pid, libc::SIGTERM) };
}

/// Platforms without a polite stop signal go straight to the kill.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn ask_to_stop(_child: &Child) {}

/// Whether part of the grace is still left after the stop request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// asked < grace
/// ```
fn grace_remains(asked: Duration, grace: Duration) -> bool {
    return asked < grace;
}

/// Whether an output of this size is a flood: only a size past the cap is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// size > MAX_PUSH_OUTPUT
/// ```
fn is_flood(size: u64) -> bool {
    return size > MAX_PUSH_OUTPUT;
}

/// What:
///  Stop a running push: ask politely,
///  wait the grace,
///  then kill and collect it.
/// Why:
///   Git's own signal handling ends the helpers it started,
///  and a push that ignores the request cannot outlive the kill.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.kill('SIGTERM'); await Promise.race([exited, delay(graceMs)]) || child.kill('SIGKILL');
/// ```
pub fn stop_push(child: &mut Child, grace: Duration) {
    ask_to_stop(child);
    let asked: Instant = Instant::now();
    while grace_remains(asked.elapsed(), grace) {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => return,
            Ok(None) => std::thread::sleep(POLL_PAUSE),
        }
    }
    // The child is being abandoned; its exit is not reported, only the cause.
    let _killed = child.kill();
    let _collected = child.wait();
}

/// Read the shared output file back from its start,
///  keeping at most the output cap so a flood is never held in memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readFile(outputPath, 'utf8')
/// ```
fn read_output(file: &mut File) -> Result<String, SingleFlightError> {
    file.seek(SeekFrom::Start(0))
        .map_err(SingleFlightError::Start)?;
    let mut bytes: Vec<u8> = Vec::new();
    file.take(MAX_PUSH_OUTPUT)
        .read_to_end(&mut bytes)
        .map_err(SingleFlightError::Start)?;
    return Ok(String::from_utf8_lossy(&bytes).into_owned());
}

/// Why a push or query was stopped.
pub enum StopCause {
    /// The deadline passed.
    Deadline,
    /// The output grew past the cap.
    Flood,
}

/// What:
///  Look at a running push until it ends,
///  its deadline passes or its output floods.
/// Why:
///   `std` has no timed wait,
///  so the parent looks with `try_wait`,
///  which never blocks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([exited, deadline, flood]);
/// ```
pub fn wait_until_done(
    child: &mut Child,
    outputs: &[&File],
    deadline: Duration,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let started: Instant = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(SingleFlightError::Start)? {
            return Ok(Ok(status));
        }
        for output in outputs {
            let size: u64 = match output.metadata() {
                Ok(metadata) => metadata.len(),
                Err(_) => u64::MAX,
            };
            if is_flood(size) {
                return Ok(Err(StopCause::Flood));
            }
        }
        let elapsed: Duration = started.elapsed();
        if elapsed >= deadline {
            return Ok(Err(StopCause::Deadline));
        }
        std::thread::sleep(POLL_PAUSE.min(deadline.saturating_sub(elapsed)));
    }
}

/// What:
///  Run a push command to its end or its deadline and capture its interleaved output.
/// Why:
///   Both streams share one anonymous file so lines keep the order Git wrote them,
///  and a file needs no reader thread that a stuck helper could keep alive.
///  A stopped push becomes a failed attempt that says so,
///  so the commit stays saved and the failure is recorded for later invocations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBoundedPush({ command, deadlineMs, graceMs }): Promise<PushAttempt>;
/// ```
pub fn run_bounded_push(
    mut command: Command,
    deadline: Duration,
    grace: Duration,
    environment: &[(OsString, OsString)],
) -> Result<PushAttempt, SingleFlightError> {
    let mut output: File = scratch_file(environment).map_err(SingleFlightError::Start)?;
    let for_stdout: File = output.try_clone().map_err(SingleFlightError::Start)?;
    let for_stderr: File = output.try_clone().map_err(SingleFlightError::Start)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(for_stdout))
        .stderr(Stdio::from(for_stderr));
    let mut child: Child = command.spawn_retrying().map_err(SingleFlightError::Start)?;
    drop(command);
    let ended: Result<ExitStatus, StopCause> = wait_until_done(&mut child, &[&output], deadline)?;
    if ended.is_err() {
        stop_push(&mut child, grace);
    }
    let captured: String = read_output(&mut output)?;
    let shown: &str = strip_one_line_break(captured.as_str());
    return Ok(match ended {
        Ok(status) => PushAttempt {
            exit_code: i64::from(status.code().unwrap_or(1)),
            output: shown.to_owned(),
        },
        Err(cause) => {
            let reason: String = match cause {
                StopCause::Deadline => format!(
                    "cli-git: git push did not finish within {} seconds and was stopped; \
                     set {PUSH_DEADLINE_VARIABLE} to allow longer. Hint: if you were pushing over a slow or \
                     offline connection, your commit is already saved; push again when the connection works.",
                    deadline.as_secs()
                ),
                StopCause::Flood => format!(
                    "cli-git: git push wrote more than {MAX_PUSH_OUTPUT} bytes of output and was stopped. Hint: if a hook or the \
                     remote prints this much, run `git ls-remote` to see whether your commits arrived, and read the \
                     hook's own output by running it directly before pushing again."
                ),
            };
            debug(TAG, &reason);
            let joined: String = if shown.is_empty() {
                reason
            } else {
                format!("{shown}\n{reason}")
            };
            PushAttempt {
                exit_code: STOPPED_EXIT_CODE,
                output: joined,
            }
        }
    });
}

/// Bounded push behavior is controlled in the test module.
#[cfg(test)]
#[path = "bounded_push_tests.rs"]
mod tests;
