//! What:
//!  The `git commit` command of the native wrapper:
//! the preparation and landing through the resident service,
//! the settled
//!  events,
//! the post-commit gate and the handoff of the automatic backup.
//! Why:
//!  The wrapper runs the commit instead of forwarding it,
//! prints the pre-forward events only after the commit returned,
//!  then runs the post-commit gate for a landed commit and,
//! when the gate passes,
//! hands the push to the service.
//! A commit
//!  that landed nothing exits with its own status.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commit = await serviceCommit({ args, gitPath, policyOptions });
//! renderPolicyEvents(commit.policyResult.events);
//! const post = await runPostCommitLifecycle({ ... }); await handOverPush({ ... });
//! ```
use crate::commit_timing::CommitTiming;
use crate::time_spent_outside_the_phases::MeasuredStandardError;
use crate::service_commit::{ServiceCommit, service_commit};
use crate::service_handoff::{ServiceHandoff, forget_the_push, hand_over, promote_the_push};
use crate::service_messages::render;
use crate::commit_transaction::{CommitInvocation, TransactionStop};
use crate::config_schema::CliGitConfig;
use crate::command_commit::CommitRegion;
use crate::diagnostics::{EngineFailureCode, render_engine_failure};
use crate::git_version::check_git_version;
use crate::policy_checks::ShippedChecks;
use crate::policy_engine::StageRequest;
use crate::policy_events::{PolicyEvent, render_policy_events};
use crate::policy_trigger::Trigger;
use crate::post_commit::{POST_COMMIT_BLOCKED_EXIT_CODE, PostCommitRequest, run_post_commit_gate};
use crate::repository_facts::RepositoryFacts;
use crate::transaction_git::GitContext;
use crate::global_arguments::without_directory_changes;
use crate::wrapper_controls::Controls;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// What:
///  The effective invocation directory:
/// the process directory moved by every `-C` of the global options.
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
///   Everything the command wrote it wrote itself and in order,
/// so only the exit status is left to the caller.
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
///  Write text to standard error,
/// ignoring a closed stream,
///  and counting the wait for its reader.
/// Why:
///   The exit status already carries the result.
///  The write is the one place a slow reader becomes time the commit spends,
///  which the timing record states as `stderrMs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stderr.write(text);
/// ```
fn write_stderr(text: &str) {
    let mut stream: MeasuredStandardError<std::io::Stderr> = MeasuredStandardError::new();
    let _ = stream.write_all(text.as_bytes());
    let _ = stream.flush();
}

/// What:
///  Print what the transaction stopped with and name the exit status.
/// Why:
///   Each stop ends differently:
/// Git's own status with nothing printed,
/// a message with status 1,
/// or a finding with
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
                hint: crate::policy_events::EngineFailureHint::OfCode,
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
    /// Real Git,
    /// its overlay and the global options.
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
///  Run one commit through the service and its lifecycle.
/// Why:
///   The service is the only writer of a commit,
/// so the command takes no lock and recovers nothing:
/// it prepares the commit,
///  sends it,
/// and reports what the service landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runCommitCommand({ ... });
/// ```
pub fn run_commit_command<F: RepositoryFacts>(command: &CommitCommand<'_>, checks: &mut ShippedChecks<F>) -> CommitEnding {
    let mut timing: CommitTiming = CommitTiming::start();
    let ending: CommitEnding = run_timed(command, checks, &mut timing);
    // One append to the repository's timing file once everything the person sees is written; the terminal never shows it.
    timing.finish(ending.code);
    return ending;
}

/// `run_commit_command`,
/// recording in `timing` how the command's time was spent and how it ended.
fn run_timed<F: RepositoryFacts>(command: &CommitCommand<'_>, checks: &mut ShippedChecks<F>, timing: &mut CommitTiming) -> CommitEnding {
    if let Err(reason) = check_git_version(command.context) {
        write_stderr(render_engine_failure(0, EngineFailureCode::ContentUnavailable, reason.as_str()).as_str());
        return CommitEnding { code: 2 };
    }
    let start: PathBuf = match std::env::current_dir() {
        Ok(found) => found,
        Err(_) => PathBuf::from("."),
    };
    let cwd: PathBuf = effective_directory(start.as_path(), command.context.global_prefix.as_slice());
    // Every Git of this commit runs in `cwd`, the directory the caller's `-C` chain already reached, so the options it forwards
    //  keep everything the caller wrote but that `-C`: forwarding it moved Git a second time and read the repository beside the
    //  one the command was in.
    let context: GitContext = GitContext {
        global_prefix: without_directory_changes(command.context.global_prefix.as_slice()),
        ..command.context.clone()
    };
    let invocation: CommitInvocation<'_> = CommitInvocation {
        context: &context,
        environment: command.environment,
        cwd: cwd.as_path(),
        arguments: command.arguments,
        subcommand_index: command.subcommand_index,
        region: command.region,
        config: command.config,
        controls: command.controls,
    };
    let request: StageRequest = StageRequest {
        trigger: Trigger::PreForward,
        config: command.config.policies.clone(),
        controls: command.controls.clone(),
        selected: Vec::new(),
    };
    // Every command line lands through the resident service; a machine with none to reach or start ends here with the plain
    // message of its platform.
    let served: ServiceCommit = match service_commit(&invocation, checks, &request, timing) {
        Ok(found) => found,
        Err(stop) => {
            // A stop inside the build phase leaves it before its figure is taken, and the record must still state what the build spent.
            timing.close_an_open_build();
            return report_stop(stop);
        }
    };
    // A test kills the wrapper here: the commit is on the branch and answered, and everything the command still owes (the
    //  push, the upkeep and the notice of them) is written after this point (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
    //  finding 3). The marker is inert unless `CLI_GIT_TEST_ONLY_SERVICE_FAULT` names it, so a command that is not a test reaches
    //  nothing.
    crate::service_fault::reach("after-answer");
    write_stderr(render_policy_events(0, served.result.events.as_slice()).as_str());
    let (Some(landed), Some(handoff)) = (served.result.landed_oid, served.handoff) else {
        timing.outcome = "ended";
        return CommitEnding { code: served.result.exit_code };
    };
    timing.outcome = "landed";
    return post_commit(command, &handoff, landed.as_str(), timing);
}

/// What:
///  The post-commit lifecycle of a commit the service landed:
/// the gate,
/// then the handoff of the push and the upkeep.
/// Why:
///   The push runs after the command returns,
/// so the commit never waits for the network;
/// a blocked gate keeps the commit
///  local and exits 2,
/// and a handoff that fails is said plainly without changing the exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = await runGate(); if (gate.blocked) return 2; await handOver(); return 0;
/// ```
fn post_commit(command: &CommitCommand<'_>, handoff: &ServiceHandoff, landed: &str, timing: &mut CommitTiming) -> CommitEnding {
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
    let mut stderr: MeasuredStandardError<std::io::Stderr> = MeasuredStandardError::new();
    // The push record is left before the gate runs, so a wrapper killed while the gate runs loses no push; the gate takes it back
    //  when it blocks, which keeps a blocked commit local, and the hand-off below wakes the record the gate allowed.

    let gating: Instant = Instant::now();
    let gate = run_post_commit_gate(&request);
    timing.gate += gating.elapsed();
    write_stderr(render_policy_events(0, gate.events.as_slice()).as_str());
    if gate.blocked {
        forget_the_push(handoff, landed);
        return CommitEnding { code: POST_COMMIT_BLOCKED_EXIT_CODE };
    }
    // The gate approved the backup: the push the service held for this commit becomes runnable, and the hand-off below wakes it.
    if let Err(failure) = promote_the_push(handoff, landed) {
        write_stderr(format!("{}\n", render(&failure)).as_str());
    }
    // A test kills the wrapper here: the push record is runnable and nothing has been woken for it yet, so the service finds it
    //  only through the note the landing wrote (`service_claim_watch.rs`), and the person only through the line the next command
    //  reads from the record (`held_push_notices`). The marker is inert unless `CLI_GIT_TEST_ONLY_SERVICE_FAULT` names it.
    crate::service_fault::reach(crate::service_fault::AFTER_PROMOTE);
    let handing_over: Instant = Instant::now();
    let handed = hand_over(handoff, landed, &mut stderr);
    timing.handoff += handing_over.elapsed();
    if let Err(failure) = handed {
        write_stderr(format!("{}\n", render(&failure)).as_str());
    }
    return CommitEnding { code: 0 };
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_command_tests.rs"]
mod tests;
