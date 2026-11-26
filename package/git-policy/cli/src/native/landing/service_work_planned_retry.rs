//! What:
//!  The words and the plan of a push that is tried again:
//!  the sentence the person's record carries (`PlannedRetry`),
//!  the journal line that says why the service is quiet,
//!  and the plan on disk that makes the sentence true.
//! Why:
//!  The promise a record makes is the plan's,
//!  so the plan is written before those words exist,
//!  and a plan that cannot be kept leaves the record with Git's own words
//!  alone;
//!  one function is the only place a retry is planned,
//!  so the round and the steps of a repository that is gone or unreachable
//!  word it the same way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said: PlannedRetry = await plannedRetry(gitDir, replanning, first, item, attempt, text);
//! ```

use super::service_work_drained_and_replanning::Replanning;
use crate::journald_native::Priority;
use crate::push_retry::{
    RetryChange,
    pause_before,
};
use crate::push_retry_cause_line::cause_or_silence;
use crate::push_retry_failure::{
    PROTECTED_BRANCH_TRIES,
    protected_branch_refusal,
};
use crate::service_timing::log_entry;
use crate::work_item::WorkItem;
use std::path::Path;
use std::time::{
    Duration,
    Instant,
};

/// What:
///  The words a record of a planned retry carries,
/// and whether the plan they promise is kept.
/// Why:
///   The promise and the plan are one fact:
/// main's words say the service keeps trying only when the plan could be kept,
///  and the notice's framing needs the same answer as a value rather than as the presence of a sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlannedRetry = { words: string; kept: boolean };
/// ```
pub(crate) struct PlannedRetry {
    /// The words for the person's record,
    /// with the promise only when the plan is kept.
    pub(crate) words: String,
    /// Whether the plan the words promise exists.
    pub(crate) kept: bool,
}

/// What:
///  Plan the next try of a push that failed for a reason that can pass,
/// say so in the journal,
/// and give the words the person's
///  failure record carries.
/// Why:
///   The journal line names the cause,
/// which is the first of Git's own lines that gives a reason rather than
///  the remote it pushed to (`cause_line`),
/// the number of the try that is planned and the pause before it,
/// so the service's quiet
///  between tries is explained;
/// the person's record says the service keeps trying,
/// so they do not push by hand for nothing,
/// and for a
///  cause the service gives a fixed number of tries it says how many there are.
///  The plan is asked of the service loop (`Replanning`),
/// which owns it,
/// and it is written before those words exist:
/// the promise a record makes
///  is the plan's,
/// so a plan that cannot be kept leaves the record with Git's own words alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = await plannedRetry(gitDir, replanning, first, item, attempt, text);
/// ```
pub(crate) async fn planned_retry(
    git_dir: &Path,
    replanning: &mut Replanning,
    first: Duration,
    item: &WorkItem,
    attempt: u32,
    text: &str,
) -> PlannedRetry {
    let pause: Duration = pause_before(attempt, first);
    // The notice below says the service keeps trying, so the plan that keeps it is written first,
    // and its answer says whether the words may promise it.
    let kept: bool = replanning.keep_the_plan(git_dir, item, attempt).await;
    replanning.changes.push(RetryChange::Plan {
        git_dir: git_dir.to_path_buf(),
        item: item.clone(),
        attempt,
        due: Instant::now() + pause,
    });
    // A protected-branch refusal is retried a fixed number of times, so a record that promised tries until the push lands would be
    //  untrue for the one cause that stops on its own; its words name the last try instead.
    let plan: String = match protected_branch_refusal(text) {
        Some(_) => format!(
            "monochromatic-git-policy-cli: the commit service tries this push again by itself in {} seconds (try {} of {}). The server's refusal of an update to a protected branch can clear by itself; if the server refuses try {} as well, the service stops trying this branch and tells you what the server said.",
            pause.as_secs(),
            attempt + 1,
            PROTECTED_BRANCH_TRIES,
            PROTECTED_BRANCH_TRIES
        ),
        None => format!(
            "monochromatic-git-policy-cli: the commit service tries this push again by itself in {} seconds (try {}), and keeps trying with longer pauses until the push lands or a conflict needs you.",
            pause.as_secs(),
            attempt + 1
        ),
    };
    let fields: [(&str, String); 4] = [
        ("CLI_GIT_BRANCH", item.branch_ref.clone()),
        ("CLI_GIT_OID", item.oid.clone()),
        ("CLI_GIT_OUTCOME", String::from("retrying")),
        (
            "CLI_GIT_REPOSITORY",
            item.repository.to_string_lossy().into_owned(),
        ),
    ];
    // The line names the repository: one journal covers every worktree of the machine, and a branch name is the same in all of
    //  them, so a retry line without it cannot say which worktree's push it plans.
    log_entry(
        Priority::Warning,
        format!(
            "push will be tried again branch={:?} oid={} repository={:?} next_try={} in_seconds={} cause: {}",
            item.branch_ref,
            item.oid,
            item.repository,
            attempt + 1,
            pause.as_secs(),
            cause_or_silence(text)
        )
        .as_str(),
        &fields,
    );
    // A plan that could not be kept is not promised: the service of this run still tries the push, and a service started in
    //  its place would not know it, so the person is left with Git's own words and pushing by hand.
    let words: String = if kept {
        format!("{plan}\n{text}")
    } else {
        text.to_string()
    };
    return PlannedRetry {
        words,
        kept,
    };
}
