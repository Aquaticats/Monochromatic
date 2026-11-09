//! What:
//!  A loopback HTTP server for the standards-review tests:
//! it takes one connection,
//! records the request it was sent,
//!  and answers with fixed bytes or stays silent.
//! Why:
//!  The review must be tested against a gateway that never leaves the machine,
//! so no repository text and no key can reach
//!  the real endpoint;
//! never part of the release executable.
//!  Every wait here is bounded and names what it waits for:
//! a call that never connects is an event the control reads,
//!  never a control left waiting at the test runner's stop,
//! and how an answer ended (written whole,
//!  cut off by the call,
//! held silent) is what the controls about cut-off calls judge instead of a clock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const server = createServer((req, res) => { seen = readBody(req); res.end(reply); }).listen(0, '127.0.0.1');
//! ```

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

/// What:
///  The quiet window a control gives a call whose subject is not the window.
/// Why:
///   A control that reads an answer must never be ended by the window before it has read that answer,
/// whatever the host is doing with this process.
///  Measured under `CPUQuota=3%` shared by the twelve controls of the client module:
/// a loopback reply was first read after more than
///  400 ms and within a few seconds,
/// so this figure is more than twenty times that delay and still far below the
///  `SCRIPT_QUIET_WINDOW` production uses.
/// A control this window does end fails with the client's own wording,
///  which names the call.
pub(crate) const ANSWER_WINDOW: Duration = Duration::from_secs(10);

/// What:
///  The ceiling of a control that expects its call to end by itself.
/// Why:
///   The outer bound ends a call that neither answers nor falls silent,
/// with the ceiling's wording naming the call,
///  well below nextest's five-minute stop.
/// A control that reaches it fails with a message naming the call instead of
///  leaving the test to be killed.
pub(crate) const CONTROL_CEILING: Duration = Duration::from_secs(30);

/// What:
///  How long the mock waits for the call it serves to connect.
/// Why:
///   `accept` has no deadline of its own,
/// so an unbounded wait left a control whose call never arrived at the runner's
///  five-minute stop (observed under a capped run:
/// the silent-server control was still running after 300 s).
///  A control that never sees its call now fails here,
/// naming this figure.
pub(crate) const CONNECTION_DEADLINE: Duration = Duration::from_secs(20);

/// How often the mock looks for its call while it waits.
const ACCEPT_POLL: Duration = Duration::from_millis(2);

/// What:
///  How long the mock waits for the request once the call connected.
/// Why:
///   A call that sends half a request and stops must end the mock,
/// not hold it.
pub(crate) const REQUEST_DEADLINE: Duration = Duration::from_secs(20);

/// How the mock answers.
#[derive(Clone, Debug)]
pub(crate) enum Answer {
    /// Write these bytes whole,
    /// then close.
    Bytes(Vec<u8>),
    /// Read the request,
    /// then hold the connection open without answering until the caller closes it,
    ///  or for this long at most.
    Silent(Duration),
    /// Read the request,
    /// then answer `200` with `body` in chunks of `chunk` bytes,
    /// framed with a chunked transfer encoding,
    ///  so the client sees several frames of one body and the mock names no length.
    Chunked {
        /// The whole body.
        body: Vec<u8>,
        /// How many body bytes each chunk carries.
        chunk: usize,
    },
    /// Read the request,
    /// then answer `200` with a body of this many bytes sent one at a time,
    /// a pause after each,
    ///  so the reply keeps arriving for as long as `pause * (bytes - 1)` and no longer idle than `pause` at any moment.
    Trickle {
        /// How many body bytes the reply holds.
        bytes: usize,
        /// The pause after each body byte.
        pause: Duration,
    },
}

/// How long a silent mock holds a connection at most:
/// far past anything a control waits for,
///  so the control reads the window's own wording and the hold is the slower alternative,
/// never waited for.
pub(crate) const SILENT_AT_MOST: Duration = Duration::from_secs(60);

/// What:
///  How the mock's answer ended.
/// Why:
///   This is the event a control about a cut-off call judges:
/// every byte written,
/// the call leaving before the end,
///  or the call closing a silence the server was only holding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AnswerEnded {
    /// Every byte of the answer was written,
    /// and the server closed the connection.
    Whole,
    /// The call closed the connection with this many body bytes still to write.
    CallerLeft {
        /// Body bytes the answer had left to write.
        unsent: usize,
    },
    /// The call closed the connection the server was only holding open,
    /// silent.
    CallerClosedTheSilence,
    /// The server's own hold on the silent connection ended first.
    HoldRanOut,
}

/// What:
///  What the one call the mock serves did.
/// Why:
///   A control reads the request and the event together,
/// so neither is guessed from a duration.
pub(crate) struct Served {
    /// The request as received,
    /// headers and body.
    pub(crate) seen: String,
    /// How the answer ended.
    pub(crate) ended: AnswerEnded,
}

/// A running mock:
/// its URL and the request bytes it recorded.
pub(crate) struct Mock {
    /// `http://127.0.0.1:<port>/v1/decisions`.
    pub(crate) url: String,
    /// The request as received,
    /// headers and body.
    pub(crate) seen: Arc<Mutex<Vec<u8>>>,
    /// The serving thread,
    /// joined by `finish`.
    handle: std::thread::JoinHandle<()>,
    /// How the answer ended,
    /// or nothing where no call connected.
    ended: Arc<Mutex<Option<AnswerEnded>>>,
}

/// An HTTP/1.1 reply with a status line,
/// a JSON content type and a content length.
pub(crate) fn http_reply(status_line: &str, body: &str) -> Vec<u8> {
    return format!("HTTP/1.1 {status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}", body.len()).into_bytes();
}

/// What:
///  The next connection within `deadline`,
/// or nothing.
/// Why:
///   A call that never arrives must be an event,
/// and this is the one wait that had no bound of its own.
pub(crate) fn accept_within(listener: &TcpListener, deadline: Duration) -> Option<TcpStream> {
    let began: Instant = Instant::now();
    loop {
        match listener.accept() {
            Ok((stream, _address)) => {
                let _blocking = stream.set_nonblocking(false);
                return Some(stream);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                let waited: Duration = began.elapsed();
                if waited >= deadline {
                    return None;
                }
                std::thread::sleep(ACCEPT_POLL.min(deadline - waited));
            }
            Err(_refused) => return None,
        }
    }
}

/// Read one request:
/// headers,
/// then as many body bytes as `content-length` says,
/// ending at the caller's close or at `REQUEST_DEADLINE`.
fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut received: Vec<u8> = Vec::new();
    let mut chunk: [u8; 4096] = [0; 4096];
    loop {
        let count: usize = stream.read(&mut chunk).unwrap_or(0);
        if count == 0 {
            break;
        }
        received.extend_from_slice(&chunk[..count]);
        let text: String = String::from_utf8_lossy(received.as_slice()).into_owned();
        if let Some(split) = text.find("\r\n\r\n") {
            let length: usize = text[..split]
                .to_ascii_lowercase()
                .lines()
                .find_map(|line| return line.strip_prefix("content-length:").and_then(|value| return value.trim().parse::<usize>().ok()))
                .unwrap_or(0);
            if received.len() >= split + 4 + length {
                break;
            }
        }
    }
    return received;
}

/// Write the answer and say how it ended:
/// every byte written,
/// the call leaving first,
/// or the call closing the silence the server held.
fn write_answer(stream: &mut TcpStream, answer: Answer) -> AnswerEnded {
    return match answer {
        Answer::Bytes(bytes) => {
            if stream.write_all(bytes.as_slice()).is_ok() { AnswerEnded::Whole } else { AnswerEnded::CallerLeft { unsent: bytes.len() } }
        }
        Answer::Chunked { body, chunk } => {
            let head: String = format!("HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ntransfer-encoding: chunked\r\nconnection: close\r\n\r\n");
            if stream.write_all(head.as_bytes()).is_err() {
                return AnswerEnded::CallerLeft { unsent: body.len() };
            }
            let mut offset: usize = 0;
            while offset < body.len() {
                let end: usize = (offset + chunk.max(1)).min(body.len());
                let piece: &[u8] = &body[offset..end];
                let size: String = format!("{:x}\r\n", piece.len());
                if stream.write_all(size.as_bytes()).is_err() || stream.write_all(piece).is_err() || stream.write_all(b"\r\n").is_err() {
                    return AnswerEnded::CallerLeft { unsent: body.len() - offset };
                }
                offset = end;
            }
            if stream.write_all(b"0\r\n\r\n").is_err() {
                return AnswerEnded::CallerLeft { unsent: 0 };
            }
            AnswerEnded::Whole
        }
        Answer::Trickle { bytes, pause } => {
            let head: String = format!("HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {bytes}\r\nconnection: close\r\n\r\n");
            if stream.write_all(head.as_bytes()).is_err() {
                return AnswerEnded::CallerLeft { unsent: bytes };
            }
            for sent in 0..bytes {
                if stream.write_all(b"x").is_err() {
                    return AnswerEnded::CallerLeft { unsent: bytes - sent };
                }
                // The pause before the next byte and never one after the last: the answer is complete at its last byte.
                if sent + 1 < bytes {
                    std::thread::sleep(pause);
                }
            }
            AnswerEnded::Whole
        }
        Answer::Silent(hold) => {
            let _bounded = stream.set_read_timeout(Some(hold));
            let mut ignored: [u8; 64] = [0; 64];
            loop {
                match stream.read(&mut ignored) {
                    Ok(0) => return AnswerEnded::CallerClosedTheSilence,
                    Ok(_said_something) => continue,
                    Err(_hold_ran_out) => return AnswerEnded::HoldRanOut,
                }
            }
        }
    };
}

/// What:
///  Serve the one call of `answer` and record what it did.
/// Why:
///   Both facts a control reads are written before the thread ends,
/// so `finish` never has to wait for one after the other.
fn serve(listener: &TcpListener, answer: Answer, recorded: &Arc<Mutex<Vec<u8>>>, ended: &Arc<Mutex<Option<AnswerEnded>>>) {
    // A call that never connects leaves `ended` empty, which is the event `finish` reports.
    let Some(mut stream) = accept_within(listener, CONNECTION_DEADLINE) else {
        return;
    };
    let _request_bound = stream.set_read_timeout(Some(REQUEST_DEADLINE));
    let request: Vec<u8> = read_request(&mut stream);
    recorded.lock().unwrap_or_else(PoisonError::into_inner).extend_from_slice(request.as_slice());
    let how: AnswerEnded = write_answer(&mut stream, answer);
    *ended.lock().unwrap_or_else(PoisonError::into_inner) = Some(how);
}

impl Mock {
    /// Start serving one connection with `answer`.
    pub(crate) fn start(answer: Answer) -> Mock {
        let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let port: u16 = listener.local_addr().expect("address").port();
        // The mock polls for its call so that a call that never arrives ends the wait with a name.
        listener.set_nonblocking(true).expect("a listener the mock polls");
        let seen: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let ended: Arc<Mutex<Option<AnswerEnded>>> = Arc::new(Mutex::new(None));
        let (recorded, ending) = (Arc::clone(&seen), Arc::clone(&ended));
        let handle: std::thread::JoinHandle<()> = std::thread::spawn(move || serve(&listener, answer, &recorded, &ending));
        return Mock { url: format!("http://127.0.0.1:{port}/v1/decisions"), seen, handle, ended };
    }

    /// What:
    ///  Wait for the serving thread and report what the call did.
    /// Why:
    ///   A mock whose call never connected fails the control here,
    /// naming the wait and its figure,
    /// so
    ///  the control it serves cannot pass having proved nothing and cannot wait at the runner's stop.
    pub(crate) fn finish(self) -> Served {
        let _joined = self.handle.join();
        let bytes: Vec<u8> = self.seen.lock().unwrap_or_else(PoisonError::into_inner).clone();
        let seen: String = String::from_utf8_lossy(bytes.as_slice()).into_owned();
        let Some(ended) = *self.ended.lock().unwrap_or_else(PoisonError::into_inner) else {
            panic!("no call reached the mock server at {} within {CONNECTION_DEADLINE:?}, so the control it serves proved nothing", self.url);
        };
        return Served { seen, ended };
    }
}
