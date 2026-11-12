//! What:
//!  The parent's run of the Git LFS upload before a push:
//! `git-lfs push <remote> <refs>` as one child it owns.
//! Why:
//!  Without an upload step a push publishes pointer files whose objects the LFS server never received.
//!  The wrapper runs `git-lfs push` itself,
//! so a failure is explained by the process that saw it and the push,
//! which
//!  then carries no hook,
//! is never started.
//! Nothing here starts the wrapper,
//! writes shell text or injects Git
//!  configuration (`doc/planning/cli-git-no-self-invocation.md`).
//! What to upload is decided in `lfs_upload_plan`.
//!  The child sees what the caller gave Git:
//! the global `-c` and `--config-env` options travel as the entries Git appends to `GIT_CONFIG_PARAMETERS`,
//! `--git-dir`,
//!  `--work-tree` and `--namespace` as their variables,
//! `-C` as the working directory,
//! and the real Git's directory
//!  leads its `PATH` so a `git` it starts is never the wrapper while `git-lfs` itself is the one the caller's `PATH`
//!  finds first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await spawn(findProgram(env.PATH), plan.arguments(), { env: withPrefix(env, globalPrefix), stdio: 'inherit' });
//! ```

/// The attempt record,
/// the coordinator's Git and its error type.
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError};
/// The deadline,
/// the grace and the push runner,
/// the wait a stalled child is ended by,
/// and the line that says how a longer one is allowed.
use crate::bounded_push::{
    STOPPED_EXIT_CODE, STOP_GRACE, StopCause, command_label, connected_window, push_patience, wait_until_done_noting, run_bounded_push,
};
/// The invocation's environment values,
/// and whether its overlay turns prompting off for the children.
use crate::child_environment::{environment_value, environment_with_overlay, prompting_disabled};
/// The caller's own command-line configuration,
/// as the variables the child reads.
pub use crate::lfs_command_line_configuration::child_configuration_variables;
/// How a child ended,
/// as a status.
use crate::forwarding::{ChildOutcome, exit_code, outcome_of};
/// The program lookup,
/// the explanation after a failure,
/// and the refusal a push without the program prints.
use crate::lfs_pre_push::{failed_message, find_program, missing_program_message};
/// Every word about a failed,
/// stopped or refused upload.
use crate::lfs_upload_explanations::{
    MAX_RELAYED_BYTES, capture_failed_message, explanation_after_failure, failed_upload, guard_failed_message, missing_program_attempt,
    stopped_upload_words,
};
/// Hides the credentials in a URL of relayed text.
use crate::lfs_output_mask::mask_url_secrets;
use crate::lfs_upload_plan::UploadPlan;
/// The anonymous file the interactive upload's standard error goes to.
use crate::scratch_location::scratch_file;
/// Relays signals to the child,
/// which starts it and hands it over still published.
use crate::signal_relay::SignalGuard;
use crate::signal_relay_relayed_child::RelayedChild;
/// The child held until it is reaped,
/// awaited on its pidfd.
use crate::child_awaited_through_a_pidfd::{OwnedChild, adopt};
/// The locks a stopped child leaves,
/// none of which the upload claims.
use crate::stopped_git_locks::LockLedger;
/// The end of a stopped child,
/// and the wait a push's patience lets pass.
use crate::stop_of_a_push_and_sweep_of_its_locks::stop_push;
/// The process's runtime,
/// which this synchronous face enters.
use crate::lazy_runtime::run_on_runtime;
/// Starts that ride out a short shortage of processes.
/// Owned OS text for arguments.
use std::ffi::OsString;
/// The real Git whose directory leads `PATH`.
use std::path::Path;
/// One job of the process's bound on blocking work,
/// for the filesystem lookups a task may not make.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The child and its standard error.
use std::process::{Command, ExitStatus, Stdio};
/// Reading the captured standard error back.
use std::io::{Read, Seek, SeekFrom};

/// `PATH` with the directory of the real Git first,
/// so a `git` that `git-lfs` starts is the real Git and never the
///  wrapper (Git does the same for the hooks it starts).
fn path_with_real_git_first(environment: &[(OsString, OsString)], real_git: &Path) -> OsString {
    let existing: OsString = environment_value(environment, "PATH").unwrap_or_default();
    let mut directories: Vec<std::path::PathBuf> = Vec::new();
    if let Some(directory) = real_git.parent() {
        directories.push(directory.to_path_buf());
    }
    directories.extend(std::env::split_paths(existing.as_os_str()));
    return std::env::join_paths(directories).unwrap_or(existing);
}


/// Why a run with captured standard error did not produce an outcome.
enum CaptureError {
    /// The file for the standard error could not be made;
    /// the text says why.
    Scratch(String),
    /// The program could not be started.
    Start,
    /// The wait for the program could not be made at all:
    /// no signal handler could be installed,
    /// or the process's runtime refused.
    /// The text says why.
    Guard(String),
}

/// What:
///  Why one bounded run of the upload produced no end.
/// Why:
///   A program that never started is explained by the missing-program refusal,
/// while a wait that could not be made is a defect of this process and is named as one,
/// so the two never share words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RunError = { kind: 'start' } | { kind: 'wait'; why: string };
/// ```
enum RunError {
    /// The program could not be started.
    Start,
    /// The wait failed,
    /// or no relay could be published;
    /// the text says why.
    Wait(String),
}

/// What:
///  How one run of the upload ended:
///  the program's own end,
/// or the end the wrapper gave it for showing no sign of work.
/// Why:
///   The two are explained in different words,
/// and a stopped run's status is the wrapper's `timeout(1)` convention rather than a status the program chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UploadEnd = { kind: 'finished'; outcome: ChildOutcome } | { kind: 'stopped'; cause: StopCause };
/// ```
enum UploadEnd {
    /// The program ended by itself.
    Finished(ChildOutcome),
    /// The wrapper ended it,
    /// and the cause says which limit came first.
    Stopped(StopCause),
}

/// What:
///  Run the command with its standard error in an anonymous file and return how it ended with that text,
/// the
///  credentials of every URL in it hidden.
/// Why:
///   Standard output stays the caller's,
/// so the progress meter is unchanged.
/// The standard error is where `git-lfs`
///  prints its errors,
/// which can carry the `lfs.url` token,
/// so it is relayed only after its URLs are masked.
/// A file needs no reader thread that a
///  stuck helper could keep alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { outcome, stderr } = await spawnCapturingStderr(command); return [outcome, maskUserinfo(stderr)];
/// ```
fn run_with_captured_stderr(
    command: &mut Command,
    environment: &[(OsString, OsString)],
) -> Result<(UploadEnd, Vec<u8>), CaptureError> {
    let mut captured: std::fs::File = match scratch_file(environment) {
        Ok(file) => file,
        Err(error) => return Err(CaptureError::Scratch(error.to_string())),
    };
    let for_child: std::fs::File = match captured.try_clone() {
        Ok(file) => file,
        Err(error) => return Err(CaptureError::Scratch(error.to_string())),
    };
    command.stderr(Stdio::from(for_child));
    // What: The program reads no update lines (its refs are arguments), so its input is the null device, as the bounded
    //  upload of the auto-push gives it.
    // Why:  An inherited input of a command nobody typed could be a pipe another program waits on.
    command.stdin(Stdio::null());
    let guard: SignalGuard = match SignalGuard::install() {
        Ok(installed) => installed,
        Err(why) => return Err(CaptureError::Guard(why.to_string())),
    };
    let ended: UploadEnd = match run_on_runtime(await_upload(command, &guard, &captured, environment)) {
        Ok(Ok(finished)) => finished,
        Ok(Err(RunError::Start)) => return Err(CaptureError::Start),
        Ok(Err(RunError::Wait(why))) => return Err(CaptureError::Guard(why)),
        Err(refusal) => return Err(CaptureError::Guard(refusal.to_string())),
    };
    let mut bytes: Vec<u8> = Vec::new();
    // The file is read only after the seek to its start succeeded, as `.and_then(...)` ran the read.
    let rewound: std::io::Result<u64> = captured.seek(SeekFrom::Start(0));
    let read: std::io::Result<usize> = match rewound {
        Ok(_) => captured.by_ref().take(MAX_RELAYED_BYTES).read_to_end(&mut bytes),
        Err(error) => Err(error),
    };
    if let Err(error) = read {
        return Err(CaptureError::Scratch(error.to_string()));
    }
    return Ok((ended, mask_url_secrets(bytes.as_slice())));
}

/// What:
///  One `git-lfs push` run to its end,
/// or until it shows no sign of work for the push's stall window,
/// with the wrapper's signals relayed to it and its standard error already captured.
/// Why:
///   A server that accepts the connection and never answers held a typed push for good before this
/// (`package/git-policy/cli/SPEC.md`,
/// once "LFS is paused");
/// the auto-push's upload was bounded and this face was not.
/// The child is stopped and its whole tree ended when the window passes,
/// and a stop is reported as a stop,
/// never as the program's answer.
/// The relay stays published for the whole wait,
/// so a signal sent to the wrapper still ends the upload.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const relayed = await guard.startRelayed(command); const child = relayed.takeChild();
/// const ended = await waitUntilDoneOrStalled(child, captured, patience); if (ended.stopped) await stopPush(child, grace);
/// return ended;
/// ```
async fn await_upload(
    command: &mut Command,
    guard: &SignalGuard,
    captured: &std::fs::File,
    environment: &[(OsString, OsString)],
) -> Result<UploadEnd, RunError> {
    let mut relayed: RelayedChild = match guard.start_relayed(command).await {
        // A program that never started is the missing-program refusal, not a defect of the wait.
        Err(_error) => return Err(RunError::Start),
        Ok(started) => started,
    };
    let started: std::process::Child = relayed.take_child().expect("the child was handed over once");
    let mut child: OwnedChild = adopt(started);
    let label: String = command_label(command);
    let mut waits: Vec<crate::push_progress::WaitReport> = Vec::new();
    // The upload leaves no reference lock, so the ledger the stop sweeps is empty.
    let locks: LockLedger = LockLedger::none();
    let patience = push_patience(environment);
    let ended: Result<ExitStatus, StopCause> = match wait_until_done_noting(
        &mut child,
        &[captured],
        patience,
        label.as_str(),
        &mut waits,
        None,
        connected_window(environment),
    )
    .await
    {
        // A wait that could not be made at all is this process's own defect, and its words say which.
        Err(wait) => return Err(RunError::Wait(wait.to_string())),
        Ok(finished) => finished,
    };
    let finished: UploadEnd = match ended {
        // `outcome_of` maps the status the way the wrapper's exit and its notices read it, a death by a signal included.
        Ok(status) => UploadEnd::Finished(outcome_of(status)),
        Err(cause) => {
            // The upload is ended with the grace every stopped child gets, and what it started goes with it.
            stop_push(child, STOP_GRACE, locks).await;
            UploadEnd::Stopped(cause)
        }
    };
    // The token stops the relay pointing at the child here, once the wait no longer owns a living one.
    drop(relayed);
    return Ok(finished);
}

/// What:
///  Run the upload as a child with the caller's streams and return the status to stop with,
/// or nothing when it succeeded.
/// Why:
///   The parent saw the child end,
/// so it prints the explanation after the program's own output;
/// the push is not started.
///  A signal sent to the wrapper is relayed to the child,
/// so stopping the wrapper stops the upload.
/// A push that also
///  pushes submodules gets a notice,
/// because their objects are not part of this upload.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await spawn('git-lfs', plan.arguments(), { stdio: 'inherit' }); if (code) { print(failedMessage(...)); return code; }
/// ```
pub fn run_upload_interactive(
    plan: &UploadPlan,
    global_prefix: &[OsString],
    directory: &Path,
    real_git: &Path,
    environment: &[(OsString, OsString)],
    overlay: &[(OsString, OsString)],
    diagnostics: &mut dyn std::io::Write,
) -> Option<i32> {
    let Some(program) = find_program(environment_value(environment, "PATH").unwrap_or_default().as_os_str()) else {
        // Terminal output error: the exit status already carries the refusal.
        let _written = diagnostics.write_all(missing_program_message().as_bytes());
        return Some(1);
    };
    if plan.submodules {
        // Terminal output error: the notice is advisory.
        let _written = diagnostics.write_all(submodule_notice().as_bytes());
    }
    // The overlay carries the never-prompt pair only when this command has no controlling terminal, and the explanation of a
    //  failed login says what the wrapper did about prompting.
    let prompting_is_disabled: bool = prompting_disabled(overlay);
    // What: The caller's configuration, as Git hands it to a child: appended to the parameters the environment already holds.
    // Why:  Git reads those parameters after the numbered pairs, so a pair written there is the newest setting of its key;
    //  a caller's pair numbered before an inherited parameter lost to it, and `-c lfs.url=...` reached the upload as the
    //  server the caller had replaced, so the object of the published pointer went to the wrong store.
    let settled: Vec<(OsString, OsString)> = environment_with_overlay(environment, overlay);
    let Some(carried) = child_configuration_variables(global_prefix, settled.as_slice()) else {
        // A `--config-env` Git refuses ends the push itself, so no object may travel to the server its absence points at.
        return None;
    };
    // A server error (an HTTP 5xx answer of the LFS server) is not run again here: it is outside our control, like a network that
    //  is down, and the commit service's push retry plan tries the push again by itself (pick 12 of
    //  `doc/planning/cli-git-shipped-limits-audit.md`, the coordinator's under the owner's delegation). A typed push ends and says so.
    for remote in &plan.remotes {
        for arguments in plan.runs(remote.as_os_str()) {
            let mut command: Command = Command::new(&program);
            command
                .current_dir(directory)
                .args(arguments.as_slice())
                .env("PATH", path_with_real_git_first(environment, real_git));
            for (name, value) in overlay.iter().cloned().chain(carried.iter().cloned()) {
                command.env(name, value);
            }
            let (ended, relayed): (UploadEnd, Vec<u8>) = match run_with_captured_stderr(&mut command, environment) {
                Ok(finished) => finished,
                Err(CaptureError::Scratch(why)) => {
                    // Terminal output error: the exit status already carries the refusal.
                    let _written = diagnostics.write_all(capture_failed_message(why.as_str()).as_bytes());
                    return Some(1);
                }
                Err(CaptureError::Start) => {
                    // Terminal output error: see above.
                    let _written = diagnostics.write_all(missing_program_message().as_bytes());
                    return Some(1);
                }
                Err(CaptureError::Guard(why)) => {
                    // Terminal output error: see above.
                    let _written = diagnostics.write_all(guard_failed_message(why.as_str()).as_bytes());
                    return Some(1);
                }
            };
            // Terminal output error: the program's own words are advisory; the exit status carries the result.
            let _relayed = diagnostics.write_all(relayed.as_slice());
            let (code, outcome): (i32, ChildOutcome) = match ended {
                UploadEnd::Finished(outcome) => (exit_code(outcome), outcome),
                UploadEnd::Stopped(cause) => {
                    // Terminal output error: the explanation carries the result.
                    let _written = diagnostics.write_all(failed_message(stopped_upload_words(&cause).as_str()).as_bytes());
                    return Some(i32::try_from(STOPPED_EXIT_CODE).unwrap_or(1));
                }
            };
            if code == 0 {
                continue;
            }
            let description: String = match outcome {
                ChildOutcome::Exited(status) => format!("exited with status {status}"),
                ChildOutcome::Signaled(signal) => format!("was ended by signal {signal}"),
            };
            // Terminal output error: see above.
            let _written = diagnostics.write_all(
                explanation_after_failure(description.as_str(), remote.as_os_str(), relayed.as_slice(), prompting_is_disabled)
                    .as_bytes(),
            );
            return Some(code);
        }
    }
    return None;
}

/// What:
///  The notice for a push that also pushes submodules.
/// Why:
///   Git pushes a submodule's commits itself;
/// their LFS objects are not uploaded by the wrapper,
/// and the pointer files
///  would be published without them.
/// Every way forward is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// 'monochromatic-git-policy-cli: this push also pushes submodules, ...'
/// ```
pub fn submodule_notice() -> String {
    return String::from(
        "monochromatic-git-policy-cli: this push also pushes submodules (`--recurse-submodules`), and the wrapper uploads Git LFS objects only \\
         for this repository's own refs. If a submodule uses Git LFS, upload its objects first by running \\
         `git lfs push --all <remote>` inside it, or push the submodule on its own; otherwise its pointer files are \\
         published without their objects.\\n",
    );
}

/// What:
///  Run the upload as a bounded child whose output is captured,
/// and return the failed attempt to stop with,
/// or nothing
///  when it succeeded.
/// Why:
///   The auto-push records and shows a failed attempt's output;
/// the explanation is part of it.
/// A stop by deadline says
///  so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = await runBoundedPush({ command: gitLfs(plan), deadlineMs }); return attempt.exitCode ? withExplanation(attempt) : undefined;
/// ```
pub fn run_upload_bounded(git: &PushGit, plan: &UploadPlan) -> Result<Option<PushAttempt>, SingleFlightError> {
    let Some(commands) = upload_commands(git, plan) else {
        return Ok(Some(missing_program_attempt()));
    };
    for (remote, command) in commands {
        let attempt: PushAttempt =
            match run_bounded_push(command, push_patience(git.environment.as_slice()), STOP_GRACE, git.environment.as_slice()) {
                Ok(finished) => finished,
                Err(SingleFlightError::Start(_)) => return Ok(Some(missing_program_attempt())),
                Err(other) => return Err(other),
            };
        if let Some(failed) = failed_upload(&attempt, remote.as_os_str(), prompting_disabled(git.environment.as_slice())) {
            return Ok(Some(failed));
        }
    }
    return Ok(None);
}

/// What:
///  The same upload,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push uploads the objects of its commits as a task,
/// and the synchronous run enters the runtime for the child's wait and is refused from there.
/// The programs,
/// their arguments and what a failure says are the synchronous face's,
/// so the two cannot
///  upload different objects or report differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const command of uploadCommands(git, plan)) { const attempt = await runBoundedPushAwaiting(command); if (attempt.exitCode) return explained(attempt); }
/// ```
pub async fn run_upload_bounded_awaiting(git: &PushGit, plan: &UploadPlan) -> Result<Option<PushAttempt>, SingleFlightError> {
    // The lookup of `git-lfs` on the caller's `PATH` asks the filesystem, so it is a job of the blocking pool and never the
    //  task that other work of the round shares (`run_blocking_job_within_the_budget`).
    let looked_up: PushGit = git.clone();
    let planned: UploadPlan = plan.clone();
    let Some(commands) = run_blocking_job_within_the_budget(move || { return upload_commands(&looked_up, &planned); })
        .await
        .map_err(refused_job)?
    else {
        return Ok(Some(missing_program_attempt()));
    };
    for (remote, command) in commands {
        let attempt: PushAttempt =
            match crate::bounded_push::run_bounded_push_awaiting(command, push_patience(git.environment.as_slice()), STOP_GRACE, git.environment.as_slice()).await {
                Ok(finished) => finished,
                Err(SingleFlightError::Start(_)) => return Ok(Some(missing_program_attempt())),
                Err(other) => return Err(other),
            };
        if let Some(failed) = failed_upload(&attempt, remote.as_os_str(), prompting_disabled(git.environment.as_slice())) {
            return Ok(Some(failed));
        }
    }
    return Ok(None);
}

/// What:
///  The commands of one upload:
/// one `git-lfs push` per remote and per run of it,
/// or nothing when no `git-lfs` is on the caller's `PATH`.
/// Why:
///   Both faces run the same programs with the same arguments,
/// so a value either face reads of the plan reaches the same child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const program = findProgram(env.PATH); if (!program) return undefined; return plan.remotes.flatMap((remote) => plan.runs(remote).map((args) => [remote, command(program, args)]));
/// ```
fn upload_commands(git: &PushGit, plan: &UploadPlan) -> Option<Vec<(OsString, Command)>> {
    let program = find_program(environment_value(git.environment.as_slice(), "PATH").unwrap_or_default().as_os_str())?;
    let mut commands: Vec<(OsString, Command)> = Vec::new();
    for remote in &plan.remotes {
        for arguments in plan.runs(remote.as_os_str()) {
            let mut command: Command = Command::new(&program);
            command
                .current_dir(&git.cwd)
                .env_clear();
            for (name, value) in &git.environment {
                command.env(name, value);
            }
            command
                .env("PATH", path_with_real_git_first(git.environment.as_slice(), git.real_git.as_path()))
                .args(arguments.as_slice());
            // The destination travels beside its command, so a failure can name the server that asked without searching the
            //  argument list again.
            commands.push((remote.clone(), command));
        }
    }
    return Some(commands);
}

/// What:
///  The failure of an upload whose job of the blocking pool the system refused.
/// Why:
///   The awaited face of the upload starts its program lookup as a job,
/// and a job the system refused a thread for never ran,
/// so the upload could not be
///  planned and the push must not be started in its place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SingleFlightError.Start(new Error(refusal.message));
/// ```
pub(crate) fn refused_job(refusal: crate::blocking_job::BlockingJobError) -> SingleFlightError {
    return SingleFlightError::Start(std::io::Error::other(refusal.to_string()));
}

/// The runs are controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_tests.rs"]
mod tests;
