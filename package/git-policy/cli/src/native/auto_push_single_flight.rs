//! What:
//!  Single-flight auto-push per branch: one owner lock serializes the pushes of a branch,
//!  and the `last-pushed` record names the most recent attempt.
//! Why:
//!  A commit that a recorded successful push already covers finishes without pushing,
//!  and a commit that waits behind a live pusher joins the push it waited for,
//!  so concurrent landings on one branch cost fewer pushes than commits.
//!  The incumbent's `src/auto-push-single-flight.ts` decides these joins,
//!  and the lock and record paths must match its paths byte for byte.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runSingleFlightPush({ gitPath, cwd, branchRef, landedOid, push, onWait });
//! ```

/// The branch key, the record and the owner lock this module coordinates through.
use super::auto_push_branch_key::encode_branch_key;
use super::auto_push_record::{
    LAST_PUSHED_SUFFIX, LastPushedOutcome, LastPushedRead, LastPushedRecord,
    read_last_pushed_record, write_last_pushed_record,
};
use super::bounded_push::{PUSH_DEADLINE_VARIABLE, STOP_GRACE, lock_wait_limit};
use super::bounded_query::{QueryOutput, query_deadline, run_bounded_query};
use super::diagnostic_log::debug;
use super::js_text::trim_javascript;
use super::owner_lock::{
    DEFAULT_POLL_DELAY, OwnerLock, OwnerLockError, WaitNotice, acquire_owner_lock_for,
};
/// Owned OS strings for Git arguments and the child environment.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Command` starts real Git.
use std::process::Command;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// Exit code `git merge-base --is-ancestor` uses for "not an ancestor".
const NOT_ANCESTOR_EXIT_CODE: i32 = 1;

/// Mode of the coordination directory, shared with the incumbent.
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

/// What:
///  Why a single-flight push could not finish.
/// Why:
///   Every variant is a coordination failure, which the caller reports and keeps the commit for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SubprocessError | TypeError | OwnerLockError | NodeJS.ErrnoException
/// ```
#[derive(Debug)]
pub enum SingleFlightError {
    /// Git could not be started, or failed in a way the incumbent does not model as an exit status.
    Start(std::io::Error),
    /// A Git query exited with a status the caller did not expect.
    Git {
        /// Arguments of the query, for the diagnostic.
        arguments: String,
        /// Exit code, or nothing when a signal ended the process.
        code: Option<i32>,
        /// Complete standard error text of the query.
        stderr: String,
    },
    /// A coordination file could not be read or published.
    Io(std::io::Error),
    /// The owner lock could not be acquired.
    Lock(OwnerLockError),
    /// A Git query was still running at its deadline and was stopped.
    TimedOut {
        /// Arguments of the query, for the diagnostic.
        arguments: String,
        /// The deadline it exceeded, in whole seconds.
        seconds: u64,
    },
}

impl std::fmt::Display for SingleFlightError {
    /// Writes the diagnostic text of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Start(error) => write!(formatter, "git could not be started: {error}"),
            Self::Git {
                arguments,
                code,
                stderr,
            } => write!(
                formatter,
                "git {arguments} exited with {code:?}: {}",
                stderr.trim_end()
            ),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Lock(error) => write!(formatter, "{error}"),
            Self::TimedOut { arguments, seconds } => write!(
                formatter,
                "git {arguments} did not finish within {seconds} seconds and was stopped; \
                 {PUSH_DEADLINE_VARIABLE} sets the limit"
            ),
        };
    }
}

impl std::error::Error for SingleFlightError {}

/// What:
///  One push this invocation ran itself.
/// Why:
///   `output` is Git's interleaved output with one trailing line break removed, as the incumbent's
///  `nano-spawn` `output` is; a successful push's output is never recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushAttempt = { exitCode: number; output: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushAttempt {
    /// Push exit code, `0` on success.
    pub exit_code: i64,
    /// Complete interleaved push output.
    pub output: String,
}

/// What:
///  How a landed commit's auto-push finished.
/// Why:
///   The four kinds keep "this invocation pushed" apart from "another invocation's push decided",
///  because only the owner of a failed push prints its full output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SingleFlightKind = 'pushed' | 'failed' | 'joined-pushed' | 'joined-failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SingleFlightKind {
    /// This invocation's push succeeded.
    Pushed,
    /// This invocation's push failed.
    Failed,
    /// Another invocation's successful push covers this commit.
    JoinedPushed,
    /// Another invocation's failed push covers this commit.
    JoinedFailed,
}

/// What:
///  How a landed commit's auto-push finished, with the deciding push's facts.
/// Why:
///   The caller prints the output and the uncovered note from these fields alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SingleFlightOutcome = { kind; tip; exitCode; output; covered };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SingleFlightOutcome {
    /// Which of the four outcomes this is.
    pub kind: SingleFlightKind,
    /// Branch tip the deciding push resolved.
    pub tip: String,
    /// Deciding push exit code.
    pub exit_code: i64,
    /// Complete output of the deciding push; empty for a joined success.
    pub output: String,
    /// Whether the deciding push's tip contains the landed commit.
    pub covered: bool,
}

/// What:
///  Paths of one branch's coordination files.
/// Why:
///   The lock and the record live beside each other under the common Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushCoordinationPaths = { lockDirectory: string; recordPath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushCoordinationPaths {
    /// Owner-lock directory.
    pub lock_directory: PathBuf,
    /// `last-pushed` record file.
    pub record_path: PathBuf,
}

/// What:
///  The real Git a coordinator runs, where, and with which environment.
/// Why:
///   The incumbent's coordination inherits the wrapper's whole environment plus the child-environment overlay,
///  so the caller passes that complete environment in (`environment_with_overlay`)
///  and nothing is read from the process environment here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushGit = { gitPath: string; cwd: string; environment: NodeJS.ProcessEnv };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushGit {
    /// Absolute real Git executable.
    pub real_git: PathBuf,
    /// Directory the commit landed in.
    pub cwd: PathBuf,
    /// Complete environment every Git child receives: the invocation's variables with the overlay applied.
    pub environment: Vec<(OsString, OsString)>,
}

/// What:
///  Build a real Git command with the given arguments, directory and environment.
/// Why:
///   Every coordination query and push starts Git this way, so none re-enters the wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nanoSpawn(gitPath, args, { cwd })
/// ```
pub fn git_command(git: &PushGit, arguments: &[OsString]) -> Command {
    let mut command = Command::new(&git.real_git);
    command
        .current_dir(&git.cwd)
        .env_clear()
        .envs(git.environment.iter().map(|pair| {
            return (&pair.0, &pair.1);
        }))
        .args(arguments);
    return command;
}

/// The arguments as one display string, for diagnostics.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// args.join(' ')
/// ```
fn describe_arguments(arguments: &[OsString]) -> String {
    return arguments
        .iter()
        .map(|argument| {
            return argument.to_string_lossy().into_owned();
        })
        .collect::<Vec<String>>()
        .join(" ");
}

/// What:
///  Run a read-only Git query and return its trimmed standard output.
///       `Result<A, B>` is either the output or a failure.
/// Why:
///   Any failed query is a coordination failure; it is never read as an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await nanoSpawn(gitPath, args, { cwd })).stdout.trim();
/// ```
pub fn git_output(git: &PushGit, arguments: &[OsString]) -> Result<String, SingleFlightError> {
    let output = run_query(git, arguments)?;
    if output.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: describe_arguments(arguments),
            code: output.code,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    let text = String::from_utf8_lossy(&output.stdout);
    return Ok(trim_javascript(&text).to_owned());
}

/// What:
///  Run a read-only Git query with its own deadline and capture its streams separately.
/// Why:
///   A query that never returns would hold the caller, and the push lock behind it, for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await nanoSpawn(gitPath, args, { cwd, signal: AbortSignal.timeout(deadlineMs) })
/// ```
pub fn run_query(git: &PushGit, arguments: &[OsString]) -> Result<QueryOutput, SingleFlightError> {
    return run_bounded_query(
        git_command(git, arguments),
        describe_arguments(arguments).as_str(),
        query_deadline(&git.environment),
        STOP_GRACE,
        git.environment.as_slice(),
    );
}

/// Argument list from string literals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['rev-parse', '--verify', ...]
/// ```
fn argv(parts: &[&str]) -> Vec<OsString> {
    return parts.iter().map(OsString::from).collect();
}

/// What:
///  Resolve one branch's coordination file paths under the common Git directory and create that directory.
/// Why:
///   Linked worktrees share one common directory, so every worktree of a repository coordinates on one lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolvePushCoordinationPaths({ gitPath, cwd, branchRef }): Promise<PushCoordinationPaths>;
/// ```
pub fn resolve_push_coordination_paths(
    git: &PushGit,
    branch_ref: &str,
) -> Result<PushCoordinationPaths, SingleFlightError> {
    let reported = git_output(
        git,
        &argv(&["rev-parse", "--path-format=absolute", "--git-common-dir"]),
    )?;
    let common = if Path::new(&reported).is_absolute() {
        PathBuf::from(&reported)
    } else {
        git.cwd.join(&reported)
    };
    let directory = common.join("cli-git").join("push");
    create_directory(&directory).map_err(SingleFlightError::Io)?;
    let key = encode_branch_key(branch_ref);
    return Ok(PushCoordinationPaths {
        lock_directory: directory.join(format!("{key}.lock")),
        record_path: directory.join(format!("{key}{LAST_PUSHED_SUFFIX}")),
    });
}

/// What:
///  Create a directory tree with mode `0700`, as the incumbent's `mkdir(..., { recursive, mode })`.
/// Why:
///   Coordination state is private to the account.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory, { recursive: true, mode: 0o700 });
/// ```
fn create_directory(directory: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    return std::fs::DirBuilder::new()
        .recursive(true)
        .mode(PRIVATE_DIRECTORY_MODE)
        .create(directory);
}

/// What:
///  Whether a commit is the given tip or one of its ancestors.
///       `Result<A, B>` is "answer or failure".
/// Why:
///   Only Git's "not an ancestor" status answers no; every other failure stops the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function tipContains({ gitPath, cwd, oid, tip }): Promise<boolean>; // throws
/// ```
pub fn tip_contains(git: &PushGit, oid: &str, tip: &str) -> Result<bool, SingleFlightError> {
    let output = run_query(git, &argv(&["merge-base", "--is-ancestor", oid, tip]))?;
    return match output.code {
        Some(0) => Ok(true),
        Some(NOT_ANCESTOR_EXIT_CODE) => Ok(false),
        code => Err(SingleFlightError::Git {
            arguments: format!("merge-base --is-ancestor {oid} {tip}"),
            code,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
    };
}

/// What:
///  The joined outcome of a record another invocation wrote.
/// Why:
///   A joined commit is covered by the record's tip by construction; the output is the record's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function joinedOutcome(record: LastPushedRecord): SingleFlightOutcome;
/// ```
fn joined_outcome(record: LastPushedRecord) -> SingleFlightOutcome {
    let kind = match record.outcome {
        LastPushedOutcome::Pushed => SingleFlightKind::JoinedPushed,
        LastPushedOutcome::Failed => SingleFlightKind::JoinedFailed,
    };
    return SingleFlightOutcome {
        kind,
        tip: record.tip,
        exit_code: record.exit_code,
        output: record.output,
        covered: true,
    };
}

/// What:
///  Adapts a closure to the owner lock's wait notice.
/// Why:
///   The lock module stays silent; this coordinator reports its own wait through the closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// onWait: () => void
/// ```
struct WaitObserver<'observer> {
    /// Called once when the first acquisition attempt finds the lock held.
    on_wait: &'observer mut dyn FnMut(),
}

impl WaitNotice for WaitObserver<'_> {
    /// Forwards the wait to the observer.
    fn waiting(&mut self) {
        (self.on_wait)();
    }
}

/// What:
///  Finish auto-push of one landed commit: join a push that covers it, or push the branch tip itself.
/// Why:
///   Three joins are possible, and each keeps the commit from pushing twice:
///  an early join on a recorded covering success, a join after waiting for the lock,
///  and the rule that a failed record joins only when a push finished after the first read.
///   The record is written before the lock is released, so the next waiter reads this attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runSingleFlightPush({ gitPath, cwd, branchRef, landedOid, push, onWait }): Promise<SingleFlightOutcome>;
/// ```
pub fn run_single_flight_push(
    git: &PushGit,
    branch_ref: &str,
    landed_oid: &str,
    push: &mut dyn FnMut() -> Result<PushAttempt, SingleFlightError>,
    on_wait: &mut dyn FnMut(),
) -> Result<SingleFlightOutcome, SingleFlightError> {
    let paths = resolve_push_coordination_paths(git, branch_ref)?;
    let before = read_record(&paths.record_path)?;
    let early = match &before {
        LastPushedRead::Record(record)
            if record.outcome == LastPushedOutcome::Pushed
                && tip_contains(git, landed_oid, &record.tip)? =>
        {
            Some(record.clone())
        }
        LastPushedRead::Record(_) | LastPushedRead::Absent => None,
    };
    if let Some(record) = early {
        debug(
            TAG,
            &format!(
                "{landed_oid} already delivered by the push of {}",
                record.tip
            ),
        );
        return Ok(joined_outcome(record));
    }
    let mut observer = WaitObserver { on_wait };
    // A live holder that outlives its own push deadline is stuck; waiting forever behind it would stall every commit.
    let lock: OwnerLock = acquire_owner_lock_for(
        &paths.lock_directory,
        DEFAULT_POLL_DELAY,
        &mut observer,
        lock_wait_limit(&git.environment),
    )
    .map_err(SingleFlightError::Lock)?;
    let after = read_record(&paths.record_path)?;
    let late = match &after {
        LastPushedRead::Record(record)
            if (record.outcome == LastPushedOutcome::Pushed
                || finished_while_waiting(&before, record))
                && tip_contains(git, landed_oid, &record.tip)? =>
        {
            Some(record.clone())
        }
        LastPushedRead::Record(_) | LastPushedRead::Absent => None,
    };
    if let Some(record) = late {
        debug(
            TAG,
            &format!(
                "{landed_oid} joined the {} push of {}",
                record.outcome.as_str(),
                record.tip
            ),
        );
        return Ok(joined_outcome(record));
    }
    let tip = git_output(
        git,
        &argv(&["rev-parse", "--verify", &format!("{branch_ref}^{{commit}}")]),
    )?;
    debug(
        TAG,
        &format!("pushing {branch_ref} at {tip} for {landed_oid}"),
    );
    let attempt = push()?;
    let pushed = attempt.exit_code == 0;
    let record = LastPushedRecord {
        tip: tip.clone(),
        outcome: if pushed {
            LastPushedOutcome::Pushed
        } else {
            LastPushedOutcome::Failed
        },
        owner_token: lock.token().to_owned(),
        owner_pid: i64::from(std::process::id()),
        exit_code: attempt.exit_code,
        output: if pushed {
            String::new()
        } else {
            attempt.output.clone()
        },
    };
    write_last_pushed_record(&paths.record_path, &record).map_err(SingleFlightError::Io)?;
    let covered = tip_contains(git, landed_oid, &tip)?;
    drop(lock);
    return Ok(SingleFlightOutcome {
        kind: if pushed {
            SingleFlightKind::Pushed
        } else {
            SingleFlightKind::Failed
        },
        tip,
        exit_code: attempt.exit_code,
        output: attempt.output,
        covered,
    });
}

/// Whether a push finished after the commit first read the record: the record's owner changed,
/// or the first read found nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (before === LAST_PUSHED_ABSENT) || (before.ownerToken !== after.ownerToken)
/// ```
fn finished_while_waiting(before: &LastPushedRead, after: &LastPushedRecord) -> bool {
    return match before {
        LastPushedRead::Absent => true,
        LastPushedRead::Record(earlier) => earlier.owner_token != after.owner_token,
    };
}

/// Read a record; an I/O failure other than absence stops the coordinator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readLastPushedRecord(recordPath)
/// ```
fn read_record(record_path: &Path) -> Result<LastPushedRead, SingleFlightError> {
    return read_last_pushed_record(record_path).map_err(SingleFlightError::Io);
}

/// Coordination behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_single_flight_tests.rs"]
mod tests;
