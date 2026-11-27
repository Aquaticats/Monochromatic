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
//! so a client that waits costs memory and no thread.
//!
//! The parts of this connection,
//! each a file of this folder,
//! in the order they build on each other:
//!
//! - `service_server_connection_handling_what_every_connection_shares.rs`:
//!   the queues,
//!   the board and the drawn shadows one service run hands every connection.
//! - `service_server_connection_handling_the_note_of_a_stuck_landing.rs`:
//!   the note a beat sends about what the landing is stuck in.
//! - `service_server_connection_handling_whose_message_it_is.rs`:
//!   whether a message is this instance's own,
//!   and the words for the ones it is not.
//! - `service_server_connection_handling_the_wait_of_a_commit.rs`:
//!   a queued commit answered with beats,
//!   and a withdrawal answered the same way.
//! - `service_server_connection_handling_one_message.rs`:
//!   what one decoded line of a connection comes to.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! server.on('connection', (socket) => handleConnection(socket, shared));
//! ```

/// What one decoded line of a connection comes to.
pub(crate) use crate::service_server_connection_handling_one_message::answer_request;
/// The point at which a beat's note reads the journal,
/// for a control that holds that read.
#[cfg(test)]
pub(crate) use crate::service_server_connection_handling_the_note_of_a_stuck_landing::note_read_point;
/// The note a connection's beat sends about what the landing thread is stuck in,
/// said again only when the words change,
/// named here for the controls of this connection.
#[cfg(test)]
pub(crate) use crate::service_server_connection_handling_the_note_of_a_stuck_landing::{
    StuckNotice,
    stuck_notice,
};
/// A commit queued and answered with beats,
/// named here for the controls of this connection.
#[cfg(test)]
pub(crate) use crate::service_server_connection_handling_the_wait_of_a_commit::queue_a_commit;
/// What every connection of one service run shares:
/// the queues to the landing and the work loop,
/// the landing's board,
/// and the release and instance the service announces and serves.
pub(crate) use crate::service_server_connection_handling_what_every_connection_shares::ConnectionShared;
/// The point at which a connection reads an instance,
/// for a control that holds that read.
#[cfg(test)]
pub(crate) use crate::service_server_connection_handling_whose_message_it_is::instance_read_point;

use crate::service_protocol::{
    MAX_LINE_BYTES,
    PROTOCOL_VERSION,
    ServiceMessage,
};
use crate::service_server_queue_types_and_send::send;
use std::time::{
    Duration,
    Instant,
};
use tokio::io::{
    AsyncBufReadExt,
    AsyncReadExt,
    BufReader,
};
use tokio::net::UnixStream;
use tokio::net::unix::{
    OwnedReadHalf,
    OwnedWriteHalf,
};

/// How long a connection may take to send its request line before it is dropped.
pub const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// The user id of the process on the other end of a connection.
fn user_of(credentials: tokio::net::unix::UCred) -> u32 {
    return credentials.uid();
}

/// Reads one request line from a new connection,
/// checks who sent it,
/// and queues it.
pub(crate) async fn handle_connection(
    stream: UnixStream,
    shared: ConnectionShared,
) {
    let accepted: Instant = Instant::now();
    let sender_user: Option<u32> = stream.peer_cred().ok().map(user_of);
    // What: Another account's process is told nothing at all, not even which release this service is.
    // Why:  A socket file is protected by the directory it stands in, and an abstract name has no file and no permission
    //       bits (`service_address_of_a_worktree.rs`), so the kernel's word about who took the connection is what keeps
    //       every other account out. Greeting first told such a process the release of this wrapper, and the check that
    //       followed read no byte of its request either.
    if sender_user != Some(rustix::process::geteuid().as_raw()) {
        return;
    }
    let (first_half, mut write_half): (OwnedReadHalf, OwnedWriteHalf) = stream.into_split();
    send(
        &mut write_half,
        &ServiceMessage::Welcome {
            version: PROTOCOL_VERSION,
            release: Some(shared.release.clone()),
        },
    )
    .await;
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
    // What: A line that ended with the stream and not with its framing newline is not a request, and nothing of it is read
    //       as a message or queued.
    // Why:  A client that gives up writing says the service never took the request, and that is true only if the service
    //       refuses a line the client could not finish; the other way round, a request it declares not made is landed by the
    //       service and the client removes the scratch tree the landing reads (the review of this change, finding 1: a
    //       request of 3,262 bytes without its newline was decoded and landed, twice, once after a full close).
    //       The line still has a bound (`REQUEST_READ_TIMEOUT`) and the same `MAX_LINE_BYTES` cap, so a client that keeps
    //       talking without a newline is cut off as before.
    if !line.ends_with('\n') {
        crate::service_warn!(
            "refused a request line that ended with its connection instead of its framing newline, so nothing of it was read as a message and nothing was queued"
        );
        return;
    }
    answer_request(&line, &mut write_half, &shared, accepted).await;
}

/// The refusal paths of a connection are controlled in the test module.
#[cfg(test)]
#[path = "service_server_connection_handling_tests.rs"]
mod tests;
