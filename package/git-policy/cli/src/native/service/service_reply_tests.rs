//! Controls for the beat a waiting client is sent and for the single writer that keeps beats and answers in order.
//!
//! The writer is a task.
//! Every control runs it on a current-thread runtime where one thread also runs the client's reads and a counting task,
//! so a writer that held its thread while it waited for a beat would leave the client no turn to read and the counter none to count.

use super::*;
use crate::service_protocol::decode_service;
use crate::ticker_beside_a_wait::with_a_ticker_beside;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixStream;
use tokio::task::JoinHandle;

/// How often the controls beat:
/// fast enough that a control needing several beats takes well under a second.
const QUICK_BEAT: Duration = Duration::from_millis(20);

/// How many beats a control waits for before it is satisfied.
const BEATS_WANTED: usize = 3;

/// A reply,
/// the task that writes it to a connection,
/// and the client's end of that connection.
fn connected() -> (Reply, JoinHandle<()>, BufReader<UnixStream>) {
    let (mut service, client) = UnixStream::pair().expect("a connected pair");
    let (reply, mut messages) = Reply::new();
    let writer = tokio::spawn(async move { write_until_answered(&mut service, &mut messages, QUICK_BEAT).await; });
    return (reply, writer, BufReader::new(client));
}

/// The next message the client reads.
async fn next(client: &mut BufReader<UnixStream>) -> ServiceMessage {
    let mut line: String = String::new();
    client.read_line(&mut line).await.expect("a line arrives");
    return decode_service(line.trim_end_matches('\n')).expect("a message");
}

/// Reads to the end of the stream and says whether every line is a beat.
async fn only_beats_until_the_end(client: &mut BufReader<UnixStream>) -> Result<(), String> {
    let mut rest: String = String::new();
    while client.read_line(&mut rest).await.expect("read") > 0 {
        if !rest.contains("heartbeat") {
            return Err(rest);
        }
        rest.clear();
    }
    return Ok(());
}

/// A request that waits for a long time is sent a beat the whole time,
/// however long the wait;
/// the beat is what the client reads
/// in place of a clock.
#[test]
fn a_waiting_client_is_sent_heartbeats_until_it_is_answered() {
    let ((), ticks) = with_a_ticker_beside(async {
        let (reply, writer, mut client) = connected();
        for _ in 0..BEATS_WANTED {
            assert_eq!(next(&mut client).await, ServiceMessage::Heartbeat);
        }
        let landed: ServiceMessage = ServiceMessage::Landed { id: String::from("x"), commit: String::from("c"), already: false, landing: None };
        reply.send(&landed);
        loop {
            let heard: ServiceMessage = next(&mut client).await;
            if heard != ServiceMessage::Heartbeat {
                assert_eq!(heard, landed);
                break;
            }
        }
        drop(reply);
        writer.await.expect("the writer ends once the reply is gone");
    });
    assert!(ticks > 0, "the writer waited for its beats on the runtime, so the counting task ran beside it");
}

/// Dropping the reply,
/// which is what answering the request does,
/// ends the writer and closes the connection:
/// the client reads
/// the end of the stream,
/// after only beats.
#[test]
fn the_beat_ends_with_the_reply_and_the_connection_closes() {
    let ((), _ticks) = with_a_ticker_beside(async {
        let (reply, writer, mut client) = connected();
        drop(reply);
        writer.await.expect("the writer ends");
        assert_eq!(only_beats_until_the_end(&mut client).await, Ok(()), "only beats may arrive after the reply is dropped");
    });
}

/// What:
///  Queues one answer `delay` after the writer started,
/// drops the reply,
/// and checks that the client reads,
/// in this order:
/// any beats,
///  the answer,
/// then only beats until the end of the stream.
/// Why:
///   Beats before an answer are allowed (the writer beats whenever nothing is queued for a beat's length),
/// so the claim is that the
///  queued answer is written before the connection closes,
/// not that it is the first line;
/// a delay longer than a beat makes the
///  beats certain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(skipBeats(lines).next()).toEqual(landed); expect(lines.rest().every(isBeat)).toBe(true);
/// ```
fn queued_answer_is_written_before_the_connection_closes(delay: Duration) {
    let ((), _ticks) = with_a_ticker_beside(async move {
        let (reply, writer, mut client) = connected();
        let landed: ServiceMessage = ServiceMessage::Landed { id: String::from("x"), commit: String::from("c"), already: false, landing: None };
        tokio::time::sleep(delay).await;
        reply.send(&landed);
        drop(reply);
        writer.await.expect("the writer ends");
        let mut first_answer: ServiceMessage = next(&mut client).await;
        while first_answer == ServiceMessage::Heartbeat {
            first_answer = next(&mut client).await;
        }
        assert_eq!(first_answer, landed);
        assert_eq!(only_beats_until_the_end(&mut client).await, Ok(()), "only beats may follow the answer");
    });
}

/// A message queued before the reply is dropped is written before the connection closes,
/// whether or not a beat came first.
#[test]
fn a_queued_answer_is_written_before_the_connection_closes() {
    queued_answer_is_written_before_the_connection_closes(Duration::ZERO);
}

/// POSITIVE CONTROL of the race:
/// the answer is queued after the writer has beaten,
/// as it is when the thread that sends it is
///  scheduled late (the gate of the branch merged with main at `944bdd30b5` read a `Heartbeat` first).
/// The delay makes the beat
///  certain,
/// and the answer still arrives before the close.
#[test]
fn a_queued_answer_is_written_before_the_connection_closes_even_when_the_sender_is_late() {
    queued_answer_is_written_before_the_connection_closes(QUICK_BEAT * 3);
}

/// A client that closed its end stops the writer on the next write.
#[test]
fn the_beat_ends_when_the_client_is_gone() {
    let ((), _ticks) = with_a_ticker_beside(async {
        let (reply, writer, client) = connected();
        drop(client);
        writer.await.expect("a write to a closed client ends the writer");
        drop(reply);
    });
}

/// Beats and answers come from one writer;
/// every line the client reads is a whole message,
/// however long,
/// and the answers keep
/// the order they were queued in.
#[test]
fn beats_and_answers_never_interleave_their_bytes_and_keep_their_order() {
    let ((), _ticks) = with_a_ticker_beside(async {
        let (reply, writer, mut client) = connected();
        // A long line, as a request sent back with many captures is, so a write is longer than one segment would be.
        let long_text: String = "w".repeat(200_000);
        // The sender is another thread, as the landing thread is.
        let answerer = std::thread::spawn(move || {
            for number in 0..BEATS_WANTED * 10 {
                reply.send(&ServiceMessage::Waiting { text: format!("{number}:{long_text}") });
            }
        });
        let mut waits: usize = 0;
        while waits < BEATS_WANTED * 10 {
            match next(&mut client).await {
                ServiceMessage::Heartbeat => {}
                ServiceMessage::Waiting { text } => {
                    assert!(text.starts_with(format!("{waits}:").as_str()), "answers keep their order: {}", &text[..8]);
                    waits += 1;
                }
                other => panic!("unexpected {other:?}"),
            }
        }
        answerer.join().expect("answerer");
        writer.await.expect("the writer ends when the reply is dropped with the answerer");
    });
}

/// The question asked at a beat may wait,
/// and a beat that waits on it does not stop the runtime:
/// a note source that takes a while to answer leaves the other tasks their turns.
/// The beat itself goes first and the note follows it,
/// because a note may wait for a place of the blocking pool that a round's removals hold,
/// and a waiting client ends a service it has heard nothing from for `HEARTBEAT_SILENCE`.
#[test]
fn a_note_that_takes_a_while_leaves_the_other_tasks_their_turns() {
    /// A note source that answers after a pause and says so.
    struct SlowNote;

    impl NoteSource for SlowNote {
        async fn note(&mut self) -> Option<ServiceMessage> {
            tokio::time::sleep(QUICK_BEAT).await;
            return Some(ServiceMessage::Waiting { text: String::from("slow") });
        }
    }

    let ((), ticks) = with_a_ticker_beside(async {
        let (mut service, mut client) = {
            let (service, client) = UnixStream::pair().expect("a connected pair");
            (service, BufReader::new(client))
        };
        let (reply, mut messages) = Reply::new();
        let writer = tokio::spawn(async move { write_until_answered_noting(&mut service, &mut messages, QUICK_BEAT, &mut SlowNote).await; });
        assert_eq!(next(&mut client).await, ServiceMessage::Heartbeat, "the beat never waits for the note that rides on it");
        assert_eq!(next(&mut client).await, ServiceMessage::Waiting { text: String::from("slow") }, "the note follows the beat that carried it");
        drop(reply);
        drop(client);
        writer.await.expect("the writer ends");
    });
    assert!(ticks > 0, "the note's wait was awaited on the runtime");
}
