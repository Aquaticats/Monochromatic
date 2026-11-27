//! What:
//!  The wait of one commit on its connection:
//!  the commit is queued and its answers are written with beats until the landing has answered,
//!  and a withdrawal is answered the same way.
//! Why:
//!   The beats begin before the check that says whether the request is this instance's has answered,
//!  and that answer goes through the same writer,
//!  so a check whose own read waits on storage never leaves the client silent;
//!  a client that hears nothing for its silence bound sends the commit again and may report it failed although it landed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await Promise.all([writeUntilAnswered(socket, messages, stuckNotice), decide().then(queueOrRefuse)]);
//! ```

use crate::blocking_job::run_blocking_job;
use crate::service_busy::{
    EarlyAnswer,
    LandingBoard,
};
use crate::service_protocol::{
    FailureCode,
    HEARTBEAT_EVERY,
    ServiceMessage,
};
use crate::service_reply::{
    Reply,
    write_until_answered_noting,
};
use crate::service_server_connection_handling_the_note_of_a_stuck_landing::{
    StuckNotice,
    stuck_notice,
};
use crate::service_server_connection_handling_what_every_connection_shares::ConnectionShared;
use crate::service_server_queue_types_and_send::{
    Job,
    send,
};
use crate::service_timing::Arrival;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::unix::OwnedWriteHalf;
use tokio::sync::mpsc::UnboundedSender;
/// What:
///  Queues a commit and writes its answers,
/// with beats,
/// until the landing has answered.
/// Why:
///   The beats begin before the check that says whether this request is this instance's has answered,
/// and that check's answer goes through the same
/// writer,
///  so a check whose own read waits on storage never leaves the client silent.
/// A client ends a service it has heard nothing from for five seconds (`HEARTBEAT_SILENCE`,
///  `service_client.rs`),
/// sends the commit again and may report it failed although it landed;
/// the check resolves a Git directory's real path
///  through the filesystem,
/// which under a busy disk took thirty-two seconds in the record of 2026-11-10T17:50:19Z.
/// The request reaches the landing only when the check says it is this instance's,
/// so a request for another
///  worktree still never lands here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [beats, queued] = await Promise.all([writeUntilAnswered(socket, messages), decide().then(queueOrRefuse)]);
/// ```
pub(crate) async fn queue_a_commit<Decide>(
    request: Box<crate::service_protocol::CommitRequest>,
    write_half: &mut OwnedWriteHalf,
    shared: &ConnectionShared,
    accepted: Instant,
    decide: Decide,
) where
    Decide: std::future::Future<Output = Option<bool>> + Send,
{
    let id: String = request.id.clone();
    let (reply, mut messages) = Reply::new();
    let answering: UnboundedSender<ServiceMessage> = reply.sender();
    let queue: UnboundedSender<Job> = shared.queue.clone();
    // What a beat asks for before it, held here because the writer borrows it for the whole wait.
    let mut notice: StuckNotice = stuck_notice(Arc::clone(&shared.board), Some(id.clone()));
    // What the check's answer is written with, and what it does with the request it decides about.
    let checking = async move {
        match decide.await {
            // A request for another worktree is not this instance's to land, so it never reaches the landing.
            Some(true) => {
                crate::service_warn!(
                    "refused a commit for {}, which another instance serves",
                    request.git_dir.display()
                );
                let _answered = answering.send(ServiceMessage::Failed {
                    id,
                    code: FailureCode::Rejected,
                });
            }
            // Nothing could say whose the request is: the client is told, and the landing never sees it.
            None => {
                let _answered = answering.send(ServiceMessage::Failed {
                    id,
                    code: FailureCode::Unavailable,
                });
            }
            Some(false) => {
                // The tree this request names is one the landing has still to read, however long the queue before it takes,
                //  so a collection spares it until the service is done with the request (`service_scratch_trees_of_queued_requests.rs`).
                crate::service_scratch_trees_of_queued_requests::note(
                    request.id.as_str(),
                    request.scratch.as_path(),
                );
                let _queued = queue.send(Job::Commit(
                    request,
                    reply,
                    Arrival {
                        accepted,
                        read: accepted.elapsed(),
                    },
                ));
                crate::service_fault::queued(id.as_str());
            }
        }
    };
    // This task has nothing else to do until the request is answered, so it writes the answers the landing thread
    //  queues and keeps the client told that the service lives, and what it is stuck in when it is stuck in a write.
    let beats =
        write_until_answered_noting(write_half, &mut messages, HEARTBEAT_EVERY, &mut notice);
    tokio::join!(beats, checking);
}
/// Answers the withdrawal of a request.
pub(crate) async fn withdraw(
    id: String,
    write_half: &mut OwnedWriteHalf,
    shared: &ConnectionShared,
) {
    // Behind a write that never returns the landing thread answers nothing, so a request that has not begun to land is
    //  withdrawn here (`service_busy.rs`); otherwise the landing thread answers between two landings, so the answer is
    //  either that the commit landed or that it never will, and meanwhile this task keeps the client told that the
    //  service lives.
    let board: Arc<LandingBoard> = Arc::clone(&shared.board);
    let asked: String = id.clone();
    // The decision is made by creating files in the journal, which can wait on storage.
    let early: EarlyAnswer = match run_blocking_job(move || {
        return board.withdraw_early(asked.as_str());
    })
    .await
    {
        Ok(answer) => answer,
        // Without a thread to decide here, the landing thread answers between two landings, as it does for a request that began.
        Err(error) => {
            crate::service_warn!(
                "the withdrawal of {id} could not be decided here and goes to the landing: {error}"
            );
            EarlyAnswer::Queued
        }
    };
    match early {
        EarlyAnswer::Landed(commit) => {
            send(
                write_half,
                &ServiceMessage::Landed {
                    id,
                    commit,
                    already: true,
                    landing: None,
                },
            )
            .await
        }
        EarlyAnswer::Withdrawn(words) => {
            send(
                write_half,
                &ServiceMessage::Waiting {
                    text: words,
                },
            )
            .await;
            send(
                write_half,
                &ServiceMessage::Withdrawn {
                    id,
                },
            )
            .await;
        }
        EarlyAnswer::Queued => {
            let (reply, mut messages) = Reply::new();
            let withdrawing: String = id.clone();
            let _queued = shared.queue.send(Job::Withdraw(id, reply));
            // The connection keeps the identity of the request, so a client that withdrew too early to have been told it is
            // inside the write is told now.
            write_until_answered_noting(
                write_half,
                &mut messages,
                HEARTBEAT_EVERY,
                &mut stuck_notice(Arc::clone(&shared.board), Some(withdrawing)),
            )
            .await;
        }
    };
}
