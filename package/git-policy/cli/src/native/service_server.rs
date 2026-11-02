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

use super::service_journal::{DONE_RETAINED, landed_commits, prune_done};
use super::service_index_lock::{Announce, unsettled_locks};
use super::service_land::{LandContext, Outcome, land_batch_timed, recover, recover_instance};
use super::service_timing::{Answered, Arrival, ExitReason, LandTiming, answered_line, exit_line, log, retire_line};
use super::private_storage::remove_tree;
use super::service_instance::instance_of;
use super::claim_time::ClaimTime;
use super::service_retry::RetrySchedule;
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
use std::path::PathBuf;
use std::time::{Duration, Instant};

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
    /// The worktree this instance serves, as `service_instance::instance_of` names it; every message about another worktree
    ///  is refused. Nothing in a development run that serves any worktree.
    pub instance: Option<String>,
}

impl ServeSettings {
    /// The settings of an ordinary run: the given idle exit and this build's release.
    pub fn new(idle: Duration) -> Self {
        return Self { idle, release: String::from(RELEASE), instance: None };
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
        Bind::Path(path) => bind_path(path),
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

/// What:
///  Binds a socket file, taking over one that no service listens on.
/// Why:
///   A service that was killed leaves its socket file behind, and the next bind of the same path then fails with
///  "Address already in use" although nothing answers there. Only a socket that refuses a connection is replaced: one that
///  answers belongs to a running service, and any other file is the person's. Two services started on one path at the same
///  moment are the caller's to avoid (the service manager's socket, which production uses, never comes here).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return listen(path) } catch (inUse) { if (await refusesConnections(path)) { unlink(path); return listen(path) } throw inUse }
/// ```
fn bind_path(path: &std::path::Path) -> std::io::Result<UnixListener> {
    use std::os::unix::fs::FileTypeExt;
    let first: std::io::Error = match UnixListener::bind(path) {
        Ok(listener) => return Ok(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => error,
        Err(error) => return Err(error),
    };
    let is_socket: bool = std::fs::symlink_metadata(path).is_ok_and(|meta| { return meta.file_type().is_socket(); });
    let refused: bool = matches!(UnixStream::connect(path), Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused);
    if !is_socket || !refused {
        return Err(first);
    }
    std::fs::remove_file(path)?;
    return UnixListener::bind(path);
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

/// A request waiting to be landed:
/// what the client sent,
/// the connection to answer it on,
/// and how it arrived.
type Queued = (Box<CommitRequest>, UnixStream, Arrival);

enum Job {
    Commit(Box<CommitRequest>, UnixStream, Arrival),
    Retire,
    /// After-commit work was announced; it only restarts the idle clock.
    Activity,
    /// A client that held a commit's scratch directory is gone; the directory is removed between batches, never during a
    ///  landing that may read it.
    Release(std::path::PathBuf),
}

/// Returns when the other end of the connection has closed it, or the connection failed. Anything the client sends is ignored.
fn wait_for_close(stream: &UnixStream) {
    let mut sink: [u8; 64] = [0; 64];
    let mut reader = stream;
    loop {
        match reader.read(&mut sink) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
    }
}

fn send(stream: &UnixStream, message: &ServiceMessage) {
    let mut line: String = encode_service(message);
    line.push('\n');
    // A client that went away has nothing to be told.
    let _written = (&*stream).write_all(line.as_bytes());
}

/// Starts a thread the service cannot do without, or ends the service with a plain line and status 4: a service that goes on
///  without its acceptor or its work thread answers no one, while an ended service is started again by the manager.
fn start_thread_or_exit(what: &str, body: impl FnOnce() + Send + 'static) {
    if let Err(error) = std::thread::Builder::new().spawn(body) {
        crate::service_log!("cannot start the thread for {what}: {error}");
        std::process::exit(4);
    }
}

/// How long the acceptor waits after a connection it could not give a thread.
const ACCEPT_BACKOFF: Duration = Duration::from_millis(100);

/// Reads one request line from a new connection, checks who sent it, and queues it.
fn handle_connection(stream: UnixStream, queue: &Sender<Job>, work: &Sender<Wake>, busy: &AtomicUsize, release: &str, instance: Option<&str>) {
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
    let foreign = |git_dir: &std::path::Path| -> bool { return instance.is_some_and(|mine| { return instance_of(git_dir) != mine; }); };
    match decode_client(line.trim_end_matches('\n')) {
        Ok(ClientMessage::Commit(request)) if foreign(request.git_dir.as_path()) => {
            crate::service_log!("refused a commit for {}, which another instance serves", request.git_dir.display());
            send(&stream, &ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::Rejected });
        }
        Ok(ClientMessage::Scratch { git_dir, .. } | ClientMessage::Work { git_dir, .. }) if foreign(git_dir.as_path()) => {
            crate::service_log!("refused a message for {}, which another instance serves", git_dir.display());
            send(&stream, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected });
        }
        Ok(ClientMessage::Commit(request)) => {
            let _queued = queue.send(Job::Commit(request, stream, Arrival { accepted, read: accepted.elapsed() }));
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
            crate::service_log!("refused a request line: {reason}");
            send(&stream, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Rejected });
        }
    }
}

/// Finishes the open batches this instance owns: those of its worktree when it serves one, all of them in a development run.
fn settle_open_batches(context: &LandContext, instance: Option<&str>) -> Vec<super::service_land::RecoveryReport> {
    return match instance {
        Some(mine) => recover_instance(context, mine),
        None => recover(context),
    };
}

/// The times of one batch that every request of it shares,
/// and the moment the landing ended.
struct BatchTimes {
    /// The batch number,
    /// zero when no batch ran (a request answered as already landed).
    batch: u64,
    /// How many requests of the branch the batch held.
    size: usize,
    /// When the group began to be handled,
    /// which ends every request's wait in the queue.
    group_began: Instant,
    /// Time to settle the batches a killed service left open.
    settle: Duration,
    /// The phases of the landing.
    landing: LandTiming,
    /// Time the landing took,
    /// from its start to its return.
    land_total: Duration,
    /// When the landing returned,
    /// which starts the answers.
    landed_at: Instant,
}

impl BatchTimes {
    /// The times of a request that no batch landed.
    fn not_landed(group_began: Instant) -> Self {
        return Self { batch: 0, size: 0, group_began, settle: Duration::ZERO, landing: LandTiming::default(), land_total: Duration::ZERO, landed_at: Instant::now() };
    }
}

/// Writes the log line for an answered request:
/// the phases of its wait,
/// from the acceptance of its connection to the answer.
fn log_answer(times: &BatchTimes, request: &CommitRequest, arrival: &Arrival, position: usize, outcome: &str) {
    let queued: Duration = times.group_began.saturating_duration_since(arrival.accepted + arrival.read);
    log(answered_line(&Answered {
        request: request.id.as_str(),
        reference: request.reference.as_str(),
        outcome,
        batch: times.batch,
        size: times.size,
        position,
        arrival: *arrival,
        queued,
        settle: times.settle,
        landing: &times.landing,
        land_total: times.land_total,
        answer: times.landed_at.elapsed(),
        total: arrival.accepted.elapsed(),
    })
    .as_str());
}

/// Lands one group of jobs that share a branch, and answers each client.
fn land_group(context: &LandContext, instance: Option<&str>, seq: &mut u64, group: Vec<Queued>, landed: &mut HashMap<String, String>) {
    let group_began: Instant = Instant::now();
    let (again, pending): (Vec<_>, Vec<_>) = group.into_iter().partition(|(request, _, _)| { return landed.contains_key(&request.id); });
    let unlanded: BatchTimes = BatchTimes::not_landed(group_began);
    for (request, stream, arrival) in again {
        send(&stream, &ServiceMessage::Landed { id: request.id.clone(), commit: landed[&request.id].clone(), already: true });
        log_answer(&unlanded, &request, &arrival, 1, "already");
    }
    // The same request sent twice into one batch (a client that reconnected) is landed once and answered twice.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let (fresh, repeats): (Vec<_>, Vec<_>) = pending.into_iter().partition(|(request, _, _)| { return seen.insert(request.id.clone()); });
    if fresh.is_empty() {
        return;
    }
    // A batch whose branch moved but whose index update had to give up (another program held Git's index lock the whole
    //  time) left its intent open; settling it first puts the index right before anything else changes it.
    let settling: Instant = Instant::now();
    let _settled = settle_open_batches(context, instance);
    let settle: Duration = settling.elapsed();
    *seq += 1;
    for (index, (_, stream, _)) in fresh.iter().enumerate().skip(1) {
        send(stream, &ServiceMessage::Position { number: index + 1, of: fresh.len() });
    }
    let requests: Vec<CommitRequest> = fresh.iter().map(|(request, _, _)| { return (**request).clone(); }).collect();
    let announce = |note: Announce| {
        for (_, stream, _) in &fresh {
            match &note {
                Announce::Waiting(text) => send(stream, &ServiceMessage::Waiting { text: text.clone() }),
                Announce::Holder(who) => send(stream, &ServiceMessage::Holder { who: who.clone() }),
            }
        }
    };
    let mut landing: LandTiming = LandTiming::default();
    let landing_began: Instant = Instant::now();
    let result = land_batch_timed(context, *seq, &requests, &announce, &mut landing);
    let times: BatchTimes = BatchTimes { batch: *seq, size: fresh.len(), group_began, settle, landing, land_total: landing_began.elapsed(), landed_at: Instant::now() };
    match result {
        Ok(outcomes) => {
            super::service_fault::reach("before-ack");
            for ((request, _, _), outcome) in fresh.iter().zip(&outcomes) {
                if let Outcome::Landed { commit } = outcome {
                    for (repeat, stream, arrival) in repeats.iter().filter(|(repeat, _, _)| { return repeat.id == request.id; }) {
                        send(stream, &ServiceMessage::Landed { id: repeat.id.clone(), commit: commit.clone(), already: true });
                        log_answer(&times, repeat, arrival, 1, "already");
                    }
                }
            }
            for (position, ((request, stream, arrival), outcome)) in fresh.iter().zip(outcomes).enumerate() {
                let word: &str = match outcome {
                    Outcome::Landed { commit } => {
                        landed.insert(request.id.clone(), commit.clone());
                        send(stream, &ServiceMessage::Landed { id: request.id.clone(), commit, already: false });
                        "landed"
                    }
                    Outcome::RetryPrepare { base } => {
                        let captures = super::service_captures::remembered(request.common_dir.as_path(), request.repository.as_path());
                        send(stream, &ServiceMessage::RetryPrepare { id: request.id.clone(), base, landed: captures });
                        "retry-prepare"
                    }
                    Outcome::BranchMoved => {
                        send(stream, &ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::BranchMoved });
                        "branch-moved"
                    }
                };
                log_answer(&times, request, arrival, position + 1, word);
            }
        }
        Err(code) => {
            for (position, (request, stream, arrival)) in fresh.iter().enumerate() {
                send(stream, &ServiceMessage::Failed { id: request.id.clone(), code });
                log_answer(&times, request, arrival, position + 1, format!("failed:{}", code.word()).as_str());
            }
        }
    }
}

/// What:
///  Runs the after-commit work of each repository named, one repository at a time, folding wakes that arrive while it
///  works into the next round, and looks again at a repository whose claims another instance held when their lease is over.
/// Why:
///   Draining claims everything pending, so a burst of wakes for one repository is one pass. The instance that held a claim
///  may have died, and nothing but this loop would then wake the work: it waits for a wake no longer than the first lease still
///  running, and the service counts itself busy meanwhile, so it does not exit idle with work claimed by a dead holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const dir of wakesOrDueRetries) await drain(dir, handlers)
/// ```
fn work_loop(receiver: &Receiver<Wake>, handlers: &GitHandlers, busy: &AtomicUsize) {
    let mut retries: RetrySchedule = RetrySchedule::default();
    let mut counted_waiting: bool = false;
    loop {
        let first: Option<Wake> = match retries.next_wait(Instant::now()) {
            None => match receiver.recv() {
                Ok(wake) => Some(wake),
                Err(_) => return,
            },
            Some(wait) => match receiver.recv_timeout(wait) {
                Ok(wake) => Some(wake),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => return,
            },
        };
        busy.fetch_add(1, Ordering::SeqCst);
        let mut wakes: Vec<Wake> = first.into_iter().chain(std::iter::from_fn(|| { return receiver.try_recv().ok(); })).collect();
        wakes.extend(retries.take_due(Instant::now()).into_iter().map(|directory| { return (directory, None); }));
        let ran: Vec<PathBuf> = wakes.iter().map(|(directory, _)| { return directory.clone(); }).collect();
        let asked: Vec<(PathBuf, Duration)> = run_round(wakes, handlers, &ClaimTime::read());
        retries.replace(ran.as_slice(), asked, Instant::now());
        // A repository waiting to be looked at again keeps the service from exiting idle, as running work does.
        if !retries.is_empty() && !counted_waiting {
            busy.fetch_add(1, Ordering::SeqCst);
            counted_waiting = true;
        } else if retries.is_empty() && counted_waiting {
            busy.fetch_sub(1, Ordering::SeqCst);
            counted_waiting = false;
        }
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
    let serving_since: Instant = Instant::now();
    let idle: Duration = settings.idle;
    let _reports = settle_open_batches(context, settings.instance.as_deref());
    // The memory of landed requests is bounded: the oldest finished batches go before the rest are read.
    let pruned: usize = prune_done(context.journal.as_path(), DONE_RETAINED);
    if pruned > 0 {
        crate::service_log!("removed the records of {pruned} old finished batches from {}", context.journal.display());
    }
    let mut landed: HashMap<String, String> = landed_commits(context.journal.as_path());
    let (sender, receiver): (Sender<Job>, Receiver<Job>) = channel();
    let acceptor_queue: Sender<Job> = sender.clone();
    // After-commit work (push, upkeep) runs on its own thread, so a slow network never delays a landing.
    let (work_sender, work_receiver): (Sender<Wake>, Receiver<Wake>) = channel();
    let handlers: GitHandlers = GitHandlers::new(context.git.clone());
    let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let worker_busy: Arc<AtomicUsize> = Arc::clone(&busy);
    start_thread_or_exit("the after-commit work", move || { work_loop(&work_receiver, &handlers, &worker_busy); });
    let announced_release: String = settings.release.clone();
    let connection_busy: Arc<AtomicUsize> = Arc::clone(&busy);
    let instance: Option<String> = settings.instance.clone();
    start_thread_or_exit("the acceptor of connections", move || {
        for connection in listener.incoming().flatten() {
            let queue: Sender<Job> = acceptor_queue.clone();
            let work: Sender<Wake> = work_sender.clone();
            let release: String = announced_release.clone();
            let counted: Arc<AtomicUsize> = Arc::clone(&connection_busy);
            let mine: Option<String> = instance.clone();
            // A connection the service cannot give a thread is told so and closed; the acceptor goes on, because a panic here
            //  would end the acceptor and leave a service that is running and answers no one.
            let notice: Option<UnixStream> = connection.try_clone().ok();
            let spawned = std::thread::Builder::new().spawn(move || { handle_connection(connection, &queue, &work, &counted, release.as_str(), mine.as_deref()); });
            if let Err(error) = spawned {
                crate::service_log!("no thread for a connection, which is closed after telling the client: {error}");
                if let Some(stream) = notice {
                    send(&stream, &ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(announced_release.clone()) });
                    send(&stream, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Unavailable });
                }
                // Starting a thread keeps failing while the pressure lasts; waiting a moment keeps the acceptor from spinning on it.
                std::thread::sleep(ACCEPT_BACKOFF);
            }
        }
    });
    // Microseconds, so that instances started at nearly the same time do not number their batches alike in a shared journal.
    let mut seq: u64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |elapsed| { return u64::try_from(elapsed.as_micros()).unwrap_or(1); });
    let mut retiring: bool = false;
    while !retiring {
        let first: Job = match receiver.recv_timeout(idle) {
            Ok(job) => job,
            // Work in flight keeps the service; otherwise it exits and the manager starts it on the next connection.
            Err(RecvTimeoutError::Timeout) if busy.load(Ordering::SeqCst) > 0 => continue,
            Err(RecvTimeoutError::Timeout) => {
                log(exit_line(ExitReason::Idle, serving_since.elapsed()).as_str());
                return;
            }
            Err(RecvTimeoutError::Disconnected) => {
                log(exit_line(ExitReason::Error, serving_since.elapsed()).as_str());
                return;
            }
        };
        let mut jobs: Vec<Job> = vec![first];
        while jobs.len() < MAX_BATCH {
            match receiver.try_recv() {
                Ok(job) => jobs.push(job),
                Err(_) => break,
            }
        }
        // Branches in order of first arrival; within a branch, arrival order.
        let mut groups: Vec<((std::path::PathBuf, String, bool), Vec<Queued>)> = Vec::new();
        let mut releases: Vec<std::path::PathBuf> = Vec::new();
        for job in jobs {
            match job {
                Job::Retire => retiring = true,
                Job::Activity => {}
                Job::Release(path) => releases.push(path),
                Job::Commit(request, stream, arrival) => {
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
                        Some((existing, members)) if !key.2 && !existing.2 => members.push((request, stream, arrival)),
                        _ => groups.push((key, vec![(request, stream, arrival)])),
                    }
                }
            }
        }
        for (_, group) in groups {
            land_group(context, settings.instance.as_deref(), &mut seq, group, &mut landed);
        }
        // A lock this thread could not settle names this process: the restart reclaims it once this process is gone.
        if unsettled_locks() > 0 {
            crate::service_log!("ending, because an index lock could not be settled; the next connection starts the service again and its recovery reclaims the lock");
            return;
        }
        for path in releases {
            if let Err(error) = remove_tree(path.as_path()) {
                crate::service_log!("the scratch directory {} could not be removed: {error}", path.display());
            }
        }
    }
    log(exit_line(ExitReason::Retire, serving_since.elapsed()).as_str());
}

/// Socket controls stay out of the release executable.
#[cfg(test)]
#[path = "service_server_tests.rs"]
mod tests;
