//! What:
//!  The Git LFS upload that runs before a push: what to upload (`git lfs push <remote> <refs>`) and how the parent runs
//!  it as one child of its own, before it starts the push.
//! Why:
//!  Without an upload step a push publishes pointer files whose objects the LFS server never received.
//!  The wrapper runs `git-lfs push` itself, as a child it owns, so a failure is explained by the process that saw it
//!  and the push, which then carries no hook at all, is never started. Nothing here starts the wrapper, writes shell
//!  text or injects Git configuration (`doc/planning/cli-git-no-self-invocation.md`).
//!  `git lfs push` takes refs and no ref-update lines, needs no value advertised by the remote, and sends objects the
//!  local clone does not already know the remote to have; objects are content addressed, so a superset is harmless.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planUpload(region); if (plan) { const code = await run('git-lfs', ['push', plan.remote, ...plan.refs]); if (code) return code; }
//! ```

/// The attempt record, the coordinator's Git and its error type.
use super::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError};
/// The deadline, the grace and the push runner.
use super::bounded_push::{STOPPED_EXIT_CODE, STOP_GRACE, push_deadline, run_bounded_push};
/// The destination and the named refs of a push.
use super::command_push::push_shape;
/// The invocation's environment values.
use super::child_environment::environment_value;
/// How a child ended, as a status.
use super::forwarding::{ChildOutcome, exit_code, outcome_of};
/// The texts, the switch and the program name.
use super::lfs_pre_push::{LFS_PROGRAM, failed_message, hook_switched_off, missing_program_message, skip_requested};
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// Owned OS text for arguments.
use std::ffi::{OsStr, OsString};
/// The real Git whose directory leads `PATH`.
use std::path::Path;
/// The child.
use std::process::Command;

/// What an upload sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UploadPlan = { remote: string; refs: string[]; everything: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadPlan {
    /// The remote name or URL `git lfs push` takes first.
    pub remote: OsString,
    /// The refs whose objects are uploaded; empty with `everything`.
    pub refs: Vec<OsString>,
    /// Upload everything reachable from every local ref (`--all`).
    pub everything: bool,
}

impl UploadPlan {
    /// The arguments of `git-lfs`: `push`, `--all` when everything is sent, the remote, then the refs.
    pub fn arguments(&self) -> Vec<OsString> {
        let mut arguments: Vec<OsString> = vec![OsString::from("push")];
        if self.everything {
            arguments.push(OsString::from("--all"));
        }
        arguments.push(self.remote.clone());
        if !self.everything {
            arguments.extend(self.refs.iter().cloned());
        }
        return arguments;
    }
}

/// The source side of a refspec, or nothing for a deletion (`:dst`); a leading `+` is dropped.
fn refspec_source(spec: &OsStr) -> Option<OsString> {
    let bytes: &[u8] = spec.as_encoded_bytes();
    let unforced: &[u8] = bytes.strip_prefix(b"+").unwrap_or(bytes);
    let source: &[u8] = unforced
        .split(|byte| {
            return *byte == b':';
        })
        .next()
        .unwrap_or_default();
    if source.is_empty() {
        return None;
    }
    return Some(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(source.to_vec()));
}

/// Settings Git reads to pick a push remote and a bare push's refs: `(key, value)` pairs from one configuration read.
fn settings(query: &mut dyn FnMut(&[&str]) -> Option<String>) -> Vec<(String, String)> {
    let listing: String = query(&[
        "config",
        "--get-regexp",
        "^(branch\\..*\\.(pushremote|remote)|remote\\.pushdefault|push\\.default)$",
    ])
    .unwrap_or_default();
    return listing
        .lines()
        .filter_map(|line| {
            return line.split_once(' ').map(|(key, value)| {
                return (key.to_owned(), value.trim().to_owned());
            });
        })
        .collect();
}

/// The value of a setting, the last one written winning as in Git.
fn setting<'a>(all: &'a [(String, String)], key: &str) -> Option<&'a str> {
    return all
        .iter()
        .rev()
        .find(|(name, _)| {
            return name == key;
        })
        .map(|(_, value)| {
            return value.as_str();
        });
}

/// The remote Git pushes to when the caller names none: the branch's push remote, the default push remote, the
///  branch's remote, then `origin`.
fn default_remote(branch: Option<&str>, all: &[(String, String)]) -> String {
    let named = |variable: &str| -> Option<String> {
        return branch.and_then(|name| {
            return setting(all, &format!("branch.{name}.{variable}")).map(str::to_owned);
        });
    };
    return named("pushremote")
        .or_else(|| {
            return setting(all, "remote.pushdefault").map(str::to_owned);
        })
        .or_else(|| {
            return named("remote");
        })
        .unwrap_or_else(|| {
            return String::from("origin");
        });
}

/// What:
///  The remote a push goes to: the one the caller named, or the one Git picks for a bare push.
/// Why:
///   The manual-push probe and the upload plan must agree on the destination, so both read it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// named ?? defaultRemote(branch, settings)
/// ```
pub fn resolve_push_remote(named: Option<OsString>, query: &mut dyn FnMut(&[&str]) -> Option<String>) -> OsString {
    if let Some(found) = named {
        return found;
    }
    let branch: Option<String> = query(&["symbolic-ref", "--quiet", "--short", "HEAD"]).filter(|name| {
        return !name.is_empty();
    });
    let all: Vec<(String, String)> = settings(query);
    return OsString::from(default_remote(branch.as_deref(), all.as_slice()));
}

/// What:
///  What to upload for a push, or nothing when the push uploads nothing.
///  `query` runs a read-only Git query and returns its trimmed standard output when it exits with status zero.
/// Why:
///   A push that bypasses the upload (`--no-verify`, `GIT_LFS_SKIP_PUSH`, the caller's switch), sends nothing (a dry
///  run, only deletions) or names no remote and branch Git could resolve uploads nothing; a push that names refs uploads
///  those refs; a bare push uploads the branch Git would send (`HEAD` when the branch has no name), and a push that
///  may send any ref uploads everything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planUpload(globalPrefix, region, environment, query): UploadPlan | undefined;
/// ```
pub fn plan_upload(
    global_prefix: &[OsString],
    region: &[OsString],
    environment: &[(OsString, OsString)],
    query: &mut dyn FnMut(&[&str]) -> Option<String>,
) -> Option<UploadPlan> {
    if hook_switched_off(global_prefix) || skip_requested(environment) {
        return None;
    }
    let region_facts = super::command_push::parse_push_region(region, &[]).ok()?;
    if region_facts.no_verify || region_facts.dry_run {
        return None;
    }
    let shape = push_shape(region)?;
    if shape.deletes {
        return None;
    }
    let mut everything: bool = shape.everything;
    let mut refs: Vec<OsString> = shape
        .refspecs
        .iter()
        .filter_map(|spec| {
            return refspec_source(spec.as_os_str());
        })
        .collect();
    let mut remote: Option<OsString> = shape.repository.clone();
    let bare: bool = shape.refspecs.is_empty() && !everything;
    if remote.is_none() || bare {
        let branch: Option<String> = query(&["symbolic-ref", "--quiet", "--short", "HEAD"]).filter(|name| {
            return !name.is_empty();
        });
        let all: Vec<(String, String)> = settings(query);
        if remote.is_none() {
            remote = Some(OsString::from(default_remote(branch.as_deref(), all.as_slice())));
        }
        if bare {
            if setting(all.as_slice(), "push.default") == Some("matching") {
                everything = true;
            } else {
                refs.push(OsString::from(branch.as_deref().unwrap_or("HEAD")));
            }
        }
    }
    if refs.is_empty() && !everything {
        return None;
    }
    return Some(UploadPlan { remote: remote?, refs, everything });
}

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
///  Run the upload as a child with the caller's streams and return the status to stop with, or nothing when it succeeded.
/// Why:
///   The parent saw the child end, so it prints the explanation after the program's own output; the push is not started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const code = await spawn('git-lfs', plan.arguments(), { stdio: 'inherit' }); if (code) { print(failedMessage(...)); return code; }
/// ```
pub fn run_upload_interactive(
    plan: &UploadPlan,
    directory: &Path,
    real_git: &Path,
    environment: &[(OsString, OsString)],
    overlay: &[(OsString, OsString)],
    diagnostics: &mut dyn std::io::Write,
) -> Option<i32> {
    let mut command: Command = Command::new(LFS_PROGRAM);
    command
        .current_dir(directory)
        .args(plan.arguments())
        .env("PATH", path_with_real_git_first(environment, real_git));
    for (name, value) in overlay {
        command.env(name, value);
    }
    let status = match command.status_retrying() {
        Ok(finished) => finished,
        Err(_) => {
            // Terminal output error: the exit status already carries the refusal.
            let _written = diagnostics.write_all(missing_program_message().as_bytes());
            return Some(1);
        }
    };
    if status.success() {
        return None;
    }
    let outcome: ChildOutcome = outcome_of(status);
    let description: String = match outcome {
        ChildOutcome::Exited(code) => format!("exited with status {code}"),
        ChildOutcome::Signaled(signal) => format!("was ended by signal {signal}"),
    };
    // Terminal output error: see above.
    let _written = diagnostics.write_all(failed_message(description.as_str()).as_bytes());
    let code: i32 = exit_code(outcome);
    return Some(if code == 0 { 1 } else { code });
}

/// What:
///  Run the upload as a bounded child whose output is captured, and return the failed attempt to stop with, or nothing
///  when it succeeded.
/// Why:
///   The auto-push records and shows a failed attempt's output; the explanation is part of it. A stop by deadline keeps
///  its own words, because the program did not exit by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = await runBoundedPush({ command: gitLfs(plan), deadlineMs }); return attempt.exitCode ? withExplanation(attempt) : undefined;
/// ```
pub fn run_upload_bounded(git: &PushGit, plan: &UploadPlan) -> Result<Option<PushAttempt>, SingleFlightError> {
    let mut command: Command = Command::new(LFS_PROGRAM);
    command
        .current_dir(&git.cwd)
        .env_clear()
        .envs(git.environment.iter().map(|(name, value)| {
            return (name, value);
        }))
        .env("PATH", path_with_real_git_first(git.environment.as_slice(), git.real_git.as_path()))
        .args(plan.arguments());
    let attempt: PushAttempt =
        match run_bounded_push(command, push_deadline(git.environment.as_slice()), STOP_GRACE, git.environment.as_slice()) {
            Ok(finished) => finished,
            Err(SingleFlightError::Start(_)) => {
                return Ok(Some(PushAttempt { exit_code: 1, output: missing_program_message().trim_end().to_owned() }));
            }
            Err(other) => return Err(other),
        };
    if attempt.exit_code == 0 {
        return Ok(None);
    }
    let description: String = if attempt.exit_code == STOPPED_EXIT_CODE {
        String::from("did not finish and was stopped")
    } else {
        format!("exited with status {}", attempt.exit_code)
    };
    let explanation: String = failed_message(description.as_str());
    let shown: &str = explanation.trim_end();
    let output: String = if attempt.output.is_empty() { shown.to_owned() } else { format!("{}\n{shown}", attempt.output) };
    return Ok(Some(PushAttempt { exit_code: attempt.exit_code, output }));
}

/// The plan and the runs are controlled in the test module.
#[cfg(test)]
#[path = "lfs_upload_tests.rs"]
mod tests;
