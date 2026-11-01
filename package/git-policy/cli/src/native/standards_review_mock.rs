//! What:
//!  A loopback HTTP server for the standards-review tests: it takes one connection, records the request it was sent,
//!  and answers with fixed bytes or stays silent.
//! Why:
//!  The review must be tested against a gateway that never leaves the machine, so no repository text and no key can reach
//!  the real endpoint; never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const server = createServer((req, res) => { seen = readBody(req); res.end(reply); }).listen(0, '127.0.0.1');
//! ```

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// How the mock answers.
#[derive(Clone, Debug)]
pub(crate) enum Answer {
    /// Write these bytes whole, then close.
    Bytes(Vec<u8>),
    /// Read the request, then hold the connection open without answering for this long.
    Silent(Duration),
}

/// A running mock: its URL and the request bytes it recorded.
pub(crate) struct Mock {
    /// `http://127.0.0.1:<port>/v1/decisions`.
    pub(crate) url: String,
    /// The request as received, headers and body.
    pub(crate) seen: Arc<Mutex<Vec<u8>>>,
    /// The serving thread, joined by `finish`.
    handle: std::thread::JoinHandle<()>,
}

/// An HTTP/1.1 reply with a status line, a JSON content type and a content length.
pub(crate) fn http_reply(status_line: &str, body: &str) -> Vec<u8> {
    return format!("HTTP/1.1 {status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}", body.len()).into_bytes();
}

/// Read one request: headers, then as many body bytes as `content-length` says.
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

impl Mock {
    /// Start serving one connection with `answer`.
    pub(crate) fn start(answer: Answer) -> Mock {
        let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let port: u16 = listener.local_addr().expect("address").port();
        let seen: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let recorded: Arc<Mutex<Vec<u8>>> = Arc::clone(&seen);
        let handle: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let request: Vec<u8> = read_request(&mut stream);
            recorded.lock().unwrap_or_else(std::sync::PoisonError::into_inner).extend_from_slice(request.as_slice());
            match answer {
                Answer::Bytes(bytes) => {
                    let _ = stream.write_all(bytes.as_slice());
                }
                Answer::Silent(hold) => std::thread::sleep(hold),
            }
        });
        return Mock { url: format!("http://127.0.0.1:{port}/v1/decisions"), seen, handle };
    }

    /// Wait for the serving thread and return the request it recorded.
    pub(crate) fn finish(self) -> String {
        let _ = self.handle.join();
        let bytes: Vec<u8> = self.seen.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        return String::from_utf8_lossy(bytes.as_slice()).into_owned();
    }
}
