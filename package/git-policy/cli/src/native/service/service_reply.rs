//! What:
//!  The service's side of one waiting client:
//! the queue of messages the landing thread answers with,
//! and the loop that writes
//!  them to the client's connection together with a heartbeat while nothing else is to say.
//! Why:
//!  A client's wait for an answer ends when the service goes silent (`service_client.rs`),
//! so what the service sends is the only sign that it is alive and working.
//! The task that read the request has nothing else to do while the request is queued or landing,
//! so it writes
//!  the heartbeats,
//! and a long batch,
//! a long queue or a slow disk in front of the landing never reads as a dead service.
//! The
//!  landing thread hands its messages to that same task through a channel instead of writing the connection itself,
//! so the
//!  connection has one writer,
//! a line is written whole,
//! and no lock guards it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [reply, messages] = makeReply(); queue.push({ request, reply }); await writeUntilAnswered(socket, messages, HEARTBEAT_EVERY);
//! ```

use crate::service_protocol::{ServiceMessage, encode_service};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// What:
///  The landing thread's end of one client's messages.
/// Why:
///   Dropping it is what answering the request ends with:
/// the writer sees the channel closed,
/// writes what is left,
/// and closes
///  the connection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Reply { constructor(private queue: Channel<ServiceMessage>) {} send(message) { this.queue.push(message) } }
/// ```
pub struct Reply {
    /// Where the messages for the client go.
    sender: UnboundedSender<ServiceMessage>,
}

/// How many requests this process has taken and not yet answered.
/// Why:
///   A landing that holds the index is what the commits behind it wait for,
/// and a count taken while it holds is
///  the only thing that says how many did;
/// the landing records it in the answer it writes.
static WAITING: AtomicUsize = AtomicUsize::new(0);

impl Reply {
    /// A reply and the end the writer task reads.
    pub fn new() -> (Self, UnboundedReceiver<ServiceMessage>) {
        let (sender, receiver) = unbounded_channel();
        let _taken = WAITING.fetch_add(1, Ordering::SeqCst);
        return (Self { sender }, receiver);
    }

    /// How many requests this process has taken and not yet answered,
    /// which is how many commits are waiting for a landing or for their turn.
    pub fn waiting() -> usize {
        return WAITING.load(Ordering::SeqCst);
    }

    /// Queues `message` for the client;
    /// a client that went away has nothing to be told.
    /// Never waits,
    /// so the landing thread that sends and a task that writes never hold each other.
    pub fn send(&self, message: &ServiceMessage) {
        let _queued = self.sender.send(message.clone());
    }

    /// What:
    ///  The channel this reply's messages go to,
    /// for a task that must answer through the connection's one writer
    ///  without counting as a request of its own.
    /// Why:
    ///   The check that decides whether a request is this instance's answers on the same connection as the landing,
    /// and two writers would interleave their bytes;
    /// a second `Reply` would count a request that never lands.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const channel = reply.sender();
    /// ```
    pub fn sender(&self) -> UnboundedSender<ServiceMessage> {
        return self.sender.clone();
    }
}

impl Drop for Reply {
    fn drop(&mut self) {
        let _answered = WAITING.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Writes `message` and a newline to `stream` as one write;
/// false when the client is gone.
async fn write_line<Stream: AsyncWrite + Unpin>(stream: &mut Stream, message: &ServiceMessage) -> bool {
    let mut line: String = encode_service(message);
    line.push('\n');
    return stream.write_all(line.as_bytes()).await.is_ok();
}

/// What:
///  Writes each queued message to the client,
/// and a heartbeat whenever `every` passes with nothing queued,
/// until the reply is
///  dropped or the client is gone.
/// Why:
///   The heartbeat is the proof of life the client's wait rests on;
/// the queued messages come first,
/// in order,
/// so a beat never
///  stands between a notice and the answer that follows it.
/// The writer is a task:
/// the wait for the next message is an `.await`,
/// so a connection that waits holds no thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const m = await messages.next({ timeout: every }); if (m === END) return; if (!write(m ?? heartbeat)) return; }
/// ```
pub async fn write_until_answered<Stream: AsyncWrite + Unpin>(stream: &mut Stream, messages: &mut UnboundedReceiver<ServiceMessage>, every: Duration) {
    write_until_answered_noting(stream, messages, every, &mut NothingToSay).await;
}

/// What:
///  Where a connection's beat asks for a message to write before the beat.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The beat asks at most once;
/// what it asks (the board,
/// the request's identity,
/// the last words said) is the state of a small
///  struct that implements this.
///  The question may wait on storage (the board is files in the journal),
/// so it is asked with an `.await`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface NoteSource { note(): Promise<ServiceMessage | undefined> }
/// ```
pub trait NoteSource {
    /// The message to write before the beat,
    /// or nothing.
    fn note(&mut self) -> impl std::future::Future<Output = Option<ServiceMessage>> + Send;
}

/// What:
///  A note source that never has a note.
/// Why:
///  `write_until_answered` is `write_until_answered_noting` with nothing to add before each beat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingToSay = { note: async (): Promise<ServiceMessage | undefined> => undefined };
/// ```
struct NothingToSay;

impl NoteSource for NothingToSay {
    async fn note(&mut self) -> Option<ServiceMessage> {
        return None;
    }
}

/// What:
///  `write_until_answered`,
/// where `note` is asked for a message whenever the beat is due,
/// and what it returns is written after that
///  beat.
/// Why:
///   A client queued behind a write that never ends is told what the service is stuck in (`service_busy.rs`);
/// the beat is the
///  one moment this task has nothing queued to write,
/// so the notice rides on it and is asked for at most once a beat.
///  The beat is written first because it is the proof of life the client's wait rests on,
/// and the note reads the journal,
///  which can wait on storage,
/// and that read is a job of the blocking pool a round of the after-commit work holds while it removes a
///  trash:
/// a note asked first would hold back the beat,
/// the answer behind it,
///  and every later beat,
/// which is what left a waiting client 27 to 32 s without a word
///  (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
///  A note still unanswered one beat later is left for the next beat,
/// so a read that storage holds costs a note and never a beat:
///  the writer's slowest interval becomes two beats,
/// which is well inside the five seconds a client allows between messages
///  (`HEARTBEAT_SILENCE`,
/// `service_protocol.rs`).
///  The source keeps the read it started (`StuckNotice`),
/// so the beat behind a slow read waits for that same read and its words are written as soon as it answers,
///  and a read that outlives many beats is made once and never queued behind itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const m = await messages.next({ timeout: every }); if (m === END) return; if (m === undefined) { if (!write(heartbeat)) return; const n = await withTimeout(note(), every); if (n && !write(n)) return; } if (!write(m ?? heartbeat)) return; }
/// ```
pub async fn write_until_answered_noting<Stream: AsyncWrite + Unpin, Notes: NoteSource>(
    stream: &mut Stream,
    messages: &mut UnboundedReceiver<ServiceMessage>,
    every: Duration,
    note: &mut Notes,
) {
    loop {
        match tokio::time::timeout(every, messages.recv()).await {
            Ok(Some(message)) => {
                if !write_line(stream, &message).await {
                    return;
                }
            }
            // The reply was dropped and everything it queued has been read.
            Ok(None) => return,
            Err(_every_passed) => {
                // The beat is the proof of life, so it is written before anything that may wait for the pool.
                if !write_line(stream, &ServiceMessage::Heartbeat).await {
                    return;
                }
                // A note still unanswered one beat later is left for the next beat, which waits for that same read rather than
                //  starting another behind a storage that is already late.
                if let Ok(Some(notice)) = tokio::time::timeout(every, note.note()).await
                    && !write_line(stream, &notice).await
                {
                    return;
                }
            }
        }
    }
}

/// The reply and its beat are controlled in the test module.
#[cfg(test)]
#[path = "service_reply_tests.rs"]
mod tests;
