//! What:
//!  Handles one client connection of the commit service:
//! reads the request line within the read timeout,
//! checks who
//!  sent it,
//! answers messages that need no landing,
//! and queues the rest;
//! also the note that tells a waiting client what the
//!  landing thread is stuck in.
//! Why:
//!  Each connection runs as a task of its own (`doc/planning/git-policy-cli-full-tokio.md`,
//! step T10),
//! so a client that waits costs memory and no thread;
//! the file keeps that task's whole life in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! server.on('connection', (socket) => handleConnection(socket, shared));
//! ```

use crate::service_server_queue_types_and_send::{Job, send};
use crate::service_timing::{Arrival, log, retire_line};
use crate::service_instance::instance_of;
use crate::service_work::Wake;
use crate::service_busy::{EarlyAnswer, LandingBoard};
use crate::blocking_job::run_blocking_job;
use crate::service_reply::{NoteSource, Reply, write_until_answered_noting};
use crate::service_protocol::{
    ClientMessage, FailureCode, HEARTBEAT_EVERY, MAX_LINE_BYTES, PROTOCOL_VERSION, ServiceMessage, decode_client,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

/// How long a connection may take to send its request line before it is dropped.
pub const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// The point at which this file reads a Git directory's instance,
/// for a control that holds that read (`service_fault_hooks_test_support.rs`).
#[cfg(test)]
const IN_THE_INSTANCE_READ: &str = "in-the-instance-read";

/// What every connection of one service run shares:
/// the queues to the landing and the work loop,
/// the count of work in flight,
/// the landing's board,
/// and what the service announces and serves.
#[derive(Clone)]
pub(crate) struct ConnectionShared {
    /// Where requests for the landing go.
    pub queue: UnboundedSender<Job>,
    /// Where after-commit work is announced.
    pub work: UnboundedSender<Wake>,
    /// The count of work in flight that keeps the service from exiting idle.
    /// What the landing is inside,
    /// for the notes a waiting client is told.
    pub board: Arc<LandingBoard>,
    /// The release the service announces to every client.
    pub release: String,
    /// The worktree this instance serves,
    /// when it serves one.
    pub instance: Option<String>,
    /// The Git directory of the worktree this instance serves,
    /// as its note or journal named it when the service started,
    /// and nothing when that could not be told.
    /// A request that names this text is this instance's without asking the filesystem whose it is.
    pub own_git_dir: Option<PathBuf>,
}


/// What:
///  The note for a connection's beat:
/// what the landing thread is stuck in when it is inside a write that is never ended,
/// said again
///  only when the words change (they name whole minutes).
/// Why:
///   A client queued behind a write that never returns is told why it waits,
/// and the words become the blocker its error names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// () => { const words = board.describe(); return words !== last ? (last = words, { type: 'waiting', text: words }) : undefined; }
/// ```
pub(crate) fn stuck_notice(board: Arc<LandingBoard>, request: Option<String>) -> StuckNotice {
    return StuckNotice { board, request, last: None };
}

/// The note source of a connection's beat:
/// what the landing thread is stuck in,
/// said again only when the words change.
pub(crate) struct StuckNotice {
    /// What the landing thread reports about itself.
    board: Arc<LandingBoard>,
    /// The identity of the request the connection serves,
    /// when it serves one.
    request: Option<String>,
    /// The key of the words said last.
    last: Option<String>,
}

/// The note the board has for a connection and the key of the words said last after it,
/// given the key said before.
/// The board is files of the journal,
/// which can wait on storage,
/// so the reads run on the blocking pool.
fn note_from_the_board(board: &LandingBoard, request: Option<&str>, last: Option<String>) -> (Option<ServiceMessage>, Option<String>) {
    // A client whose own request is in the group being landed is told so when the write starts, because its commit may be on
    // the branch already; the key changes when the longer words of a minute's stuck write replace the first ones.
    let inside: Option<(String, String)> = match request {
        Some(id) => board.inside_notice(id),
        None => None,
    };
    if let Some((key, words)) = inside {
        if last.as_deref() == Some(key.as_str()) {
            return (None, last);
        }
        return (Some(ServiceMessage::InsideWrite { text: words }), Some(key));
    }
    let Some(words) = board.describe() else { return (None, last) };
    if last.as_deref() == Some(words.as_str()) {
        return (None, last);
    }
    return (Some(ServiceMessage::Waiting { text: words.clone() }), Some(words));
}

impl NoteSource for StuckNotice {
    async fn note(&mut self) -> Option<ServiceMessage> {
        let board: Arc<LandingBoard> = Arc::clone(&self.board);
        let request: Option<String> = self.request.clone();
        let last: Option<String> = self.last.take();
        return match run_blocking_job(move || { return note_from_the_board(&board, request.as_deref(), last); }).await {
            Ok((message, last_said)) => {
                self.last = last_said;
                message
            }
            Err(error) => {
                // No note this beat: the beat itself is the proof of life and is still written.
                crate::service_warn!("the note of what the landing is inside could not be read for a waiting client: {error}");
                None
            }
        };
    }
}

/// The user id of the process on the other end of a connection.
fn user_of(credentials: tokio::net::unix::UCred) -> u32 {
    return credentials.uid();
}

/// Reads one request line from a new connection,
/// checks who sent it,
/// and queues it.
pub(crate) async fn handle_connection(stream: UnixStream, shared: ConnectionShared) {
    let accepted: Instant = Instant::now();
    let sender_user: Option<u32> = stream.peer_cred().ok().map(user_of);
    let (first_half, mut write_half): (OwnedReadHalf, OwnedWriteHalf) = stream.into_split();
    send(&mut write_half, &ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(shared.release.clone()) }).await;
    if sender_user != Some(rustix::process::geteuid().as_raw()) {
        return;
    }
    let limit: u64 = u64::try_from(MAX_LINE_BYTES).unwrap_or(u64::MAX) + 1;
    let mut line: String = String::new();
    let mut reader = BufReader::new(first_half.take(limit));
    match tokio::time::timeout(REQUEST_READ_TIMEOUT, reader.read_line(&mut line)).await {
        Ok(Ok(_read)) => {}
        // A line that is not text, a failed read or a request line that is too slow: the connection is dropped.
        Ok(Err(_)) | Err(_) => return,
    }
    if line.is_empty() {
        // A connection that only looked (a readiness probe) sends nothing and is not an error.
        return;
    }
    answer_request(&line, &mut write_half, &shared, accepted).await;
}

/// Whether a Git directory belongs to another instance of the service than this one,
/// asked on the blocking pool because naming the instance resolves the path;
/// nothing when the pool could not run it.
/// Why:
///  A request that names this worktree's own Git directory exactly is this instance's without that ask:
/// the same text
///  names the same directory,
/// so the same instance,
/// and the filesystem is asked only for a request that spells it another way
///  or belongs to another worktree.
///  The ask resolves a real path,
/// which under a busy disk took thirty-two seconds in the record of
///  2026-11-10T17:50:19Z,
/// and it is on the answering path of every commit and every wake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isForeign(shared, gitDir) // shared.ownGitDir === gitDir ? false : await named(gitDir) !== shared.instance
/// ```
async fn is_foreign(shared: &ConnectionShared, git_dir: PathBuf) -> Option<bool> {
    let Some(mine) = shared.instance.clone() else { return Some(false) };
    // The text this service was named for: the same text names the same directory, whatever a resolution would say.
    if shared.own_git_dir.as_deref() == Some(git_dir.as_path()) {
        return Some(false);
    }
    return match run_blocking_job(move || {
        // A control holds this read on whichever thread runs it, which is a pool thread and not the thread that set the hold.
        #[cfg(test)]
        crate::service_fault::hooks::run_on_any_thread(IN_THE_INSTANCE_READ);
        return instance_of(git_dir.as_path()) != mine;
    }).await {
        Ok(foreign) => Some(foreign),
        Err(error) => {
            crate::service_error!("the instance of a Git directory could not be looked at: {error}");
            None
        }
    };
}

/// Refuses a message about a Git directory that another instance serves.
async fn refuse_a_message_for(git_dir: &std::path::Path, write_half: &mut OwnedWriteHalf) {
    crate::service_warn!("refused a message for {}, which another instance serves", git_dir.display());
    send(write_half, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected }).await;
}

/// Tells a client that the service cannot take its request now.
async fn tell_unavailable(write_half: &mut OwnedWriteHalf) {
    send(write_half, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Unavailable }).await;
}

/// Acts on the request line of a connection:
/// refuses what another instance serves,
/// answers what needs no landing,
/// and queues the rest.
async fn answer_request(line: &str, write_half: &mut OwnedWriteHalf, shared: &ConnectionShared, accepted: Instant) {
    // A message about another worktree is not this instance's to act on: each worktree has its own instance, so one that
    //  reaches the wrong socket is a mistake of the sender's, refused where it can be seen.
    match decode_client(line.trim_end_matches('\n')) {
        Ok(ClientMessage::Commit(request)) => match is_foreign(shared, request.git_dir.clone()).await {
            Some(true) => {
                crate::service_warn!("refused a commit for {}, which another instance serves", request.git_dir.display());
                send(write_half, &ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::Rejected }).await;
            }
            Some(false) => {
                let deciding = is_foreign(shared, request.git_dir.clone());
                queue_a_commit(request, write_half, shared, accepted, deciding).await;
            }
            None => send(write_half, &ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::Unavailable }).await,
        },
        Ok(ClientMessage::Work { git_dir, review_key }) => match is_foreign(shared, git_dir.clone()).await {
            Some(true) => refuse_a_message_for(git_dir.as_path(), write_half).await,
            Some(false) => {
                let _queued = shared.work.send((git_dir, review_key));
                let _touched = shared.queue.send(Job::Activity);
            }
            None => tell_unavailable(write_half).await,
        },
        Ok(ClientMessage::Warm { repository, git_dir }) => match is_foreign(shared, git_dir.clone()).await {
            Some(true) => refuse_a_message_for(git_dir.as_path(), write_half).await,
            // Nothing is answered: the compile is a job of the blocking pool, and the connection closes with this function.
            Some(false) => crate::service_rules_precompile::warm_in_background(repository),
            None => tell_unavailable(write_half).await,
        },
        Ok(ClientMessage::Withdraw { id }) => withdraw(id, write_half, shared).await,
        Ok(ClientMessage::Retire) => {
            log(retire_line(shared.release.as_str()).as_str());
            let _queued = shared.queue.send(Job::Retire);
        }
        Err(reason) => {
            crate::service_warn!("refused a request line: {reason}");
            send(write_half, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected }).await;
        }
    };
}

/// What:
///  Queues a commit and writes its answers,
/// with beats,
/// until the landing has answered.
/// Why:
///   The beats begin with the connection,
/// and the answer of the check that says whether this request is this instance's goes through the same
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
async fn queue_a_commit<Decide>(request: Box<crate::service_protocol::CommitRequest>, write_half: &mut OwnedWriteHalf, shared: &ConnectionShared, accepted: Instant, decide: Decide)
where
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
                let _answered = answering.send(ServiceMessage::Failed { id, code: FailureCode::Rejected });
            }
            // Nothing could say whose the request is: the client is told, and the landing never sees it.
            None => {
                let _answered = answering.send(ServiceMessage::Failed { id, code: FailureCode::Unavailable });
            }
            Some(false) => {
                let _queued = queue.send(Job::Commit(request, reply, Arrival { accepted, read: accepted.elapsed() }));
                crate::service_fault::queued(id.as_str());
            }
        }
    };
    // This task has nothing else to do until the request is answered, so it writes the answers the landing thread
    //  queues and keeps the client told that the service lives, and what it is stuck in when it is stuck in a write.
    let beats = write_until_answered_noting(write_half, &mut messages, HEARTBEAT_EVERY, &mut notice);
    tokio::join!(beats, checking);
}

/// Answers the withdrawal of a request.
async fn withdraw(id: String, write_half: &mut OwnedWriteHalf, shared: &ConnectionShared) {
    // Behind a write that never returns the landing thread answers nothing, so a request that has not begun to land is
    //  withdrawn here (`service_busy.rs`); otherwise the landing thread answers between two landings, so the answer is
    //  either that the commit landed or that it never will, and meanwhile this task keeps the client told that the
    //  service lives.
    let board: Arc<LandingBoard> = Arc::clone(&shared.board);
    let asked: String = id.clone();
    // The decision is made by creating files in the journal, which can wait on storage.
    let early: EarlyAnswer = match run_blocking_job(move || { return board.withdraw_early(asked.as_str()); }).await {
        Ok(answer) => answer,
        // Without a thread to decide here, the landing thread answers between two landings, as it does for a request that began.
        Err(error) => {
            crate::service_warn!("the withdrawal of {id} could not be decided here and goes to the landing: {error}");
            EarlyAnswer::Queued
        }
    };
    match early {
        EarlyAnswer::Landed(commit) => send(write_half, &ServiceMessage::Landed { id, commit, already: true, landing: None }).await,
        EarlyAnswer::Withdrawn(words) => {
            send(write_half, &ServiceMessage::Waiting { text: words }).await;
            send(write_half, &ServiceMessage::Withdrawn { id }).await;
        }
        EarlyAnswer::Queued => {
            let (reply, mut messages) = Reply::new();
            let withdrawing: String = id.clone();
            let _queued = shared.queue.send(Job::Withdraw(id, reply));
            // The connection keeps the identity of the request, so a client that withdrew too early to have been told it is
            // inside the write is told now.
            write_until_answered_noting(write_half, &mut messages, HEARTBEAT_EVERY, &mut stuck_notice(Arc::clone(&shared.board), Some(withdrawing))).await;
        }
    };
}

/// The refusal paths of a connection are controlled in the test module.
#[cfg(test)]
#[path = "service_server_connection_handling_tests.rs"]
mod tests;
