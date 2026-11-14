//! What:
//!  The lowest steps of the client of the commit service:
//! connecting to the service's address,
//! reading one line as a message,
//!  a close or silence,
//! writing a request within a bound,
//! and asking an older service to retire.
//! Why:
//!  The client (`service_client.rs`) and the provisioning of the service units (`service_provision.rs`) both need them;
//! a file
//!  that uses neither lets each use it without using the other in a circle.
//!  Every one of these steps waits in the kernel for a program that may have stopped,
//! so every one of them is
//!  bounded by the bound its caller names (`wait_ceilings.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stream = connect(address); const heard = listen(reader, within); writeWithin(stream, line, 5_000, 10_000);
//! ```

use crate::service_client_connect_within_a_bound::connect_within;
use crate::service_protocol::{
    MAX_LINE_BYTES, ServiceMessage, decode_service, encode_retire,
};
use crate::service_server::Bind;
use std::io::{BufRead, BufReader};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};
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
    // The connect itself is bounded, because a socket whose queue of unaccepted connections is full takes no connection
    //  until something accepts, which a stopped service never does (`wait_ceilings.rs`, `CONNECT_CEILING`).
    return connect_within(address);
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
/// or silence,
/// within the read timeout the connection carries.
/// Why:
///   A read that times out is the service saying nothing for the silence bound,
/// which is not the same as a service that
///  closed its end:
/// the first is asked again with the same request,
/// the second is a service that went away.
///  The line is read in chunks here rather than by one `read_line`,
///  because a service that sends a byte now and then and never ends its line keeps `read_line` inside one call for
///  ever:
/// each chunk it takes restarts the operating system's read timeout,
/// so no bound of the client is looked at again.
/// The
///  bound is the same read timeout the callers already give the connection,
/// counted from the first byte waited for, so a line
///  that never ends ends the wait like a silence does,
/// and the caller reaches the service again on a new connection,
/// which
///  is what it does with a silence.
/// A line longer than the protocol allows is not a message,
/// so waiting for its end waits for bytes this client never accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heard = listen(reader) // { message } | 'closed' | 'silent'
/// ```
pub(crate) fn listen(reader: &mut BufReader<UnixStream>) -> Heard {
    let within: Duration = match reader.get_ref().read_timeout() {
        Ok(Some(given)) => given,
        // A connection whose caller gave it no bound is read as one line, as it was before this file bounded a line: the
        //  callers that read an answer always set one.
        _ => return listen_one_line(reader),
    };
    let heard: Heard = listen_within(reader, within);
    // The connection keeps the bound its caller gave it, whatever part of it this line used, so the next line is waited for
    //  as long as this one was granted.
    let _restored = reader.get_ref().set_read_timeout(Some(within));
    return heard;
}

/// What:
///  `listen`, reading for no longer than `within`.
/// Why:
///   The bound is one look at the clock between two chunks,
/// so bytes that never become a message move nothing and a
///  line that ends after them ends the wait like a silence does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listenWithin(reader, 5_000) // { message } | 'closed' | 'silent'
/// ```
fn listen_within(reader: &mut BufReader<UnixStream>, within: Duration) -> Heard {
    let until: Instant = Instant::now() + within;
    let mut line: Vec<u8> = Vec::new();
    loop {
        let left: Duration = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Heard::Silent;
        }
        // No read waits past the bound, so every chunk the operating system hands over costs one look at the clock.
        let _timeout = reader.get_ref().set_read_timeout(Some(left));
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
        if line.len() > MAX_LINE_BYTES {
            return Heard::Closed;
        }
        if line.last() != Some(&b'\n') {
            continue;
        }
        let Ok(text) = String::from_utf8(line) else { return Heard::Closed };
        return decode_service(text.trim_end_matches('\n')).map_or(Heard::Closed, Heard::Message);
    }
}

/// What:
///  One line read in a single call, for a connection no caller gave a bound.
/// Why:
///   The behaviour every reader had before a line was read in chunks is kept for the one kind of caller that names no
///  bound at all,
/// so this file changes nothing for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readLine(reader).then((line) => line.length > 0 ? decode(line) : 'closed')
/// ```
fn listen_one_line(reader: &mut BufReader<UnixStream>) -> Heard {
    let mut line: String = String::new();
    return match reader.read_line(&mut line) {
        Ok(read) if read > 0 => decode_service(line.trim_end_matches('\n')).map_or(Heard::Closed, Heard::Message),
        Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => Heard::Silent,
        _ => Heard::Closed,
    };
}

/// What became of writing one line into the connection:
/// every byte went in,
/// or the step gave up and why.
/// Why:
///   A write of a request waits for the service to read it,
/// and a service that stopped reading never takes a byte,
///  so the step that writes says which of the two ends it reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Wrote = 'all' | 'nothing-taken' | 'too-slow' | 'ended'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Wrote {
    /// Every byte went into the connection.
    All,
    /// The service took no byte for the whole silence window:
    /// it stopped reading.
    NothingTaken,
    /// The service took bytes but the whole line did not go in within the ceiling.
    TooSlow,
    /// The connection ended before the whole line went in.
    Ended,
}

/// What:
///  Writes every byte of `bytes` into the connection,
/// giving up when the service takes nothing for `silence` or when the whole write outlasts `ceiling`.
/// Why:
///   One `write_all` of a request waits in the kernel until the other end reads it,
/// and a service that stopped
///   reading never does:
/// a request of one mebibyte fills both buffers of the connection (233,152 bytes measured here) and then
///  waits with no end,
/// so no bound of the client is ever looked at again
/// (`wait_ceilings.rs`, `REQUEST_WRITE_SILENCE`).
/// The wait is on progress, not on a clock: a service that takes bytes is reading,
/// so the silence window starts
///  again with every byte the kernel accepts,
/// and only the whole write has a ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeWithin(socket, bytes, 5_000, 10_000) // 'all' | 'nothing-taken' | 'too-slow' | 'ended'
/// ```
pub(crate) fn write_within(stream: &UnixStream, bytes: &[u8], silence: Duration, ceiling: Duration) -> Wrote {
    let began: Instant = Instant::now();
    let mut written: usize = 0;
    let mut took_a_byte: Instant = began;
    while written < bytes.len() {
        let whole_left: Duration = ceiling.saturating_sub(began.elapsed());
        let silence_left: Duration = silence.saturating_sub(took_a_byte.elapsed());
        if whole_left.is_zero() {
            return Wrote::TooSlow;
        }
        if silence_left.is_zero() {
            return Wrote::NothingTaken;
        }
        let mut waiting: libc::pollfd = libc::pollfd { fd: stream.as_raw_fd(), events: libc::POLLOUT, revents: 0 };
        let millis: libc::c_int = libc::c_int::try_from(whole_left.min(silence_left).as_millis().max(1)).unwrap_or(libc::c_int::MAX);
        // SAFETY: `waiting` is one valid `pollfd` for the call, which reads and writes only that.
        let ready: libc::c_int = unsafe { libc::poll(&raw mut waiting, 1, millis) };
        if ready < 0 {
            // A signal that came first is not the end of the write, so the loop looks at the clock again.
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Wrote::Ended;
        }
        // Nothing to write to a connection that reported a hang-up or an error; the send below would say the same.
        if ready == 0 {
            continue;
        }
        // One send at a time, and never waiting in the kernel: the room the poll just proved is what it takes, and a send
        //  that would wait returns instead, so this loop keeps the clock it reads.
        // SAFETY: the pointer and length are this function's own slice, and `MSG_DONTWAIT` and `MSG_NOSIGNAL` are flags the
        //  kernel takes for a send on a connected socket.
        let sent: isize = unsafe {
            libc::send(stream.as_raw_fd(), bytes[written..].as_ptr().cast(), bytes.len() - written, libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL)
        };
        if sent < 0 {
            let failed: std::io::Error = std::io::Error::last_os_error();
            if failed.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if failed.kind() == std::io::ErrorKind::WouldBlock {
                // The socket takes nothing yet: the loop waits for room again, under the two bounds.
                continue;
            }
            return Wrote::Ended;
        }
        if sent == 0 {
            return Wrote::Ended;
        }
        written += usize::try_from(sent).unwrap_or(usize::MAX);
        took_a_byte = Instant::now();
    }
    return Wrote::All;
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
    // A connect that waits in the kernel for a queue nothing empties is no one to ask, and the wait has an end.
    let Ok(stream) = crate::service_client_connect_within_a_bound::connect_within(&Bind::Path(socket.to_path_buf())) else { return };
    let _timeout = stream.set_read_timeout(Some(START_WAIT));
    let mut reader: BufReader<UnixStream> = BufReader::new(stream);
    let Some(ServiceMessage::Welcome { version, .. }) = read_message(&mut reader) else { return };
    let mut retire: String = encode_retire(version);
    retire.push('\n');
    // The one line of the ask is written under the same bounds as any other, so a service that stops reading holds nothing.
    let _sent = write_within(reader.get_ref(), retire.as_bytes(), crate::wait_ceilings::REQUEST_WRITE_SILENCE, crate::wait_ceilings::REQUEST_WRITE_CEILING);
}

/// Controls of the two bounds of one exchange stay out of the release executable.
#[cfg(test)]
#[path = "service_client_listen_and_ask_to_retire_tests.rs"]
mod tests;
