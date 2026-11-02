//! Controls for the client's wait bounds and for how it reads the lines the service sends.

use super::*;
use crate::service_protocol::encode_service;

/// The documented wait of a commit in the queue: ten minutes, configurable
///  (`doc/planning/cli-git-resident-service.md`, "bounded commit wait").
const DOCUMENTED_COMMIT_WAIT: Duration = Duration::from_secs(10 * 60);

/// A commit queued behind a full batch of landings waits for them; a bound of about a minute would fail commits that were
/// only waiting their turn, so the default is the documented ten minutes, which is also longer than a service start.
#[test]
fn a_commit_waits_the_documented_ten_minutes_in_the_queue() {
    assert_eq!(COMMIT_WAIT, DOCUMENTED_COMMIT_WAIT);
    assert!(COMMIT_WAIT > START_WAIT);
}

/// A reader over a connection whose service sent `bytes` and then closed it.
fn reader_after(bytes: &[u8]) -> BufReader<UnixStream> {
    let (client, mut service) = UnixStream::pair().expect("a connected pair");
    service.write_all(bytes).expect("the service writes");
    drop(service);
    return BufReader::new(client);
}

/// A line the service sent is read as its message, and a connection the service closed reads as no message.
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

/// A line that is not a message reads as none, which is what the empty line of a closed connection is too: the check that a
/// read took some bytes, before decoding, changes no result, because the empty line decodes to nothing by itself.
#[test]
fn an_empty_or_damaged_line_decodes_to_no_message() {
    assert!(decode_service("").is_err(), "an empty line is not a message");
    assert_eq!(read_message(&mut reader_after(b"not a message\n")), None);
}
