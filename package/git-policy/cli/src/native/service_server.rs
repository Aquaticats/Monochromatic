//! What:
//!  The resident service's loop: accept connections, queue requests in arrival order, land what is queued as a batch,
//!  answer each client, and exit when told to retire.
//! Why:
//!  One writer lands every commit, so there is no lock to take, hold or wait for: order is the order of the queue, the
//!  cost of the one durable write is shared by the batch, and a client that waits blocks on its socket instead of
//!  polling (`doc/planning/cli-git-resident-service.md`). The service never starts itself: the operating system hands it
//!  the listening socket (socket activation), or a development run binds one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const batch of queue.batches()) { const outcomes = await landBatch(batch); for (...) reply(outcome); }
//! ```

use super::service_journal::landed_commits;
use super::service_land::{LandContext, Outcome, land_batch, recover};
use super::service_protocol::{
    ClientMessage, CommitRequest, FailureCode, MAX_LINE_BYTES, PROTOCOL_VERSION, ServiceMessage, decode_client, encode_service,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// The most requests landed as one batch.
pub const MAX_BATCH: usize = 64;

/// How long a connection may take to send its request line before it is dropped.
pub const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// File descriptor the service manager passes the listening socket on (`SD_LISTEN_FDS_START`).
pub const INHERITED_DESCRIPTOR: i32 = 3;

/// Where the listening socket comes from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Bind {
    /// A Linux abstract name: one binder at a time, gone with the process, no file to go stale.
    Abstract(String),
    /// A path in a private directory, for platforms without abstract names.
    Path(std::path::PathBuf),
    /// The socket the service manager opened and passed in.
    Inherited,
}

/// What:
///  Opens the listening socket.
/// Why:
///   Socket activation hands over a bound socket, which is the normal way; the others serve tests and development runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listener = bind(address)
/// ```
pub fn bind(address: &Bind) -> std::io::Result<UnixListener> {
    return match address {
        #[cfg(target_os = "linux")]
        Bind::Abstract(name) => {
            use std::os::linux::net::SocketAddrExt;
            let socket = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes())?;
            UnixListener::bind_addr(&socket)
        }
        #[cfg(not(target_os = "linux"))]
        Bind::Abstract(_) => Err(std::io::Error::other("abstract sockets need Linux")),
        Bind::Path(path) => UnixListener::bind(path),
        Bind::Inherited => {
            use std::os::fd::FromRawFd;
            let announced_for: bool = std::env::var("LISTEN_PID").is_ok_and(|pid| { return pid == std::process::id().to_string(); });
            let count: i32 = std::env::var("LISTEN_FDS").ok().and_then(|text| { return text.parse().ok(); }).unwrap_or(0);
            if !announced_for || count < 1 {
                return Err(std::io::Error::other("no socket was passed in"));
            }
            // SAFETY: the manager passed this descriptor to this process and nothing else in the process owns it.
            Ok(unsafe { UnixListener::from_raw_fd(INHERITED_DESCRIPTOR) })
        }
    };
}

/// The user id of the process on the other end of a connection.
fn peer_user(stream: &UnixStream) -> Option<u32> {
    use std::os::fd::AsRawFd;
    let mut credentials = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: `credentials` and `length` are valid for the call and `length` is its size.
    let status = unsafe {
        libc::getsockopt(stream.as_raw_fd(), libc::SOL_SOCKET, libc::SO_PEERCRED, (&raw mut credentials).cast(), &raw mut length)
    };
    return if status == 0 { Some(credentials.uid) } else { None };
}

enum Job {
    Commit(Box<CommitRequest>, UnixStream),
    Retire,
}

fn send(stream: &UnixStream, message: &ServiceMessage) {
    let mut line: String = encode_service(message);
    line.push('\n');
    // A client that went away has nothing to be told.
    let _written = (&*stream).write_all(line.as_bytes());
}

/// Reads one request line from a new connection, checks who sent it, and queues it.
fn handle_connection(stream: UnixStream, queue: &Sender<Job>) {
    send(&stream, &ServiceMessage::Welcome { version: PROTOCOL_VERSION });
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
    let _timeout = stream.set_read_timeout(None);
    if line.is_empty() {
        // A connection that only looked (a readiness probe) sends nothing and is not an error.
        return;
    }
    match decode_client(line.trim_end_matches('\n')) {
        Ok(ClientMessage::Commit(request)) => {
            let _queued = queue.send(Job::Commit(Box::new(request), stream));
        }
        Ok(ClientMessage::Retire) => {
            let _queued = queue.send(Job::Retire);
        }
        Err(reason) => {
            eprintln!("cli-git-service: refused a request line: {reason}");
            send(&stream, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected });
        }
    }
}

/// Lands one group of jobs that share a branch, and answers each client.
fn land_group(context: &LandContext, seq: &mut u64, group: Vec<(Box<CommitRequest>, UnixStream)>, landed: &mut HashMap<String, String>) {
    let (again, fresh): (Vec<_>, Vec<_>) = group.into_iter().partition(|(request, _)| { return landed.contains_key(&request.id); });
    for (request, stream) in again {
        send(&stream, &ServiceMessage::Landed { id: request.id.clone(), commit: landed[&request.id].clone(), already: true });
    }
    // The same request sent twice into one batch (a client that reconnected) is landed once and answered twice.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let (fresh, repeats): (Vec<_>, Vec<_>) = fresh.into_iter().partition(|(request, _)| { return seen.insert(request.id.clone()); });
    if fresh.is_empty() {
        return;
    }
    *seq += 1;
    for (index, (_, stream)) in fresh.iter().enumerate().skip(1) {
        send(stream, &ServiceMessage::Position { number: index + 1, of: fresh.len() });
    }
    let requests: Vec<CommitRequest> = fresh.iter().map(|(request, _)| { return (**request).clone(); }).collect();
    match land_batch(context, *seq, &requests) {
        Ok(outcomes) => {
            super::service_fault::reach("before-ack");
            for ((request, _), outcome) in fresh.iter().zip(&outcomes) {
                if let Outcome::Landed { commit } = outcome {
                    for (repeat, stream) in repeats.iter().filter(|(repeat, _)| { return repeat.id == request.id; }) {
                        send(stream, &ServiceMessage::Landed { id: repeat.id.clone(), commit: commit.clone(), already: true });
                    }
                }
            }
            for ((request, stream), outcome) in fresh.iter().zip(outcomes) {
                match outcome {
                    Outcome::Landed { commit } => {
                        landed.insert(request.id.clone(), commit.clone());
                        send(stream, &ServiceMessage::Landed { id: request.id.clone(), commit, already: false });
                    }
                    Outcome::RetryPrepare { base } => send(stream, &ServiceMessage::RetryPrepare { id: request.id.clone(), base }),
                }
            }
        }
        Err(code) => {
            for (request, stream) in &fresh {
                send(stream, &ServiceMessage::Failed { id: request.id.clone(), code });
            }
        }
    }
}

/// What:
///  Runs the service until it is told to retire.
/// Why:
///   Recovery runs before the first connection is accepted; then the loop takes everything queued as one batch, groups it
///  by branch in order of first arrival, lands each group, and answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await serve(listener, context)
/// ```
pub fn serve(listener: UnixListener, context: &LandContext) {
    let _reports = recover(context);
    let mut landed: HashMap<String, String> = landed_commits(context.journal.as_path());
    let (sender, receiver): (Sender<Job>, Receiver<Job>) = channel();
    let acceptor_queue: Sender<Job> = sender.clone();
    std::thread::spawn(move || {
        for connection in listener.incoming().flatten() {
            let queue: Sender<Job> = acceptor_queue.clone();
            std::thread::spawn(move || { handle_connection(connection, &queue); });
        }
    });
    let mut seq: u64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |elapsed| { return u64::try_from(elapsed.as_millis()).unwrap_or(1); });
    let mut retiring: bool = false;
    while !retiring {
        let Ok(first) = receiver.recv() else { return };
        let mut jobs: Vec<Job> = vec![first];
        while jobs.len() < MAX_BATCH {
            match receiver.try_recv() {
                Ok(job) => jobs.push(job),
                Err(_) => break,
            }
        }
        // Branches in order of first arrival; within a branch, arrival order.
        let mut groups: Vec<((std::path::PathBuf, String), Vec<(Box<CommitRequest>, UnixStream)>)> = Vec::new();
        for job in jobs {
            match job {
                Job::Retire => retiring = true,
                Job::Commit(request, stream) => {
                    let key = (request.common_dir.clone(), request.reference.clone());
                    match groups.iter_mut().find(|(existing, _)| { return *existing == key; }) {
                        Some((_, members)) => members.push((request, stream)),
                        None => groups.push((key, vec![(request, stream)])),
                    }
                }
            }
        }
        for (_, group) in groups {
            land_group(context, &mut seq, group, &mut landed);
        }
    }
}
