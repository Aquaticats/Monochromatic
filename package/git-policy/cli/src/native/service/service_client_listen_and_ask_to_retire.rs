//! What:
//!  The lowest steps of the client of the commit service:
//! connecting to the service's address,
//! reading one line as a message,
//!  a close or silence,
//! and asking an older service to retire.
//! Why:
//!  The client (`service_client.rs`) and the provisioning of the service units (`service_provision.rs`) both need them;
//! a file
//!  that uses neither lets each use it without using the other in a circle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stream = connect(address); const heard = await listen(reader); await askToRetire(socket);
//! ```

use crate::service_protocol::{
    ServiceMessage, decode_service, encode_retire,
};
use crate::service_server::Bind;
use std::io::{BufRead, BufReader, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::time::Duration;
/// How long the client waits for a service to start or to retire when it cannot ask the service manager how the start is going.
///  Ten times the slowest of 52 cold starts in this repository's commit timing log (`startWaitMs`,
/// longest 1041 ms,
///  median 47 ms,
/// 2026-11-03);
/// the 70 commits that also asked an older service to retire all reached the service within
///  that same 1041 ms (`reachMs`).
/// A client that can ask the manager has no such bound:
/// it ends when the manager says the
///  start failed.
pub const START_WAIT: Duration = Duration::from_secs(10);

/// What the client heard from the service in one read.
pub(crate) enum Heard {
    /// A message.
    Message(ServiceMessage),
    /// The service closed the connection,
    /// or sent something that is not a message.
    Closed,
    /// Nothing came within the silence bound.
    Silent,
}

/// What:
///  A connection to the service's address, made within `CONNECT_CEILING`.
/// Why:
///   A listening socket whose backlog is full takes no connection until its program accepts,
/// and a program stopped at any instruction never accepts,
///  so a plain connect waits for as long as that process stays stopped.
///  The review of `3d5c1a0d8c` measured a commit still waiting after 30 s for that reason
/// (`foundation/unix_socket_connect_within_a_deadline.rs`, and `CONNECT_CEILING` for the number).
///  A connection that only queued is never read as an answer either:
/// the caller reads a greeting from what it gets, and a full backlog answers none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// connect(address) // a stream within CONNECT_CEILING, or the failure that ended the attempt
/// ```
pub(crate) fn connect(address: &Bind) -> std::io::Result<UnixStream> {
    return match address {
        #[cfg(target_os = "linux")]
        Bind::Abstract(name) => crate::unix_socket_connect_within_a_deadline::connect_abstract_within(name.as_str(), crate::wait_ceilings::CONNECT_CEILING),
        #[cfg(not(target_os = "linux"))]
        Bind::Abstract(_) => Err(std::io::Error::other("abstract sockets need Linux")),
        Bind::Path(path) => crate::unix_socket_connect_within_a_deadline::connect_within(path, crate::wait_ceilings::CONNECT_CEILING),
        Bind::Inherited => Err(std::io::Error::other("a client does not inherit the socket")),
    };
}

pub(crate) fn read_message(reader: &mut BufReader<UnixStream>) -> Option<ServiceMessage> {
    return match listen(reader) {
        Heard::Message(message) => Some(message),
        Heard::Closed | Heard::Silent => None,
    };
}

/// What:
///  Reads one line from the service and says whether it was a message,
/// a close,
/// or silence.
/// Why:
///   A read that times out is the service saying nothing for the silence bound,
/// which is not the same as a service that
///  closed its end:
/// the first is asked again with the same request,
/// the second is a service that went away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heard = await listen(reader) // { message } | 'closed' | 'silent'
/// ```
pub(crate) fn listen(reader: &mut BufReader<UnixStream>) -> Heard {
    let mut line: String = String::new();
    return match reader.read_line(&mut line) {
        Ok(read) if read > 0 => decode_service(line.trim_end_matches('\n')).map_or(Heard::Closed, Heard::Message),
        Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => Heard::Silent,
        _ => Heard::Closed,
    };
}

/// What:
///  Whether a line from the service has already arrived:
/// a line the reader still holds,
/// bytes the connection has,
/// or the service's close.
/// Why:
///   A waiting notice tells a person about a wait that is still on,
/// and a client that speaks before reading what has already arrived tells of a wait that is over:
/// a client the machine held up said its commit
///  was inside a write after the service had landed it.
/// The asking is `MSG_DONTWAIT` on the connection the client already holds,
/// so it waits for nothing and
///  leaves what arrived to the read that follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aMessageIsAlreadyHere(reader) // false only when a read would wait
/// ```
pub(crate) fn a_message_is_already_here(reader: &mut BufReader<UnixStream>) -> bool {
    if !reader.buffer().is_empty() {
        return true;
    }
    let mut first: [u8; 1] = [0; 1];
    // SAFETY: `first` is a buffer the call writes at most one byte into, and the descriptor is the live connection the
    //  reader holds its own reference to.
    let peeked: isize = unsafe { libc::recv(reader.get_ref().as_raw_fd(), first.as_mut_ptr().cast(), first.len(), libc::MSG_PEEK | libc::MSG_DONTWAIT) };
    if peeked >= 0 {
        // A closed connection is answered at once as well: the read that follows returns with the close rather than waiting
        //  for a line the closed service will never send.
        return true;
    }
    // Only a connection with nothing to read leaves the read that follows waiting, and a notice is for that wait alone.
    return !matches!(std::io::Error::last_os_error().kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut);
}

/// What:
///  Asks the service listening at `socket` to finish its batch and exit,
/// when one answers;
/// nothing else is waited for.
/// Why:
///   The single service of the earlier releases is replaced by one service per worktree.
/// It is asked politely first,
/// in the
///  protocol version it announced,
/// so that a batch it is landing is finished;
/// the manager then stops its units.
/// A socket
///  nobody listens on,
/// or a service that does not greet,
/// is no one to ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await askToRetire(socket)
/// ```
pub fn ask_to_retire(socket: &std::path::Path) {
    // A person's command must not wait on a service whose backlog is full (`CONNECT_CEILING`).
    let Ok(stream) = crate::unix_socket_connect_within_a_deadline::connect_within(socket, crate::wait_ceilings::CONNECT_CEILING) else { return };
    let _timeout = stream.set_read_timeout(Some(START_WAIT));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream);
    let Some(ServiceMessage::Welcome { version, .. }) = read_message(&mut reader) else { return };
    let mut retire: String = encode_retire(version);
    retire.push('\n');
    let _sent = reader.get_mut().write_all(retire.as_bytes());
}
