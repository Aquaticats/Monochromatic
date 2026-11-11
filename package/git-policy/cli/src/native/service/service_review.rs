//! What:
//!  The service's handler for a `review` work item:
//! read the landed commit,
//! ask the decision gateway,
//! and append the
//!  entry to the repository's live shadow log,
//! every step ending on no progress and none on the clock.
//! Why:
//!  This replaces the repository's post-commit hook that ran `mise run shadow-review` (`doc/runbook/standards-review-gate.md`).
//!  The review shares the after-commit work with pushes and upkeep,
//! so every step ends by itself:
//! the gateway call on the
//!  idle timeouts of its transport,
//! the Git reads under the watch of `child_watch.rs`;
//! the local write of the entry has no
//!  judge (pick 9 of `doc/planning/cli-git-shipped-limits-audit.md`,
//! the coordinator's under the owner's delegation and open to
//!  the owner's veto;
//! it replaces the 20 second budget around the whole run).
//! A review that fails leaves its item in
//!  `failed` as a plain record and the log shows the gateway's failure as an entry;
//! nothing is ever left claimed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await reviewLanded(names, sha, subject, read, options);
//! ```

use crate::standards_review_input::{ChangedPath, LandedCommit, has_shadow_log};
use crate::standards_review_run::{Containment, Contained, EnvironmentVariables, LandedCommitReader, ReviewOptions, ReviewOutcome, options_from_environment, review_landed, run_contained_in, run_contained_in_awaiting};
use crate::standards_review_source::{read_landed_commit, read_message, read_names};
use crate::standards_review_text::commit_subject;
use crate::service_review_key::{KEY_VARIABLES, ReviewKey};
use crate::service_work::Verdict;
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long the gateway call may stay silent before it is ended.
///  Citation:
/// none,
/// and **unmeasured as a quiet window**.
/// The figure was the call's total deadline,
/// "several times the largest
///  figure on record (1.4 s in `doc/audit/standards-review-gate-shadow.md`)";
/// that is a whole time,
/// and a whole time is an upper
///  bound of a quiet gap,
/// not the gap.
/// The gap of every call is now kept in the ledger (`quiet_ledger.rs`).
pub const CALL_QUIET_WINDOW: Duration = Duration::from_secs(10);

/// What:
///  The common Git directory of a worktree's Git directory:
/// the directory its `commondir` file names,
/// else the directory
///  itself.
/// Why:
///   A linked worktree's items carry that worktree's own Git directory (`.git/worktrees/<name>`),
/// which Git deletes with the
///  worktree,
/// while the repository's one log lives in the common directory.
/// Git strips every trailing carriage return and
///  line feed and resolves a relative path against the Git directory (`get_common_dir_noenv` in `path.c`),
/// as the layout
///  reader does;
/// a path that cannot be resolved is used as written,
/// so the entry still goes to the common directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync(join(gitDir, 'commondir')) ? realpath(resolve(gitDir, readFile(...).trimEnd())) : gitDir
/// ```
pub fn common_dir_of(git_dir: &Path) -> PathBuf {
    let Ok(text) = std::fs::read(git_dir.join("commondir")) else {
        return git_dir.to_path_buf();
    };
    // Everything up to the last byte that is not a line break; nothing when the file holds only line breaks.
    let mut end: usize = text.len();
    while end > 0 && (text[end - 1] == b'\r' || text[end - 1] == b'\n') {
        end -= 1;
    }
    if end == 0 {
        return git_dir.to_path_buf();
    }
    #[cfg(unix)]
    let named: PathBuf = PathBuf::from(<std::ffi::OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(&text[..end]));
    #[cfg(not(unix))]
    let named: PathBuf = PathBuf::from(String::from_utf8_lossy(&text[..end]).into_owned());
    let joined: PathBuf = git_dir.join(named);
    return std::fs::canonicalize(&joined).unwrap_or(joined);
}

/// What:
///  The options for one item:
/// the script's environment variables read from the service's environment,
/// and the call
///  deadline;
/// the log is the item's repository's live log in its common Git directory.
/// Why:
///   Every worktree of a repository appends to that repository's one log,
/// and a service that serves several repositories
///   writes each one's own;
/// the item names its worktree's Git directory,
/// and the common one follows from it with no Git start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// optionsFor(item, process.env)
/// ```
pub fn options_for(item: &WorkItem, variable: &dyn EnvironmentVariables) -> ReviewOptions {
    let common: PathBuf = common_dir_of(item.git_dir.as_path());
    let mut options: ReviewOptions = options_from_environment(common.as_path(), variable);
    options.quiet_window = CALL_QUIET_WINDOW;
    return options;
}

/// What:
///  The text of a review key.
/// Why:
///  `review_variables` keeps the key as text so the lookup it returns can hand it out again and again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exposeAsText = (key: ReviewKey) => key.expose();
/// ```
fn expose_as_text(held: &ReviewKey) -> String {
    return held.expose().to_string();
}

/// What:
///  The variable lookup the review's options read:
/// the credential variables answer from the key that came in the wake
///  message and from nowhere else,
/// and every other variable answers from the service's environment.
/// Why:
///   The credential is never in the service's environment or in a unit file,
/// so a service that did not receive a key in
///   the message must record "no key" and not pick up a stray value;
/// the service's environment carries no secret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (name) => keyNames.includes(name) ? fromMessage(name) : process.env[name]
/// ```
pub fn review_variables(key: Option<&ReviewKey>) -> ReviewVariables {
    return ReviewVariables { exposed: key.map(expose_as_text) };
}

/// The variable lookup of a review:
/// the exposed key answers for the credential variables,
/// every other name from the environment.
pub struct ReviewVariables {
    /// The key that came in the wake message,
    /// as text,
    /// or nothing.
    exposed: Option<String>,
}

impl EnvironmentVariables for ReviewVariables {
    fn variable(&self, name: &str) -> Option<String> {
        if name == KEY_VARIABLES[0] {
            return self.exposed.clone();
        }
        if name == KEY_VARIABLES[1] {
            return None;
        }
        return std::env::var(name).ok();
    }
}

/// What:
///  The read of the landed commit that the review asks for when the commit has code files.
/// Why:
///   The read needs the Git context,
/// the directory,
/// the commit and the subject,
/// which the review does not carry itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReadLandedCommit implements LandedCommitReader { read(files) { return readLandedCommit(context, cwd, oid, subject, files); } }
/// ```
struct ReadLandedCommit<'a> {
    /// The Git program and the directories the read uses.
    context: &'a GitContext,
    /// The repository the commit landed in.
    cwd: &'a Path,
    /// The commit.
    oid: &'a str,
    /// The commit's subject line.
    subject: &'a str,
}

impl LandedCommitReader for ReadLandedCommit<'_> {
    fn read(&self, files: &[ChangedPath]) -> Result<LandedCommit, String> {
        return read_landed_commit(self.context, self.cwd, self.oid, self.subject, files);
    }
}

/// The Git reads,
/// the subject and the review,
/// without the budget.
fn run_review(context: &GitContext, item: &WorkItem, options: &ReviewOptions, now_millis: &dyn Fn() -> u64) -> ReviewOutcome {
    let cwd: PathBuf = item.repository.clone();
    let names: Vec<ChangedPath> = match read_names(context, cwd.as_path(), item.oid.as_str()) {
        Ok(found) => found,
        Err(message) => return ReviewOutcome::Unread(message),
    };
    let subject: String = match read_message(context, cwd.as_path(), item.oid.as_str()) {
        Ok(message) => commit_subject(message.as_str()),
        Err(message) => return ReviewOutcome::Unread(message),
    };
    let read: ReadLandedCommit<'_> = ReadLandedCommit { context, cwd: cwd.as_path(), oid: item.oid.as_str(), subject: subject.as_str() };
    return review_landed(names.as_slice(), item.oid.as_str(), subject.as_str(), &read, options, now_millis);
}

/// The wall clock in milliseconds since 1970.
fn wall_clock_millis() -> u64 {
    return match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
        Err(_before_the_epoch) => 0,
    };
}

/// What:
///  Review one item:
/// `Done` when an entry was written,
/// was not needed,
/// or the worktree does not hold the tracked shadow log that
///  switches the review on;
/// `Failed` with the reason when the commit could not be read,
/// the log could not be written,
/// or the
///  review stopped with an internal error.
/// Why:
///   The caller settles `Failed` items into `failed`,
/// so a review that cannot finish is a plain record and never a stuck claim.
///  No budget covers the whole run;
/// each step ends on no progress by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reviewItem(context, item, options)
/// ```
pub fn review_item(context: &GitContext, item: &WorkItem, options: ReviewOptions) -> Verdict {
    return review_item_within(context, item, options, crate::wait_ceilings::REVIEW_CEILING);
}

/// `review_item` where the whole review is waited for at most `ceiling`;
/// a control shortens it.
pub fn review_item_within(context: &GitContext, item: &WorkItem, options: ReviewOptions, ceiling: Duration) -> Verdict {
    return review_item_in(&crate::standards_review_run::RUNNER, context, item, options, ceiling);
}

/// What:
///  `review_item_within` on a runner of the caller's own.
/// Why:
///   The look for the repository's shadow log is inside the contained job:
/// it reads the worktree,
/// which can stall like any other
///  read,
/// and nothing that can block runs in the after-commit work outside the ceiling (sol-2 round 2).
/// Controls give each its
///  own runner,
/// so they do not see one another's reviews.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runner.contain(() => hasShadowLog(repository) ? reviewLanded(...) : undefined, ceiling)
/// ```
pub fn review_item_in(runner: &'static Containment, context: &GitContext, item: &WorkItem, options: ReviewOptions, ceiling: Duration) -> Verdict {
    let owned_context: GitContext = GitContext { common_dir: Some(item.git_dir.clone()), ..context.clone() };
    let owned: WorkItem = item.clone();
    let outcome: Contained<Option<ReviewOutcome>> = run_contained_in(runner, move || {
        return run_the_review(owned_context, owned, options);
    }, ceiling);
    return verdict_of(outcome, item);
}

/// What:
///  `review_item`,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The round of the service's work is tasks
/// (`landing/service_round.rs`),
/// and the wait for a review is the runtime's timer there:
/// the job starts Git,
/// so the synchronous wait cannot be made from a task at all,
/// and a worker that blocked on it would hold the verifier it waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reviewItemAwaiting(context, item, options)
/// ```
pub async fn review_item_awaiting(context: &GitContext, item: &WorkItem, options: ReviewOptions) -> Verdict {
    return review_item_within_awaiting(context, item, options, crate::wait_ceilings::REVIEW_CEILING).await;
}

/// `review_item_awaiting` where the whole review is waited for at most `ceiling`;
/// a control shortens it.
pub async fn review_item_within_awaiting(context: &GitContext, item: &WorkItem, options: ReviewOptions, ceiling: Duration) -> Verdict {
    return review_item_in_awaiting(&crate::standards_review_run::RUNNER, context, item, options, ceiling).await;
}

/// What:
///  `review_item_in` on a runner of the caller's own,
/// its wait made on the runtime.
/// Why:
///   The job,
/// the look for the shadow log and the runner's admission are the synchronous face's;
/// only the wait for the job is the runtime's,
///  so the two cannot answer differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runner.containAwaiting(() => hasShadowLog(repository) ? reviewLanded(...) : undefined, ceiling)
/// ```
pub async fn review_item_in_awaiting(runner: &'static Containment, context: &GitContext, item: &WorkItem, options: ReviewOptions, ceiling: Duration) -> Verdict {
    let owned_context: GitContext = GitContext { common_dir: Some(item.git_dir.clone()), ..context.clone() };
    let owned: WorkItem = item.clone();
    let outcome: Contained<Option<ReviewOutcome>> = run_contained_in_awaiting(runner, move || {
        return run_the_review(owned_context, owned, options);
    }, ceiling)
    .await;
    return verdict_of(outcome, item);
}

/// What:
///  The job one review runs in its contained thread:
/// nothing when the repository keeps no shadow log,
/// else the review itself.
/// Why:
///   Both faces run the same job,
/// so a review cannot depend on which one asked for it.
/// The Git directory the input file needs is the item's,
/// because the service's own context names no repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const job = () => hasShadowLog(repository) ? reviewLanded(context, item, options, clock) : undefined
/// ```
fn run_the_review(context: GitContext, item: WorkItem, options: ReviewOptions) -> Option<ReviewOutcome> {
    if !has_shadow_log(item.repository.as_path()) {
        return None;
    }
    return Some(run_review(&context, &item, &options, &wall_clock_millis));
}

/// What:
///  What a contained review came to,
/// as the person's item is settled.
/// Why:
///   Both faces answer through this one function,
/// so the verdict of a review cannot depend on which face asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdictOf = (outcome, item, ceiling) => ...
/// ```
fn verdict_of(outcome: Contained<Option<ReviewOutcome>>, item: &WorkItem) -> Verdict {
    return match outcome {
        Contained::Panicked => Verdict::Failed(format!("the review of {} stopped with an internal error", item.oid)),
        Contained::NoThread(refused) => Verdict::Failed(format!("the review of {} was not started: {refused}", item.oid)),
        Contained::CeilingReached(reached) => Verdict::Failed(ceiling_reason(item.oid.as_str(), reached)),
        Contained::Busy => Verdict::Failed(busy_reason(item.oid.as_str())),
        Contained::Finished(None) => Verdict::Done,
        Contained::Finished(Some(ReviewOutcome::Unread(message) | ReviewOutcome::Unwritable(message))) => Verdict::Failed(message),
        Contained::Finished(Some(ReviewOutcome::NoEntry | ReviewOutcome::Skipped | ReviewOutcome::Reviewed(_) | ReviewOutcome::AlreadyLogged)) => Verdict::Done,
    };
}

/// What:
///  The reason of a review that was not started because an earlier one,
/// stopped at its ceiling,
/// is still running.
/// Why:
///   The earlier review may hold the review log,
/// and a second one would only queue behind it (sol-2,
/// B);
/// this one is refused with
///  the command that does it when the earlier one has ended,
/// and the service starts no further thread meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the standards review of ${oid} was not started because an earlier review is still running; run SHADOW_REVIEW_SHA=${oid} mise run shadow-review`
/// ```
pub fn busy_reason(oid: &str) -> String {
    return format!(
        "the standards review of {oid} was not started because an earlier review, stopped at its ceiling, is still running and may hold the \
         review log; review it again, once that review has ended, with `SHADOW_REVIEW_SHA={oid} mise run shadow-review`"
    );
}

/// What:
///  The reason of a review that ran as long as the service lets one run.
/// Why:
///   It names what was running (the standards review of the commit),
/// the ceiling,
/// and the command that does it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the standards review of ${oid} ran for ${seconds} seconds, ...; run SHADOW_REVIEW_SHA=${oid} mise run shadow-review`
/// ```
pub fn ceiling_reason(oid: &str, ceiling: Duration) -> String {
    return format!(
        "the standards review of {oid} ran for {} seconds, the longest the service lets one review run, and the service stopped waiting for it; \
         review it again with `SHADOW_REVIEW_SHA={oid} mise run shadow-review`",
        ceiling.as_secs()
    );
}

/// Review controls stay out of the release executable.
#[cfg(test)]
#[path = "service_review_tests.rs"]
mod tests;
