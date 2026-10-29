//! What:
//!  Running automatic maintenance after a landing:
//!  one `git config` read for the maintenance keys (or the commit's configuration snapshot),
//!  then `git maintenance run --auto --quiet`, detached or not, in the real worktree.
//!  Its exit status is returned for the caller to ignore, as native Git does.
//! Why:
//!  Every landing migrates its objects into one pack;
//!  without this step the pack count grows by one per commit and every later Git process searches every pack.
//!  Failures are logged and never fail the landed commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runAutoMaintenance({ gitPath, cwd, globalArgs });
//! ```

/// The decision and its error type.
use super::config_snapshot::ConfigSnapshot;
use super::maintenance_decision::{
    MAINTENANCE_CONFIG_PATTERN, MaintenanceDecision, MaintenanceError, auto_maintenance_arguments,
    parse_config_records,
};
/// Diagnostics on standard error, switched like the incumbent's console sink.
use super::diagnostic_log;
/// The real-Git context and the captured output of one start.
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// Ordered map of the maintenance keys to their canonical text.
use std::collections::BTreeMap;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Tag the incumbent's logger prints for this function, so diagnostics match.
const TAG: &str = "runAutoMaintenance";

/// Exit status `parse_options` uses for an unknown option,
///  which Git before 2.47 reports for `maintenance run --detach`.
const USAGE_EXIT_CODE: i32 = 129;

/// Exit status of `git config --get-regexp` when no key matches.
const CONFIG_UNSET_EXIT_CODE: i32 = 1;

/// What:
///  What automatic maintenance did.
/// Why:
///   Configuration can turn it off, which is not an exit status;
///  the caller distinguishes the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = typeof AUTO_MAINTENANCE_DISABLED | number;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutoMaintenanceOutcome {
    /// Configuration disabled maintenance; no maintenance Git ran.
    Disabled,
    /// Git's exit status for the maintenance run, or for the config read when that read failed.
    Exited(i32),
}

/// What:
///  Git's exit status, where a signal ending Git counts as `1`.
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
///  The maintenance keys as the decision reads them, or the status of a config read that failed.
/// Why:
///   A failed read ends maintenance with Git's status, which is not an error and not a decision.
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
///   The incumbent's read, kept for a repository whose snapshot is missing or holds a value it does not model.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['config', '--type=bool-or-int', '--null', '--get-regexp', PATTERN]);
/// ```
fn read_maintenance_config(context: &GitContext, cwd: &Path) -> Result<MaintenanceConfig, MaintenanceError> {
    // Config lookup; exit 1 means no key is set.
    let lookup: GitOutput = run_git(
        context,
        &GitRequest::new(
            cwd,
            &[
                "config",
                "--type=bool-or-int",
                "--null",
                "--get-regexp",
                MAINTENANCE_CONFIG_PATTERN,
            ],
        ),
    )
    .map_err(|error| return start_failure("git config", error))?;
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
    let config_bytes: &[u8] = if lookup_code == 0 {
        lookup.stdout.as_slice()
    } else {
        &[]
    };
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
///  Run automatic maintenance, taking the maintenance keys from the commit's configuration snapshot when it
///  models them.
/// Why:
///   The snapshot was read when the commit began, so a configuration that changed during the commit is not seen
///  (a hook that sets `maintenance.auto` is the example);
///  a snapshot read with global options, or one holding a value it does not model, starts `git config` as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // config from the snapshot, else a read; then `git maintenance run --auto --quiet --detach|--no-detach`
/// ```
pub fn run_auto_maintenance_with(
    context: &GitContext,
    cwd: &Path,
    snapshot: Option<&ConfigSnapshot>,
) -> Result<AutoMaintenanceOutcome, MaintenanceError> {
    let from_snapshot: Option<BTreeMap<String, String>> = snapshot
        .filter(|candidate| { return candidate.without_global_options(); })
                .and_then(ConfigSnapshot::maintenance_records);
    let config: BTreeMap<String, String> = match from_snapshot {
        Some(records) => records,
        None => match read_maintenance_config(context, cwd)? {
            MaintenanceConfig::Records(records) => records,
            MaintenanceConfig::Exited(code) => return Ok(AutoMaintenanceOutcome::Exited(code)),
        },
    };
    // Native maintenance arguments, or the disabled outcome.
    let arguments: Vec<&'static str> = match auto_maintenance_arguments(&config) {
        MaintenanceDecision::Disabled => {
            diagnostic_log::debug(
                TAG,
                "automatic maintenance disabled by configuration",
            );
            return Ok(AutoMaintenanceOutcome::Disabled);
        }
        MaintenanceDecision::Run(arguments) => arguments,
    };
    diagnostic_log::debug(TAG, format!("running git {}", arguments.join(" ")).as_str());
    // Maintenance outcome; a detached run returns once the background process is forked.
    let result: GitOutput = run_git(
        context,
        &GitRequest::new(cwd, arguments.as_slice()),
    )
    .map_err(|error| return start_failure("git maintenance run", error))?;
    let result_code: i32 = exit_status(result.code);
    if result_code != USAGE_EXIT_CODE {
        diagnostic_log::debug(
            TAG,
            format!("automatic maintenance exited {result_code}").as_str(),
        );
        return Ok(AutoMaintenanceOutcome::Exited(result_code));
    }
    // Git before 2.47 has no detach option; its native form omits it and gc detaches on its own.
    let flag: &str = arguments.last().copied().unwrap_or("");
    diagnostic_log::debug(
        TAG,
        format!(
            "git rejected {flag}; retrying in the pre-2.47 form: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )
        .as_str(),
    );
    let fallback_arguments: &[&str] = &arguments[..arguments.len() - 1];
    let fallback: GitOutput = run_git(
        context,
        &GitRequest::new(cwd, fallback_arguments),
    )
    .map_err(|error| return start_failure("git maintenance run", error))?;
    let fallback_code: i32 = exit_status(fallback.code);
    diagnostic_log::debug(
        TAG,
        format!("automatic maintenance exited {fallback_code}").as_str(),
    );
    return Ok(AutoMaintenanceOutcome::Exited(fallback_code));
}

/// Runner controls live beside the runner.
#[cfg(test)]
#[path = "maintenance_run_tests.rs"]
mod tests;
