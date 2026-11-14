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
///  Opens a client's connection to the service at `address`,
///  and says what came back.
/// Why:
///  This is the one place a client opens one,
/// so the question every client asks of the process that took it is asked once,
///  and the package's own controls reach a fixture's service through the same code the wrapper uses (rule `XPT`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reached = connect(address); // { kind: 'ours', stream } | { kind: 'nothing' } | { kind: 'other-account' }
/// ```
pub fn connect(address: &Bind) -> Reached {
    let stream: std::io::Result<UnixStream> = match address {
        #[cfg(target_os = "linux")]
        Bind::Abstract(name) => {
            use std::os::linux::net::SocketAddrExt;
            match std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()) {
                Ok(socket_address) => UnixStream::connect_addr(&socket_address),
                Err(error) => Err(error),
            }
        }
        #[cfg(not(target_os = "linux"))]
        Bind::Abstract(_) => Err(std::io::Error::other("abstract sockets need Linux")),
        Bind::Path(path) => UnixStream::connect(path),
        Bind::Inherited => Err(std::io::Error::other("a client does not inherit the socket")),
    };
    return match stream {
        Ok(accepted) if connection_is_this_account(&accepted) => Reached::Ours(accepted),
        Ok(_another_account) => Reached::OtherAccount,
        Err(_nothing) => Reached::Nothing,
    };
}

/// What a connection to a service's address came to.
pub enum Reached {
    /// A process of this account accepted it.
    Ours(UnixStream),
    /// Nothing listens at the address.
    Nothing,
    /// A process of another account took it,
    /// which no service of this command can be.
    OtherAccount,
}

/// What:
///  Whether the process on the other end of a connection is this account's.
/// Why:
///   A socket file is protected by the directory it stands in,
/// whose mode keeps another account out,
/// and an abstract name has no file and no permission bits
/// (`service_address_of_a_worktree.rs`).
/// What both ends read instead is the kernel's own word about the other
/// (`SO_PEERCRED`):
/// a connection taken by another account is no service of this command,
/// and one whose peer cannot be read is not proven to be one.
/// The service asks the same question of every client before it reads a byte
///  (`service_server_connection_handling.rs`).
/// On a system with no abstract namespace every address is a socket file in a directory only this account may enter,
///  so the directory's own mode is the protection and there is nothing more to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// connectionIsThisAccount(stream) // getsockopt(SO_PEERCRED).uid === process.geteuid()
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn connection_is_this_account(stream: &UnixStream) -> bool {
    return a_connection_of_the_account(the_users_id_on_the_other_end(stream), own_users_id());
}

/// The user id of this process.
#[cfg(target_os = "linux")]
fn own_users_id() -> u32 {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    return unsafe { libc::geteuid() };
}

/// What:
///  The user id the kernel reports for the process on the other end of a connection,
///  or nothing when the kernel would not say.
/// Why:
///   `SO_PEERCRED` is the kernel's own answer about the peer,
/// which a process cannot claim otherwise,
///  and it is the fact that stands in for the directory's mode on an address that has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const uid = getsockopt(stream, SOL_SOCKET, SO_PEERCRED).uid; // undefined when the call fails
/// ```
#[cfg(target_os = "linux")]
fn the_users_id_on_the_other_end(stream: &UnixStream) -> Option<u32> {
    use std::os::fd::AsRawFd;
    let mut credentials: libc::ucred = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut length: libc::socklen_t = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: the descriptor is the live connection this call holds, `credentials` is a `ucred` of the size it is given, and
    //  `length` is written with the bytes the kernel filled in, which is at most that size.
    let read: libc::c_int = unsafe { libc::getsockopt(stream.as_raw_fd(), libc::SOL_SOCKET, libc::SO_PEERCRED, std::ptr::from_mut(&mut credentials).cast(), &mut length) };
    if read != 0 {
        return None;
    }
    return Some(credentials.uid);
}

/// What:
///  Whether the user id the kernel reported is the one this process runs as.
/// Why:
///   A connection another account's process took is no service of this command,
/// and a connection the kernel would not speak about is not proven to be one,
///  so both answer no.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aConnectionOfTheAccount(1001, 1000) // false
/// ```
fn a_connection_of_the_account(their_user: Option<u32>, our_user: u32) -> bool {
    return their_user == Some(our_user);
}

/// `connection_is_this_account` where the account is the directory's to keep out,
/// so there is no second question to ask.
#[cfg(not(target_os = "linux"))]
pub(crate) fn connection_is_this_account(_stream: &UnixStream) -> bool {
    return true;
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
    let Ok(stream) = UnixStream::connect(socket) else { return };
    let _timeout = stream.set_read_timeout(Some(START_WAIT));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream);
    let Some(ServiceMessage::Welcome { version, .. }) = read_message(&mut reader) else { return };
    let mut retire: String = encode_retire(version);
    retire.push('\n');
    let _sent = reader.get_mut().write_all(retire.as_bytes());
}

/// The controls of a connection's account stay out of the release executable.
#[cfg(all(test, target_os = "linux"))]
#[path = "service_client_listen_and_ask_to_retire_tests.rs"]
mod tests;
