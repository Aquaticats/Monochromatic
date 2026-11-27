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
//! One run of the child,
//! the standard error captured from it and the wait that ends it
//! live in `lfs_upload_child_run`.
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
use crate::auto_push_single_flight::{
    PushAttempt,
    PushGit,
    SingleFlightError,
};
/// One job of the process's bound on blocking work,
/// for the filesystem lookups a task may not make.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// What ended a stopped child,
/// which the controls of the stop's words still ask for.
#[cfg(test)]
use crate::bounded_push::StopCause;
/// The grace a stopped child gets,
/// the push runner,
/// and how long a push's patience lets a wait pass.
use crate::bounded_push::{
    STOP_GRACE,
    STOPPED_EXIT_CODE,
    push_patience,
    run_bounded_push_naming,
};
/// The invocation's environment values,
/// merged with the overlay its children run under.
use crate::child_environment::{
    environment_value,
    environment_with_overlay,
};
/// How a child ended,
/// as a status.
use crate::forwarding::{
    ChildOutcome,
    exit_code,
};
/// The caller's own command-line configuration,
/// as the variables the child reads.
pub use crate::lfs_command_line_configuration::child_configuration_variables;
/// The hosts a text names.
use crate::lfs_output_mask::hosts_in;
/// The program lookup.
use crate::lfs_pre_push::find_program;
/// The folders searched and the refusal that names them.
use crate::lfs_program_search::{
    ProgramSearch,
    missing_program_message,
};
/// The command of one upload run,
/// its captured standard error,
/// and the wait that ends it.
use crate::lfs_upload_child_run::{
    CaptureError,
    UploadEnd,
    path_with_real_git_first,
    run_with_captured_stderr,
    upload_commands,
};
/// The masked head of a run's captured standard error,
/// which the controls of that read still ask for.
#[cfg(test)]
use crate::lfs_upload_child_run::{
    relayed_head,
    shortened_relay_notice,
};
/// The cap on the bytes of a standard error that are relayed,
/// which the controls of the cut still ask for.
#[cfg(test)]
use crate::lfs_upload_explanations::MAX_RELAYED_BYTES;
/// Every word about a failed,
/// stopped or refused upload.
use crate::lfs_upload_explanations::{
    capture_failed_message,
    explanation_after_failure,
    failed_upload,
    guard_failed_message,
    missing_program_attempt,
    start_failed_attempt,
    stopped_upload_tail,
    stopped_upload_words,
};
/// What to upload.
use crate::lfs_upload_plan::UploadPlan;
/// Owned OS text for arguments.
use std::ffi::OsString;
/// The real Git whose directory leads `PATH`.
use std::path::Path;
/// The child of one run.
use std::process::Command;
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
    let Some(program) = find_program(
        environment_value(environment, "PATH")
            .unwrap_or_default()
            .as_os_str(),
        directory,
    ) else {
        // Terminal output error: the exit status already carries the refusal.
        let _written = diagnostics
            .write_all(missing_program_message(&ProgramSearch::of(environment)).as_bytes());
        return Some(1);
    };
    if plan.submodules {
        // Terminal output error: the notice is advisory.
        let _written = diagnostics.write_all(submodule_notice().as_bytes());
    }
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
            let (ended, relayed): (UploadEnd, Vec<u8>) =
                match run_with_captured_stderr(&mut command, environment) {
                    Ok(finished) => finished,
                    Err(CaptureError::Scratch(why)) => {
                        // Terminal output error: the exit status already carries the refusal.
                        let _written =
                            diagnostics.write_all(capture_failed_message(why.as_str()).as_bytes());
                        return Some(1);
                    }
                    Err(CaptureError::Start(why)) => {
                        // Terminal output error: see above.
                        let _written = diagnostics.write_all(why.as_bytes());
                        return Some(1);
                    }
                    Err(CaptureError::Guard(why)) => {
                        // Terminal output error: see above.
                        let _written =
                            diagnostics.write_all(guard_failed_message(why.as_str()).as_bytes());
                        return Some(1);
                    }
                };
            // Terminal output error: the program's own words are advisory; the exit status carries the result.
            let _relayed = diagnostics.write_all(relayed.as_slice());
            // The explanation is a paragraph of its own: a program that ended without a last line break must not have it
            //  glued to its last line.
            if !relayed.is_empty() && !relayed.ends_with(b"\n") {
                let _written = diagnostics.write_all(b"\n");
            }
            let (code, outcome): (i32, ChildOutcome) = match ended {
                UploadEnd::Finished(outcome) => (exit_code(outcome), outcome),
                UploadEnd::Stopped(cause) => {
                    // Terminal output error: the explanation carries the result.
                    //  The stop's own sentence already ends with a full stop, so the tail starts after one full stop and not
                    //  after a second one.
                    let server: Option<String> = upload_server_of(plan, relayed.as_slice());
                    let _written = diagnostics.write_all(
                        format!(
                            "{} {}\n",
                            stopped_upload_words(&cause, server.as_deref()),
                            stopped_upload_tail()
                        )
                        .as_bytes(),
                    );
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
                explanation_after_failure(
                    description.as_str(),
                    remote.as_os_str(),
                    relayed.as_slice(),
                )
                .as_bytes(),
            );
            return Some(code);
        }
    }
    return None;
}

/// What:
///  The LFS server a stopped upload was connected to:
///  the one the plan read from the configuration,
///  confirmed by the program's own words when they name that same server,
///  and the program's own words alone when the plan read none.
/// Why:
///   A stop that held a connection must say which server it waited for
///  (`package/git-policy/cli/SPEC.md`,
///  "A connected stop names the server it waited for");
///  the plan answers with the endpoint git-lfs itself picks,
///  and a program that printed nothing confirms nothing.
///  The program's words are not trusted on their own when the plan named a server:
///  a documentation link printed before the error named a host the upload never
///  contacted (the short review's reproduction),
///  which is the one thing this naming exists to prevent.
///  Neither form repeats a credential:
///  the host comes from the same authority rule the mask uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = hostsIn(text)[0]; named === plan.lfsServer || plan.lfsServer === undefined ? named ?? plan.lfsServer : plan.lfsServer
/// ```
fn upload_server_of(
    plan: &UploadPlan,
    relayed: &[u8],
) -> Option<String> {
    // `from_utf8_lossy` reads the program's words whatever their encoding; only the host search matters here.
    let shown: String = String::from_utf8_lossy(relayed).into_owned();
    let from_program: Option<String> = hosts_in(shown.as_str()).into_iter().next();
    return match (plan.lfs_server.as_deref(), from_program) {
        (Some(planned), Some(named)) if planned == named => Some(named),
        (Some(planned), _named) => Some(planned.to_owned()),
        (None, named) => named,
    };
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
pub fn run_upload_bounded(
    git: &PushGit,
    plan: &UploadPlan,
) -> Result<Option<PushAttempt>, SingleFlightError> {
    let Some(commands) = upload_commands(git, plan) else {
        return Ok(Some(missing_program_attempt(&ProgramSearch::of(
            git.environment.as_slice(),
        ))));
    };
    for (remote, command) in commands {
        // The lookup already found the program, so a start the system refused is not a missing program: its own error says why.
        let program: String = command.get_program().to_string_lossy().into_owned();
        let attempt: PushAttempt = match run_bounded_push_naming(
            command,
            push_patience(git.environment.as_slice()),
            STOP_GRACE,
            git.environment.as_slice(),
            plan.lfs_server.as_deref(),
        ) {
            Ok(finished) => finished,
            Err(SingleFlightError::Start(error)) => {
                return Ok(Some(start_failed_attempt(
                    program.as_str(),
                    error.to_string().as_str(),
                )));
            }
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
pub async fn run_upload_bounded_awaiting(
    git: &PushGit,
    plan: &UploadPlan,
) -> Result<Option<PushAttempt>, SingleFlightError> {
    // The lookup of `git-lfs` on the caller's `PATH` asks the filesystem, so it is a job of the blocking pool and never the
    //  task that other work of the round shares (`run_blocking_job_within_the_budget`).
    let looked_up: PushGit = git.clone();
    let planned: UploadPlan = plan.clone();
    let Some(commands) = run_blocking_job_within_the_budget(move || {
        return upload_commands(&looked_up, &planned);
    })
    .await
    .map_err(refused_job)?
    else {
        return Ok(Some(missing_program_attempt(&ProgramSearch::of(
            git.environment.as_slice(),
        ))));
    };
    for (remote, command) in commands {
        let program: String = command.get_program().to_string_lossy().into_owned();
        let attempt: PushAttempt = match crate::bounded_push::run_bounded_push_awaiting_naming(
            command,
            push_patience(git.environment.as_slice()),
            STOP_GRACE,
            git.environment.as_slice(),
            plan.lfs_server.as_deref(),
        )
        .await
        {
            Ok(finished) => finished,
            Err(SingleFlightError::Start(error)) => {
                return Ok(Some(start_failed_attempt(
                    program.as_str(),
                    error.to_string().as_str(),
                )));
            }
            Err(other) => return Err(other),
        };
        if let Some(failed) = failed_upload(&attempt, remote.as_os_str()) {
            return Ok(Some(failed));
        }
    }
    return Ok(None);
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
