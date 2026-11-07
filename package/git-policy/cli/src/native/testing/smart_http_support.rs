//! What:
//!  A minimal HTTP/1.1 server for tests that puts `git http-backend` (Git's own smart-HTTP CGI program) behind a loopback port.
//! Why:
//!  `git daemon` is not installed with Git here,
//! and a push over HTTP is the transport whose bytes the push meter cannot see
//!  (`libcurl` uses `send` and `recv`).
//! The server runs the real `git http-backend` for every request,
//! so the remote side of the
//!  push (`receive-pack`,
//! its hooks,
//! its keepalive) is Git's,
//! and only the HTTP framing is the test's:
//! one request per
//!  connection,
//! `Connection: close`,
//! a body with `Content-Length` or `chunked`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const server = await SmartHttp.start(projectRoot); await push(`http://127.0.0.1:${server.port}/remote.git`); server.stop();
//! ```
#![cfg(unix)]

use crate::test_support::REAL_GIT;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

/// A running server:
/// the port it listens on,
/// and the thread that accepts.
pub(crate) struct SmartHttp {
    /// The loopback port,
    /// chosen by the system.
    port: u16,
    /// Set when the server is being stopped,
    /// so the accepting thread ends at the next connection.
    stopping: Arc<AtomicBool>,
    /// The thread that accepts connections and joins the threads that serve them.
    acceptor: Option<JoinHandle<()>>,
}

impl SmartHttp {
    /// Start serving the repositories under `project_root`.
    pub(crate) fn start(project_root: &Path) -> Self {
        let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
        let port: u16 = listener.local_addr().expect("its address").port();
        let stopping: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let flag: Arc<AtomicBool> = Arc::clone(&stopping);
        let root: PathBuf = project_root.to_path_buf();
        let acceptor: JoinHandle<()> = std::thread::spawn(move || {
            let mut serving: Vec<JoinHandle<()>> = Vec::new();
            for incoming in listener.incoming() {
                if flag.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = incoming else { continue };
                let served_root: PathBuf = root.clone();
                serving.push(std::thread::spawn(move || { serve(stream, served_root.as_path()); }));
            }
            for thread in serving {
                let _ended = thread.join();
            }
        });
        return Self { port, stopping, acceptor: Some(acceptor) };
    }

    /// The port the server listens on.
    pub(crate) fn port(&self) -> u16 {
        return self.port;
    }
}

impl Drop for SmartHttp {
    /// Stop accepting:
    /// set the flag,
    /// make one connection so the blocked accept returns,
    /// and wait for the threads.
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        let _woken = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(acceptor) = self.acceptor.take() {
            let _ended = acceptor.join();
        }
    }
}

/// Read one chunked body:
/// size line in hexadecimal,
/// that many bytes and a line break,
/// until a chunk of size zero.
fn read_chunked(reader: &mut BufReader<TcpStream>) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    loop {
        let mut line: String = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return body;
        }
        let size: usize = usize::from_str_radix(line.trim().split(';').next().unwrap_or("0"), 16).unwrap_or(0);
        if size == 0 {
            let mut trailer: String = String::new();
            let _read = reader.read_line(&mut trailer);
            return body;
        }
        let mut chunk: Vec<u8> = vec![0; size];
        if reader.read_exact(&mut chunk).is_err() {
            return body;
        }
        body.extend_from_slice(&chunk);
        let mut break_line: String = String::new();
        let _read = reader.read_line(&mut break_line);
    }
}

/// Serve one connection:
/// one request,
/// answered by `git http-backend`.
fn serve(stream: TcpStream, project_root: &Path) {
    let Ok(clone) = stream.try_clone() else { return };
    let mut reader: BufReader<TcpStream> = BufReader::new(clone);
    let mut request_line: String = String::new();
    if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
        return;
    }
    let mut parts = request_line.split_whitespace();
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else { return };
    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut line: String = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    if headers.get("expect").is_some_and(|value| { return value.eq_ignore_ascii_case("100-continue"); }) {
        let mut writer: &TcpStream = &stream;
        let _sent = writer.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
    }
    let body: Vec<u8> = if headers.get("transfer-encoding").is_some_and(|value| { return value.contains("chunked"); }) {
        read_chunked(&mut reader)
    } else {
        let length: usize = headers.get("content-length").and_then(|value| { return value.parse::<usize>().ok(); }).unwrap_or(0);
        let mut bytes: Vec<u8> = vec![0; length];
        if reader.read_exact(&mut bytes).is_err() {
            return;
        }
        bytes
    };
    let (path, query): (&str, &str) = target.split_once('?').unwrap_or((target, ""));
    let mut cgi: Command = Command::new(REAL_GIT);
    cgi.arg("http-backend")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_PROJECT_ROOT", project_root)
        .env("GIT_HTTP_EXPORT_ALL", "1")
        .env("REQUEST_METHOD", method)
        .env("PATH_INFO", path)
        .env("QUERY_STRING", query)
        .env("CONTENT_LENGTH", body.len().to_string())
        .env("REMOTE_ADDR", "127.0.0.1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(kind) = headers.get("content-type") {
        cgi.env("CONTENT_TYPE", kind);
    }
    if let Some(protocol) = headers.get("git-protocol") {
        cgi.env("GIT_PROTOCOL", protocol);
    }
    let Ok(mut child) = cgi.spawn() else { return };
    if let Some(mut input) = child.stdin.take() {
        let _written = input.write_all(&body);
    }
    let Ok(finished) = child.wait_with_output() else { return };
    let output: Vec<u8> = finished.stdout;
    let split: usize = output.windows(4).position(|window| { return window == b"\r\n\r\n"; }).map_or(output.len(), |at| { return at; });
    let head: String = String::from_utf8_lossy(&output[..split]).into_owned();
    let payload: &[u8] = output.get(split + 4..).unwrap_or(&[]);
    let mut status: String = String::from("200 OK");
    let mut response_headers: String = String::new();
    for line in head.lines() {
        if let Some(value) = line.strip_prefix("Status:") {
            status = value.trim().to_owned();
        } else if !line.is_empty() {
            response_headers.push_str(line);
            response_headers.push_str("\r\n");
        }
    }
    let mut writer: &TcpStream = &stream;
    let _sent = writer.write_all(format!("HTTP/1.1 {status}\r\n{response_headers}Content-Length: {}\r\nConnection: close\r\n\r\n", payload.len()).as_bytes());
    let _payload_sent = writer.write_all(payload);
}
