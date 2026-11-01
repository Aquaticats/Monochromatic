//! What:
//!  The parent's run of the Git LFS upload before a push: `git-lfs push <remote> <refs>` as one child it owns.
//! Why:
//!  Without an upload step a push publishes pointer files whose objects the LFS server never received.
//!  The wrapper runs `git-lfs push` itself, so a failure is explained by the process that saw it and the push, which
//!  then carries no hook, is never started. Nothing here starts the wrapper, writes shell text or injects Git
//!  configuration (`doc/planning/cli-git-no-self-invocation.md`). What to upload is decided in `lfs_upload_plan`.
//!  The child sees what the caller gave Git: the global `-c` options travel as `GIT_CONFIG_COUNT` pairs, `--git-dir`,
//!  `--work-tree` and `--namespace` as their variables, `-C` as the working directory, and the real Git's directory
//!  leads its `PATH` so a `git` it starts is never the wrapper while `git-lfs` itself is the one the caller's `PATH`
//!  finds first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const code = await spawn(findProgram(env.PATH), plan.arguments(), { env: withPrefix(env, globalPrefix), stdio: 'inherit' });
//! ```

/// The attempt record, the coordinator's Git and its error type.
use super::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError};
/// The deadline, the grace and the push runner.
use super::bounded_push::{STOPPED_EXIT_CODE, STOP_GRACE, budget_spent_message, push_deadline, run_bounded_push};
/// The invocation's environment values.
use super::child_environment::environment_value;
/// How a child ended, as a status.
use super::forwarding::{ChildOutcome, exit_code, outcome_of};
/// The texts, the program lookup and the plan.
use super::lfs_pre_push::{failed_message, find_program, missing_program_message};
use super::lfs_upload_plan::UploadPlan;
/// Relays signals to the child.
use super::signal_relay::SignalGuard;
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// Owned OS text for arguments.
use std::ffi::OsString;
/// The real Git whose directory leads `PATH`.
use std::path::Path;
/// The child.
use std::process::Command;

/// `PATH` with the directory of the real Git first, so a `git` that `git-lfs` starts is the real Git and never the
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
///  (a `-c` without `=` is the boolean true), `--git-dir`, `--work-tree` and `--namespace` become `GIT_DIR`,
///  `GIT_WORK_TREE` and `GIT_NAMESPACE`; `-C` is the working directory and is applied by the caller.
///  No shell is involved, and an `-c http.extraHeader=...` or `-c lfs.url=...` reaches `git-lfs` as it reached the
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
    let existing: usize = environment_value(environment, "GIT_CONFIG_COUNT")
        .and_then(|value| {
            return value.to_str().and_then(|count| {
                return count.parse::<usize>().ok();
            });
        })
        .unwrap_or(0);
    if !pairs.is_empty() {
        variables.push((OsString::from("GIT_CONFIG_COUNT"), OsString::from((existing + pairs.len()).to_string())));
    }
    for (offset, (name, value)) in pairs.into_iter().enumerate() {
        variables.push((OsString::from(format!("GIT_CONFIG_KEY_{}", existing + offset)), name));
        variables.push((OsString::from(format!("GIT_CONFIG_VALUE_{}", existing + offset)), value));
    }
    return variables;
}

/// What:
///  Run the upload as a child with the caller's streams and return the status to stop with, or nothing when it succeeded.
/// Why:
///   The parent saw the child end, so it prints the explanation after the program's own output; the push is not started.
///  A signal sent to the wrapper is relayed to the child, so stopping the wrapper stops the upload. A push that also
///  pushes submodules gets a notice, because their objects are not part of this upload.
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
    for remote in &plan.remotes {
        let mut command: Command = Command::new(&program);
        command
            .current_dir(directory)
            .args(plan.arguments(remote.as_os_str()))
            .env("PATH", path_with_real_git_first(environment, real_git));
        for (name, value) in prefix_variables(global_prefix, environment).into_iter().chain(overlay.iter().cloned()) {
            command.env(name, value);
        }
        let outcome: std::io::Result<ChildOutcome> = match SignalGuard::install() {
            Ok(guard) => guard.run_child(&mut command),
            Err(_) => command.status_retrying().map(outcome_of),
        };
        let Ok(ended) = outcome else {
            // Terminal output error: see above.
            let _written = diagnostics.write_all(missing_program_message().as_bytes());
            return Some(1);
        };
        let code: i32 = exit_code(ended);
        if code == 0 {
            continue;
        }
        let description: String = match ended {
            ChildOutcome::Exited(status) => format!("exited with status {status}"),
            ChildOutcome::Signaled(signal) => format!("was ended by signal {signal}"),
        };
        // Terminal output error: see above.
        let _written = diagnostics.write_all(failed_message(description.as_str()).as_bytes());
        return Some(code);
    }
    return None;
}

/// What:
///  The notice for a push that also pushes submodules.
/// Why:
///   Git pushes a submodule's commits itself; their LFS objects are not uploaded by the wrapper, and the pointer files
///  would be published without them. Every way forward is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// 'cli-git: this push also pushes submodules, ...'
/// ```
pub fn submodule_notice() -> String {
    return String::from(
        "cli-git: this push also pushes submodules (`--recurse-submodules`), and the wrapper uploads Git LFS objects only \\
         for this repository's own refs. If a submodule uses Git LFS, upload its objects first by running \\
         `git lfs push --all <remote>` inside it, or push the submodule on its own; otherwise its pointer files are \\
         published without their objects.\\n",
    );
}

/// What:
///  Run the upload as a bounded child whose output is captured, and return the failed attempt to stop with, or nothing
///  when it succeeded.
/// Why:
///   The auto-push records and shows a failed attempt's output; the explanation is part of it. A stop by deadline says
///  so, and a step the holder's time budget left no time for is not started and says that instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = await runBoundedPush({ command: gitLfs(plan), deadlineMs }); return attempt.exitCode ? withExplanation(attempt) : undefined;
/// ```
pub fn run_upload_bounded(git: &PushGit, plan: &UploadPlan) -> Result<Option<PushAttempt>, SingleFlightError> {
    if super::holder_budget::exhausted() {
        return Ok(Some(PushAttempt { exit_code: STOPPED_EXIT_CODE, output: budget_spent_message() }));
    }
    let Some(program) = find_program(environment_value(git.environment.as_slice(), "PATH").unwrap_or_default().as_os_str())
    else {
        return Ok(Some(PushAttempt { exit_code: 1, output: missing_program_message().trim_end().to_owned() }));
    };
    for remote in &plan.remotes {
        let mut command: Command = Command::new(&program);
        command
            .current_dir(&git.cwd)
            .env_clear()
            .envs(git.environment.iter().map(|(name, value)| {
                return (name, value);
            }))
            .env("PATH", path_with_real_git_first(git.environment.as_slice(), git.real_git.as_path()))
            .args(plan.arguments(remote.as_os_str()));
        let attempt: PushAttempt =
            match run_bounded_push(command, push_deadline(git.environment.as_slice()), STOP_GRACE, git.environment.as_slice()) {
                Ok(finished) => finished,
                Err(SingleFlightError::Start(_)) => {
                    return Ok(Some(PushAttempt { exit_code: 1, output: missing_program_message().trim_end().to_owned() }));
                }
                Err(other) => return Err(other),
            };
        if attempt.exit_code != 0 {
            return Ok(Some(explained(attempt)));
        }
    }
    return Ok(None);
}

/// The failed attempt with the explanation after the program's own output; a stop by deadline says so.
fn explained(attempt: PushAttempt) -> PushAttempt {
    let description: String = if attempt.exit_code == STOPPED_EXIT_CODE {
        String::from("did not finish and was stopped")
    } else {
        format!("exited with status {}", attempt.exit_code)
    };
    let explanation: String = failed_message(description.as_str());
    let shown: &str = explanation.trim_end();
    let output: String = if attempt.output.is_empty() { shown.to_owned() } else { format!("{}\n{shown}", attempt.output) };
    return PushAttempt { exit_code: attempt.exit_code, output };
}

/// The runs are controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_tests.rs"]
mod tests;
