//! What:
//!  The wrapper's side of a commit through the resident service: connect, wait honestly while the operating system
//!  starts the service, send the request, show the queue position, survive a service that dies mid-request, and turn
//!  every failure into the plain text of `service_messages.rs`.
//! Why:
//!  A failure the system can resolve is resolved here without the user: a service that is starting is waited for, one
//!  that crashed is reconnected to with the same request identity, one that is older than this wrapper is asked to retire
//!  and is reached again once the manager has started the new one. Only what cannot be resolved ends the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await submit(config, request, stderr); // landed, or prepare again, or a ClientFailure
//! ```

use super::service_messages::{ClientFailure, Platform, STARTING_NOTICE, UPDATING_NOTICE, position_notice};
use super::service_protocol::{
    ClientMessage, CommitRequest, FailureCode, PROTOCOL_VERSION, ServiceMessage, decode_service, encode_client,
};
use super::service_server::Bind;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

/// How long the client waits for the operating system to start the service.
pub const START_WAIT: Duration = Duration::from_secs(10);

/// How long a start takes before the client says it is starting the service.
pub const START_NOTICE_GRACE: Duration = Duration::from_secs(1);

/// How long a commit waits in the queue before the client gives up and says so.
pub const COMMIT_WAIT: Duration = Duration::from_secs(10 * 60);

/// How many times a request is sent again after the connection broke before the client says it cannot be sure.
pub const RECONNECTS: usize = 5;

/// How many times the client asks an older service to retire.
pub const REPLACEMENTS: usize = 2;

/// The pause between connection tries while the service starts.
const CONNECT_PAUSE: Duration = Duration::from_millis(25);

/// What the client may rely on the operating system to do.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Activation {
    /// The manager owns the socket and starts the service on demand.
    Managed,
    /// Nothing can start the service on this computer: the activation is not installed.
    NotInstalled,
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
    /// Longest wait for a start.
    pub start_wait: Duration,
    /// Longest wait in the queue.
    pub commit_wait: Duration,
}

/// What a request came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Submitted {
    /// The commit is on the branch.
    Landed { commit: String, already: bool },
    /// A named path changed on the branch: prepare again from this base and submit again with the same identity.
    PrepareAgain { base: String },
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
    let mut line: String = String::new();
    return match reader.read_line(&mut line) {
        Ok(read) if read > 0 => decode_service(line.trim_end_matches('\n')).ok(),
        _ => None,
    };
}

/// What:
///  Connects, waiting for a start the operating system performs, and returns the stream with the service's version.
/// Why:
///   The wait is bounded and honest: one notice after the grace, then silence until the bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stream, version } = await reach(config, notices)
/// ```
fn reach(config: &ClientConfig, notices: &mut dyn Write, announced: &mut bool) -> Result<(BufReader<UnixStream>, u32), ClientFailure> {
    let started: Instant = Instant::now();
    loop {
        match connect(&config.address) {
            Ok(stream) => {
                let _timeout = stream.set_read_timeout(Some(config.start_wait));
                let mut reader: BufReader<UnixStream> = BufReader::new(stream);
                if let Some(ServiceMessage::Welcome { version }) = read_message(&mut reader) {
                    let _timeout = reader.get_ref().set_read_timeout(Some(config.commit_wait));
                    return Ok((reader, version));
                }
            }
            Err(_) => match config.activation {
                Activation::NotInstalled => return Err(ClientFailure::NotInstalled),
                Activation::Unsupported(platform) => return Err(ClientFailure::NoActivation(platform)),
                Activation::Managed => {}
            },
        }
        if started.elapsed() >= config.start_wait {
            return Err(ClientFailure::DidNotStart);
        }
        if !*announced && started.elapsed() >= START_NOTICE_GRACE {
            *announced = true;
            let _written = writeln!(notices, "{STARTING_NOTICE}");
        }
        std::thread::sleep(CONNECT_PAUSE);
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
    let line: String = encode_client(&ClientMessage::Commit(request.clone()));
    let waiting_since: Instant = Instant::now();
    let mut announced: bool = false;
    let mut replacements: usize = 0;
    let mut breaks: usize = 0;
    let mut last_position: Option<usize> = None;
    loop {
        if waiting_since.elapsed() >= config.commit_wait {
            return Err(ClientFailure::WaitedTooLong { minutes: config.commit_wait.as_secs() / 60 });
        }
        let (mut reader, version) = reach(config, notices, &mut announced)?;
        if version != PROTOCOL_VERSION {
            // An older service is asked to finish and exit; the manager starts the current one for the next connection.
            if version > PROTOCOL_VERSION || replacements >= REPLACEMENTS {
                return Err(ClientFailure::NoCommonVersion);
            }
            replacements += 1;
            let _written = writeln!(notices, "{UPDATING_NOTICE}");
            let mut retire: String = encode_client(&ClientMessage::Retire);
            retire.push('\n');
            let _sent = reader.get_mut().write_all(retire.as_bytes());
            std::thread::sleep(CONNECT_PAUSE * 4);
            continue;
        }
        let mut payload: String = line.clone();
        payload.push('\n');
        if reader.get_mut().write_all(payload.as_bytes()).is_err() {
            breaks += 1;
            if breaks > RECONNECTS {
                return Err(ClientFailure::MayNotHaveLanded);
            }
            continue;
        }
        loop {
            match read_message(&mut reader) {
                Some(ServiceMessage::Position { number, of }) => {
                    if last_position != Some(number) {
                        last_position = Some(number);
                        let _written = writeln!(notices, "{}", position_notice(number, of));
                    }
                }
                Some(ServiceMessage::Landed { commit, already, .. }) => return Ok(Submitted::Landed { commit, already }),
                Some(ServiceMessage::RetryPrepare { base, .. }) => return Ok(Submitted::PrepareAgain { base }),
                Some(ServiceMessage::Failed { code, .. }) => return Err(failure_for(code, request)),
                Some(ServiceMessage::Welcome { .. }) => {}
                None => break,
            }
        }
        // The connection broke before an answer: the service died or was replaced. Send the same request again.
        breaks += 1;
        if breaks > RECONNECTS {
            return Err(ClientFailure::MayNotHaveLanded);
        }
        std::thread::sleep(CONNECT_PAUSE * u32::try_from(breaks).unwrap_or(1));
    }
}
