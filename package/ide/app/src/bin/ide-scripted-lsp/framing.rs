//! `Content-Length` framing on the standard streams,
//!  the report file,
//!  and pending client replies.

/// `json_value!` is `json!` with the one lint allowance the repository's `unwrap` ban needs.
use ide_app::json_value;
/// What:
///  `Value` is any JSON value;
///  `json!` builds one from literal syntax.
/// Why:
///  Every protocol message is one JSON object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::{Value, json};
/// What:
///  `HashMap` is a key-value table;
///  `File` an open file;
///  `BufRead` and `Write` are
///       the reading and writing interfaces;
///  `Mutex` guards a value so one thread uses it at a
///       time;
///  `mpsc` channels pass owned messages between threads;
///  `Duration` is a time span;
///       `SystemTime` and `UNIX_EPOCH` read the wall clock.
/// Why:
///  Delayed hover answers and the probe run on helper threads that share the output stream,
///      and every report line carries the wall-clock time it was written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { appendFileSync } from 'node:fs';
/// ```
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, BufRead, Write},
    path::Path,
    sync::{
        Mutex,
        mpsc::{Sender, channel},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// What:
///  Everything threads share.
///  Each field sits in a `Mutex`,
///  which hands out exclusive access.
/// Why:
///  Two threads writing one message each must not interleave their bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Wire { waiting = new Map<number, (reply: unknown) => void>(); nextId = 1000; }
/// ```
pub struct Wire {
    /// Standard output,
    ///  where framed messages go.
    output: Mutex<io::Stdout>,
    /// Report file,
    ///  when a test asked for one.
    report: Mutex<Option<File>>,
    /// Requests sent to the client that still await a reply,
    ///  by request number.
    waiting: Mutex<HashMap<u64, Sender<Value>>>,
    /// Next request number;
    ///  starts high so it is recognizable in logs.
    next_id: Mutex<u64>,
}

/// Shared-stream operations.
impl Wire {
    /// What:
    ///  Open the report file for appending when a path is given.
    ///  `Option<&Path>` is "a
    ///       borrowed path,
    ///  or nothing";
    ///  `io::Result<Self>` is success with a value or an I/O error.
    /// Why:
    ///  Tests read this file to learn what the server received and what text it holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(report?: string) { this.report = report; }
    /// ```
    pub fn new(report: Option<&Path>) -> io::Result<Self> {
        let file = match report {
            // The trailing `?` returns the I/O error to the caller when the file cannot be opened.
            Some(path) => Some(OpenOptions::new().create(true).append(true).open(path)?),
            None => None,
        };
        // `Ok(...)` is the success variant of `Result`.
        return Ok(Self {
            output: Mutex::new(io::stdout()),
            report: Mutex::new(file),
            waiting: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1000),
        });
    }

    /// What:
    ///  Write one framed message,
    ///  then record in the report that it was written.
    ///       `mut message` lets this function add the version field.
    /// Why:
    ///  The protocol requires a byte-length header before every JSON body.
    ///  The `sent` line
    ///      tells a test,
    ///  or a person reading a failure,
    ///  when the server wrote each message.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// send(message: object) {
    ///   const body = JSON.stringify({ jsonrpc: '2.0', ...message });
    ///   process.stdout.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
    /// }
    /// ```
    pub fn send(&self, mut message: Value) -> io::Result<()> {
        message["jsonrpc"] = json_value!("2.0");
        let body = message.to_string();
        // What: `lock()` waits for exclusive access and returns a guard; `expect` stops the
        //       program with this message if another thread panicked while holding the lock.
        // Why: Header and body must reach the stream as one uninterrupted unit.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // single-threaded: no lock needed
        // ```
        let mut output = self.output.lock().expect("output lock is not poisoned");
        write!(output, "Content-Length: {}\r\n\r\n{body}", body.len())?;
        output.flush()?;
        // `drop` releases the output lock before the report lock is taken.
        drop(output);
        self.record(json_value!({ "sent": { "id": message["id"], "method": message["method"] } }));
        return Ok(());
    }

    /// What:
    ///  Append one JSON line to the report,
    ///  if a report was requested,
    ///  with the wall-clock
    ///       time in milliseconds as `at`.
    ///  `mut entry` lets this function add that field.
    /// Why:
    ///  The times of the server's lines and of the client's log tell which side stalled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// record(entry: object) { appendFileSync(report, JSON.stringify({ ...entry, at: Date.now() }) + '\n'); }
    /// ```
    pub fn record(&self, mut entry: Value) {
        // What: `SystemTime::now().duration_since(UNIX_EPOCH)` is the time since 1970 as a
        //       `Result`; the `match` turns it into fractional milliseconds, or zero for a clock set
        //       before 1970. `as_secs_f64` gives seconds as a 64-bit floating-point number.
        // Why: JSON numbers are floating-point; milliseconds with a fraction keep sub-millisecond order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // entry.at = Date.now();
        // ```
        let at = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(since) => since.as_secs_f64() * 1000.0,
            Err(_) => 0.0,
        };
        entry["at"] = json_value!(at);
        let mut report = self.report.lock().expect("report lock is not poisoned");
        // `as_mut()` borrows the file inside the `Option` for writing.
        if let Some(file) = report.as_mut() {
            // A failed report write is only visible on standard error; the session continues.
            if let Err(error) = writeln!(file, "{entry}") {
                eprintln!("scripted language server cannot write its report: {error}");
            }
        }
    }

    /// What:
    ///  Send a request to the client and wait up to ten seconds for its reply.
    /// Why:
    ///  The probe needs each reply before sending the next request,
    ///  like a real server
    ///      that awaits `client/registerCapability`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async ask(method: string, params: unknown): Promise<unknown>
    /// ```
    pub fn ask(&self, method: &str, params: Value) -> Value {
        // `channel()` creates a sender and a receiver; the reply travels through them.
        let (sender, receiver) = channel();
        let id = {
            let mut next = self.next_id.lock().expect("id lock is not poisoned");
            // `*next` reads and writes the number behind the guard.
            *next += 1;
            *next
        };
        self.waiting
            .lock()
            .expect("waiting lock is not poisoned")
            .insert(id, sender);
        if let Err(error) = self.send(json_value!({ "id": id, "method": method, "params": params }))
        {
            return json_value!({ "sendFailed": error.to_string() });
        }
        // `recv_timeout` returns `Err` when nothing arrived in time; `unwrap_or_else` substitutes a marker.
        return match receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(answer) => answer,
            Err(_) => json!({ "unanswered": true }),
        };
    }

    /// Hand a client reply to the thread that asked.
    pub fn resolve(&self, id: u64, reply: Value) {
        let sender = self
            .waiting
            .lock()
            .expect("waiting lock is not poisoned")
            .remove(&id);
        if let Some(waiting) = sender {
            // The asking thread may have given up; a failed hand-over is not an error here.
            if let Err(error) = waiting.send(reply) {
                eprintln!("scripted language server dropped a late client reply: {error}");
            }
        }
    }
}

/// What:
///  Read one framed message,
///  or nothing at end of input.
///  `&mut impl BufRead` lends any
///       buffered reader for reading.
/// Why:
///  The client closes standard input when it drops the server;
///  that must end the loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readMessage(input: Reader): unknown | undefined
/// ```
pub fn read_message(input: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length: usize = 0;
    loop {
        // `String::new()` creates an empty owned text buffer for one header line.
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            // `None` inside `Ok`: the stream ended cleanly.
            return Ok(None);
        }
        let header = line.trim();
        if header.is_empty() {
            break;
        }
        // `strip_prefix` returns the rest of the line when it starts with the header name.
        if let Some(value) = header.strip_prefix("Content-Length: ") {
            length = value.parse().map_err(io::Error::other)?;
        }
    }
    // `vec![0; length]` creates a zero-filled byte list of the announced size.
    let mut body = vec![0; length];
    input.read_exact(&mut body)?;
    let message = serde_json::from_slice(&body).map_err(io::Error::other)?;
    return Ok(Some(message));
}
