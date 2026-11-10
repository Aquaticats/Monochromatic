//! What:
//!  The commit service's push of a landed commit's branch:
//! which push runs,
//! what its output shows,
//!  and how a failure is reported without failing the commit.
//! Why:
//!  Every commit in this repository is backed up (`AGENTS.md` rule APG),
//!  the service pushes the branch its record names and never reads `HEAD`,
//!  prints only GitHub `remote:` lines on success and the full output on failure,
//!  and leaves a notice for the next command when the push fails.
//!  The service is the only pusher of the repositories it serves and coalesces pending pushes of one branch
//!  into one call,
//! so the push needs no lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await pushOnce({ gitPath, cwd }, branchRef, landedOid);
//! ```

/// Where a plain push of a named branch goes.
use crate::auto_push_destination::{NamedPush, named_push, named_push_awaiting};
/// The reconcile of a push that a moved remote branch rejected,
/// and the lines it adds to the output.
use crate::auto_push_reconcile::{RECONCILE_FAILED_PREFIX, RECONCILE_LINE_PREFIX, reconcile_divergent_push};
/// The same reconcile,
/// awaited by a caller that is itself a task of the runtime.
use crate::auto_push_reconcile_awaiting::reconcile_divergent_push_awaiting;
/// The service's Git,
/// the failed attempt it reports and its error type.
use crate::auto_push_single_flight::{PushAttempt, PushGit, SingleFlightError, run_query, run_query_awaiting};
/// The auto-push's bounded `git push` with the Git LFS upload.
use crate::lfs_bounded_push::{run_push, run_push_awaiting};
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// Owned OS strings for Git arguments.
use std::ffi::OsString;
/// `Write` is the stderr sink the terminal lines go to.
use std::io::Write;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// Prefix of the GitHub server-side lines a clean push surfaces.
const REMOTE_LINE_PREFIX: &str = "remote: ";

/// Note printed after every failed push:
/// the commit stays local.
pub const PUSH_FAILED_NOTE: &str =
    "monochromatic-git-policy-cli: commit saved locally, but auto-push to origin failed; run `git push` when ready.";

/// What Git prints when another process holds the lock of a reference the push updates.
const REFERENCE_LOCK_REFUSAL: &str = "cannot lock ref";

/// How many times the service pushes again when a held reference lock refused the update of a remote-tracking reference.
pub const PUSH_LOCK_RETRIES: u32 = 3;

/// The pause before the first of those pushes;
/// each later one waits a multiple of it.
const PUSH_LOCK_PAUSE: std::time::Duration = std::time::Duration::from_millis(250);

/// What:
///  Which branch the auto-push took.
/// Why:
///   `Skipped` covers both "nowhere to back up to" and a setting under which Git pushes the branch nowhere;
/// the first
///  exits 0 silently,
/// the second with a note that the person must see.
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
///   `shown` is what the terminal received,
/// so tests compare the surfaced text directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AutoPushResult = { outcome; exitCode; shown };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoPushResult {
    /// Which branch the auto-push took.
    pub outcome: AutoPushOutcome,
    /// Push exit code;
    /// `0` when skipped or pushed,
    /// Git's code (or `1`) on failure.
    pub exit_code: i64,
    /// Text surfaced to stderr;
    /// empty on a silent skip or a quiet clean push,
    /// the note on a skip with one.
    pub shown: String,
}

/// What:
///  Select the output a push shows:
/// only `remote:` lines after a clean push,
/// everything after a failure,
/// and the refusal of a reference lock in either case.
/// Why:
///   A rejection,
/// a hook refusal or an offline error stays diagnosable only when the full output shows.
///   A push whose remote-tracking reference Git could not update exits `0` because the commits were delivered,
/// so the refusal
///  is the only sign the local view is stale:
/// without it `git status` says the branch is ahead of a remote that already has the commits,
/// and nothing says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filterPushOutput({ output, exitCode }): string;
/// ```
pub fn filter_push_output(output: &str, exit_code: i64) -> String {
    if exit_code != 0 {
        return output.to_owned();
    }
    let mut kept: Vec<&str> = Vec::new();
    let refused: bool = output.contains(REFERENCE_LOCK_REFUSAL);
    for line in output.split('\n') {
        let about_the_refusal: bool = refused && line.contains(REFERENCE_LOCK_REFUSAL);
        if line.starts_with(REMOTE_LINE_PREFIX) || line.starts_with(RECONCILE_LINE_PREFIX) || about_the_refusal {
            kept.push(line);
        }
    }
    return kept.join("\n");
}

/// What:
///  Whether a read-only Git query exits zero.
/// Why:
///   A non-zero exit is an answer ("no upstream",
/// "no origin");
/// a failure to start Git is not an answer.
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
///  Run one real `git push` and capture its interleaved output,
/// with one trailing line break removed,
///  stopping it when its deadline passes.
/// Why:
///   Git's output of stdout and stderr shares one file so the lines interleave in the order they were written,
///  the trailing line break is removed as the incumbent's captured output is,
///  and a push that stalls must not hold the service's push of the branch forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { exitCode, output } = await nanoSpawn(gitPath, pushArgs, { cwd }) // SubprocessError -> { exitCode ?? 1, output }
/// ```
fn run_push_command(git: &PushGit, arguments: &[&str]) -> Result<PushAttempt, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return run_push(git, &argv);
}

/// What a push plan decided before anything was pushed.
enum Planned {
    /// Nothing to push;
    /// the result says why.
    Skip(AutoPushResult),
    /// Push the branch with these arguments.
    Push {
        /// The branch HEAD names.
        branch_ref: String,
        /// Whether the branch has an upstream,
        /// which makes a moved remote branch reconcilable.
        has_upstream: bool,
        /// The `git push` arguments.
        arguments: Vec<String>,
    },
}

/// What:
///  The plan that skips a branch with a note,
/// after writing the note to the terminal sink.
/// Why:
///   The note is the only diagnostic of a push that never ran,
/// so the result carries it as well,
/// for the caller that
///  reports it elsewhere than the sink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skippedWith(note: string, err: Writable): Planned;
/// ```
fn skipped_with(note: String, err: &mut dyn Write) -> Planned {
    // Terminal output error: the note itself is the only diagnostic; a failed write to stderr is not a push failure.
    let _written = writeln!(err, "{note}");
    return Planned::Skip(AutoPushResult { outcome: AutoPushOutcome::Skipped, exit_code: 0, shown: note });
}

/// What:
///  Decide how to push a recorded branch whatever the worktree has checked out:
/// no upstream and no origin skips,
/// otherwise
///  the push names the branch on both sides and never reads `HEAD`.
/// Why:
///   The service pushes after the command returned,
/// so the person may switch branches or detach the head at any instant,
///  including between a check and the push;
/// the record names the branch that received the commit,
/// and the plain
///  `git push` would publish the checked-out one instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = await planNamedPush({ gitPath, cwd }, branchRef);
/// ```
fn plan_named_push(git: &PushGit, branch_ref: &str, err: &mut dyn Write) -> Result<Planned, SingleFlightError> {
    // Git reads `refs/heads/x@{upstream}` as a branch named `refs/heads/x` ("no such branch"), so the query takes the short name;
    //  a ref outside `refs/heads` has no upstream.
    let has_upstream: bool = match branch_ref.strip_prefix("refs/heads/") {
        Some(name) => {
            let upstream_name: String = format!("{name}@{{upstream}}");
            query_succeeds(git, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", upstream_name.as_str()])?
        }
        None => false,
    };
    if !has_upstream && !query_succeeds(git, &["remote", "get-url", "origin"])? {
        debug(TAG, "no upstream and no origin remote; skipping auto-push");
        return Ok(Planned::Skip(AutoPushResult { outcome: AutoPushOutcome::Skipped, exit_code: 0, shown: String::new() }));
    }
    let arguments: Vec<String> = if has_upstream {
        let (remote, targets): (String, Vec<String>) = match named_push(git, branch_ref)? {
            NamedPush::Push { remote, targets } => (remote, targets),
            NamedPush::Matching { remote } => {
                // Git matches the branches itself: the plain push of the remote, with no refspec.
                return Ok(Planned::Push { branch_ref: branch_ref.to_owned(), has_upstream, arguments: vec![String::from("push"), remote] });
            }
            NamedPush::Skip(note) => return Ok(skipped_with(note, err)),
        };
        let mut arguments: Vec<String> = vec![String::from("push"), remote];
        for target in &targets {
            arguments.push(format!("{branch_ref}:{target}"));
        }
        arguments
    } else {
        vec![String::from("push"), String::from("--set-upstream"), String::from("origin"), format!("{branch_ref}:{branch_ref}")]
    };
    return Ok(Planned::Push { branch_ref: branch_ref.to_owned(), has_upstream, arguments });
}

/// What:
///  Push a landed commit's branch once,
/// with no coordination with other processes.
/// Why:
///   The service's push consumer is the only pusher of the repositories it serves and coalesces pending pushes of one
///  branch into this single call,
/// so it needs no lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = await pushOnce({ gitPath, cwd }, landedOid);
/// ```
pub fn push_once(git: &PushGit, branch_ref: &str, landed_oid: &str, err: &mut dyn Write) -> Result<AutoPushResult, SingleFlightError> {
    // The record names the branch, and the worktree may switch away from it at any instant, including between a check of
    //  `HEAD` and the push itself: the plan never reads `HEAD`, so what is pushed is always the recorded branch.
    let planned: Planned = plan_named_push(git, branch_ref, err)?;
    let (pushed_ref, has_upstream, mut attempt): (String, bool, PushAttempt) = match planned {
        Planned::Skip(result) => return Ok(result),
        Planned::Push { branch_ref: planned_ref, has_upstream, arguments } => {
            let argument_views: Vec<&str> = arguments.iter().map(String::as_str).collect();
            debug(TAG, &format!("pushing {landed_oid} with {argument_views:?} on {planned_ref}"));
            let mut attempt: PushAttempt = run_push_command(git, argument_views.as_slice())?;
            // The instances of two worktrees share the repository's references. While another holds the lock of the remote-tracking
            //  reference this push updates, Git still delivers the push, prints the refusal to update the reference, and exits 0, which
            //  leaves the reference stale; the same push again finds nothing to deliver and brings the reference up to date.
            let mut pause: u32 = 0;
            while attempt.output.contains(REFERENCE_LOCK_REFUSAL) && pause < PUSH_LOCK_RETRIES {
                pause += 1;
                std::thread::sleep(PUSH_LOCK_PAUSE * pause);
                attempt = run_push_command(git, argument_views.as_slice())?;
            }
            (planned_ref, has_upstream, attempt)
        }
    };
    if has_upstream && attempt.exit_code != 0 {
        attempt = reconcile_divergent_push(git, &pushed_ref, attempt)?;
    }
    let failed: bool = attempt.exit_code != 0;
    let shown: String = filter_push_output(&attempt.output, attempt.exit_code);
    if !shown.is_empty() {
        writeln!(err, "{shown}").map_err(SingleFlightError::Io)?;
    }
    if failed && !attempt.output.contains(RECONCILE_FAILED_PREFIX) {
        writeln!(err, "{PUSH_FAILED_NOTE}").map_err(SingleFlightError::Io)?;
    }
    return Ok(AutoPushResult {
        outcome: if failed { AutoPushOutcome::Failed } else { AutoPushOutcome::Pushed },
        exit_code: attempt.exit_code,
        shown,
    });
}

/// What:
///  Whether a read-only Git query exits zero,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push asks this as a task,
/// and the synchronous query enters the runtime and is refused from there.
/// The question and the reading of the exit status are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await gitQuerySucceedsOnTheRuntime({ gitPath, cwd, args });
/// ```
async fn query_succeeds_awaiting(git: &PushGit, arguments: &[&str]) -> Result<bool, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return Ok(run_query_awaiting(git, &argv).await?.code == Some(0));
}

/// What:
///  The same plan of a push,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The plan asks Git twice and reads the destination's configuration,
/// and every one of those starts is refused from inside the runtime.
/// The questions and the arguments are the synchronous face's,
/// so the two plan the same push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = await planNamedPushOnTheRuntime({ gitPath, cwd }, branchRef);
/// ```
async fn plan_named_push_awaiting(
    git: &PushGit,
    branch_ref: &str,
    err: &mut (dyn Write + Send),
) -> Result<Planned, SingleFlightError> {
    // Git reads `refs/heads/x@{upstream}` as a branch named `refs/heads/x` ("no such branch"), so the query takes the short name;
    //  a ref outside `refs/heads` has no upstream.
    let has_upstream: bool = match branch_ref.strip_prefix("refs/heads/") {
        Some(name) => {
            let upstream_name: String = format!("{name}@{{upstream}}");
            query_succeeds_awaiting(git, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", upstream_name.as_str()]).await?
        }
        None => false,
    };
    if !has_upstream && !query_succeeds_awaiting(git, &["remote", "get-url", "origin"]).await? {
        debug(TAG, "no upstream and no origin remote; skipping auto-push");
        return Ok(Planned::Skip(AutoPushResult { outcome: AutoPushOutcome::Skipped, exit_code: 0, shown: String::new() }));
    }
    let arguments: Vec<String> = if has_upstream {
        let (remote, targets): (String, Vec<String>) = match named_push_awaiting(git, branch_ref).await? {
            NamedPush::Push { remote, targets } => (remote, targets),
            NamedPush::Matching { remote } => {
                // Git matches the branches itself: the plain push of the remote, with no refspec.
                return Ok(Planned::Push { branch_ref: branch_ref.to_owned(), has_upstream, arguments: vec![String::from("push"), remote] });
            }
            NamedPush::Skip(note) => return Ok(skipped_with(note, err)),
        };
        let mut arguments: Vec<String> = vec![String::from("push"), remote];
        for target in &targets {
            arguments.push(format!("{branch_ref}:{target}"));
        }
        arguments
    } else {
        vec![String::from("push"), String::from("--set-upstream"), String::from("origin"), format!("{branch_ref}:{branch_ref}")]
    };
    return Ok(Planned::Push { branch_ref: branch_ref.to_owned(), has_upstream, arguments });
}

/// What:
///  Push a landed commit's branch once,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The push of the landing round is a task,
/// and every step below it starts Git,
/// which the synchronous face enters the runtime for and is refused from there.
/// The plan,
/// the held-reference retry,
/// the reconcile and the words the push shows are the synchronous face's,
/// so a push and its retry cannot differ between the two.
///  The pause between the retries is the runtime's own,
/// so no worker is held while the other worktree's lock is waited out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = await pushOnceOnTheRuntime({ gitPath, cwd }, branchRef, landedOid);
/// ```
pub async fn push_once_awaiting(
    git: &PushGit,
    branch_ref: &str,
    landed_oid: &str,
    err: &mut (dyn Write + Send),
) -> Result<AutoPushResult, SingleFlightError> {
    let planned: Planned = plan_named_push_awaiting(git, branch_ref, err).await?;
    let (pushed_ref, has_upstream, mut attempt): (String, bool, PushAttempt) = match planned {
        Planned::Skip(result) => return Ok(result),
        Planned::Push { branch_ref: planned_ref, has_upstream, arguments } => {
            let argument_views: Vec<&str> = arguments.iter().map(String::as_str).collect();
            debug(TAG, &format!("pushing {landed_oid} with {argument_views:?} on {planned_ref}"));
            let mut attempt: PushAttempt = run_push_command_awaiting(git, argument_views.as_slice()).await?;
            // The instances of two worktrees share the repository's references. While another holds the lock of the remote-tracking
            //  reference this push updates, Git still delivers the push, prints the refusal to update the reference, and exits 0, which
            //  leaves the reference stale; the same push again finds nothing to deliver and brings the reference up to date.
            let mut pause: u32 = 0;
            while attempt.output.contains(REFERENCE_LOCK_REFUSAL) && pause < PUSH_LOCK_RETRIES {
                pause += 1;
                tokio::time::sleep(PUSH_LOCK_PAUSE * pause).await;
                attempt = run_push_command_awaiting(git, argument_views.as_slice()).await?;
            }
            (planned_ref, has_upstream, attempt)
        }
    };
    if has_upstream && attempt.exit_code != 0 {
        attempt = reconcile_divergent_push_awaiting(git, &pushed_ref, attempt).await?;
    }
    let failed: bool = attempt.exit_code != 0;
    let shown: String = filter_push_output(&attempt.output, attempt.exit_code);
    if !shown.is_empty() {
        writeln!(err, "{shown}").map_err(SingleFlightError::Io)?;
    }
    if failed && !attempt.output.contains(RECONCILE_FAILED_PREFIX) {
        writeln!(err, "{PUSH_FAILED_NOTE}").map_err(SingleFlightError::Io)?;
    }
    return Ok(AutoPushResult {
        outcome: if failed { AutoPushOutcome::Failed } else { AutoPushOutcome::Pushed },
        exit_code: attempt.exit_code,
        shown,
    });
}

/// What:
///  Run one real `git push` and capture its interleaved output,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The awaited push is the synchronous one with its wait on the runtime,
/// so the arguments and the captured output are the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const attempt = await runPushAwaiting(git, pushArgs);
/// ```
async fn run_push_command_awaiting(git: &PushGit, arguments: &[&str]) -> Result<PushAttempt, SingleFlightError> {
    let argv: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    return run_push_awaiting(git, &argv).await;
}

/// Auto-push behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_tests.rs"]
mod tests;

/// Controls that the service never pushes where a plain `git push` of the branch would not.
#[cfg(test)]
#[path = "auto_push_semantics_tests.rs"]
mod semantics_tests;
