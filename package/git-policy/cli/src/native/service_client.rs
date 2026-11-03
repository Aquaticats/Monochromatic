//! What:
//!  The wrapper's side of a commit through the resident service: connect, wait honestly while the operating system
//!  starts the service, send the request, show the queue position, survive a service that dies mid-request, and turn
//!  every failure into the plain text of `service_messages.rs`.
//! Why:
//!  A failure the system can resolve is resolved here without the user: a service that is starting is waited for, one
//!  that crashed is reconnected to with the same request identity, one that was started from another release of the
//!  wrapper (older or newer, found by the release its welcome names) is asked to retire and is reached again once the
//!  manager has started the current one. Only what cannot be resolved ends the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await submit(config, request, stderr); // landed, or prepare again, or a ClientFailure
//! ```

use super::commit_timing::{ReachTrace, SubmitTrace, holder_kind_of};
use super::service_messages::{ClientFailure, Platform, SILENT_NOTICE, STARTING_NOTICE, UPDATING_NOTICE, position_notice};
use super::service_protocol::{
    ClientMessage, CommitRequest, FailureCode, LandedCapture, PROTOCOL_VERSION, RELEASE, ServiceMessage, decode_service, encode_client, encode_retire,
};
use super::service_instance::service_unit_instance;
use super::service_provision::ProvisionHost;
use super::service_review_key::ReviewKey;
use super::systemd_dbus::{Bus, EXCHANGE_WAIT};
use super::service_server::Bind;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

/// How long the client waits for a service to start or to retire when it cannot ask the service manager how the start is going.
///  Ten times the slowest of 52 cold starts in this repository's commit timing log (`startWaitMs`, longest 1041 ms,
///  median 47 ms, 2026-11-03); the 70 commits that also asked an older service to retire all reached the service within
///  that same 1041 ms (`reachMs`). A client that can ask the manager has no such bound: it ends when the manager says the
///  start failed.
pub const START_WAIT: Duration = Duration::from_secs(10);

/// How long a start takes before the client says it is starting the service.
pub const START_NOTICE_GRACE: Duration = Duration::from_secs(1);

/// How many times a request is sent again after the connection broke before the client says it cannot be sure.
pub const RECONNECTS: usize = 5;

/// The pause between connection tries while the service starts.
const CONNECT_PAUSE: Duration = Duration::from_millis(25);

/// What the client may rely on the operating system to do.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Activation {
    /// The manager owns the socket and starts the service on demand.
    Managed,
    /// This platform cannot start the service on demand.
    Unsupported(Platform),
}

/// What the client needs to reach the service.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// Where the service listens.
    pub address: Bind,
    /// What the operating system can do about a service that is not running.
    pub activation: Activation,
    /// Longest wait for a start when `start_watch` is absent; with a watch the wait ends only when the manager says the start
    ///  failed, and this is the wait for an older service to retire.
    pub start_wait: Duration,
    /// How long the service may send nothing, heartbeats included, before the client treats it as stopped. There is no limit
    ///  on how long a request waits while the service keeps sending.
    pub heartbeat_silence: Duration,
    /// How to ask the service manager whether the service failed to start; absent where the client cannot ask.
    pub start_watch: Option<ManagerWatch>,
}

/// What:
///  How the client asks the service manager whether the service unit has failed.
/// Why:
///   A service that is starting sends nothing, so the wait for its first greeting has no heartbeat to listen for. The manager
///  knows whether the start failed, which is the one thing that ends that wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManagerWatch = { bus: string; unit: string; every: number; hasFailed: (bus: string, unit: string) => boolean };
/// ```
#[derive(Clone, Debug)]
pub struct ManagerWatch {
    /// The session bus of the account.
    pub bus: std::path::PathBuf,
    /// The service unit of the worktree.
    pub unit: String,
    /// How often the manager is asked while the greeting has not come.
    pub every: Duration,
    /// Whether the manager reports the unit as failed now; nothing when the manager cannot be asked, which leaves the wait
    ///  to the start wait.
    pub has_failed: fn(&std::path::Path, &str) -> Option<bool>,
}

/// How often the service manager is asked whether the service unit failed, while a start is waited for.
pub const MANAGER_ASK_EVERY: Duration = START_NOTICE_GRACE;

/// Whether the manager reports `unit` as failed: nothing when the bus or the unit cannot be asked about.
fn unit_has_failed(bus: &std::path::Path, unit: &str) -> Option<bool> {
    let mut connection: Bus = Bus::connect(bus, EXCHANGE_WAIT).ok()?;
    let state: Option<String> = connection.active_state(unit).ok()?;
    return Some(state.as_deref() == Some("failed"));
}

/// What:
///  The watch that asks the account's service manager about the worktree's service unit.
/// Why:
///   The route knows the session bus and the worktree's instance, which is all the unit's name needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = managerWatchFor(host)
/// ```
pub fn manager_watch_for(host: &ProvisionHost) -> ManagerWatch {
    return ManagerWatch { bus: host.bus.clone(), unit: service_unit_instance(host.instance.as_str()), every: MANAGER_ASK_EVERY, has_failed: unit_has_failed };
}

/// What the client heard from the service in one read.
enum Heard {
    /// A message.
    Message(ServiceMessage),
    /// The service closed the connection, or sent something that is not a message.
    Closed,
    /// Nothing came within the silence bound.
    Silent,
}

/// What a request came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Submitted {
    /// The commit is on the branch.
    Landed { commit: String, already: bool },
    /// A named path changed on the branch: prepare again from this base and submit again with the same identity.
    ///  `landed` tells when the bytes of the commits the service landed since were captured, for the ones it knows.
    PrepareAgain { base: String, landed: Vec<LandedCapture> },
    /// The branch is no longer at the base of a commit that cannot be moved to another tip (an amend, or a commit Git built).
    ///  The caller decides whether to build it again on the new tip or to stop.
    Moved,
}

fn connect(address: &Bind) -> std::io::Result<UnixStream> {
    return match address {
        #[cfg(target_os = "linux")]
        Bind::Abstract(name) => {
            use std::os::linux::net::SocketAddrExt;
            UnixStream::connect_addr(&std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes())?)
        }
        #[cfg(not(target_os = "linux"))]
        Bind::Abstract(_) => Err(std::io::Error::other("abstract sockets need Linux")),
        Bind::Path(path) => UnixStream::connect(path),
        Bind::Inherited => Err(std::io::Error::other("a client does not inherit the socket")),
    };
}

fn read_message(reader: &mut BufReader<UnixStream>) -> Option<ServiceMessage> {
    return match listen(reader) {
        Heard::Message(message) => Some(message),
        Heard::Closed | Heard::Silent => None,
    };
}

/// What:
///  Reads one line from the service and says whether it was a message, a close, or silence.
/// Why:
///   A read that times out is the service saying nothing for the silence bound, which is not the same as a service that
///  closed its end: the first is asked again with the same request, the second is a service that went away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heard = await listen(reader) // { message } | 'closed' | 'silent'
/// ```
fn listen(reader: &mut BufReader<UnixStream>) -> Heard {
    let mut line: String = String::new();
    return match reader.read_line(&mut line) {
        Ok(read) if read > 0 => decode_service(line.trim_end_matches('\n')).map_or(Heard::Closed, Heard::Message),
        Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => Heard::Silent,
        _ => Heard::Closed,
    };
}

/// What:
///  Connects, waiting for a start the operating system performs, and returns the stream with the service's version.
/// Why:
///   With a manager to ask, the wait has no clock: it ends when the service greets, or when the manager has said twice in a
///  row that the unit failed. Where the manager cannot be asked it ends at the start wait, as it always did. Once a service
///  has greeted during this command (`service_seen`), a service that then greets nothing within the silence bound is the
///  one that stopped, and that ends the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stream, version } = await reach(config, notices, serviceSeen)
/// ```
fn reach(
    config: &ClientConfig,
    notices: &mut dyn Write,
    announced: &mut bool,
    trace: &mut ReachTrace,
    service_seen: bool,
) -> Result<(BufReader<UnixStream>, u32, Option<String>), ClientFailure> {
    let started: Instant = Instant::now();
    let mut refused_here: bool = false;
    let mut last_asked: Instant = started;
    let mut failed_answers: u32 = 0;
    let mut cannot_ask: bool = false;
    loop {
        trace.tries += 1;
        match connect(&config.address) {
            Ok(stream) => {
                let patience: Duration = match (&config.start_watch, service_seen) {
                    (_, true) => config.heartbeat_silence,
                    (Some(watch), false) => watch.every,
                    (None, false) => config.start_wait,
                };
                let _timeout = stream.set_read_timeout(Some(patience));
                let mut reader: BufReader<UnixStream> = BufReader::new(stream);
                match listen(&mut reader) {
                    Heard::Message(ServiceMessage::Welcome { version, release }) => {
                        let _widened = reader.get_ref().set_read_timeout(Some(config.heartbeat_silence));
                        trace.saw_release(release.as_deref());
                        if refused_here {
                            trace.start_waited += started.elapsed();
                        }
                        return Ok((reader, version, release));
                    }
                    // A service that greeted before and now greets nothing is the one that stopped.
                    Heard::Silent if service_seen => return Err(ClientFailure::ServiceSilent),
                    // Connected, but nothing greeted: the service is still starting or is going away.
                    Heard::Message(_) | Heard::Closed | Heard::Silent => {
                        trace.refused += 1;
                        refused_here = true;
                    }
                }
            }
            Err(_) => match config.activation {
                Activation::Unsupported(platform) => return Err(ClientFailure::NoActivation(platform)),
                Activation::Managed => {
                    trace.refused += 1;
                    refused_here = true;
                }
            },
        }
        match &config.start_watch {
            Some(watch) => {
                if last_asked.elapsed() >= watch.every {
                    last_asked = Instant::now();
                    let answer: Option<bool> = (watch.has_failed)(watch.bus.as_path(), watch.unit.as_str());
                    cannot_ask = answer.is_none();
                    failed_answers = if answer == Some(true) { failed_answers + 1 } else { 0 };
                }
                // One answer of "failed" can be the unit's last run, before the connection made the manager start it again.
                if failed_answers >= 2 || (cannot_ask && started.elapsed() >= config.start_wait) {
                    return Err(ClientFailure::DidNotStart);
                }
            }
            None => {
                if started.elapsed() >= config.start_wait {
                    return Err(ClientFailure::DidNotStart);
                }
            }
        }
        if !*announced && started.elapsed() >= START_NOTICE_GRACE {
            *announced = true;
            let _written = writeln!(notices, "{STARTING_NOTICE}");
        }
        std::thread::sleep(CONNECT_PAUSE);
    }
}

/// What:
///  Reaches a service that speaks this wrapper's protocol and was started from this wrapper's release, asking any other to
///  retire first.
/// Why:
///   The manager keeps its socket across an upgrade or a downgrade, so a service from the other release may answer: one
///  whose program was since removed, or one that behaves as the other release did. It is asked to finish its batch and
///  exit, and the next connection makes the manager start the program the rewritten unit names. The asking is bounded by
///  the start wait from the first ask (the same wait the manager's start gets), because the retiring service still accepts connections until it has exited, and
///  each connection it accepts in that time is asked again. A service of a newer protocol cannot be asked (it refuses a
///  message it cannot read as its own), so that ends the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = await reachCurrent(config, notices, announced)
/// ```
fn reach_current(config: &ClientConfig, notices: &mut dyn Write, announced: &mut bool, trace: &mut ReachTrace, service_seen: bool) -> Result<BufReader<UnixStream>, ClientFailure> {
    let began: Instant = Instant::now();
    let reached = reach_current_untimed(config, notices, announced, trace, service_seen);
    trace.elapsed += began.elapsed();
    return reached;
}

/// `reach_current` without the clock around it.
fn reach_current_untimed(config: &ClientConfig, notices: &mut dyn Write, announced: &mut bool, trace: &mut ReachTrace, service_seen: bool) -> Result<BufReader<UnixStream>, ClientFailure> {
    let mut give_up_at: Option<Instant> = None;
    loop {
        let (mut reader, version, release) = reach(config, notices, announced, trace, service_seen)?;
        if version == PROTOCOL_VERSION && release.as_deref() == Some(RELEASE) {
            trace.answered_by = release;
            return Ok(reader);
        }
        if version > PROTOCOL_VERSION {
            return Err(ClientFailure::NoCommonVersion);
        }
        let deadline: Instant = *give_up_at.get_or_insert_with(|| {
            let _written = writeln!(notices, "{UPDATING_NOTICE}");
            return Instant::now() + config.start_wait;
        });
        if Instant::now() >= deadline {
            return Err(ClientFailure::OtherRelease);
        }
        // Written in the version the service announced: a service of an earlier release reads a retire in no other.
        let mut retire: String = encode_retire(version);
        retire.push('\n');
        let _sent = reader.get_mut().write_all(retire.as_bytes());
        trace.retires_sent += 1;
        std::thread::sleep(CONNECT_PAUSE * 4);
    }
}

/// The user-facing failure for a code the service gave.
pub fn failure_for(code: FailureCode, request: &CommitRequest) -> ClientFailure {
    let repository: String = request.repository.display().to_string();
    return match code {
        FailureCode::RepositoryFull => ClientFailure::RepositoryFull(repository),
        FailureCode::RepositoryReadOnly => ClientFailure::RepositoryReadOnly(repository),
        FailureCode::IndexInUse => ClientFailure::IndexInUse(String::from("Another program")),
        FailureCode::Rejected => ClientFailure::Refused,
        FailureCode::BranchKeepsChanging => ClientFailure::BranchKeepsChanging {
            branch: request.reference.trim_start_matches("refs/heads/").to_string(),
            times: super::service_land::LANDING_ATTEMPTS,
        },
        FailureCode::BranchMoved => ClientFailure::BranchMoved { branch: request.reference.trim_start_matches("refs/heads/").to_string() },
        FailureCode::Unavailable => ClientFailure::Unavailable,
    };
}

/// What:
///  Sends a commit request and waits until it is answered.
/// Why:
///   The same identity goes with every send, so a service that landed the commit and died before answering reports it
///  as already landed, and nothing lands twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await submit(config, request, notices)
/// ```
pub fn submit(config: &ClientConfig, request: &CommitRequest, notices: &mut dyn Write) -> Result<Submitted, ClientFailure> {
    return submit_traced(config, request, notices, &mut SubmitTrace::default());
}

/// What:
///  `submit`,
/// leaving in `trace` how the service was reached and how long the answer took.
/// Why:
///   The wrapper's timing record states the tries,
/// the retires and the releases of the reach,
/// the queue positions the service
///  reported and the time between the request being written and the answer;
/// the trace is filled as the send proceeds,
/// so a
///  send that fails still reports what it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await submitTraced(config, request, notices, trace)
/// ```
pub fn submit_traced(config: &ClientConfig, request: &CommitRequest, notices: &mut dyn Write, trace: &mut SubmitTrace) -> Result<Submitted, ClientFailure> {
    let began: Instant = Instant::now();
    let answered = submit_loop(config, request, notices, trace);
    trace.total += began.elapsed();
    return answered;
}

/// The sends and the waits of `submit_traced`.
///
/// The wait for the answer has no clock: it lasts as long as the service keeps sending, and ends when the service goes
///  silent for the silence bound. The request is then sent again with the same identity, so nothing lands twice, and the
///  command ends only when every try has met silence or a closed connection.
fn submit_loop(config: &ClientConfig, request: &CommitRequest, notices: &mut dyn Write, trace: &mut SubmitTrace) -> Result<Submitted, ClientFailure> {
    let line: String = encode_client(&ClientMessage::Commit(Box::new(request.clone())));
    let mut announced: bool = false;
    let mut breaks: usize = 0;
    let mut last_position: Option<usize> = None;
    let mut holder: Option<String> = None;
    let mut service_seen: bool = false;
    let mut last_break_was_silence: bool = false;
    loop {
        let mut reader: BufReader<UnixStream> = reach_current(config, notices, &mut announced, &mut trace.reach, service_seen)?;
        service_seen = true;
        let mut payload: String = line.clone();
        payload.push('\n');
        if reader.get_mut().write_all(payload.as_bytes()).is_err() {
            breaks += 1;
            trace.reconnects += 1;
            if breaks > RECONNECTS {
                return Err(ClientFailure::MayNotHaveLanded);
            }
            continue;
        }
        trace.sends += 1;
        let written_at: Instant = Instant::now();
        let answered: Option<Result<Submitted, ClientFailure>> = loop {
            match listen(&mut reader) {
                Heard::Message(ServiceMessage::Position { number, of }) => {
                    if last_position != Some(number) {
                        last_position = Some(number);
                        trace.positions.push(number);
                        let _written = writeln!(notices, "{}", position_notice(number, of));
                    }
                }
                Heard::Message(ServiceMessage::Landed { commit, already, .. }) => break Some(Ok(Submitted::Landed { commit, already })),
                Heard::Message(ServiceMessage::RetryPrepare { base, landed, .. }) => break Some(Ok(Submitted::PrepareAgain { base, landed })),
                Heard::Message(ServiceMessage::Failed { code: FailureCode::BranchMoved, .. }) => break Some(Ok(Submitted::Moved)),
                Heard::Message(ServiceMessage::Failed { code: FailureCode::IndexInUse, .. }) => {
                    break Some(Err(ClientFailure::IndexInUse(holder.take().unwrap_or_else(|| { return String::from("Another program"); }))));
                }
                Heard::Message(ServiceMessage::Failed { code, .. }) => break Some(Err(failure_for(code, request))),
                Heard::Message(ServiceMessage::Waiting { text }) => {
                    trace.waiting_notices += 1;
                    trace.holder_kind = Some(holder_kind_of(text.as_str()));
                    let _written = write!(notices, "{text}");
                }
                Heard::Message(ServiceMessage::Holder { who }) => holder = Some(who),
                // Each message, this one too, restarts the silence bound; nothing else is needed.
                Heard::Message(ServiceMessage::Welcome { .. } | ServiceMessage::Heartbeat) => {}
                Heard::Silent => {
                    last_break_was_silence = true;
                    break None;
                }
                Heard::Closed => {
                    last_break_was_silence = false;
                    break None;
                }
            }
        };
        trace.answer_wait += written_at.elapsed();
        if let Some(done) = answered {
            return done;
        }
        // The service went silent or the connection broke before an answer: the service died, hung or was replaced. Send the
        //  same request again.
        breaks += 1;
        if breaks > RECONNECTS {
            return Err(if last_break_was_silence { ClientFailure::ServiceSilent } else { ClientFailure::MayNotHaveLanded });
        }
        if last_break_was_silence {
            let _written = writeln!(notices, "{SILENT_NOTICE}");
        }
        std::thread::sleep(CONNECT_PAUSE * u32::try_from(breaks).unwrap_or(1));
    }
}

/// What:
///  Asks the service listening at `socket` to finish its batch and exit, when one answers; nothing else is waited for.
/// Why:
///   The single service of the earlier releases is replaced by one service per worktree. It is asked politely first, in the
///  protocol version it announced, so that a batch it is landing is finished; the manager then stops its units. A socket
///  nobody listens on, or a service that does not greet, is no one to ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await askToRetire(socket)
/// ```
pub fn ask_to_retire(socket: &std::path::Path) {
    let Ok(stream) = UnixStream::connect(socket) else { return };
    let _timeout = stream.set_read_timeout(Some(START_WAIT));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream);
    let Some(ServiceMessage::Welcome { version, .. }) = read_message(&mut reader) else { return };
    let mut retire: String = encode_retire(version);
    retire.push('\n');
    let _sent = reader.get_mut().write_all(retire.as_bytes());
}

/// A scratch directory handed to the service: the service removes it when this value is dropped, and when the process that
///  holds it dies by any means, because the connection is the only thing that says the directory is still wanted.
pub struct ScratchWatch {
    /// The open connection. Close-on-exec, so no program the wrapper starts keeps it open after the wrapper is gone.
    _connection: UnixStream,
}

/// What:
///  Hands the commit's scratch directory to the service, which removes it when the connection closes.
/// Why:
///   A client killed without warning cannot remove what it made, and no one can tell afterwards whether it is gone or
///  stopped: the open connection is the signal, without a process id and without a lock. The announcement is written at once
///  and the greeting is not waited for, so a service that is frozen or still starting takes it when it reads, and a client
///  killed meanwhile still loses its directory. A service that is not running takes nothing, and a service of another release
///  refuses a message it does not know and closes nothing of the client's; in both the client removes the directory itself
///  when it ends, as it always has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = watchScratch(config, gitDir, path) // undefined when nothing listens
/// ```
pub fn watch_scratch(config: &ClientConfig, git_dir: &std::path::Path, path: &std::path::Path) -> Option<ScratchWatch> {
    let mut connection: UnixStream = connect(&config.address).ok()?;
    let mut line: String = encode_client(&ClientMessage::Scratch { git_dir: git_dir.to_path_buf(), path: path.to_path_buf() });
    line.push('\n');
    connection.write_all(line.as_bytes()).ok()?;
    return Some(ScratchWatch { _connection: connection });
}

/// What:
///  Tells the service that work items were written for a repository, starting it through the operating system when it
///  is not running, and returns once the service has taken the message.
/// Why:
///   The wrapper never starts the service or runs the work itself; the message is the only thing the service needs to
///  look at the repository's work directory. The connection closes when the service has read the line, which is the
///  confirmation that it will act.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await wake(config, gitDir, reviewKey)
/// ```
pub fn wake(config: &ClientConfig, git_dir: &std::path::Path, review_key: Option<&ReviewKey>, notices: &mut dyn Write) -> Result<(), ClientFailure> {
    let mut line: String = encode_client(&ClientMessage::Work { git_dir: git_dir.to_path_buf(), review_key: review_key.cloned() });
    line.push('\n');
    let mut announced: bool = false;
    let mut reader: BufReader<UnixStream> = reach_current(config, notices, &mut announced, &mut ReachTrace::default(), false)?;
    if reader.get_mut().write_all(line.as_bytes()).is_err() {
        return Err(ClientFailure::Unavailable);
    }
    // The service closes the connection after it has read the line.
    while read_message(&mut reader).is_some() {}
    return Ok(());
}

/// The client's bounds and how it reads the service's lines.
#[cfg(test)]
#[path = "service_client_tests.rs"]
mod tests;
