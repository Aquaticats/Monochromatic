//! What:
//!  The face a task of the runtime asks of the same removals:
//! every Git child awaited on the runtime,
//! and the read of a batch's journal record a job of the blocking pool awaited under a ceiling.
//! Why:
//!  A task cannot enter a step that waits for a child or a file
//! (`transaction_git_awaiting.rs`),
//! and a landing that becomes a task does this through this face;
//! its steps,
//! the three answers of a reference that is not there,
//! names another commit or gives no answer,
//! and the words each of them says are the synchronous face's
//! (`service_scratch_ref_of_a_dropped_batch.rs`).
//!  A look at the intent that gave no answer leaves the reference where it is and says why,
//! so no commit is lost and nothing claims to have removed what it never read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removal = await clearAfterAFailedLandingAwaiting(context, requests, seq)
//! ```

use crate::blocking_jobs_within_the_budget::{
    TheAnswerOfAJobAwaited,
    run_a_job_awaited_no_longer_than,
};
use crate::service_journal::{
    OPEN_EXTENSION,
    batch_path,
};
use crate::service_land_context::LandContext;
use crate::service_protocol::CommitRequest;
/// The synchronous reader,
/// which the controls here compare this one with.
#[cfg(test)]
use crate::service_scratch_ref_names_and_values::value_of;
use crate::service_scratch_ref_names_and_values::{
    RefValue,
    ScratchRefRemoval,
    kept_ref,
    left,
    scratch_ref,
    value_of_awaiting,
    write_ref_awaiting,
};
/// The synchronous face of each removal,
/// which the controls here compare this one with.
#[cfg(test)]
use crate::service_scratch_ref_of_a_dropped_batch::{
    clear_after_a_failed_landing,
    clear_after_dropped_batch,
};
use crate::transaction_git::{
    GitContext,
    NO_REFLOG_SETTING,
};
use crate::wait_ceilings::LANDING_STATE_CEILING;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// What:
///  `clear_after_dropped_batch` with every Git step of it awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and the removal is a step of the
///  landing once that landing is a task.
/// The order of the steps,
/// the three answers of a reference that is not there,
/// names another commit or gives no answer,
/// and the words each of them says are the synchronous face's,
/// pinned by
///  `the_awaited_clearing_answers_what_the_synchronous_one_answers`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = await clearAfterDroppedBatchAwaiting(git, repository, seq)
/// ```
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"
    )
)]
pub(crate) async fn clear_after_dropped_batch_awaiting(
    git: &GitContext,
    repository: &Path,
    seq: u64,
) -> ScratchRefRemoval {
    let scratch: String = scratch_ref(seq);
    let kept: String = kept_ref(seq);
    let tip: String = match value_of_awaiting(git, repository, scratch.as_str()).await {
        RefValue::Names(found) => found,
        // No scratch reference at all: the landing made no objects, or its branch write removed it.
        RefValue::Absent => return ScratchRefRemoval::Removed,
        // A symbolic reference is not one of the wrapper's writes, so what it holds is not proven to be the batch's commit: nothing
        //  is removed on it (`RefValue::Symbolic`).
        RefValue::Symbolic(target) => {
            left(seq, repository, format!("{scratch} is a symbolic reference to {target}, not a reference of the wrapper's own, so nothing is removed").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Unasked => {
            left(
                seq,
                repository,
                format!("{scratch} cannot be read, so nothing is removed").as_str(),
            );
            return ScratchRefRemoval::Left;
        }
    };
    let already: RefValue = value_of_awaiting(git, repository, kept.as_str()).await;
    match &already {
        RefValue::Names(named) if *named == tip => {}
        RefValue::Names(named) => {
            left(seq, repository, format!("{kept} names {named} and the scratch reference {scratch} names {tip}, so {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        // The commit is held by the reference this one leads to, which its owner can move: the batch's commit is not proven to be
        //  safe elsewhere, so the scratch reference stays.
        RefValue::Symbolic(target) => {
            left(seq, repository, format!("{kept} is a symbolic reference to {target} and the scratch reference {scratch} names {tip}, so {scratch} is left as it is").as_str());
            return ScratchRefRemoval::Left;
        }
        RefValue::Unasked => {
            left(
                seq,
                repository,
                format!(
                    "{kept} cannot be read, so the scratch reference {scratch} is left as it is"
                )
                .as_str(),
            );
            return ScratchRefRemoval::Left;
        }
        RefValue::Absent => {
            // The empty old value asks Git for a write of a name that names nothing: a reference another command writes between this
            //  read and the write is never replaced (`keep_on_a_side_ref`, the owner's rule "never discard user data").
            if !write_ref_awaiting(
                git,
                repository,
                &[
                    "-c",
                    NO_REFLOG_SETTING,
                    "update-ref",
                    "--no-deref",
                    kept.as_str(),
                    tip.as_str(),
                    "",
                ],
            )
            .await
            {
                left(
                    seq,
                    repository,
                    format!("the commit {tip} of a batch that did not land could not be kept at {kept}, so its scratch reference {scratch} is left as it is").as_str(),
                );
                return ScratchRefRemoval::Left;
            }
            crate::service_log!(
                "batch {} {}: the commit {tip} of a batch that did not land is kept at {kept}",
                seq,
                crate::service_timing::in_the_repository_at(repository)
            );
        }
    }
    // The old value is the one just read: a reference another command wrote in between is never removed, because the commit it
    //  names may be the only thing naming it (the owner's rule "never discard user data").
    if write_ref_awaiting(
        git,
        repository,
        &[
            "-c",
            NO_REFLOG_SETTING,
            "update-ref",
            "-d",
            "--no-deref",
            scratch.as_str(),
            tip.as_str(),
        ],
    )
    .await
    {
        crate::service_log!(
            "batch {} {}: the scratch reference {scratch} of a batch that did not land is removed",
            seq,
            crate::service_timing::in_the_repository_at(repository)
        );
        return ScratchRefRemoval::Removed;
    }
    left(seq, repository, format!("the scratch reference {scratch} could not be removed; its commit {tip} is kept at {kept}").as_str());
    return ScratchRefRemoval::Left;
}

/// The one path one open-intent question names,
/// owned so that a job of the blocking pool can take it.
struct OpenIntentQuestion {
    /// The journal record of the batch's intent.
    path: PathBuf,
}

/// The look one job of the pool does:
/// whether the journal holds an open intent for the batch.
fn an_intent_is_open(question: OpenIntentQuestion) -> bool {
    return question.path.exists();
}

/// What:
///  Whether the journal holds an open intent for batch `seq`,
/// as a job of the blocking pool awaited no longer than
///  `LANDING_STATE_CEILING`:
/// `Some(true)` or `Some(false)` when the look answered,
/// and nothing when it gave no answer.
/// Why:
///   The look is a `stat` of a file in the instance's journal,
/// which waits in the kernel on a mount that stopped answering,
/// and a
///  task of the runtime may not wait there.
/// A look that gave nothing in time is told apart from "no intent",
/// because
///  "no intent" sets the removal of the reference in motion and a look nobody answered must set nothing in motion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const open = await anOpenIntentAwaiting(journal, seq); // null when the look gave no answer
/// ```
async fn an_open_intent_awaiting(
    journal: &Path,
    seq: u64,
) -> Option<bool> {
    return an_open_intent_within(LANDING_STATE_CEILING, journal, seq).await;
}

/// `an_open_intent_awaiting` with the wait `ceiling` grants.
async fn an_open_intent_within(
    ceiling: Duration,
    journal: &Path,
    seq: u64,
) -> Option<bool> {
    let question: OpenIntentQuestion = OpenIntentQuestion {
        path: batch_path(journal, seq, OPEN_EXTENSION),
    };
    return match run_a_job_awaited_no_longer_than(ceiling, question, an_intent_is_open).await {
        TheAnswerOfAJobAwaited::Ran(open) => Some(open),
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}

/// What:
///  `clear_after_a_failed_landing` with the look at the intent a job of the pool and every Git step of it awaited
///  on the runtime.
/// Why:
///   A task of the runtime may wait on neither a file nor a child it starts itself,
/// so a landing that becomes a task
///  does this through this face.
/// A look that gave no answer leaves the reference where it is and says why,
/// so
///  no commit is lost and nothing claims to have removed what it never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = await clearAfterAFailedLandingAwaiting(context, requests, seq)
/// ```
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"
    )
)]
pub(crate) async fn clear_after_a_failed_landing_awaiting(
    context: &LandContext,
    requests: &[CommitRequest],
    seq: u64,
) -> ScratchRefRemoval {
    // No request is no import, so the batch made no reference at all.
    let Some(first) = requests.first() else {
        return ScratchRefRemoval::Removed;
    };
    // An open intent names the batch's own worktree, and the restart that settles it removes the reference then.
    match an_open_intent_awaiting(context.journal.as_path(), seq).await {
        Some(true) => return ScratchRefRemoval::Removed,
        Some(false) => {}
        None => {
            left(
                seq,
                first.repository.as_path(),
                "the record of this batch's intent cannot be read, so its scratch reference is left as it is and a restart settles the batch",
            );
            return ScratchRefRemoval::Left;
        }
    }
    let git: GitContext = GitContext {
        common_dir: Some(first.common_dir.clone()),
        ..context.git.clone()
    };
    return clear_after_dropped_batch_awaiting(&git, first.repository.as_path(), seq).await;
}

/// Controls for the awaited faces of these removals stay out of the release executable.
#[cfg(test)]
#[path = "service_scratch_ref_awaiting_tests.rs"]
mod awaiting_tests;
