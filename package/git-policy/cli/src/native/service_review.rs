//! What:
//!  The service's handler for a `review` work item: read the landed commit, ask the decision gateway, and append the
//!  entry to the worktree's shadow log, all inside a fixed time budget.
//! Why:
//!  This replaces the repository's post-commit hook that ran `mise run shadow-review` (`doc/runbook/standards-review-gate.md`).
//!  The review shares the service's work thread with pushes and upkeep, so every step is bounded: the gateway call by a
//!  deadline, the Git reads and the write by the budget around the whole run. A review that fails leaves its item in
//!  `failed` as a plain record and the log shows the gateway's failure as an entry; nothing is ever left claimed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await Promise.race([reviewLanded(names, sha, subject, read, options), wait(budget)]);
//! ```

use super::standards_review_append::spool_directory;
use super::standards_review_input::has_shadow_log;
use super::standards_review_run::{ReviewOptions, ReviewOutcome, options_from_environment, review_landed, run_within};
use super::standards_review_source::{read_landed_commit, read_message, read_names};
use super::standards_review_text::commit_subject;
use super::service_review_key::{KEY_VARIABLES, ReviewKey};
use super::service_work::Verdict;
use super::transaction_git::GitContext;
use super::work_item::WorkItem;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long the gateway call may take: several times the largest figure on record (1.4 s in
/// `doc/audit/standards-review-gate-shadow.md`), and a third of the script's 30 s, because the work thread is shared.
pub const CALL_DEADLINE: Duration = Duration::from_secs(10);

/// How long the whole review may take, Git reads and write included; past it the item is failed and the work thread moves on.
pub const BUDGET: Duration = Duration::from_secs(20);

/// What:
///  The options for one item: the script's environment variables read from the service's environment, the call deadline,
///  and the spool beside the item's Git directory.
/// Why:
///   The log is the one under the item's own worktree, so a service that serves several repositories writes each one's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// optionsFor(item, process.env)
/// ```
pub fn options_for(item: &WorkItem, variable: &dyn Fn(&str) -> Option<String>) -> ReviewOptions {
    let mut options: ReviewOptions = options_from_environment(Path::new(item.repository.as_str()), variable);
    options.deadline = CALL_DEADLINE;
    options.spool = Some(spool_directory(Path::new(item.git_dir.as_str()), item.repository.as_str()));
    return options;
}

/// What:
///  The variable lookup the review's options read: the credential variables answer from the key that came in the wake
///  message and from nowhere else, and every other variable answers from the service's environment.
/// Why:
///   The credential is never in the service's environment or in a unit file, so a service that did not receive a key in
///   the message must record "no key" and not pick up a stray value; the service's environment carries no secret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (name) => keyNames.includes(name) ? fromMessage(name) : process.env[name]
/// ```
pub fn review_variables(key: Option<&ReviewKey>) -> impl Fn(&str) -> Option<String> {
    let exposed: Option<String> = key.map(|held| { return held.expose().to_string(); });
    return move |name: &str| {
        if name == KEY_VARIABLES[0] {
            return exposed.clone();
        }
        if name == KEY_VARIABLES[1] {
            return None;
        }
        return std::env::var(name).ok();
    };
}

/// The Git reads, the subject and the review, without the budget.
fn run_review(context: &GitContext, item: &WorkItem, options: &ReviewOptions, now_millis: &dyn Fn() -> u64) -> ReviewOutcome {
    let cwd: PathBuf = PathBuf::from(item.repository.as_str());
    let names: Vec<String> = match read_names(context, cwd.as_path(), item.oid.as_str()) {
        Ok(found) => found,
        Err(message) => return ReviewOutcome::Unread(message),
    };
    let subject: String = match read_message(context, cwd.as_path(), item.oid.as_str()) {
        Ok(message) => commit_subject(message.as_str()),
        Err(message) => return ReviewOutcome::Unread(message),
    };
    let read = |files: &[String]| { return read_landed_commit(context, cwd.as_path(), item.oid.as_str(), subject.as_str(), files); };
    return review_landed(names.as_slice(), item.oid.as_str(), subject.as_str(), &read, options, now_millis);
}

/// The wall clock in milliseconds since 1970.
fn wall_clock_millis() -> u64 {
    return std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| { return u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX); });
}

/// What:
///  Review one item within `budget`: `Done` when an entry was written, was not needed, or the repository has no shadow log;
///  `Failed` with the reason when the commit could not be read, the log could not be written, or the budget ran out.
/// Why:
///   The caller settles `Failed` items into `failed`, so a review that cannot finish is a plain record and never a stuck claim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reviewItem(context, item, options, budget)
/// ```
pub fn review_item(context: &GitContext, item: &WorkItem, options: ReviewOptions, budget: Duration) -> Verdict {
    if !has_shadow_log(Path::new(item.repository.as_str())) {
        return Verdict::Done;
    }
    // The service serves many repositories, so its own context names none; the Git command that takes input needs the
    // item's Git directory for its input file, as the landing sets it for each request.
    let owned_context: GitContext = GitContext { common_dir: Some(PathBuf::from(item.git_dir.as_str())), ..context.clone() };
    let owned: WorkItem = item.clone();
    let outcome: Option<ReviewOutcome> = run_within(budget, move || { return run_review(&owned_context, &owned, &options, &wall_clock_millis); });
    return match outcome {
        None => Verdict::Failed(format!("the review of {} did not finish within {} s", item.oid, budget.as_secs())),
        Some(ReviewOutcome::Unread(message) | ReviewOutcome::Unwritable(message)) => Verdict::Failed(message),
        Some(ReviewOutcome::NoEntry | ReviewOutcome::Skipped | ReviewOutcome::Reviewed(_) | ReviewOutcome::AlreadyLogged) => Verdict::Done,
    };
}

/// Review controls stay out of the release executable.
#[cfg(test)]
#[path = "service_review_tests.rs"]
mod tests;
