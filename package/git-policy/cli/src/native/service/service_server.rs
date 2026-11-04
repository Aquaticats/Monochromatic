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

use crate::service_journal::{BatchClaim, DONE_RETAINED, Recovery, claim_batch, landed_commits, prune_done};
use crate::service_land_follow::storage_failure;
use crate::service_index_lock::{Announce, unsettled_locks};
use crate::service_index_prestage::{PrestageGuard, start as start_prestage};
use crate::service_captures::CaptureMemory;
use crate::service_land::{LandContext, Outcome, land_batch_timed, recover, recover_instance};
use crate::journald_native::Priority;
use crate::service_timing::{Answered, Arrival, ExitReason, LandTiming, answered_line, exit_line, log, log_entry, retire_line};
use crate::private_storage::remove_tree;
use crate::service_instance::instance_of;
use crate::claim_time::ClaimTime;
use crate::service_retry::RetrySchedule;
use crate::push_retry::{PLAN_DIRECTORY, PushRetries};
use crate::service_work::{GitHandlers, Wake, run_round};
use crate::service_busy::{EarlyAnswer, LandingBoard};
use crate::service_reply::{Reply, write_until_answered_noting};
use crate::wait_ceilings::NEVER_ENDED_STEP_SLOW_AFTER;
use crate::service_protocol::{
    ClientMessage, CommitRequest, FailureCode, HEARTBEAT_EVERY, MAX_LINE_BYTES, PROTOCOL_VERSION, RELEASE, ServiceMessage, decode_client, encode_service,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::Arc;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The largest batch whose landing still costs about what its size predicts: 256. Measured with the service binary on throwaway
///  repositories, 2026-11-03, bursts of simultaneous commit requests (`MAX_BATCH` raised to 4096 for the probe, three rounds
///  alternating over 64, 128, 256, 512 and 1024 clients, other sessions running, a debug build): a landing took 71 to 76 ms for
///  62 to 63 requests, 82 to 85 ms for 126, 104 to 109 ms for 253 to 255, 193 to 204 ms for 509 to 511 (once 2,002 ms, a host stall)
///  and 525 to 547 ms for 1,021 to 1,023. The cost of each added request is 0.16 ms up to 254 requests, 0.36 ms from there to
///  510, and 0.66 ms to 1,022, so the cost per request has not yet doubled at 256. The time a client waits for a burst of 128 was
///  176 to 206 ms in one landing and 243 to 249 ms in three at the old cap of 64.
pub const BATCH_COST_KNEE: usize = 256;

/// The most requests landed as one batch: `BATCH_COST_KNEE`. A batch is answered all at once and holds the index lock while it
///  lands, so the cap bounds both. The prototype's burst reached 64, and production never reached more than one request in a
///  batch (1,302 answered requests, 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z), so the cap bites only in a burst. Option B of the
///  batch size question, picked by the coordinator under the owner's delegation, open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Picked for these follow-ups").
pub const MAX_BATCH: usize = BATCH_COST_KNEE;

/// How long a connection may take to send its request line before it is dropped.
pub const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// How long the service waits with nothing to do before it exits; the manager's socket stays, so the next connection starts it
///  again. Bazel's server does the same with `--max_idle_secs`, whose default is three hours, which is this value's precedent.
///  Measured cost of exceeding it: of 1,051 commits in the commit timing log (2026-11-02T09:01:37Z to 2026-11-03T15:10:12Z), the 10
///  that followed an idle gap of more than three hours reached the service in at most 70 ms (`reachMs`) after a start wait of at
///  most 47 ms (`startWaitMs`); 110 idle exits happened in that window. The 28 running services held 212 MiB together (7.9 MiB each,
///  `systemctl --user show --property=MemoryCurrent`). A shorter exit would trade at most that start for that memory; the owner's
///  bound is that a commit should not take 5 s longer than native Git (`SPEC.md`). Kept at three hours as the coordinator's pick under the owner's delegation, open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`, "Sizes and ages of the service's own housekeeping").
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
    /// How long a write the service never ends runs before the clients behind it are told, and may be withdrawn while the
    ///  service is inside it (`wait_ceilings.rs`, `NEVER_ENDED_STEP_SLOW_AFTER`); a control shortens it.
    pub stuck_after: Duration,
}

impl ServeSettings {
    /// The settings of an ordinary run: the given idle exit and this build's release.
    pub fn new(idle: Duration) -> Self {
        return Self { idle, release: String::from(RELEASE), instance: None, stuck_after: NEVER_ENDED_STEP_SLOW_AFTER };
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
            ensure_socket_passed_in(std::env::var("LISTEN_PID").ok().as_deref(), std::env::var("LISTEN_FDS").ok().as_deref(), std::process::id())?;
            // SAFETY: the manager passed this descriptor to this process and nothing else in the process owns it.
            Ok(unsafe { UnixListener::from_raw_fd(INHERITED_DESCRIPTOR) })
        }
    };
}

/// What:
///  Checks the service manager's socket-activation variables:
/// `listen_pid` must name this process (`own_pid`) and
///  `listen_fds` must count at least one descriptor.
/// Why:
///   The descriptor the manager passes is taken over as the listening socket,
/// so a variable set for another process,
/// or a
///  count of none,
/// must be refused instead of adopting a descriptor that was never passed to this process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (listenPid !== String(ownPid) || !(Number.parseInt(listenFds ?? '0') >= 1)) { throw new Error('no socket was passed in') }
/// ```
fn ensure_socket_passed_in(listen_pid: Option<&str>, listen_fds: Option<&str>, own_pid: u32) -> std::io::Result<()> {
    let announced_for: bool = listen_pid.is_some_and(|pid| { return pid == own_pid.to_string(); });
    let count: i32 = listen_fds.and_then(|text| { return text.parse().ok(); }).unwrap_or(0);
    if !announced_for || count < 1 {
        return Err(std::io::Error::other("no socket was passed in"));
    }
    return Ok(());
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
type Queued = (Box<CommitRequest>, Reply, Arrival);

/// What a landing group shares:
/// the common Git directory,
/// the queue reference,
/// and whether its request lands alone.
type GroupKey = (std::path::PathBuf, String, bool);

/// What:
///  Puts `member` into the latest group of its common Git directory and queue reference,
/// or into a group of its own when
///  it lands alone,
/// when that group's request does,
/// or when no group of that branch exists yet.
/// Why:
///   Requests of one branch that may share a batch land together and in arrival order;
/// an amend is a group of its own and
///  what arrives after it starts another group,
/// so the order on one branch is the order of arrival,
/// and a request never
///  lands in a batch of another branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const joined = groups.findLast((group) => group.common === key.common && group.reference === key.reference)
/// if (joined && !key.alone && !joined.alone) { joined.members.push(member) } else { groups.push({ key, members: [member] }) }
/// ```
fn place_in_groups<T>(groups: &mut Vec<(GroupKey, Vec<T>)>, key: GroupKey, member: T) {
    let joins = groups.iter_mut().rev().find(|((common, reference, _), _)| { return *common == key.0 && *reference == key.1; });
    match joins {
        Some((existing, members)) if !key.2 && !existing.2 => members.push(member),
        _ => groups.push((key, vec![member])),
    }
}

enum Job {
    Commit(Box<CommitRequest>, Reply, Arrival),
    Retire,
    /// The client stopped waiting for the commit of this identity (its ceiling passed): answered with the landed commit, or
    ///  withdrawn so that it never lands.
    Withdraw(String, Reply),
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
        crate::service_error!("cannot start the thread for {what}: {error}");
        std::process::exit(4);
    }
}

/// How many withdrawn identities the service remembers before it forgets them all: the same count as the finished batches it
///  remembers (`DONE_RETAINED`), since a client that withdrew sends nothing more for its request.
const WITHDRAWN_RETAINED: usize = DONE_RETAINED;

/// How long the acceptor waits after a connection it could not give a thread.
const ACCEPT_BACKOFF: Duration = Duration::from_millis(100);

/// What:
///  The note for a connection's beat: what the landing thread is stuck in when it is inside a write that is never ended, said again
///  only when the words change (they name whole minutes).
/// Why:
///   A client queued behind a write that never returns is told why it waits, and the words become the blocker its error names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// () => { const words = board.describe(); return words !== last ? (last = words, { type: 'waiting', text: words }) : undefined; }
/// ```
fn stuck_notice(board: &LandingBoard) -> impl FnMut() -> Option<ServiceMessage> + '_ {
    let mut last: Option<String> = None;
    return move || {
        let words: String = board.describe()?;
        if last.as_deref() == Some(words.as_str()) {
            return None;
        }
        last = Some(words.clone());
        return Some(ServiceMessage::Waiting { text: words });
    };
}

/// Reads one request line from a new connection, checks who sent it, and queues it.
fn handle_connection(stream: UnixStream, queue: &Sender<Job>, work: &Sender<Wake>, busy: &AtomicUsize, board: &LandingBoard, release: &str, instance: Option<&str>) {
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
            crate::service_warn!("refused a commit for {}, which another instance serves", request.git_dir.display());
            send(&stream, &ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::Rejected });
        }
        Ok(ClientMessage::Scratch { git_dir, .. } | ClientMessage::Work { git_dir, .. }) if foreign(git_dir.as_path()) => {
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
            write_until_answered_noting(&stream, &messages, HEARTBEAT_EVERY, &mut stuck_notice(board));
        }
        Ok(ClientMessage::Withdraw { id }) => {
            // Behind a write that never returns the landing thread answers nothing, so a request that has not begun to land is
            //  withdrawn here (`service_busy.rs`); otherwise the landing thread answers between two landings, so the answer is
            //  either that the commit landed or that it never will, and meanwhile this thread keeps the client told that the
            //  service lives.
            match board.withdraw_early(id.as_str()) {
                EarlyAnswer::Landed(commit) => send(&stream, &ServiceMessage::Landed { id, commit, already: true }),
                EarlyAnswer::Withdrawn(words) => {
                    send(&stream, &ServiceMessage::Waiting { text: words });
                    send(&stream, &ServiceMessage::Withdrawn { id });
                }
                EarlyAnswer::Queued => {
                    let (reply, messages) = Reply::new();
                    let _queued = queue.send(Job::Withdraw(id, reply));
                    write_until_answered_noting(&stream, &messages, HEARTBEAT_EVERY, &mut stuck_notice(board));
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

/// Finishes the open batches this instance owns: those of its worktree when it serves one, all of them in a development run.
fn settle_open_batches(context: &LandContext, instance: Option<&str>) -> Vec<crate::service_land::RecoveryReport> {
    let reports: Vec<crate::service_land::RecoveryReport> = match instance {
        Some(mine) => recover_instance(context, mine),
        None => recover(context),
    };
    // An open batch means an earlier service stopped between its intent and its end; each one settled is said.
    for report in &reports {
        log_entry(
            Priority::Warning,
            format!("settled the batch an earlier service left open batch={} decision={}", report.seq, if report.decision == Recovery::Finish { "finish" } else { "drop" }).as_str(),
            &[("CLI_GIT_BATCH", report.seq.to_string())],
        );
    }
    return reports;
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
    // What the Git steps and tools since the previous answer showed; the answer that finds them takes them.
    let quiet: Vec<(String, Duration)> = crate::quiet_ledger::drain();
    let line: String = answered_line(&Answered {
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
        quiet: quiet.as_slice(),
    });
    // A request that was answered with a failure is a warning, so `journalctl --priority=warning` finds the commits that failed.
    let priority: Priority = if outcome.starts_with("failed") { Priority::Warning } else { Priority::Info };
    let fields: [(&str, String); 4] = [
        ("CLI_GIT_REQUEST_ID", request.id.clone()),
        ("CLI_GIT_BRANCH", request.reference.clone()),
        ("CLI_GIT_OUTCOME", outcome.to_string()),
        ("CLI_GIT_BATCH", times.batch.to_string()),
    ];
    log_entry(priority, line.as_str(), &fields);
}

/// Lands one group of jobs that share a branch, and answers each client.
fn land_group(context: &LandContext, instance: Option<&str>, seq: &mut u64, group: Vec<Queued>, landed: &mut HashMap<String, String>, captures: &mut CaptureMemory) {
    let group_began: Instant = Instant::now();
    let (again, pending): (Vec<_>, Vec<_>) = group.into_iter().partition(|(request, _, _)| { return landed.contains_key(&request.id); });
    let unlanded: BatchTimes = BatchTimes::not_landed(group_began);
    for (request, stream, arrival) in again {
        stream.send(&ServiceMessage::Landed { id: request.id.clone(), commit: landed[&request.id].clone(), already: true });
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
    // The number is taken from the journal, not from this service's count alone: another service of the same journal may count
    //  from the same clock reading, and two landings with one number would overwrite each other's records.
    let claim: BatchClaim = match claim_batch(context.journal.as_path(), *seq + 1) {
        Ok(claim) => claim,
        Err(error) => {
            let code: FailureCode = storage_failure(&error);
            crate::service_error!("no batch number could be taken in {}: {error}", context.journal.display());
            for (position, (request, stream, arrival)) in fresh.iter().enumerate() {
                stream.send(&ServiceMessage::Failed { id: request.id.clone(), code });
                log_answer(&unlanded, request, arrival, position + 1, format!("failed:{}", code.word()).as_str());
            }
            return;
        }
    };
    *seq = claim.seq();
    for (index, (_, stream, _)) in fresh.iter().enumerate().skip(1) {
        stream.send(&ServiceMessage::Position { number: index + 1, of: fresh.len() });
    }
    let requests: Vec<CommitRequest> = fresh.iter().map(|(request, _, _)| { return (**request).clone(); }).collect();
    let announce = |note: Announce| {
        for (_, stream, _) in &fresh {
            match &note {
                Announce::Waiting(text) => stream.send(&ServiceMessage::Waiting { text: text.clone() }),
                Announce::Holder(who) => stream.send(&ServiceMessage::Holder { who: who.clone() }),
            }
        }
    };
    let mut landing: LandTiming = LandTiming::default();
    let landing_began: Instant = Instant::now();
    let result = land_batch_timed(context, *seq, &requests, &announce, &mut landing);
    // The intent or the finished batch keeps the number now; a landing that wrote none gives the number back.
    drop(claim);
    let times: BatchTimes = BatchTimes { batch: *seq, size: fresh.len(), group_began, settle, landing, land_total: landing_began.elapsed(), landed_at: Instant::now() };
    match result {
        Ok(outcomes) => {
            // What this batch landed is remembered before any answer, so a commit sent back in the same batch is told about it.
            for ((request, _, _), outcome) in fresh.iter().zip(&outcomes) {
                if let Outcome::Landed { commit } = outcome {
                    captures.record_landed(request.common_dir.as_path(), request.repository.as_path(), commit.as_str(), request.captured);
                }
            }
            crate::service_fault::reach("before-ack");
            for ((request, _, _), outcome) in fresh.iter().zip(&outcomes) {
                if let Outcome::Landed { commit } = outcome {
                    for (repeat, stream, arrival) in repeats.iter().filter(|(repeat, _, _)| { return repeat.id == request.id; }) {
                        stream.send(&ServiceMessage::Landed { id: repeat.id.clone(), commit: commit.clone(), already: true });
                        log_answer(&times, repeat, arrival, 1, "already");
                    }
                }
            }
            for (position, ((request, stream, arrival), outcome)) in fresh.iter().zip(outcomes).enumerate() {
                let word: &str = match outcome {
                    Outcome::Landed { commit } => {
                        landed.insert(request.id.clone(), commit.clone());
                        stream.send(&ServiceMessage::Landed { id: request.id.clone(), commit, already: false });
                        "landed"
                    }
                    Outcome::RetryPrepare { base } => {
                        let known = captures.remembered(request.common_dir.as_path(), request.repository.as_path());
                        stream.send(&ServiceMessage::RetryPrepare { id: request.id.clone(), base, landed: known });
                        "retry-prepare"
                    }
                    Outcome::BranchMoved => {
                        stream.send(&ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::BranchMoved });
                        "branch-moved"
                    }
                };
                log_answer(&times, request, arrival, position + 1, word);
            }
        }
        Err(code) => {
            for (position, (request, stream, arrival)) in fresh.iter().enumerate() {
                stream.send(&ServiceMessage::Failed { id: request.id.clone(), code });
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
///  running, and the service counts itself busy meanwhile, so it does not exit idle with work claimed by a dead holder. The same
///  holds for a push the service stopped: the loop owns the plan of its new tries (`PushRetries`), each round hands the workers
///  what is due and applies what they ask, and the wake for the earliest plan is kept in the same schedule as the lease wakes, so
///  the service neither forgets the plan nor exits idle while a try is pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const dir of wakesOrDueRetries) await drain(dir, handlers)
/// ```
pub fn work_loop(receiver: &Receiver<Wake>, handlers: &GitHandlers, busy: &AtomicUsize, mut push_retries: PushRetries) {
    let mut retries: RetrySchedule = RetrySchedule::default();
    let mut counted_waiting: bool = false;
    // A plan that is already there when the loop starts (one rebuilt from disk) is woken for like one made by a round.
    let started: Instant = Instant::now();
    let restored: Vec<(PathBuf, Duration)> = push_retries
        .directories()
        .into_iter()
        .filter_map(|directory| { return push_retries.next_wait(directory.as_path(), started).map(|wait| { return (directory, wait); }); })
        .collect();
    retries.replace(&[], restored, started);
    if !retries.is_empty() {
        busy.fetch_add(1, Ordering::SeqCst);
        counted_waiting = true;
    }
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
        let mut ran: Vec<PathBuf> = wakes.iter().map(|(directory, _)| { return directory.clone(); }).collect();
        let asked: Vec<(PathBuf, Duration)> = run_round(wakes, handlers, &mut push_retries, &ClaimTime::read());
        // A directory the round named for its plan alone (the common directory of a bare repository) replaces its old entry too.
        ran.extend(asked.iter().map(|(directory, _)| { return directory.clone(); }));
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
    // The identities of the requests their clients withdrew; a request with one of these is never landed.
    let mut withdrawn: std::collections::HashSet<String> = std::collections::HashSet::new();
    // What the landed commits were captured at: owned by this loop, the only thread that lands and answers.
    let mut captures: CaptureMemory = CaptureMemory::default();
    let (sender, receiver): (Sender<Job>, Receiver<Job>) = channel();
    let acceptor_queue: Sender<Job> = sender.clone();
    // After-commit work (push, upkeep) runs on its own thread, so a slow network never delays a landing.
    let (work_sender, work_receiver): (Sender<Wake>, Receiver<Wake>) = channel();
    let handlers: GitHandlers = GitHandlers::new(context.git.clone());
    let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let worker_busy: Arc<AtomicUsize> = Arc::clone(&busy);
    // What this landing thread is inside, for the threads that talk to clients (`service_busy.rs`).
    let board: Arc<LandingBoard> = LandingBoard::new(context.journal.clone(), settings.stuck_after);
    crate::service_busy::use_on_this_thread(Arc::clone(&board));
    let connection_board: Arc<LandingBoard> = Arc::clone(&board);
    // The plan of the pushes waiting for a new try is kept in this instance's own journal, so a restarted service rebuilds it.
    let plan_directory: PathBuf = context.journal.join(PLAN_DIRECTORY);
    start_thread_or_exit("the after-commit work", move || { work_loop(&work_receiver, &handlers, &worker_busy, PushRetries::restored_from_environment(plan_directory.as_path())); });
    let announced_release: String = settings.release.clone();
    let connection_busy: Arc<AtomicUsize> = Arc::clone(&busy);
    let instance: Option<String> = settings.instance.clone();
    start_thread_or_exit("the acceptor of connections", move || {
        for connection in listener.incoming().flatten() {
            let queue: Sender<Job> = acceptor_queue.clone();
            let work: Sender<Wake> = work_sender.clone();
            let release: String = announced_release.clone();
            let counted: Arc<AtomicUsize> = Arc::clone(&connection_busy);
            let record: Arc<LandingBoard> = Arc::clone(&connection_board);
            let mine: Option<String> = instance.clone();
            // A connection the service cannot give a thread is told so and closed; the acceptor goes on, because a panic here
            //  would end the acceptor and leave a service that is running and answers no one.
            let notice: Option<UnixStream> = connection.try_clone().ok();
            let spawned = std::thread::Builder::new().spawn(move || { handle_connection(connection, &queue, &work, &counted, &record, release.as_str(), mine.as_deref()); });
            if let Err(error) = spawned {
                crate::service_error!("no thread for a connection, which is closed after telling the client: {error}");
                if let Some(stream) = notice {
                    send(&stream, &ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(announced_release.clone()) });
                    send(&stream, &ServiceMessage::Failed { id: String::new(), code: FailureCode::Unavailable });
                }
                // Starting a thread keeps failing while the pressure lasts; waiting a moment keeps the acceptor from spinning on it.
                std::thread::sleep(ACCEPT_BACKOFF);
            }
        }
    });
    // Microseconds, so that instances started at different times count from different numbers; the number each batch takes is
    //  still claimed in the journal (`claim_batch`), because equal readings and counts that overtake another's are possible.
    let clock_seed: u64 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |elapsed| { return u64::try_from(elapsed.as_micros()).unwrap_or(1); });
    let mut seq: u64 = crate::service_fault::pinned_first_batch().unwrap_or(clock_seed);
    // The marker made ahead is removed whichever way this loop ends.
    let _prestaged = PrestageGuard;
    let mut retiring: bool = false;
    while !retiring {
        let first: Job = match receiver.recv_timeout(idle) {
            Ok(job) => job,
            // Work in flight keeps the service; otherwise it exits and the manager starts it on the next connection.
            Err(RecvTimeoutError::Timeout) if busy.load(Ordering::SeqCst) > 0 => continue,
            Err(RecvTimeoutError::Timeout) => {
                // Leaving: the syncs of the landings that were answered are made before the service goes (a signal cannot wait for them).
                crate::service_durable::drain();
                log(exit_line(ExitReason::Idle, serving_since.elapsed()).as_str());
                return;
            }
            Err(RecvTimeoutError::Disconnected) => {
                // Leaving: the syncs of the landings that were answered are made before the service goes (a signal cannot wait for them).
                crate::service_durable::drain();
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
        // Withdrawals are answered before anything lands, on this one thread, so each is either after a landing that is over (the
        //  commit is on the branch, and that is the answer) or before the landing of its request begins (the request is dropped
        //  and remembered, so it can never land, not even when it reaches the queue after the withdrawal).
        if withdrawn.len() > WITHDRAWN_RETAINED {
            withdrawn.clear();
        }
        for job in &jobs {
            if let Job::Withdraw(id, reply) = job {
                match landed.get(id) {
                    Some(commit) => reply.send(&ServiceMessage::Landed { id: id.clone(), commit: commit.clone(), already: true }),
                    None => {
                        withdrawn.insert(id.clone());
                        reply.send(&ServiceMessage::Withdrawn { id: id.clone() });
                    }
                }
            }
        }
        // Branches in order of first arrival; within a branch, arrival order.
        let mut groups: Vec<(GroupKey, Vec<Queued>)> = Vec::new();
        let mut releases: Vec<std::path::PathBuf> = Vec::new();
        for job in jobs {
            match job {
                Job::Withdraw(..) => {}
                Job::Commit(request, _dropped, _arrival) if withdrawn.contains(&request.id) => {
                    crate::service_log!("dropped the request {} that its client withdrew", request.id);
                }
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
                    let key: GroupKey = (request.common_dir.clone(), queue_reference, request.lands_alone());
                    place_in_groups(&mut groups, key, (request, stream, arrival));
                }
            }
        }
        for (_, group) in groups {
            let index: Option<std::path::PathBuf> = group.first().map(|(request, _, _)| { return request.index.clone(); });
            // A request whose client was answered "withdrawn" while this thread was stuck in a write is dropped here, and the
            //  rest are marked as being landed, under one lock (`service_busy.rs`).
            let ids: Vec<String> = group.iter().map(|(request, _, _)| { return request.id.clone(); }).collect();
            let dropped: Vec<String> = board.begin_group(ids.as_slice());
            for id in &dropped {
                crate::service_log!("dropped the request {id} that its client withdrew while the service was stuck in a write");
            }
            let group: Vec<Queued> = group.into_iter().filter(|(request, _, _)| { return !dropped.contains(&request.id); }).collect();
            if group.is_empty() {
                board.end_group();
                continue;
            }
            land_group(context, settings.instance.as_deref(), &mut seq, group, &mut landed, &mut captures);
            board.end_group();
            // The batch after this one is numbered one more and lands on this index (a worktree has one), so its marker is
            //  made and flushed now, while the service is idle, and the next commit only places it.
            if let Some(next_index) = index {
                start_prestage(next_index.as_path(), seq + 1);
            }
        }
        // A lock this thread could not settle names this process: the restart reclaims it once this process is gone.
        if unsettled_locks() > 0 {
            crate::service_error!("ending, because an index lock could not be settled; the next connection starts the service again and its recovery reclaims the lock");
            return;
        }
        for path in releases {
            if let Err(error) = remove_tree(path.as_path()) {
                crate::service_warn!("the scratch directory {} could not be removed: {error}", path.display());
            }
        }
    }
    // Leaving: the syncs of the landings that were answered are made before the service goes (a signal cannot wait for them).
    crate::service_durable::drain();
    log(exit_line(ExitReason::Retire, serving_since.elapsed()).as_str());
}

/// Socket controls stay out of the release executable.
#[cfg(test)]
#[path = "service_server_tests.rs"]
mod tests;
