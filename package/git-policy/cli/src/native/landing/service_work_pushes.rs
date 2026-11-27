//! What:
//!  The push of one branch:
//!  the commit the push may send,
//!  the records that hold it back,
//!  and the words each answer of Git and of the server comes to.
//! Why:
//!   A commit is pushed as the branch stands at the moment the push reads it,
//!  never as the newest record the drain claimed,
//!  so no commit the person has not decided on is published before its own gate answered.
//!  Both faces of the push are here,
//!  so the synchronous one cannot answer differently from the awaited one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function pushAwaiting(handlers: GitHandlers, items: WorkItem[]): Promise<Verdict> { /* ... */ }
//! ```

use super::service_work_acceptable_records_and_claims::work_directory;
use super::service_work_branch_is_gone::branch_is_gone_words_awaiting;
use super::service_work_branch_questions::{
    branch_exists_awaiting,
    branch_tip_awaiting,
};
use super::service_work_git_handlers::GitHandlers;
use super::service_work_handlers_and_verdicts::{
    Verdict,
    WorkHandlers,
};
use crate::auto_push::AutoPushOutcome;
use crate::auto_push_awaiting::push_once_awaiting;
use crate::auto_push_single_flight::PushGit;
use crate::push_retry_failure::{
    final_failure,
    plain_words,
    retried_failure_record,
    unrecognised_rejection,
};
use crate::work_item::WorkItem;
use crate::work_item_holding::{
    BlockedVerdict,
    blocked_verdicts,
};
use std::path::PathBuf;

pub(super) fn push(
    handlers: &GitHandlers,
    items: &[WorkItem],
) -> Verdict {
    return match crate::lazy_runtime::run_on_runtime(handlers.push_awaiting(items)) {
        Ok(verdict) => verdict,
        // The work did not happen, so the item is tried again rather than reported as a push that failed.
        Err(refusal) => Verdict::Retry(format!("the push could not run: {refusal}")),
    };
}

pub(super) fn push_awaiting<'a>(
    handlers: &'a GitHandlers,
    items: &'a [WorkItem],
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
    return Box::pin(async move {
        let Some(newest) = items.last() else {
            return Verdict::Done;
        };
        let git: PushGit = handlers.push_git_awaiting(newest).await;
        // What: A push whose command chose its transport stops here when this service no longer holds that choice, or holds
        //  it as a value this release cannot write in a line (`push_without_its_transport::gone`).
        // Why:  Those settings live only in the memory of the service that received them, so running the push through the
        //  manager's environment would speak a transport nobody asked for; nothing was sent, so the person is told why and
        //  given the command that pushes.
        let held: Option<crate::service_push_programs_held_in_memory::PushPrograms> =
            crate::service_push_programs_held_in_memory::held(
                newest.git_dir.as_path(),
                newest.id().as_str(),
            );
        if let Some(gone) = crate::push_without_its_transport::gone(held.as_ref(), newest.transport)
        {
            return Verdict::Skipped(
                crate::push_without_its_transport::words(&git, newest, gone).await,
            );
        }
        // The commit this push may send is fixed before the worktree's records are read. Every commit's record is written before its
        //  branch moves, so every commit of the branch at this reading has a record in the reading, while a commit that lands after
        //  it has none; sending the branch's reference instead would publish that later commit before its own gate answered, and that
        //  gate may keep it local.
        let Some(sent_up_to) = branch_tip_awaiting(&git, newest.branch_ref.as_str()).await else {
            // The branch named no commit at that reading, so nothing of it can be sent: it is either gone, which is said with wherever
            //  the commit still lives, or Git could not be asked, and the push is tried again. Never is the branch's reference sent
            //  instead, because it may name a commit by now.
            return match branch_exists_awaiting(&git, newest.branch_ref.as_str()).await {
                Some(false) => Verdict::Skipped(branch_is_gone_words_awaiting(&git, newest).await),
                _ => Verdict::Retry(format!(
                    "the commit {} was not pushed: {} named no commit in this repository when the push read it, and the push is tried again",
                    newest.oid, newest.branch_ref
                )),
            };
        };
        // The verdicts a post-commit gate kept are the worktree's own records: they are what makes this push stop before a commit the
        //  gate refused, however far the branch has moved since.
        let work: PathBuf = work_directory(newest.git_dir.as_path());
        let blocked: Vec<BlockedVerdict> = blocked_verdicts(work.as_path());
        // A commit whose own gate still runs is not decided either, and a push of the branch sends every commit up to the tip: the
        //  record is what keeps the tip where the gate has not answered yet (`work_item_holding.rs`).
        let undecided: Vec<String> =
            crate::work_item_holding::undecided_commits(work.as_path(), newest.branch_ref.as_str());
        let mut output: Vec<u8> = Vec::new();
        return match push_once_awaiting(
            &git,
            newest.branch_ref.as_str(),
            sent_up_to.as_str(),
            blocked.as_slice(),
            undecided.as_slice(),
            &mut output,
        )
        .await
        {
            // A skip sent nothing: with words they are told; with none (no upstream and no origin) the repository has nowhere
            //  to back up to and there is nothing to say, but an earlier failure of the branch is still true.
            Ok(result) if result.outcome == AutoPushOutcome::Skipped => {
                Verdict::Skipped(result.shown)
            }
            // A verdict held the push back: nothing was sent and nothing failed, so the branch is tried again by the retry plan the
            //  service keeps, and the words go to the person's record with the promise that plan keeps.
            Ok(result) if result.outcome == AutoPushOutcome::HeldBack => {
                Verdict::Retry(result.shown)
            }
            Ok(result) if result.exit_code == 0 => {
                // A matching push can carry a sibling the remote refused while this branch arrived: the branch is done, and the
                //  refused sibling is published nowhere, so the journal names it here and the record the next command shows carries
                //  it to the person.
                for line in crate::matching_push_delivered_the_recorded_branch::refused_lines(
                    result.shown.as_str(),
                ) {
                    crate::service_warn!(
                        "a branch of this push was refused and was not published: {line}"
                    );
                }
                // A push that exits 0 still holds words the person needs: the server's own `remote:` lines, the account of a
                //  reconcile it made, and the refusal of a remote-tracking reference it could not update, which is the only sign
                //  that the local view stayed behind. The command that ran the push returned long before it ended, so those words
                //  go to a record the next command shows once, which the step that settles this verdict writes. A push that simply
                //  landed says nothing and leaves no record.
                // The record is named for the commit this push sent, never for the newest record the drain claimed: a commit that
                //  landed after that claim is what the push sends (`sent_up_to`, read from the branch), so naming the record for the
                //  claimed commit would overwrite an unread record of that commit with words about another one, and label them as its.
                Verdict::Delivered {
                    commit: sent_up_to,
                    said: result.shown,
                }
            }
            // A branch the person deleted is not there to push, now or ever: nothing was sent, so nothing can pass by waiting, and
            //  the commit is named for the person with wherever it still lives. The repository is asked, never the words of the
            //  output: any program a push runs can print a line about a branch of its own.
            Ok(_result)
                if branch_exists_awaiting(&git, newest.branch_ref.as_str()).await
                    == Some(false) =>
            {
                Verdict::Skipped(branch_is_gone_words_awaiting(&git, newest).await)
            }
            // A conflict, a login the service does not have and a server hook's decline need the person and stop the retry;
            //  every other failure can pass (a stall, the network, an LFS server error, a held lock, a host key this machine
            //  does not know), and the record of one whose words ask something of the person says what to do and that the
            //  service keeps its schedule.
            Ok(result) => match final_failure(result.shown.as_str()) {
                Some(cause) => Verdict::Failed(match plain_words(cause) {
                    Some(words) => format!("{words}\n{}", result.shown),
                    None => result.shown,
                }),
                None => {
                    // A rejection by the server whose reason is not known is tried again, and named so that it can be learned.
                    if let Some(line) = unrecognised_rejection(result.shown.as_str()) {
                        crate::service_warn!(
                            "the server rejected the push {} for a reason this release does not know, so it is tried again: {line}",
                            crate::service_timing::in_the_repository_at(
                                newest.repository.as_path()
                            )
                        );
                    }
                    Verdict::Retry(retried_failure_record(result.shown.as_str()))
                }
            },
            Err(error) => Verdict::Retry(format!("{error}")),
        };
    });
}
