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
use super::service_index_lock::Announce;
use super::service_land::{LandContext, Outcome, land_batch_announcing, recover};
use super::service_work::{GitHandlers, Wake, run_round};
use super::service_protocol::{
    ClientMessage, CommitRequest, FailureCode, MAX_LINE_BYTES, PROTOCOL_VERSION, RELEASE, ServiceMessage, decode_client, encode_service,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::Arc;
use std::time::Duration;

/// The most requests landed as one batch.
pub const MAX_BATCH: usize = 64;

/// How long a connection may take to send its request line before it is dropped.
pub const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// How long the service waits with nothing to do before it exits; the manager's socket stays, so the next connection starts it
///  again. Bazel's server does the same with `--max_idle_secs`, whose default is three hours, which is this value's precedent.
pub const IDLE_EXIT: Duration = Duration::from_secs(3 * 60 * 60);

/// File descriptor the service manager passes the listening socket on (`SD_LISTEN_FDS_START`).
pub const INHERITED_DESCRIPTOR: i32 = 3;

/// How a service run behaves apart from where it listens and what it lands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServeSettings {
    /// How long the service waits with nothing to do before it exits.
    pub idle: Duration,
    /// The release the service announces to every client. It is this build's release (`RELEASE`) except in a test that
    ///  needs a service of another release.
    pub release: String,
}

impl ServeSettings {
    /// The settings of an ordinary run: the given idle exit and this build's release.
    pub fn new(idle: Duration) -> Self {
        return Self { idle, release: String::from(RELEASE) };
    }
}

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
    /// After-commit work was announced; it only restarts the idle clock.
    Activity,
}

fn send(stream: &UnixStream, message: &ServiceMessage) {
    let mut line: String = encode_service(message);
    line.push('\n');
    // A client that went away has nothing to be told.
    let _written = (&*stream).write_all(line.as_bytes());
}

/// Reads one request line from a new connection, checks who sent it, and queues it.
fn handle_connection(stream: UnixStream, queue: &Sender<Job>, work: &Sender<Wake>, release: &str) {
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
    match decode_client(line.trim_end_matches('\n')) {
        Ok(ClientMessage::Commit(request)) => {
            let _queued = queue.send(Job::Commit(request, stream));
        }
        Ok(ClientMessage::Retire) => {
            let _queued = queue.send(Job::Retire);
        }
        Ok(ClientMessage::Work { common_dir, review_key }) => {
            let _queued = work.send((common_dir, review_key));
            let _touched = queue.send(Job::Activity);
        }
        Err(reason) => {
            eprintln!("cli-git-service: refused a request line: {reason}");
            send(&stream, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected });
        }
    }
}

/// Lands one group of jobs that share a branch, and answers each client.
fn land_group(context: &LandContext, seq: &mut u64, group: Vec<(Box<CommitRequest>, UnixStream)>, landed: &mut HashMap<String, String>) {
    let (again, pending): (Vec<_>, Vec<_>) = group.into_iter().partition(|(request, _)| { return landed.contains_key(&request.id); });
    for (request, stream) in again {
        send(&stream, &ServiceMessage::Landed { id: request.id.clone(), commit: landed[&request.id].clone(), already: true });
    }
    // The same request sent twice into one batch (a client that reconnected) is landed once and answered twice.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let (fresh, repeats): (Vec<_>, Vec<_>) = pending.into_iter().partition(|(request, _)| { return seen.insert(request.id.clone()); });
    if fresh.is_empty() {
        return;
    }
    // A batch whose branch moved but whose index update had to give up (another program held Git's index lock the whole
    //  time) left its intent open; settling it first puts the index right before anything else changes it.
    // A batch whose branch moved but whose index update had to give up (another program held Git's index lock the whole
    //  time) left its intent open; settling it first puts the index right before anything else changes it.
    let _settled = recover(context);
    *seq += 1;
    for (index, (_, stream)) in fresh.iter().enumerate().skip(1) {
        send(stream, &ServiceMessage::Position { number: index + 1, of: fresh.len() });
    }
    let requests: Vec<CommitRequest> = fresh.iter().map(|(request, _)| { return (**request).clone(); }).collect();
    let announce = |note: Announce| {
        for (_, stream) in &fresh {
            match &note {
                Announce::Waiting(text) => send(stream, &ServiceMessage::Waiting { text: text.clone() }),
                Announce::Holder(who) => send(stream, &ServiceMessage::Holder { who: who.clone() }),
            }
        }
    };
    match land_batch_announcing(context, *seq, &requests, &announce) {
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
                    Outcome::RetryPrepare { base } => {
                        let captures = super::service_captures::remembered(request.common_dir.as_path(), request.repository.as_path());
                        send(stream, &ServiceMessage::RetryPrepare { id: request.id.clone(), base, landed: captures });
                    }
                    Outcome::BranchMoved => send(stream, &ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::BranchMoved }),
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
///  Runs the after-commit work of each repository named, one repository at a time, folding wakes that arrive while it
///  works into the next round.
/// Why:
///   Draining claims everything pending, so a burst of wakes for one repository is one pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const dir of wakes) await drain(dir, handlers)
/// ```
fn work_loop(receiver: &Receiver<Wake>, handlers: &GitHandlers, busy: &AtomicUsize) {
    while let Ok(first) = receiver.recv() {
        busy.fetch_add(1, Ordering::SeqCst);
        let wakes: Vec<Wake> = std::iter::once(first).chain(std::iter::from_fn(|| { return receiver.try_recv().ok(); })).collect();
        run_round(wakes, handlers, std::time::SystemTime::now());
        busy.fetch_sub(1, Ordering::SeqCst);
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
pub fn serve(listener: UnixListener, context: &LandContext, settings: &ServeSettings) {
    let idle: Duration = settings.idle;
    let _reports = recover(context);
    let mut landed: HashMap<String, String> = landed_commits(context.journal.as_path());
    let (sender, receiver): (Sender<Job>, Receiver<Job>) = channel();
    let acceptor_queue: Sender<Job> = sender.clone();
    // After-commit work (push, upkeep) runs on its own thread, so a slow network never delays a landing.
    let (work_sender, work_receiver): (Sender<Wake>, Receiver<Wake>) = channel();
    let handlers: GitHandlers = GitHandlers::new(context.git.clone());
    let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let worker_busy: Arc<AtomicUsize> = Arc::clone(&busy);
    std::thread::spawn(move || { work_loop(&work_receiver, &handlers, &worker_busy); });
    let announced_release: String = settings.release.clone();
    std::thread::spawn(move || {
        for connection in listener.incoming().flatten() {
            let queue: Sender<Job> = acceptor_queue.clone();
            let work: Sender<Wake> = work_sender.clone();
            let release: String = announced_release.clone();
            std::thread::spawn(move || { handle_connection(connection, &queue, &work, release.as_str()); });
        }
    });
    let mut seq: u64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |elapsed| { return u64::try_from(elapsed.as_millis()).unwrap_or(1); });
    let mut retiring: bool = false;
    while !retiring {
        let first: Job = match receiver.recv_timeout(idle) {
            Ok(job) => job,
            // Work in flight keeps the service; otherwise it exits and the manager starts it on the next connection.
            Err(RecvTimeoutError::Timeout) if busy.load(Ordering::SeqCst) > 0 => continue,
            Err(_) => return,
        };
        let mut jobs: Vec<Job> = vec![first];
        while jobs.len() < MAX_BATCH {
            match receiver.try_recv() {
                Ok(job) => jobs.push(job),
                Err(_) => break,
            }
        }
        // Branches in order of first arrival; within a branch, arrival order.
        let mut groups: Vec<((std::path::PathBuf, String, bool), Vec<(Box<CommitRequest>, UnixStream)>)> = Vec::new();
        for job in jobs {
            match job {
                Job::Retire => retiring = true,
                Job::Activity => {}
                Job::Commit(request, stream) => {
                    // An amend is a group of its own, and what arrives after it starts another group, so the order on
                    //  one branch is the order of arrival.
                    // `HEAD` is a head of one worktree, so two detached worktrees of one repository are two branches.
                    let queue_reference: String = if request.reference == "HEAD" {
                        format!("HEAD@{}", request.repository.display())
                    } else {
                        request.reference.clone()
                    };
                    let key = (request.common_dir.clone(), queue_reference, request.lands_alone());
                    let joins = groups.iter_mut().rev().find(|((common, reference, _), _)| { return *common == key.0 && *reference == key.1; });
                    match joins {
                        Some((existing, members)) if !key.2 && !existing.2 => members.push((request, stream)),
                        _ => groups.push((key, vec![(request, stream)])),
                    }
                }
            }
        }
        for (_, group) in groups {
            land_group(context, &mut seq, group, &mut landed);
        }
    }
}
