//! Controls for the client's wait bounds and for how it reads the lines the service sends.

use crate::service_protocol::{PROTOCOL_VERSION, RELEASE, decode_service};
use super::*;
use crate::service_protocol::{HEARTBEAT_EVERY, HEARTBEAT_SILENCE, encode_service};

/// A commit queued behind a full batch of landings waits for them for as long as the service keeps sending,
/// so there is no
/// queue wait to read;
/// what bounds the wait is the silence of the service,
/// which is the service's own beat several times over.
#[test]
fn the_silence_bound_is_several_beats_so_one_late_beat_is_not_a_stopped_service() {
    assert!(HEARTBEAT_SILENCE >= HEARTBEAT_EVERY * MINIMUM_BEATS_MISSED, "{HEARTBEAT_SILENCE:?} against a beat every {HEARTBEAT_EVERY:?}");
}

/// The fewest beats of silence a stopped service is told from a late one by.
const MINIMUM_BEATS_MISSED: u32 = 3;

/// A reader over a connection whose service sent `bytes` and then closed it.
fn reader_after(bytes: &[u8]) -> BufReader<UnixStream> {
    let (client, mut service) = UnixStream::pair().expect("a connected pair");
    service.write_all(bytes).expect("the service writes");
    drop(service);
    return BufReader::new(client);
}

/// A line the service sent is read as its message,
/// and a connection the service closed reads as no message.
#[test]
fn a_line_is_read_as_a_message_and_a_closed_connection_as_none() {
    let greeting: ServiceMessage = ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) };
    let mut line: String = encode_service(&greeting);
    line.push('\n');
    let mut reader: BufReader<UnixStream> = reader_after(line.as_bytes());
    assert_eq!(read_message(&mut reader), Some(greeting));
    assert_eq!(read_message(&mut reader), None, "the service closed the connection after its line");
    assert_eq!(read_message(&mut reader_after(b"")), None, "a connection closed with nothing sent");
}

/// A line that is not a message reads as none,
/// which is what the empty line of a closed connection is too:
/// the check that a
/// read took some bytes,
/// before decoding,
/// changes no result,
/// because the empty line decodes to nothing by itself.
#[test]
fn an_empty_or_damaged_line_decodes_to_no_message() {
    assert!(decode_service("").is_err(), "an empty line is not a message");
    assert_eq!(read_message(&mut reader_after(b"not a message\n")), None);
}
