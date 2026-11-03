//! Controls for the beat a waiting client is sent and for the turns the beat and the answer take on one connection.

use super::*;
use crate::service_protocol::decode_service;
use std::io::{BufRead, BufReader};

/// How often the controls beat: fast enough that a control needing several beats takes well under a second.
const QUICK_BEAT: Duration = Duration::from_millis(20);

/// How many beats a control waits for before it is satisfied.
const BEATS_WANTED: usize = 3;

/// A reply and the client's end of its connection.
fn connected() -> (Reply, BufReader<UnixStream>) {
    let (service, client) = UnixStream::pair().expect("a connected pair");
    return (Reply::new(service), BufReader::new(client));
}

/// The next message the client reads.
fn next(client: &mut BufReader<UnixStream>) -> ServiceMessage {
    let mut line: String = String::new();
    client.read_line(&mut line).expect("a line arrives");
    return decode_service(line.trim_end_matches('\n')).expect("a message");
}

/// A request that waits for a long time is sent a beat the whole time, however long the wait; the beat is what the client
/// reads in place of a clock.
#[test]
fn a_waiting_client_is_sent_heartbeats_until_it_is_answered() {
    let (reply, mut client) = connected();
    let handle = reply.beat_handle();
    let beater = std::thread::spawn(move || { beat_while_held(&handle, QUICK_BEAT); });
    for _ in 0..BEATS_WANTED {
        assert_eq!(next(&mut client), ServiceMessage::Heartbeat);
    }
    reply.send(&ServiceMessage::Landed { id: String::from("x"), commit: String::from("c"), already: false });
    drop(reply);
    beater.join().expect("the beat ends once the reply is gone");
}

/// Dropping the reply, which is what answering the request does, ends the beat and closes the connection: the client
/// reads the end of the stream and no further beat.
#[test]
fn the_beat_ends_with_the_reply_and_the_connection_closes() {
    let (reply, mut client) = connected();
    let handle = reply.beat_handle();
    let beater = std::thread::spawn(move || { beat_while_held(&handle, QUICK_BEAT); });
    drop(reply);
    beater.join().expect("the beat ends");
    let mut rest: String = String::new();
    while client.read_line(&mut rest).expect("read") > 0 {
        assert!(rest.contains("heartbeat"), "only beats may arrive after the reply is dropped: {rest}");
        rest.clear();
    }
}

/// A client that closed its end stops the beat on the next write.
#[test]
fn the_beat_ends_when_the_client_is_gone() {
    let (reply, client) = connected();
    let handle = reply.beat_handle();
    drop(client);
    let beater = std::thread::spawn(move || { beat_while_held(&handle, QUICK_BEAT); });
    beater.join().expect("a write to a closed client ends the beat");
    drop(reply);
}

/// The beat and the answer come from two threads on one connection; every line the client reads is a whole message.
#[test]
fn beats_and_answers_never_interleave_their_bytes() {
    let (reply, mut client) = connected();
    let handle = reply.beat_handle();
    let beater = std::thread::spawn(move || { beat_while_held(&handle, Duration::from_millis(1)); });
    // A long line, as a request sent back with many captures is, so a write is longer than one segment would be.
    let long_text: String = "w".repeat(200_000);
    let answerer = std::thread::spawn(move || {
        for _ in 0..BEATS_WANTED * 10 {
            reply.send(&ServiceMessage::Waiting { text: long_text.clone() });
        }
    });
    let mut waits: usize = 0;
    while waits < BEATS_WANTED * 10 {
        match next(&mut client) {
            ServiceMessage::Heartbeat => {}
            ServiceMessage::Waiting { text } => {
                assert_eq!(text.len(), 200_000);
                waits += 1;
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    answerer.join().expect("answerer");
    beater.join().expect("the beat ends when the reply is dropped with the answerer");
}
