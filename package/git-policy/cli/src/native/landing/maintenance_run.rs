//! What:
//!  Running automatic maintenance after a landing:
//!  one `git config` read for the maintenance keys (or the commit's configuration snapshot),
//!  then `git maintenance run --auto`,
//! detached or not,
//! in the real worktree,
//! with Git's words kept in the journal.
//!  Its exit status is returned for the caller to ignore,
//! as native Git does.
//! Why:
//!  A landing writes its objects with `git fast-import`,
//! which leaves them loose when the batch holds at most
//!  `fastimport.unpackLimit` objects (100 by default;
//! Git 2.56.0 `builtin/fast-import.c`,
//! `end_packfile`) and keeps one pack
//!  otherwise;
//! without this step the loose objects and packs accumulate and every later Git process searches them all.
//!  Failures are logged and never fail the landed commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runAutoMaintenance({ gitPath, cwd, globalArgs });
//! ```

/// The decision and its error type.
use crate::config_snapshot::ConfigSnapshot;
use crate::maintenance_decision::{
    MAINTENANCE_CONFIG_PATTERN, MaintenanceDecision, MaintenanceError, auto_maintenance_arguments,
    parse_config_records,
};
/// Diagnostics on standard error,
/// switched like the incumbent's console sink.
use crate::diagnostic_log;
/// Where Git's words go now that the run carries no `--quiet`.
use crate::inner_git_words::record_words;
/// The real-Git context and the captured output of one start.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// The same start awaited by a task of the runtime.
use crate::transaction_git_awaiting::run_git_awaiting;
/// Ordered map of the maintenance keys to their canonical text.
use std::collections::BTreeMap;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Tag the incumbent's logger prints for this function,
/// so diagnostics match.
const TAG: &str = "runAutoMaintenance";

/// Exit status `parse_options` uses for an unknown option,
///  which Git before 2.47 reports for `maintenance run --detach`.
const USAGE_EXIT_CODE: i32 = 129;

/// Exit status of `git config --get-regexp` when no key matches.
const CONFIG_UNSET_EXIT_CODE: i32 = 1;

/// What:
///  What automatic maintenance did.
/// Why:
///   Configuration can turn it off,
/// which is not an exit status;
///  the caller distinguishes the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = typeof AUTO_MAINTENANCE_DISABLED | number;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutoMaintenanceOutcome {
    /// Configuration disabled maintenance;
    /// no maintenance Git ran.
    Disabled,
    /// Git's exit status for the maintenance run,
    /// or for the config read when that read failed.
    Exited(i32),
}

/// What:
///  Git's exit status,
/// where a signal ending Git counts as `1`.
/// Why:
///   The incumbent reads a missing exit code as `1` (`exitCode ?? 1`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.exitCode ?? 1
/// ```
fn exit_status(code: Option<i32>) -> i32 {
    return code.unwrap_or(1);
}

/// What:
///  The failure of a `git config` that could not start.
/// Why:
///  `map_err` hands over the error of the start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configStartFailure = (error: Error) => startFailure('git config', error);
/// ```
fn config_start_failure(error: std::io::Error) -> MaintenanceError {
    return start_failure("git config", error);
}

/// What:
///  The failure of a `git maintenance run` that could not start.
/// Why:
///  `map_err` hands over the error of the start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const maintenanceStartFailure = (error: Error) => startFailure('git maintenance run', error);
/// ```
fn maintenance_start_failure(error: std::io::Error) -> MaintenanceError {
    return start_failure("git maintenance run", error);
}

/// What:
///  Map a start failure to a maintenance error that names the command.
/// Why:
///   The caller logs one line that says which Git start could not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new CommitTransactionGitError(`${command} could not start`);
/// ```
fn start_failure(command: &str, error: std::io::Error) -> MaintenanceError {
    return MaintenanceError::Start(format!("{command} could not start: {error}"));
}

/// What:
///  The maintenance keys as the decision reads them,
/// or the status of a config read that failed.
/// Why:
///   A failed read ends maintenance with Git's status,
/// which is not an error and not a decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MaintenanceConfig = { records: Map<string, string> } | { exited: number };
/// ```
enum MaintenanceConfig {
    /// Each set key with the text Git prints for its last value.
    Records(BTreeMap<String, String>),
    /// The read failed with this status.
    Exited(i32),
}

/// What:
///  Read the maintenance keys by starting `git config`.
/// Why:
///   The incumbent's read,
/// kept for a repository whose snapshot is missing or holds a value it does not model.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['config', '--type=bool-or-int', '--null', '--get-regexp', PATTERN]);
/// ```
fn read_maintenance_config(context: &GitContext, cwd: &Path) -> Result<MaintenanceConfig, MaintenanceError> {
    // Config lookup; exit 1 means no key is set.
    let lookup: GitOutput = run_git(context, &maintenance_config_request(cwd)).map_err(config_start_failure)?;
    return maintenance_config_of(&lookup);
}

/// What:
///  The same read,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The synchronous face enters the runtime for the pipes of a step that is ended at its ceiling,
/// and that entering is refused from inside the runtime
/// (`lazy_runtime::run_on_runtime`),
///  so the landing round,
/// which runs as tasks,
/// asks for the read here.
/// The request and the parse are the synchronous face's,
/// so the two cannot differ in what they read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['config', '--type=bool-or-int', '--null', '--get-regexp', PATTERN]);
/// ```
async fn read_maintenance_config_awaiting(context: &GitContext, cwd: &Path) -> Result<MaintenanceConfig, MaintenanceError> {
    let lookup: GitOutput = run_git_awaiting(context, &maintenance_config_request(cwd)).await.map_err(config_start_failure)?;
    return maintenance_config_of(&lookup);
}

/// What:
///  The one question both faces of the config read put to Git.
/// Why:
///   A read whose command line differed between the faces would read a different configuration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitRequest(cwd, ['config', '--type=bool-or-int', '--null', '--get-regexp', PATTERN]);
/// ```
fn maintenance_config_request(cwd: &Path) -> GitRequest {
    return GitRequest::new(cwd, &["config", "--type=bool-or-int", "--null", "--get-regexp", MAINTENANCE_CONFIG_PATTERN]);
}

/// What:
///  The maintenance keys one finished config read answered,
/// or the status it failed with.
/// Why:
///   The read's parse,
/// its warning and its exit rule are the same whichever face ran the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// configOf(lookup); // { records } | { exited }
/// ```
fn maintenance_config_of(lookup: &GitOutput) -> Result<MaintenanceConfig, MaintenanceError> {
    let lookup_code: i32 = exit_status(lookup.code);
    if lookup_code != 0 && lookup_code != CONFIG_UNSET_EXIT_CODE {
        diagnostic_log::warn(
            TAG,
            format!(
                "skipping automatic maintenance: git config exited {lookup_code}: {}",
                String::from_utf8_lossy(&lookup.stderr).trim()
            )
            .as_str(),
        );
        return Ok(MaintenanceConfig::Exited(lookup_code));
    }
    // An unset key prints nothing, so an exit of 1 parses as an empty configuration.
    let config_bytes: &[u8] = if lookup_code == 0 { lookup.stdout.as_slice() } else { &[] };
    return Ok(MaintenanceConfig::Records(parse_config_records(config_bytes)?));
}

/// What:
///  Run automatic maintenance in the real worktree with the caller's global options,
///  reading the maintenance keys by starting `git config`.
/// Why:
///   The read and the run happen in the same order the incumbent uses,
///  so the decision and its effect agree on one configuration snapshot per read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runAutoMaintenance({ gitPath, cwd, globalArgs });
/// ```
pub fn run_auto_maintenance(
    context: &GitContext,
    cwd: &Path,
) -> Result<AutoMaintenanceOutcome, MaintenanceError> {
    return run_auto_maintenance_with(context, cwd, None);
}

/// What:
///  Run automatic maintenance,
/// taking the maintenance keys from the commit's configuration snapshot when it
///  models them.
/// Why:
///   The snapshot was read when the commit began,
/// so a configuration that changed during the commit is not seen
///  (a hook that sets `maintenance.auto` is the example);
///  a snapshot read with global options,
/// or one holding a value it does not model,
/// starts `git config` as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // config from the snapshot, else a read; then `git maintenance run --auto --detach|--no-detach`
/// ```
pub fn run_auto_maintenance_with(
    context: &GitContext,
    cwd: &Path,
    snapshot: Option<&ConfigSnapshot>,
) -> Result<AutoMaintenanceOutcome, MaintenanceError> {
    let config: BTreeMap<String, String> = match config_from_snapshot(snapshot) {
        Some(records) => records,
        None => match read_maintenance_config(context, cwd)? {
            MaintenanceConfig::Records(records) => records,
            MaintenanceConfig::Exited(code) => return Ok(AutoMaintenanceOutcome::Exited(code)),
        },
    };
    let Some(arguments) = planned_arguments(&config) else { return Ok(AutoMaintenanceOutcome::Disabled) };
    let result: GitOutput = run_git(context, &maintenance_request(cwd, arguments.as_slice())).map_err(maintenance_start_failure)?;
    let fallback_arguments: &[&str] = match ended(arguments.as_slice(), &result) {
        Ended::Finished(outcome) => return Ok(outcome),
        Ended::RetryWithoutDetach(fallback_arguments) => fallback_arguments,
    };
    let fallback: GitOutput = run_git(context, &maintenance_request(cwd, fallback_arguments)).map_err(maintenance_start_failure)?;
    return Ok(reported_fallback(fallback_arguments, &fallback));
}

/// What:
///  The same run,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round of the service runs as tasks of the runtime
/// (`foundation/parallel_runner_on_the_runtime_awaiting.rs`),
/// and the synchronous faces of a Git step enter the runtime and are refused from inside it,
/// so a round
///  that ran maintenance after a landing could not call them.
/// The read,
/// the decision and the journal lines are the synchronous face's,
/// so the two cannot differ in what they
///  run or record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // config from the snapshot, else an awaited read; then an awaited `git maintenance run --auto --detach|--no-detach`
/// ```
pub async fn run_auto_maintenance_with_awaiting(
    context: &GitContext,
    cwd: &Path,
    snapshot: Option<&ConfigSnapshot>,
) -> Result<AutoMaintenanceOutcome, MaintenanceError> {
    let config: BTreeMap<String, String> = match config_from_snapshot(snapshot) {
        Some(records) => records,
        None => match read_maintenance_config_awaiting(context, cwd).await? {
            MaintenanceConfig::Records(records) => records,
            MaintenanceConfig::Exited(code) => return Ok(AutoMaintenanceOutcome::Exited(code)),
        },
    };
    let Some(arguments) = planned_arguments(&config) else { return Ok(AutoMaintenanceOutcome::Disabled) };
    let result: GitOutput = run_git_awaiting(context, &maintenance_request(cwd, arguments.as_slice())).await.map_err(maintenance_start_failure)?;
    let fallback_arguments: &[&str] = match ended(arguments.as_slice(), &result) {
        Ended::Finished(outcome) => return Ok(outcome),
        Ended::RetryWithoutDetach(fallback_arguments) => fallback_arguments,
    };
    let fallback: GitOutput =
        run_git_awaiting(context, &maintenance_request(cwd, fallback_arguments)).await.map_err(maintenance_start_failure)?;
    return Ok(reported_fallback(fallback_arguments, &fallback));
}

/// What:
///  The maintenance keys a commit's configuration snapshot answers,
/// for the repositories it models.
/// Why:
///   A snapshot that was read with global options in front answers nothing:
/// its records may not be the repository's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.withoutGlobalOptions ? snapshot.maintenanceRecords() : undefined;
/// ```
fn config_from_snapshot(snapshot: Option<&ConfigSnapshot>) -> Option<BTreeMap<String, String>> {
    return match snapshot {
        Some(candidate) if candidate.without_global_options() => candidate.maintenance_records(),
        _ => None,
    };
}

/// What:
///  The maintenance arguments the configuration asks for,
/// or nothing when it turns maintenance off.
/// Why:
///   Both faces decide before they run anything,
/// and a disabled run starts no Git at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decision(config).kind === 'disabled' ? undefined : decision.arguments;
/// ```
fn planned_arguments(config: &BTreeMap<String, String>) -> Option<Vec<&'static str>> {
    // Native maintenance arguments, or the disabled outcome.
    let arguments: Vec<&'static str> = match auto_maintenance_arguments(config) {
        MaintenanceDecision::Disabled => {
            diagnostic_log::debug(TAG, "automatic maintenance disabled by configuration");
            return None;
        }
        MaintenanceDecision::Run(arguments) => arguments,
    };
    diagnostic_log::debug(TAG, format!("running git {}", arguments.join(" ")).as_str());
    return Some(arguments);
}

/// What:
///  The one request both faces put to Git for the maintenance run.
/// Why:
///   A run whose command line differed between the faces would run a different kind of maintenance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitRequest(cwd, arguments);
/// ```
fn maintenance_request(cwd: &Path, arguments: &[&'static str]) -> GitRequest {
    return GitRequest::new(cwd, arguments);
}

/// What:
///  What one finished maintenance run came to:
/// its outcome,
/// or the arguments of the retry Git before 2.47 needs.
/// Why:
///   A refused `--detach` is the one result that asks for another run,
/// and both faces must read it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ended = { kind: 'finished'; outcome } | { kind: 'retry'; arguments };
/// ```
enum Ended<'a> {
    /// The run finished;
    /// what it came to is the run's answer.
    Finished(AutoMaintenanceOutcome),
    /// Git before 2.47 refused `--detach`;
    /// these are the arguments of the form it has.
    RetryWithoutDetach(&'a [&'static str]),
}

/// What:
///  What one finished maintenance run came to,
/// keeping what it said in the journal.
/// Why:
///   What the run said (the foreground tasks before it detaches;
/// a detached part writes to `/dev/null`,
/// Git 2.56.0 `setup.c`
///  `daemonize`) is kept in the journal;
/// a refused `--detach` of Git before 2.47 is logged instead,
/// and the pre-2.47 form omits it
///  because gc detaches on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (code !== 129) { recordWords(arguments, result); return { kind: 'finished', outcome: code } } return { kind: 'retry', arguments: arguments.slice(0, -1) };
/// ```
fn ended<'a>(arguments: &'a [&'static str], result: &GitOutput) -> Ended<'a> {
    let result_code: i32 = exit_status(result.code);
    if result_code != USAGE_EXIT_CODE {
        record_words(arguments, result);
        diagnostic_log::debug(TAG, format!("automatic maintenance exited {result_code}").as_str());
        return Ended::Finished(AutoMaintenanceOutcome::Exited(result_code));
    }
    // Git before 2.47 has no detach option; its native form omits it and gc detaches on its own.
    let flag: &str = arguments.last().copied().unwrap_or("");
    diagnostic_log::debug(
        TAG,
        format!("git rejected {flag}; retrying in the pre-2.47 form: {}", String::from_utf8_lossy(&result.stderr).trim()).as_str(),
    );
    return Ended::RetryWithoutDetach(&arguments[..arguments.len().saturating_sub(1)]);
}

/// What:
///  The outcome of the pre-2.47 retry,
/// with what it said kept in the journal.
/// Why:
///   Every step of the run records its words and its status,
/// whatever the form Git has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordWords(arguments, fallback); return fallback.exitCode;
/// ```
fn reported_fallback(arguments: &[&'static str], fallback: &GitOutput) -> AutoMaintenanceOutcome {
    let fallback_code: i32 = exit_status(fallback.code);
    record_words(arguments, fallback);
    diagnostic_log::debug(TAG, format!("automatic maintenance exited {fallback_code}").as_str());
    return AutoMaintenanceOutcome::Exited(fallback_code);
}

/// Runner controls live beside the runner.
#[cfg(test)]
#[path = "maintenance_run_tests.rs"]
mod tests;
