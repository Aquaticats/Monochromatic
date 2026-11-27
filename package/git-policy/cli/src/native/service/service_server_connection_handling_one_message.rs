//! What:
//!  Acts on one decoded line of a client connection:
//!  answers a commit's question about a drawn shadow,
//!  refuses a message another instance serves,
//!  answers what needs no landing,
//!  and queues the rest.
//! Why:
//!  Every message that needs no landing is answered here and not by the landing thread,
//!  so its answer cannot wait behind another commit's landing;
//!  what needs landing goes to the queue this file names once,
//!  and the connection's own file only reads the line and hands it over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! switch (decodeClient(line).type) { case 'commit': ...; case 'withdraw': ...; case 'retire': ...; }
//! ```

use crate::service_protocol::{
    ClientMessage,
    FailureCode,
    ServiceMessage,
    decode_client,
};
use crate::service_server_connection_handling_the_wait_of_a_commit::{
    queue_a_commit,
    withdraw,
};
use crate::service_server_connection_handling_what_every_connection_shares::ConnectionShared;
use crate::service_server_connection_handling_whose_message_it_is::{
    is_foreign,
    refuse_a_message_for,
    tell_unavailable,
};
use crate::service_server_queue_types_and_send::{
    Job,
    send,
};
use crate::service_timing::{
    log,
    retire_line,
};
use std::time::Instant;
use tokio::net::unix::OwnedWriteHalf;
/// What:
///  Answers a commit's question about the finished shadow drawn ahead of it:
///  the tree,
/// or that the stock holds none.
/// Why:
///   The question is answered here and not by the landing,
///  from a list held in memory,
///  so its answer cannot wait for another commit's landing.
///  A tree leaves the stock with this answer,
///  so two commits that ask at once never get the same one:
///  the second is told there is none and takes the pool's member or its own creation.
///  The commit's scratch tree names the attempt that asked,
///  which the answer needs nothing more of:
///  the tree's life is the commit's renewal and the sweep's (`shadow_drawn.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = shared.drawn.handOut(); send(writeHalf, tree ? { type: 'drawn', tree } : { type: 'no-shadow' });
/// ```
pub(crate) async fn answer_the_shadow_question(
    scratch: &std::path::Path,
    write_half: &mut OwnedWriteHalf,
    shared: &ConnectionShared,
) {
    match shared.drawn.hand_out() {
        Some(tree) => {
            crate::diagnostic_log::debug(
                "shadowDrawn",
                format!(
                    "handed {} to the commit that asked from {}",
                    tree.display(),
                    scratch.display()
                )
                .as_str(),
            );
            send(
                write_half,
                &ServiceMessage::Drawn {
                    tree,
                },
            )
            .await;
        }
        None => send(write_half, &ServiceMessage::NoShadow).await,
    };
}
/// Acts on the request line of a connection:
/// refuses what another instance serves,
/// answers what needs no landing,
/// and queues the rest.
pub(crate) async fn answer_request(
    line: &str,
    write_half: &mut OwnedWriteHalf,
    shared: &ConnectionShared,
    accepted: Instant,
) {
    // A message about another worktree is not this instance's to act on: each worktree has its own instance, so one that
    //  reaches the wrong socket is a mistake of the sender's, refused where it can be seen.
    match decode_client(line.trim_end_matches('\n')) {
        Ok(ClientMessage::Commit(request)) => {
            // The question of whose this request is runs beside the beats of its connection, not before them, so a check
            //  whose own read waits on storage never leaves the client silent (`queue_a_commit`).
            let deciding = is_foreign(shared, request.git_dir.clone());
            queue_a_commit(request, write_half, shared, accepted, deciding).await;
        }
        Ok(ClientMessage::Work {
            git_dir,
            review_key,
            push_programs,
        }) => match is_foreign(shared, git_dir.clone()).await {
            Some(true) => refuse_a_message_for(git_dir.as_path(), write_half).await,
            Some(false) => {
                // What: The programs and transport settings of the command that wrote these push records are held here, for the
                //  records it named.
                // Why:  Nothing of them is written (the two command lines can hold a password), and the round that runs a push
                //  looks them up by its record's id, so each push speaks and searches with the values of the command that asked
                //  for it (`service_push_programs_held_in_memory`).
                for (id, programs) in push_programs {
                    crate::service_push_programs_held_in_memory::remember(
                        git_dir.as_path(),
                        id.as_str(),
                        programs,
                    );
                }
                let _queued = shared.work.send((git_dir, review_key));
                let _touched = shared.queue.send(Job::Activity);
            }
            None => tell_unavailable(write_half).await,
        },
        // What: The push records this command settled or removed can no longer run, so the values held against them are given
        //  up here.
        // Why:  A record the committing wrapper removed leaves nothing the service could learn it from, and one of the values
        //  can hold a password (`service_push_programs_held_in_memory`). Nothing is queued: no push is left to run.
        Ok(ClientMessage::ForgetPrograms {
            git_dir,
            records,
        }) => match is_foreign(shared, git_dir.clone()).await {
            Some(true) => refuse_a_message_for(git_dir.as_path(), write_half).await,
            Some(false) => {
                for id in records {
                    crate::service_push_programs_held_in_memory::forget(
                        git_dir.as_path(),
                        id.as_str(),
                    );
                }
            }
            None => tell_unavailable(write_half).await,
        },
        // What: A commit asks whether a finished shadow was drawn ahead of it, and where it is.
        // Why:  The answer is taken out of the service's own stock and written by this connection's task, never by the
        //       landing thread: a commit that waited for its tree through the landing queue would queue behind every other
        //       commit's landing, which is the wait the hand-out exists to remove.
        Ok(ClientMessage::Shadow {
            git_dir,
            scratch,
        }) => match is_foreign(shared, git_dir.clone()).await {
            Some(true) => refuse_a_message_for(git_dir.as_path(), write_half).await,
            Some(false) => answer_the_shadow_question(scratch.as_path(), write_half, shared).await,
            None => tell_unavailable(write_half).await,
        },
        Ok(ClientMessage::Warm {
            repository,
            git_dir,
        }) => match is_foreign(shared, git_dir.clone()).await {
            Some(true) => refuse_a_message_for(git_dir.as_path(), write_half).await,
            // Nothing is answered: the compile is a job of the blocking pool, and the connection closes with this function.
            Some(false) => crate::service_rules_precompile::warm_in_background(repository),
            None => tell_unavailable(write_half).await,
        },
        Ok(ClientMessage::Withdraw {
            id,
        }) => withdraw(id, write_half, shared).await,
        Ok(ClientMessage::Retire) => {
            log(retire_line(shared.release.as_str()).as_str());
            let _queued = shared.queue.send(Job::Retire);
        }
        Err(reason) => {
            crate::service_warn!("refused a request line: {reason}");
            send(
                write_half,
                &ServiceMessage::Failed {
                    id: String::new(),
                    code: FailureCode::Rejected,
                },
            )
            .await;
        }
    };
}
