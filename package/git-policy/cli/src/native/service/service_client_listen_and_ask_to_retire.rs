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
    let _timeout = stream.set_read_timeout(Some(START_WAIT));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream);
    let Some(ServiceMessage::Welcome { version, .. }) = read_message(&mut reader) else { return };
    let mut retire: String = encode_retire(version);
    retire.push('\n');
    let _sent = reader.get_mut().write_all(retire.as_bytes());
}
