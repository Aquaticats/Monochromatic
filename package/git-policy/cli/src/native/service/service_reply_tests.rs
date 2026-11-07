//! Controls for the beat a waiting client is sent and for the single writer that keeps beats and answers in order.

use super::*;
use crate::service_protocol::decode_service;
use std::io::{BufRead, BufReader};

/// How often the controls beat: fast enough that a control needing several beats takes well under a second.
const QUICK_BEAT: Duration = Duration::from_millis(20);

/// How many beats a control waits for before it is satisfied.
const BEATS_WANTED: usize = 3;

/// A reply, the thread that writes it to a connection, and the client's end of that connection.
fn connected() -> (Reply, std::thread::JoinHandle<()>, BufReader<UnixStream>) {
    let (service, client) = UnixStream::pair().expect("a connected pair");
    let (reply, messages) = Reply::new();
    let writer = std::thread::spawn(move || { write_until_answered(&service, &messages, QUICK_BEAT); });
    return (reply, writer, BufReader::new(client));
}

/// The next message the client reads.
fn next(client: &mut BufReader<UnixStream>) -> ServiceMessage {
    let mut line: String = String::new();
    client.read_line(&mut line).expect("a line arrives");
    return decode_service(line.trim_end_matches('\n')).expect("a message");
}

/// A request that waits for a long time is sent a beat the whole time, however long the wait; the beat is what the client reads
/// in place of a clock.
#[test]
fn a_waiting_client_is_sent_heartbeats_until_it_is_answered() {
    let (reply, writer, mut client) = connected();
    for _ in 0..BEATS_WANTED {
        assert_eq!(next(&mut client), ServiceMessage::Heartbeat);
    }
    let landed: ServiceMessage = ServiceMessage::Landed { id: String::from("x"), commit: String::from("c"), already: false };
    reply.send(&landed);
    loop {
        let heard: ServiceMessage = next(&mut client);
        if heard != ServiceMessage::Heartbeat {
            assert_eq!(heard, landed);
            break;
        }
    }
    drop(reply);
    writer.join().expect("the writer ends once the reply is gone");
}

/// Dropping the reply, which is what answering the request does, ends the writer and closes the connection: the client reads
/// the end of the stream, after only beats.
#[test]
fn the_beat_ends_with_the_reply_and_the_connection_closes() {
    let (reply, writer, mut client) = connected();
    drop(reply);
    writer.join().expect("the writer ends");
    let mut rest: String = String::new();
    while client.read_line(&mut rest).expect("read") > 0 {
        assert!(rest.contains("heartbeat"), "only beats may arrive after the reply is dropped: {rest}");
        rest.clear();
    }
}

/// What:
///  Queues one answer `delay` after the writer started, drops the reply, and checks that the client reads, in this order: any beats,
///  the answer, then only beats until the end of the stream.
/// Why:
///   Beats before an answer are allowed (the writer beats whenever nothing is queued for a beat's length), so the claim is that the
///  queued answer is written before the connection closes, not that it is the first line; a delay longer than a beat makes the
///  beats certain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(skipBeats(lines).next()).toEqual(landed); expect(lines.rest().every(isBeat)).toBe(true);
/// ```
fn queued_answer_is_written_before_the_connection_closes(delay: Duration) {
    let (reply, writer, mut client) = connected();
    let landed: ServiceMessage = ServiceMessage::Landed { id: String::from("x"), commit: String::from("c"), already: false };
    std::thread::sleep(delay);
    reply.send(&landed);
    drop(reply);
    writer.join().expect("the writer ends");
    let mut first_answer: ServiceMessage = next(&mut client);
    while first_answer == ServiceMessage::Heartbeat {
        first_answer = next(&mut client);
    }
    assert_eq!(first_answer, landed);
    let mut rest: String = String::new();
    while client.read_line(&mut rest).expect("read") > 0 {
        assert!(rest.contains("heartbeat"), "only beats may follow the answer: {rest}");
        rest.clear();
    }
}

/// A message queued before the reply is dropped is written before the connection closes, whether or not a beat came first.
#[test]
fn a_queued_answer_is_written_before_the_connection_closes() {
    queued_answer_is_written_before_the_connection_closes(Duration::ZERO);
}

/// POSITIVE CONTROL of the race: the answer is queued after the writer has beaten, as it is when the thread that sends it is
///  scheduled late (the gate of the branch merged with main at `944bdd30b5` read a `Heartbeat` first). The delay makes the beat
///  certain, and the answer still arrives before the close.
#[test]
fn a_queued_answer_is_written_before_the_connection_closes_even_when_the_sender_is_late() {
    queued_answer_is_written_before_the_connection_closes(QUICK_BEAT * 3);
}

/// A client that closed its end stops the writer on the next write.
#[test]
fn the_beat_ends_when_the_client_is_gone() {
    let (reply, writer, client) = connected();
    drop(client);
    writer.join().expect("a write to a closed client ends the writer");
    drop(reply);
}

/// Beats and answers come from one writer; every line the client reads is a whole message, however long, and the answers keep
/// the order they were queued in.
#[test]
fn beats_and_answers_never_interleave_their_bytes_and_keep_their_order() {
    let (reply, writer, mut client) = connected();
    // A long line, as a request sent back with many captures is, so a write is longer than one segment would be.
    let long_text: String = "w".repeat(200_000);
    let answerer = std::thread::spawn(move || {
        for number in 0..BEATS_WANTED * 10 {
            reply.send(&ServiceMessage::Waiting { text: format!("{number}:{long_text}") });
        }
    });
    let mut waits: usize = 0;
    while waits < BEATS_WANTED * 10 {
        match next(&mut client) {
            ServiceMessage::Heartbeat => {}
            ServiceMessage::Waiting { text } => {
                assert!(text.starts_with(format!("{waits}:").as_str()), "answers keep their order: {}", &text[..8]);
                waits += 1;
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    answerer.join().expect("answerer");
    writer.join().expect("the writer ends when the reply is dropped with the answerer");
}
