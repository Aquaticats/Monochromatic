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
use crate::service_client_patience::Patience;
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
pub(crate) fn connect(address: &Bind) -> std::io::Result<UnixStream> {
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

pub(crate) fn read_message(reader: &mut BufReader<UnixStream>, patience: &Patience, own: Duration) -> Option<ServiceMessage> {
    return match listen(reader, patience, own) {
        Heard::Message(message) => Some(message),
        Heard::Closed | Heard::Silent => None,
    };
}

/// What:
///  Reads one line from the service and says whether it was a message,
/// a close,
/// or silence,
/// within the budget the command gave the service.
/// Why:
///   A read that times out is the service saying nothing for the silence bound,
/// which is not the same as a service that
///  closed its end:
/// the first is asked again with the same request,
/// the second is a service that went away.
///  A line is read in chunks here rather than by one `read_line`,
///  because a service that sends bytes and never ends its line keeps `read_line` inside one call for ever:
///  every chunk it returns restarts the operating system's read timeout,
///  so no bound of the client is ever looked at again
///  (the third audit of release 0.6.1-alpha.12,
///  finding r01).
///  The budget is looked at between two chunks, so bytes that never become a message move nothing,
///  and a line longer than the protocol allows is not a message at all.
///  `own` is the bound this one step would have without a budget: the silence bound, or the wait a start is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heard = await listen(reader, patience, own) // { message } | 'closed' | 'silent'
/// ```
pub(crate) fn listen(reader: &mut BufReader<UnixStream>, patience: &Patience, own: Duration) -> Heard {
    let mut line: Vec<u8> = Vec::new();
    loop {
        if patience.spent() {
            return Heard::Silent;
        }
        // No read waits past this step's own bound or what the command has left, whichever is shorter.
        let _timeout = reader.get_ref().set_read_timeout(Some(patience.bound(own)));
        let available: usize = match reader.fill_buf() {
            Ok(bytes) if bytes.is_empty() => return Heard::Closed,
            Ok(bytes) => bytes.len(),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => return Heard::Silent,
            Err(_) => return Heard::Closed,
        };
        let taken: usize = match reader.buffer().iter().position(|byte| return *byte == b'\n') {
            Some(end) => end + 1,
            None => available,
        };

        line.extend_from_slice(&reader.buffer()[..taken]);
        reader.consume(taken);
        // A line the protocol does not allow is not a message, and no service sends one: waiting for its end would wait for
        //  bytes this client never accepts.
        if line.len() > crate::service_protocol::MAX_LINE_BYTES {
            return Heard::Closed;
        }
        if line.last() != Some(&b'\n') {
            continue;
        }
        let text: String = match String::from_utf8(line) {
            Ok(read) => read,
            Err(_not_text) => return Heard::Closed,
        };
        return decode_service(text.trim_end_matches('\n')).map_or(Heard::Closed, Heard::Message);
    }
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
    let Ok(stream) = UnixStream::connect(socket) else { return };
    let mut reader: BufReader<UnixStream> = BufReader::new(stream);
    let patience: Patience = Patience::of(START_WAIT);
    let Some(ServiceMessage::Welcome { version, .. }) = read_message(&mut reader, &patience, START_WAIT) else { return };
    let mut retire: String = encode_retire(version);
    retire.push('\n');
    let _bounded = reader.get_mut().set_write_timeout(Some(patience.wait()));
    let _sent = reader.get_mut().write_all(retire.as_bytes());
}
