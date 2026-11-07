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
//!  Each connection runs on a thread of its own;
//! the file keeps that thread's whole life in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! server.on('connection', (socket) => handleConnection(socket, queue, work, busy, board, release, instance));
//! ```

use crate::service_server_queue_types_and_send::{Job, send};
use crate::service_timing::{Arrival, log, retire_line};
use crate::service_instance::instance_of;
use crate::service_work::Wake;
use crate::service_busy::{EarlyAnswer, LandingBoard};
use crate::service_reply::{NoteSource, Reply, write_until_answered_noting};
use crate::service_protocol::{
    ClientMessage, FailureCode, HEARTBEAT_EVERY, MAX_LINE_BYTES, PROTOCOL_VERSION, ServiceMessage, decode_client,
};
use std::io::{BufRead, BufReader, Read};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
/// How long a connection may take to send its request line before it is dropped.
pub const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// The user id of the process on the other end of a connection.
pub(crate) fn peer_user(stream: &UnixStream) -> Option<u32> {
    use std::os::fd::AsRawFd;
    let mut credentials = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: `credentials` and `length` are valid for the call and `length` is its size.
    let status = unsafe {
        libc::getsockopt(stream.as_raw_fd(), libc::SOL_SOCKET, libc::SO_PEERCRED, (&raw mut credentials).cast(), &raw mut length)
    };
    return if status == 0 { Some(credentials.uid) } else { None };
}

/// Returns when the other end of the connection has closed it,
/// or the connection failed.
/// Anything the client sends is ignored.
pub(crate) fn wait_for_close(stream: &UnixStream) {
    let mut sink: [u8; 64] = [0; 64];
    let mut reader = stream;
    loop {
        match reader.read(&mut sink) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
    }
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
pub(crate) fn stuck_notice<'a>(board: &'a LandingBoard, request: Option<&'a str>) -> StuckNotice<'a> {
    return StuckNotice { board, request, last: None };
}

/// The note source of a connection's beat:
/// what the landing thread is stuck in,
/// said again only when the words change.
pub(crate) struct StuckNotice<'a> {
    /// What the landing thread reports about itself.
    board: &'a LandingBoard,
    /// The identity of the request the connection serves,
    /// when it serves one.
    request: Option<&'a str>,
    /// The key of the words said last.
    last: Option<String>,
}

impl NoteSource for StuckNotice<'_> {
    fn note(&mut self) -> Option<ServiceMessage> {
        // A client whose own request is in the group being landed is told so when the write starts, because its commit may be on
        // the branch already; the key changes when the longer words of a minute's stuck write replace the first ones.
        let inside: Option<(String, String)> = match self.request {
            Some(id) => self.board.inside_notice(id),
            None => None,
        };
        if let Some((key, words)) = inside {
            if self.last.as_deref() == Some(key.as_str()) {
                return None;
            }
            self.last = Some(key);
            return Some(ServiceMessage::InsideWrite { text: words });
        }
        let words: String = self.board.describe()?;
        if self.last.as_deref() == Some(words.as_str()) {
            return None;
        }
        self.last = Some(words.clone());
        return Some(ServiceMessage::Waiting { text: words });
    }
}

/// Reads one request line from a new connection,
/// checks who sent it,
/// and queues it.
pub(crate) fn handle_connection(stream: UnixStream, queue: &Sender<Job>, work: &Sender<Wake>, busy: &AtomicUsize, board: &LandingBoard, release: &str, instance: Option<&str>) {
    let accepted: Instant = Instant::now();
    send(&stream, &ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(release.to_string()) });
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    if peer_user(&stream) != Some(unsafe { libc::geteuid() }) {
        return;
    }
    let _timeout = stream.set_read_timeout(Some(REQUEST_READ_TIMEOUT));
    let Ok(reader_stream) = stream.try_clone() else { return };
    let mut line: String = String::new();
    let limit: u64 = u64::try_from(MAX_LINE_BYTES).unwrap_or(u64::MAX) + 1;
    if BufReader::new(reader_stream.take(limit)).read_line(&mut line).is_err() {
        return;
    }
    let _cleared = stream.set_read_timeout(None);
    if line.is_empty() {
        // A connection that only looked (a readiness probe) sends nothing and is not an error.
        return;
    }
    // A message about another worktree is not this instance's to act on: each worktree has its own instance, so one that
    //  reaches the wrong socket is a mistake of the sender's, refused where it can be seen.
    match decode_client(line.trim_end_matches('\n')) {
        Ok(ClientMessage::Commit(request)) if is_foreign(instance, request.git_dir.as_path()) => {
            crate::service_warn!("refused a commit for {}, which another instance serves", request.git_dir.display());
            send(&stream, &ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::Rejected });
        }
        Ok(ClientMessage::Scratch { git_dir, .. } | ClientMessage::Work { git_dir, .. }) if is_foreign(instance, git_dir.as_path()) => {
            crate::service_warn!("refused a message for {}, which another instance serves", git_dir.display());
            send(&stream, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected });
        }
        Ok(ClientMessage::Commit(request)) => {
            let id: String = request.id.clone();
            let (reply, messages) = Reply::new();
            let _queued = queue.send(Job::Commit(request, reply, Arrival { accepted, read: accepted.elapsed() }));
            crate::service_fault::queued(id.as_str());
            // This thread has nothing else to do until the request is answered, so it writes the answers the landing thread
            //  queues and keeps the client told that the service lives, and what it is stuck in when it is stuck in a write.
            write_until_answered_noting(&stream, &messages, HEARTBEAT_EVERY, &mut stuck_notice(board, Some(id.as_str())));
        }
        Ok(ClientMessage::Withdraw { id }) => {
            // Behind a write that never returns the landing thread answers nothing, so a request that has not begun to land is
            //  withdrawn here (`service_busy.rs`); otherwise the landing thread answers between two landings, so the answer is
            //  either that the commit landed or that it never will, and meanwhile this thread keeps the client told that the
            //  service lives.
            let withdrawing: String = id.clone();
            match board.withdraw_early(id.as_str()) {
                EarlyAnswer::Landed(commit) => send(&stream, &ServiceMessage::Landed { id, commit, already: true }),
                EarlyAnswer::Withdrawn(words) => {
                    send(&stream, &ServiceMessage::Waiting { text: words });
                    send(&stream, &ServiceMessage::Withdrawn { id });
                }
                EarlyAnswer::Queued => {
                    let (reply, messages) = Reply::new();
                    let _queued = queue.send(Job::Withdraw(id, reply));
                    // The connection keeps the identity of the request, so a client that withdrew too early to have been told it is
                    // inside the write is told now.
                    write_until_answered_noting(&stream, &messages, HEARTBEAT_EVERY, &mut stuck_notice(board, Some(withdrawing.as_str())));
                }
            }
        }
        Ok(ClientMessage::Retire) => {
            log(retire_line(release).as_str());
            let _queued = queue.send(Job::Retire);
        }
        Ok(ClientMessage::Scratch { git_dir, path }) => {
            // The connection is the client's claim on the directory. While it is open the service is not idle, and when it
            //  closes, by the client's end or its death, the directory is no one's.
            busy.fetch_add(1, Ordering::SeqCst);
            wait_for_close(&stream);
            busy.fetch_sub(1, Ordering::SeqCst);
            let _unused = git_dir;
            let _queued = queue.send(Job::Release(path));
        }
        Ok(ClientMessage::Work { git_dir, review_key }) => {
            let _queued = work.send((git_dir, review_key));
            let _touched = queue.send(Job::Activity);
        }
        Err(reason) => {
            crate::service_warn!("refused a request line: {reason}");
            send(&stream, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected });
        }
    }
}

/// What:
///  Whether a Git directory belongs to another instance of the service than this one.
/// Why:
///  Each worktree has its own instance,
/// so a message that reaches the wrong socket is a mistake of the sender's and is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isForeign = (instance: string | undefined, gitDir: string) => instance !== undefined && instanceOf(gitDir) !== instance;
/// ```
fn is_foreign(instance: Option<&str>, git_dir: &std::path::Path) -> bool {
    return match instance {
        Some(mine) => instance_of(git_dir) != mine,
        None => false,
    };
}
