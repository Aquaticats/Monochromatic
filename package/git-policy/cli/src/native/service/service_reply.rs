//! What:
//!  The service's side of one waiting client: the queue of messages the landing thread answers with, and the loop that writes
//!  them to the client's connection together with a heartbeat while nothing else is to say.
//! Why:
//!  A client waits for its answer with no clock of its own (`service_client.rs`), so the only thing that ends its wait is
//!  silence. The thread that read the request has nothing else to do while the request is queued or landing, so it writes
//!  the heartbeats, and a long batch, a long queue or a slow disk in front of the landing never reads as a dead service. The
//!  landing thread hands its messages to that same thread through a channel instead of writing the connection itself, so the
//!  connection has one writer, a line is written whole, and no lock guards it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [reply, messages] = makeReply(); queue.push({ request, reply }); await writeUntilAnswered(socket, messages, HEARTBEAT_EVERY);
//! ```

use crate::service_protocol::{ServiceMessage, encode_service};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

/// What:
///  The landing thread's end of one client's messages.
/// Why:
///   Dropping it is what answering the request ends with: the writer sees the channel closed, writes what is left, and closes
///  the connection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Reply { constructor(private queue: Channel<ServiceMessage>) {} send(message) { this.queue.push(message) } }
/// ```
pub struct Reply {
    /// Where the messages for the client go.
    sender: Sender<ServiceMessage>,
}

impl Reply {
    /// A reply and the end the writer loop reads.
    pub fn new() -> (Self, Receiver<ServiceMessage>) {
        let (sender, receiver) = channel();
        return (Self { sender }, receiver);
    }

    /// Queues `message` for the client; a client that went away has nothing to be told.
    pub fn send(&self, message: &ServiceMessage) {
        let _queued = self.sender.send(message.clone());
    }
}

/// Writes `message` and a newline to `stream` as one write; false when the client is gone.
fn write_line(stream: &UnixStream, message: &ServiceMessage) -> bool {
    let mut line: String = encode_service(message);
    line.push('\n');
    let mut writer: &UnixStream = stream;
    return writer.write_all(line.as_bytes()).is_ok();
}

/// What:
///  Writes each queued message to the client, and a heartbeat whenever `every` passes with nothing queued, until the reply is
///  dropped or the client is gone.
/// Why:
///   The heartbeat is the proof of life the client's wait rests on; the queued messages come first, in order, so a beat never
///  stands between a notice and the answer that follows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const m = await messages.next({ timeout: every }); if (m === END) return; if (!write(m ?? heartbeat)) return; }
/// ```
pub fn write_until_answered(stream: &UnixStream, messages: &Receiver<ServiceMessage>, every: Duration) {
    loop {
        match messages.recv_timeout(every) {
            Ok(message) => {
                if !write_line(stream, &message) {
                    return;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if !write_line(stream, &ServiceMessage::Heartbeat) {
                    return;
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

/// The reply and its beat are controlled in the test module.
#[cfg(test)]
#[path = "service_reply_tests.rs"]
mod tests;
