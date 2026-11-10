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
//! the global `-c` options travel as `GIT_CONFIG_COUNT` pairs,
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
    STOPPED_EXIT_CODE, STOP_GRACE, StopCause, command_label, push_patience, wait_until_done_noting, run_bounded_push,
};
/// The invocation's environment values.
use crate::child_environment::environment_value;
/// How a child ended,
/// as a status.
use crate::forwarding::{ChildOutcome, exit_code, outcome_of};
/// The texts,
/// the program lookup,
/// and the reading that says a failure was a login.
use crate::lfs_pre_push::{
    environment_config_count, failed_message, find_program, login_needed_message, missing_program_message, reads_as_a_login_failure,
};
/// Hides the credentials in a URL of relayed text,
/// and reads the host of a destination to name it.
use crate::lfs_output_mask::{host_of, mask_userinfo, mask_userinfo_text};
use crate::lfs_upload_plan::UploadPlan;
/// The anonymous file the interactive upload's standard error goes to.
use crate::scratch_location::scratch_file;
/// Relays signals to the child,
/// which starts it and hands it over still published.
use crate::signal_relay::{RelayedChild, SignalGuard};
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

/// What:
///  The variables that carry the global options of a push into a child that reads Git's configuration.
/// Why:
///   `-c name=value` becomes `GIT_CONFIG_KEY_n`/`GIT_CONFIG_VALUE_n` after any pairs the environment already holds
///  (a `-c` without `=` is the boolean true),
/// `--git-dir`,
/// `--work-tree` and `--namespace` become `GIT_DIR`,
///  `GIT_WORK_TREE` and `GIT_NAMESPACE`;
/// `-C` is the working directory and is applied by the caller.
///  No shell is involved,
/// and an `-c http.extraHeader=...` or `-c lfs.url=...` reaches `git-lfs` as it reached the
///  Git that used to run the hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prefixVariables(globalPrefix: string[], environment): [string, string][];
/// ```
pub fn prefix_variables(global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    let mut pairs: Vec<(OsString, OsString)> = Vec::new();
    let mut variables: Vec<(OsString, OsString)> = Vec::new();
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let text: String = token.to_string_lossy().into_owned();
        if text == "-c" {
            if let Some(setting) = tokens.next() {
                let setting_text: String = setting.to_string_lossy().into_owned();
                let (name, value) = setting_text.split_once('=').unwrap_or((setting_text.as_str(), "true"));
                pairs.push((OsString::from(name), OsString::from(value)));
            }
            continue;
        }
        for (option, variable) in [("--git-dir", "GIT_DIR"), ("--work-tree", "GIT_WORK_TREE"), ("--namespace", "GIT_NAMESPACE")] {
            if text == option {
                if let Some(value) = tokens.next() {
                    variables.push((OsString::from(variable), value.clone()));
                }
            } else if let Some(value) = text.strip_prefix(format!("{option}=").as_str()) {
                variables.push((OsString::from(variable), OsString::from(value)));
            }
        }
    }
    let existing: usize = environment_config_count(environment);
    if !pairs.is_empty() {
        variables.push((OsString::from("GIT_CONFIG_COUNT"), OsString::from((existing + pairs.len()).to_string())));
    }
    for (offset, (name, value)) in pairs.into_iter().enumerate() {
        variables.push((OsString::from(format!("GIT_CONFIG_KEY_{}", existing + offset)), name));
        variables.push((OsString::from(format!("GIT_CONFIG_VALUE_{}", existing + offset)), value));
    }
    return variables;
}

/// Why a run with captured standard error did not produce an outcome.
enum CaptureError {
    /// The file for the standard error could not be made;
    /// the text says why.
    Scratch(String),
    /// The program could not be started.
    Start,
    /// The wait for the program could not be made at all:
    /// no signal handler could be installed, or the process's runtime refused.
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

/// The most standard error text one run keeps and relays;
/// the rest is dropped,
/// so a flood cannot fill memory.
const MAX_RELAYED_BYTES: u64 = 4 * 1024 * 1024;

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
    return Ok((ended, mask_userinfo(bytes.as_slice())));
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
    let mut relayed: RelayedChild = guard.start_relayed(command).await.map_err(|_error| RunError::Start)?;
    let started: std::process::Child = relayed.take_child().expect("the child was handed over once");
    let mut child: OwnedChild = adopt(started);
    let label: String = command_label(command);
    let mut waits: Vec<crate::push_progress::WaitReport> = Vec::new();
    // The upload leaves no reference lock, so the ledger the stop sweeps is empty.
    let locks: LockLedger = LockLedger::none();
    let patience = push_patience(environment);
    let ended: Result<ExitStatus, StopCause> =
        wait_until_done_noting(&mut child, &[captured], patience, label.as_str(), &mut waits, None).await.map_err(|wait| RunError::Wait(wait.to_string()))?;
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
///  The words a run the wrapper stopped is explained with.
/// Why:
///  The program chose no status of its own,
/// so the explanation says which limit came first and what the person can do about a
///  slow line, as the auto-push's upload does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cause.kind === 'stalled' ? `made no progress for ${limit} seconds and was stopped` : `...`
/// ```
fn stopped_upload_words(cause: &StopCause) -> String {
    return match cause {
        StopCause::Deadline => String::from("did not finish in time and was stopped"),
        StopCause::Stalled(report) => format!(
            "made no progress for {} seconds and was stopped (bytes_moved={}); {}",
            report.limit.as_secs(),
            report.moved,
            crate::push_deadline_advice::extend_clause()
        ),
        StopCause::Flood => format!("wrote more than {MAX_RELAYED_BYTES} bytes of output and was stopped"),
        // A typed upload carries no arm for the service's stop, so this ending cannot be reached here.
        StopCause::ServiceStopped => String::from("was ended because the service was stopped"),
    };
}

/// What:
///  How a refusal names the destination whose upload needed a login:
///  the server the destination URL names,
/// or, for a remote name or a local path,
/// the destination as typed with any userinfo hidden.
/// Why:
///   A refusal a person acts on says which server asked,
/// and never repeats a token that a repository's `lfs.url` carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hostOf(text) ? `the server \`${hostOf(text)}\`` : `the destination \`${maskUserinfoText(text)}\``
/// ```
fn login_subject(destination: &std::ffi::OsStr) -> String {
    // `to_string_lossy` reads a destination that is not UTF-8 without failing; a remote name or URL is text in practice.
    let text: String = destination.to_string_lossy().into_owned();
    return match host_of(text.as_str()) {
        Some(host) => format!("the server `{host}`"),
        None => format!("the destination `{}`", mask_userinfo_text(text.as_str())),
    };
}

/// What:
///  The explanation printed after a failed upload:
/// the general one,
/// and,
/// when the program's own words say it needed a login,
/// the one that names the server and every way to give it one.
/// Why:
///   The wrapper is what refused to ask a person,
/// so that failure is the one whose way forward only the wrapper can explain
///  (`doc/troubleshooting/lfs-credential-in-url.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// failedMessage(status) + (readsAsALoginFailure(output) ? loginNeededMessage(loginSubject(destination)) : '')
/// ```
fn explanation_after_failure(status: &str, destination: &std::ffi::OsStr, output: &[u8]) -> String {
    let mut text: String = failed_message(status);
    // `from_utf8_lossy` reads the program's words whatever their encoding; only the search matters here.
    let shown: String = String::from_utf8_lossy(output).into_owned();
    if reads_as_a_login_failure(shown.as_str()) {
        text.push_str(login_needed_message(login_subject(destination).as_str()).as_str());
    }
    return text;
}

/// The refusal printed when the standard error of the upload cannot be captured.
fn capture_failed_message(why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: this push was stopped before it sent anything. The wrapper reads the output of `git-lfs push` through \
         a private temporary file, to hide the credentials in the URLs it names, and that file could not be made ({why}). Ways forward: fix the cause named in the parentheses (the \
         wrapper needs a writable `XDG_STATE_HOME`, or a `HOME`, for its private scratch directory) and push again; or \
         skip the upload for this push with `GIT_LFS_SKIP_PUSH=1` or `git push --no-verify`, which publishes pointer \
         files without their objects.\n"
    );
}

/// What:
///  The refusal printed when no signal handler could be installed for the upload's wait.
/// Why:
///  Without a guard the wait would have no bound and no relay,
/// so the push is stopped before it sends anything rather than left to wait
///  where nothing bounds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: this push was stopped before it sent anything. ... (${why}) ...`
/// ```
fn guard_failed_message(why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: this push was stopped before it sent anything. The wrapper bounds the run of `git-lfs push` \
         and relays a signal sent to it to that program, and it could not begin that wait ({why}). Ways forward: push again, because \
         the cause may be transient; or upload by hand with `git lfs push --all <remote>` and push with `git push --no-verify`.\n"
    );
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
            for (name, value) in prefix_variables(global_prefix, environment).into_iter().chain(overlay.iter().cloned()) {
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
            let (code, ended): (i32, ChildOutcome) = match ended {
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
            let description: String = match ended {
                ChildOutcome::Exited(status) => format!("exited with status {status}"),
                ChildOutcome::Signaled(signal) => format!("was ended by signal {signal}"),
            };
            // Terminal output error: see above.
            let _written = diagnostics.write_all(explanation_after_failure(description.as_str(), remote.as_os_str(), relayed.as_slice()).as_bytes());
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
        if let Some(failed) = failed_upload(&attempt, remote.as_os_str()) {
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
        if let Some(failed) = failed_upload(&attempt, remote.as_os_str()) {
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

/// What:
///  What a person is told when no `git-lfs` is on the caller's `PATH`.
/// Why:
///   Both faces refuse the upload with the same words,
/// and the push that would have followed it is never started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ exitCode: 1, output: missingProgramMessage().trimEnd() })
/// ```
fn missing_program_attempt() -> PushAttempt {
    return PushAttempt { exit_code: 1, output: missing_program_message().trim_end().to_owned() };
}

/// What:
///  The failed attempt of one finished upload,
/// with the credentials in its output hidden,
/// or nothing when it succeeded.
/// Why:
///  A server error is not run again here,
/// for the reason given in `run_upload_interactive`:
/// the service's push retry plan
///  tries the push again by itself.
/// Both faces read one finished attempt here,
/// so the two cannot report a failure differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// attempt.exitCode === 0 ? undefined : explained({ exitCode, output: maskUserinfoText(attempt.output) });
/// ```
fn failed_upload(attempt: &PushAttempt, destination: &std::ffi::OsStr) -> Option<PushAttempt> {
    if attempt.exit_code == 0 {
        return None;
    }
    let shown: String = mask_userinfo_text(attempt.output.as_str());
    return Some(explained(PushAttempt { exit_code: attempt.exit_code, output: shown }, destination));
}

/// The failed attempt with the explanation after the program's own output;
/// a stop by deadline says so,
/// and a login the program needed says how to give it one.
fn explained(attempt: PushAttempt, destination: &std::ffi::OsStr) -> PushAttempt {
    let description: String = if attempt.exit_code == STOPPED_EXIT_CODE {
        String::from("did not finish and was stopped")
    } else {
        format!("exited with status {}", attempt.exit_code)
    };
    let explanation: String = explanation_after_failure(description.as_str(), destination, attempt.output.as_bytes());
    let shown: &str = explanation.trim_end();
    let output: String = if attempt.output.is_empty() { shown.to_owned() } else { format!("{}\n{shown}", attempt.output) };
    return PushAttempt { exit_code: attempt.exit_code, output };
}

/// The runs are controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_tests.rs"]
mod tests;
