//! What:
//!  The `git commit` command of the native wrapper: the transaction, the settled events, the post-commit gate and
//!  the automatic backup, in the incumbent's order.
//! Why:
//!  The incumbent (`src/bin.ts`) runs the commit transaction instead of forwarding the commit, prints the
//!  pre-forward events only after the transaction returned, then runs the post-commit lifecycle for a landed commit
//!  and, when the gate passes, the automatic push. A commit that landed nothing exits with the transaction's status.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const transaction = await runCommitTransactionBoundary({ args, gitPath, policyOptions, concurrency });
//! renderPolicyEvents(transaction.policyResult.events);
//! const post = await runPostCommitLifecycle({ ... }); await autoPush({ ... });
//! ```
use super::auto_push_single_flight::PushGit;
use super::child_environment::environment_with_overlay;
use super::commit_transaction::{CommitInvocation, TransactionResult, TransactionStop, run_commit_transaction};
use super::config_schema::CliGitConfig;
use super::command_commit::CommitRegion;
use super::diagnostic_log::warn;
use super::diagnostics::{EngineFailureCode, render_engine_failure};
use super::git_version::check_git_version;
use super::policy_checks::ShippedChecks;
use super::policy_engine::StageRequest;
use super::policy_events::{PolicyEvent, render_policy_events};
use super::policy_trigger::Trigger;
use super::post_commit::{PostCommitOutcome, PostCommitRequest, run_post_commit};
use super::repository_facts::RepositoryFacts;
use super::transaction_git::GitContext;
use super::transaction_lease::has_valid_preparation_lease;
use super::wrapper_controls::Controls;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "runCommitCommand";

/// What:
///  The effective invocation directory: the process directory moved by every `-C` of the global options.
/// Why:
///   Relative pathspecs and the transaction's Git queries resolve from where Git itself would run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseGlobalOptions(args).effectiveCwd
/// ```
pub fn effective_directory(start: &Path, global_prefix: &[OsString]) -> PathBuf {
    let mut directory: PathBuf = start.to_path_buf();
    let mut index: usize = 0;
    while index < global_prefix.len() {
        if global_prefix[index] == "-C"
            && let Some(value) = global_prefix.get(index + 1)
        {
            directory = directory.join(value);
            index += 2;
            continue;
        }
        index += 1;
    }
    return directory;
}

/// What:
///  The ending of one commit command.
/// Why:
///   Everything the command wrote it wrote itself and in order, so only the exit status is left to the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = code;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitEnding {
    /// The process exit status.
    pub code: i32,
}

/// What:
///  Write text to standard error, ignoring a closed stream.
/// Why:
///   The exit status already carries the result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stderr.write(text);
/// ```
fn write_stderr(text: &str) {
    let mut stream = std::io::stderr();
    let _ = stream.write_all(text.as_bytes());
    let _ = stream.flush();
}

/// What:
///  Print what the transaction stopped with and name the exit status.
/// Why:
///   Each stop ends differently: Git's own status with nothing printed, a message with status 1, or a finding with
///   status 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// catch (error) { ... process.exitCode = ... }
/// ```
fn report_stop(stop: TransactionStop) -> CommitEnding {
    match stop {
        TransactionStop::NativeExit(code) => return CommitEnding { code },
        TransactionStop::Message(message) => {
            write_stderr(format!("{message}\n").as_str());
            return CommitEnding { code: 1 };
        }
        TransactionStop::Failed { code, message } => {
            let event: PolicyEvent = PolicyEvent::EngineFailure {
                code,
                message,
                trigger: Some(Trigger::PreForward),
                policy: None,
                path: None,
            };
            write_stderr(render_policy_events(0, std::slice::from_ref(&event)).as_str());
            return CommitEnding { code: 2 };
        }
    }
}

/// What:
///  Everything the commit command reads from the wrapped invocation.
/// Why:
///   The command needs the identity and the loaded configuration the lifecycle already resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { rawArgs, gitPath, runtimeResolution, ... }
/// ```
pub struct CommitCommand<'a> {
    /// Real Git, its overlay and the global options.
    pub context: &'a GitContext,
    /// The invocation's environment.
    pub environment: &'a [(OsString, OsString)],
    /// The arguments with the wrapper's controls removed.
    pub arguments: &'a [OsString],
    /// The index of `commit` within the arguments.
    pub subcommand_index: usize,
    /// The commit options Git's table read.
    pub region: &'a CommitRegion,
    /// The loaded configuration.
    pub config: &'a CliGitConfig,
    /// The wrapper controls given on the command line.
    pub controls: &'a Controls,
}

/// What:
///  Run one `git commit` through the transaction and its lifecycle.
/// Why:
///   Startup recovery ran before configuration loaded, so the transaction starts from a clean registry unless a live
///   transaction belongs to another invocation, which does not stop it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runCommitCommand({ ... });
/// ```
pub fn run_commit_command<F: RepositoryFacts>(command: &CommitCommand<'_>, checks: &mut ShippedChecks<F>) -> CommitEnding {
    if let Err(reason) = check_git_version(command.context) {
        write_stderr(render_engine_failure(0, EngineFailureCode::ContentUnavailable, reason.as_str()).as_str());
        return CommitEnding { code: 2 };
    }
    let start: PathBuf = std::env::current_dir().unwrap_or_else(|_| { return PathBuf::from("."); });
    let cwd: PathBuf = effective_directory(start.as_path(), command.context.global_prefix.as_slice());
    let executable: PathBuf = match std::env::current_exe() {
        Ok(found) => found,
        Err(error) => {
            write_stderr(format!("cli-git: cannot determine this executable: {error}\n").as_str());
            return CommitEnding { code: 2 };
        }
    };
    let invocation: CommitInvocation<'_> = CommitInvocation {
        context: command.context,
        environment: command.environment,
        cwd: cwd.as_path(),
        arguments: command.arguments,
        subcommand_index: command.subcommand_index,
        region: command.region,
        config: command.config,
        controls: command.controls,
        inherited_lease_valid: has_valid_preparation_lease(command.environment),
        executable: executable.as_path(),
    };
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: command.config.policies.clone(),
        controls: command.controls.clone(),
        selected: Vec::new(),
    };
    let result: TransactionResult = match run_commit_transaction(&invocation, checks, &request) {
        Ok(found) => found,
        Err(stop) => return report_stop(stop),
    };
    write_stderr(render_policy_events(0, result.events.as_slice()).as_str());
    let Some(landed) = result.landed_oid else {
        return CommitEnding { code: result.exit_code };
    };
    return post_commit(command, &invocation, landed.as_str(), result.uses_lfs);
}

/// What:
///  The post-commit lifecycle of a landed commit: the gate over its changes, then the automatic backup.
/// Why:
///   A blocked gate keeps the commit local and exits 2; a passing gate lets the backup run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const post = await runPostCommitLifecycle({ ... }); if (post.blocked) throw new PolicyDecisionError(2); await autoPush({ ... });
/// ```
fn post_commit(command: &CommitCommand<'_>, invocation: &CommitInvocation<'_>, landed: &str, uses_lfs: bool) -> CommitEnding {
    let request: PostCommitRequest<'_> = PostCommitRequest {
        real_git: command.context.real_git.as_path(),
        landed_oid: landed,
        arguments: command.arguments,
        global_prefix: command.context.global_prefix.as_slice(),
        overlay: command.context.overlay.as_slice(),
        environment: command.environment,
        config: command.config.policies.clone(),
        controls: command.controls.clone(),
    };
    let git: PushGit = PushGit {
        real_git: command.context.real_git.clone(),
        cwd: invocation.cwd.to_path_buf(),
        environment: environment_with_overlay(command.environment, command.context.overlay.as_slice()),
        lfs_pre_push: if uses_lfs { Some(invocation.executable.to_path_buf()) } else { None },
    };
    let mut stderr = std::io::stderr();
    match run_post_commit(&request, &git, &mut stderr) {
        Ok(PostCommitOutcome { exit_code, .. }) => return CommitEnding { code: exit_code },
        Err(error) => {
            warn(TAG, format!("the post-commit lifecycle failed: {error:?}").as_str());
            write_stderr(format!("{error:?}\n").as_str());
            return CommitEnding { code: 1 };
        }
    }
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_command_tests.rs"]
mod tests;
