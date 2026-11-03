//! What:
//!  The service's side of one waiting client: the connection the answer will be written to, and the beat that tells the
//!  client the service is alive until the answer is written.
//! Why:
//!  A client waits for its answer with no clock of its own (`service_client.rs`), so the only thing that ends its wait is
//!  silence. The beat comes from the thread that read the request, which has nothing else to do while the request is queued
//!  or landing, so a long batch, a long queue or a slow disk in front of the landing never reads as a dead service. The
//!  landing thread writes the same connection, so the two take turns on it: a line is written whole or not at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reply = new Reply(socket); queue.push({ request, reply }); await beatWhileHeld(reply.weak(), HEARTBEAT_EVERY);
//! ```

use super::service_protocol::{ServiceMessage, encode_service};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::time::Duration;

/// What:
///  The connection of one queued request, shared by the thread that beats and the thread that answers.
/// Why:
///   The lock covers one write of one line and nothing else: a heartbeat and an answer from two threads must not interleave
///  their bytes on one stream, and a line of the answer can be longer than one kernel buffer segment. No other lock is held
///  while it is taken, so nothing can wait on it for longer than one write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Reply { constructor(private socket: Mutex<Socket>) {} send(message) { ... } }
/// ```
pub struct Reply {
    /// The connection; dropping the last strong holder closes it.
    stream: Arc<Mutex<UnixStream>>,
}

impl Reply {
    /// The reply over `stream`.
    pub fn new(stream: UnixStream) -> Self {
        return Self { stream: Arc::new(Mutex::new(stream)) };
    }

    /// Writes `message` as one line; a client that went away has nothing to be told.
    pub fn send(&self, message: &ServiceMessage) {
        write_line(&self.stream, message);
    }

    /// A handle that does not keep the connection open, for the thread that beats.
    pub fn beat_handle(&self) -> Weak<Mutex<UnixStream>> {
        return Arc::downgrade(&self.stream);
    }
}

/// Writes `message` and a newline to the stream behind `shared`, as one write.
fn write_line(shared: &Mutex<UnixStream>, message: &ServiceMessage) {
    let mut line: String = encode_service(message);
    line.push('\n');
    // A poisoned lock means another writer panicked between two writes of its own; this line is still whole.
    let stream = shared.lock().unwrap_or_else(PoisonError::into_inner);
    let mut writer: &UnixStream = &stream;
    let _written = writer.write_all(line.as_bytes());
}

/// What:
///  Sends a heartbeat every `every` until the reply is gone or the client is.
/// Why:
///   The reply is gone when the request has been answered, so the beat ends with the answer and needs no flag; a write
///  that fails means the client closed its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (true) { await sleep(every); const reply = weak.deref(); if (!reply || !reply.tryWrite(heartbeat)) return; }
/// ```
pub fn beat_while_held(handle: &Weak<Mutex<UnixStream>>, every: Duration) {
    loop {
        std::thread::sleep(every);
        let Some(shared) = handle.upgrade() else { return };
        let mut line: String = encode_service(&ServiceMessage::Heartbeat);
        line.push('\n');
        let stream = shared.lock().unwrap_or_else(PoisonError::into_inner);
        let mut writer: &UnixStream = &stream;
        if writer.write_all(line.as_bytes()).is_err() {
            return;
        }
    }
}

/// The reply and its beat are controlled in the test module.
#[cfg(test)]
#[path = "service_reply_tests.rs"]
mod tests;
