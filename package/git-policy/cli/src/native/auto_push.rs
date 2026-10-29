//! What:
//!  Post-commit auto-push of a landed commit: which push runs, what its output shows,
//!  and how a coordination failure is reported without failing the commit.
//! Why:
//!  Every commit in this repository is backed up (`AGENTS.md` rule APG),
//!  and the incumbent's `src/auto-push.ts` decides the push argv from the upstream state,
//!  prints only GitHub `remote:` lines on success and the full output on failure,
//!  and keeps the commit's exit status when the push fails.
//!  The landed commit is an input, so the integrator can call this after any landing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await autoPush({ gitPath, cwd, landedOid });
//! ```

/// The reconcile of a push that a moved remote branch rejected, and the lines it adds to the output.
use super::auto_push_reconcile::{
    RECONCILE_FAILED_PREFIX, RECONCILE_LINE_PREFIX, named_destination_push, reconcile_divergent_push,
};
/// The single-flight coordinator, its Git command builder and its error type.
use super::auto_push_single_flight::{
    PushAttempt, PushGit, SingleFlightError, SingleFlightKind, SingleFlightOutcome, push_command,
    resolve_push_coordination_paths, run_query, run_single_flight_push_repeating,
};
/// The deadline-bound push runner and the limits it applies.
use super::bounded_push::{
    LOCK_WAIT_VARIABLE, STOP_GRACE, apply_low_speed_defaults, lock_wait_limit, push_deadline,
    run_bounded_push,
};
/// The diagnostic log of the wrapper.
use super::diagnostic_log::debug;
/// The lock record whose owner the waiting notice names.
use super::owner_lock_record::{PublishedOwner, read_owner_lock_record};
/// Owned OS strings for Git arguments.
use std::ffi::OsString;
/// `Write` is the stderr sink the terminal lines go to.
use std::io::Write;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// Prefix of the GitHub server-side lines a clean push surfaces.
const REMOTE_LINE_PREFIX: &str = "remote: ";

/// Note printed when HEAD is detached: pushing `HEAD` needs a branch.
pub const DETACHED_HEAD_NOTE: &str = "cli-git: HEAD is detached (mid-rebase/cherry-pick or a detached checkout); skipping auto-push.";

/// Note printed after every failed push, owned or joined: the commit stays local.
pub const PUSH_FAILED_NOTE: &str =
    "cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.";

/// Push argv for a branch without an upstream: create it on origin and record the upstream.
const FIRST_PUSH_ARGUMENTS: &[&str] = &["push", "--set-upstream", "origin", "HEAD"];

/// Push argv for a branch with an upstream: follow the configured upstream, wherever it lives.
const UPSTREAM_PUSH_ARGUMENTS: &[&str] = &["push"];

/// What:
///  Which branch the auto-push took.
/// Why:
///   `Skipped` covers both "nowhere to back up to" and "detached HEAD"; both exit 0 silently or with a note.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AutoPushOutcome = 'skipped' | 'pushed' | 'failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutoPushOutcome {
    /// Nothing was pushed.
    Skipped,
    /// A successful push delivered the commit.
    Pushed,
    /// The push that would deliver the commit failed or could not run.
    Failed,
}

/// What:
///  The result of one auto-push attempt.
/// Why:
///   `shown` is what the terminal received, so tests compare the surfaced text directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AutoPushResult = { outcome; exitCode; shown; joined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoPushResult {
    /// Which branch the auto-push took.
    pub outcome: AutoPushOutcome,
    /// Push exit code; `0` when skipped or pushed, Git's code (or `1`) on failure.
    pub exit_code: i64,
    /// Text surfaced to stderr; empty on a skip, a quiet clean push, or a joined success.
    pub shown: String,
    /// Whether another invocation's push decided the outcome.
    pub joined: bool,
}

/// What:
///  Select the output a push shows: only `remote:` lines after a clean push, everything after a failure.
/// Why:
///   A rejection, a hook refusal or an offline error stays diagnosable only when the full output shows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filterPushOutput({ output, exitCode }): string;
/// ```
pub fn filter_push_output(output: &str, exit_code: i64) -> String {
    if exit_code != 0 {
        return output.to_owned();
    }
    return output
        .split('\n')
        .filter(|line| {
            return line.starts_with(REMOTE_LINE_PREFIX) || line.starts_with(RECONCILE_LINE_PREFIX);
        })
        .collect::<Vec<&str>>()
        .join("\n");
}

/// What:
///  Whether a read-only Git query exits zero.
/// Why:
///   A non-zero exit is an answer ("no upstream", "no origin"); a failure to start Git is not an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await gitQuerySucceeds({ gitPath, cwd, args }); // false on SubprocessError only
/// ```
fn query_succeeds(git: &PushGit, arguments: &[&str]) -> Result<bool, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return Ok(run_query(git, &argv)?.code == Some(0));
}

/// What:
///  The branch HEAD names, or nothing when HEAD is detached.
/// Why:
///   `symbolic-ref --quiet HEAD` exits non-zero exactly when HEAD is not a branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readSymbolicHead({ gitPath, cwd }): Promise<string | typeof HEAD_DETACHED>;
/// ```
fn read_symbolic_head(git: &PushGit) -> Result<Option<String>, SingleFlightError> {
    let argv: Vec<OsString> = ["symbolic-ref", "--quiet", "HEAD"]
        .iter()
        .map(OsString::from)
        .collect();
    let output = run_query(git, &argv)?;
    if output.code != Some(0) {
        return Ok(None);
    }
    return Ok(Some(
        super::js_text::trim_javascript(&String::from_utf8_lossy(&output.stdout)).to_owned(),
    ));
}

/// What:
///  Run one real `git push` and capture its interleaved output, with one trailing line break removed,
///  stopping it when its deadline passes.
/// Why:
///   Git's output of stdout and stderr shares one file so the lines interleave in the order they were written,
///  the trailing line break is removed as the incumbent's captured output is,
///  and a push that stalls must not hold the branch's push lock forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { exitCode, output } = await nanoSpawn(gitPath, pushArgs, { cwd }) // SubprocessError -> { exitCode ?? 1, output }
/// ```
fn run_push_command(git: &PushGit, arguments: &[&str]) -> Result<PushAttempt, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    let mut command = push_command(git, &argv);
    apply_low_speed_defaults(&mut command, &git.environment);
    return run_bounded_push(command, push_deadline(&git.environment), STOP_GRACE, git.environment.as_slice());
}

/// What:
///  The stderr line shown when this commit has to wait behind another process's push.
/// Why:
///   A silent wait looks like a hang; the line names the process to look at and how long the wait lasts,
///  and says the commit is already saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: waiting for process ${pid} to finish pushing ${branchRef}; giving up after ${seconds} seconds`
/// ```
fn waiting_line(git: &PushGit, branch_ref: &str, waited: Option<std::time::Duration>) -> String {
    let holder: String = match holder_pid(git, branch_ref) {
        Some(pid) => format!("process {pid}"),
        None => String::from("another cli-git process"),
    };
    // A repeat names how long the wait has lasted; the first line has just begun.
    let progress: String = match waited {
        Some(elapsed) => format!(" ({} seconds so far)", elapsed.as_secs()),
        None => String::new(),
    };
    return format!(
        "cli-git: waiting for {holder} to finish pushing {branch_ref}{progress}; giving up after {} seconds \
         ({LOCK_WAIT_VARIABLE} changes this). The commit is already saved.",
        lock_wait_limit(&git.environment).as_secs()
    );
}

/// The process that holds the branch's push lock, when its record can be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readOwnerLockRecord(lockDirectory))?.ownerPid
/// ```
fn holder_pid(git: &PushGit, branch_ref: &str) -> Option<i64> {
    let paths = resolve_push_coordination_paths(git, branch_ref).ok()?;
    return match read_owner_lock_record(&paths.lock_directory) {
        Ok(PublishedOwner::Owner(record)) => Some(record.owner_pid),
        Ok(PublishedOwner::Busy) | Err(_) => None,
    };
}

/// What:
///  Surface a settled single-flight outcome and convert it into the auto-push result.
/// Why:
///   Lines go to `err` exactly as Git printed them, followed by the uncovered note and the failure note,
///  so the commit's terminal output matches the incumbent's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportSingleFlightOutcome({ outcome, landedOid, branchRef }): AutoPushResult;
/// ```
fn report_outcome(
    outcome: &SingleFlightOutcome,
    landed_oid: &str,
    branch_ref: &str,
    err: &mut dyn Write,
) -> std::io::Result<AutoPushResult> {
    let joined = matches!(
        outcome.kind,
        SingleFlightKind::JoinedPushed | SingleFlightKind::JoinedFailed
    );
    let failed = matches!(
        outcome.kind,
        SingleFlightKind::Failed | SingleFlightKind::JoinedFailed
    );
    let shown = if outcome.kind == SingleFlightKind::JoinedPushed {
        String::new()
    } else {
        filter_push_output(&outcome.output, outcome.exit_code)
    };
    if !shown.is_empty() {
        writeln!(err, "{shown}")?;
    }
    if !outcome.covered {
        writeln!(
            err,
            "cli-git: landed commit {landed_oid} is no longer on {branch_ref}; auto-push pushed {} without it.",
            outcome.tip
        )?;
    }
    // A reconcile failure already says why the commit is unpublished; "run `git push`" would repeat a push that cannot succeed.
    if failed && !outcome.output.contains(RECONCILE_FAILED_PREFIX) {
        writeln!(err, "{PUSH_FAILED_NOTE}")?;
    }
    return Ok(AutoPushResult {
        outcome: if failed {
            AutoPushOutcome::Failed
        } else {
            AutoPushOutcome::Pushed
        },
        exit_code: outcome.exit_code,
        shown,
        joined,
    });
}

/// What:
///  Back up a landed commit by pushing its branch, single-flight per branch.
/// Why:
///   The commit has happened by the time this runs and Git ignores a post-commit hook's status,
///  so a failed push or a failed coordination is printed and returned as `Failed`, and never fails the commit.
///   Failures before the push decision (an unreadable repository, a Git that cannot start) are returned as errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function autoPush({ gitPath, cwd, landedOid }): Promise<AutoPushResult>;
/// ```
pub fn auto_push(
    git: &PushGit,
    landed_oid: &str,
    err: &mut dyn Write,
) -> Result<AutoPushResult, SingleFlightError> {
    let has_upstream = query_succeeds(
        git,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    )?;
    if !has_upstream && !query_succeeds(git, &["remote", "get-url", "origin"])? {
        debug(TAG, "no upstream and no origin remote; skipping auto-push");
        return Ok(AutoPushResult {
            outcome: AutoPushOutcome::Skipped,
            exit_code: 0,
            shown: String::new(),
            joined: false,
        });
    }
    let Some(branch_ref) = read_symbolic_head(git)? else {
        // Terminal output error: the note itself is the only diagnostic; a failed write to stderr is not a push failure.
        let _written = writeln!(err, "{DETACHED_HEAD_NOTE}");
        return Ok(AutoPushResult {
            outcome: AutoPushOutcome::Skipped,
            exit_code: 0,
            shown: DETACHED_HEAD_NOTE.to_owned(),
            joined: false,
        });
    };
    let named: Option<Vec<String>> = if has_upstream {
        named_destination_push(git, &branch_ref)?
    } else {
        None
    };
    let named_arguments: Vec<&str> = named
        .iter()
        .flatten()
        .map(String::as_str)
        .collect();
    let push_arguments: &[&str] = if !named_arguments.is_empty() {
        named_arguments.as_slice()
    } else if has_upstream {
        UPSTREAM_PUSH_ARGUMENTS
    } else {
        FIRST_PUSH_ARGUMENTS
    };
    debug(
        TAG,
        &format!("auto-pushing {landed_oid} with {push_arguments:?} on {branch_ref}"),
    );
    let coordinated = {
        let mut push = || {
            let attempt = run_push_command(git, push_arguments)?;
            // Only a branch with an upstream can be rejected for a moved remote branch that a merge cures.
            if has_upstream && attempt.exit_code != 0 {
                return reconcile_divergent_push(git, &branch_ref, attempt);
            }
            return Ok(attempt);
        };
        let mut on_wait = |waited: Option<std::time::Duration>| {
            let line = waiting_line(git, &branch_ref, waited);
            debug(TAG, &line);
            // Terminal output error: the notice is advisory; a failed write to stderr must not stop the wait.
            let _written = writeln!(err, "{line}");
        };
        run_single_flight_push_repeating(git, &branch_ref, landed_oid, &mut push, &mut on_wait)
    };
    return match coordinated {
        Ok(outcome) => {
            report_outcome(&outcome, landed_oid, &branch_ref, err).map_err(SingleFlightError::Io)
        }
        Err(error) => {
            debug(TAG, &format!("auto-push coordination failed: {error}"));
            let _written = writeln!(
                err,
                "cli-git: auto-push of {landed_oid} could not run: {error}"
            );
            let _noted = writeln!(err, "{PUSH_FAILED_NOTE}");
            Ok(AutoPushResult {
                outcome: AutoPushOutcome::Failed,
                exit_code: 1,
                shown: error.to_string(),
                joined: false,
            })
        }
    };
}

/// Auto-push behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_tests.rs"]
mod tests;

/// Racing auto-push processes on one branch, killed pushers and joins.
#[cfg(test)]
#[path = "auto_push_concurrency_tests.rs"]
mod concurrency_tests;
