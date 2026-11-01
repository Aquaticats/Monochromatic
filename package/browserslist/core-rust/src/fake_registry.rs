//! What:
//!  A tiny HTTP server on a loopback port for tests,
//! serving fixed bodies by request path.
//! Why:
//!  The decision record keeps network-dependent tests off the real registry;
//!  this stands in for `registry.npmjs.org` and Mozilla product-details.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const server = http.createServer((req, res) => { /* look up req.url in a routes map */ }).listen(0, '127.0.0.1');
//! ```

/// What:
///  Import `HashMap` from `std::collections::HashMap`.
/// Why:
///  Hash tables give linear-time lookups.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { HashMap } from 'std/collections';
/// ```
use std::collections::HashMap;
/// What:
///  Import `Read, Write` from `std::io`.
/// Why:
///  Byte streams are read and written through these traits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Read, Write } from 'std/io';
/// ```
use std::io::{Read, Write};
/// What:
///  Import `Shutdown, TcpListener, TcpStream` from `std::net`.
/// Why:
///  The fake registry listens on a loopback socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Shutdown, TcpListener, TcpStream } from 'std/net';
/// ```
use std::net::{Shutdown, TcpListener, TcpStream};
/// What:
///  Import `AtomicBool, Ordering` from `std::sync::atomic`.
/// Why:
///  The server thread and the test share state safely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AtomicBool, Ordering } from 'std/sync/atomic';
/// ```
use std::sync::atomic::{AtomicBool, Ordering};
/// What:
///  Import `Arc, Mutex` from `std::sync`.
/// Why:
///  The server thread and the test share state safely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arc, Mutex } from 'std/sync';
/// ```
use std::sync::{Arc, Mutex};
/// What:
///  Import `JoinHandle, sleep, spawn` from `std::thread`.
/// Why:
///  The fake registry serves from its own thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JoinHandle, sleep, spawn } from 'std/thread';
/// ```
use std::thread::{JoinHandle, sleep, spawn};
/// What:
///  Import `Duration` from `std::time::Duration`.
/// Why:
///  Waits and timeouts are bounded durations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Duration } from 'std/time';
/// ```
use std::time::Duration;

/// One canned answer:
/// status code and body.
type Route = (u16, Vec<u8>);

/// What:
///  A running fake server;
/// dropping it stops the server thread and joins it.
/// Why:
///  Tests must leave no thread or socket behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FakeRegistry { url(): string; requests(): string[]; setRoute(path: string, status: number, body: Uint8Array): void; close(): void }
/// ```
pub struct FakeRegistry {
    /// The base URL,
    /// such as `http://127.0.0.1:41234`.
    address: String,
    /// Request path to canned answer;
    /// shared with the server thread.
    routes: Arc<Mutex<HashMap<String, Route>>>,
    /// Every path served so far,
    /// in order.
    requests: Arc<Mutex<Vec<String>>>,
    /// Set to true to ask the server thread to finish.
    stop: Arc<AtomicBool>,
    /// The server thread;
    /// taken out when the registry is dropped.
    thread: Option<JoinHandle<()>>,
}

/// Read a request head and return its path,
/// or `None` when the client sent nothing usable.
fn read_path(stream: &mut TcpStream) -> Option<String> {
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let mut head: Vec<u8> = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !head.windows(4).any(|window| return window == b"\r\n\r\n") {
        let count = stream.read(&mut chunk).ok()?;
        if count == 0 {
            return None;
        }
        head.extend_from_slice(&chunk[..count]);
    }
    let text = String::from_utf8_lossy(&head).into_owned();
    let first_line = text.lines().next()?;
    return first_line.split(' ').nth(1).map(str::to_string);
}

/// Answer one connection from the routes table.
fn serve(mut stream: TcpStream, routes: &Mutex<HashMap<String, Route>>, requests: &Mutex<Vec<String>>) {
    let Some(path) = read_path(&mut stream) else {
        return;
    };
    requests.lock().expect("requests lock").push(path.clone());
    let (status, body) = routes.lock().expect("routes lock").get(&path).cloned().unwrap_or((404, b"not found".to_vec()));
    let head = format!("HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&body);
    let _ = stream.flush();
    let _ = stream.shutdown(Shutdown::Both);
}

/// What:
///  Methods of `FakeRegistry`.
/// Why:
///  Tests start the server,
/// read its URL and request log,
/// and change its routes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FakeRegistry { static start(routes): FakeRegistry; url(): string; requests(): string[]; setRoute(): void }
/// ```
impl FakeRegistry {
    /// Start serving `routes` (path to status and body) on a free loopback port.
    pub fn start(routes: Vec<(&str, u16, Vec<u8>)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        listener.set_nonblocking(true).expect("nonblocking listener");
        let address = format!("http://{}", listener.local_addr().expect("local address"));
        let table: HashMap<String, Route> =
            routes.into_iter().map(|(path, status, body)| return (path.to_string(), (status, body))).collect();
        let routes_shared = Arc::new(Mutex::new(table));
        let requests_shared = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_routes = Arc::clone(&routes_shared);
        let thread_requests = Arc::clone(&requests_shared);
        let thread_stop = Arc::clone(&stop);
        let thread = spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => serve(stream, &thread_routes, &thread_requests),
                    Err(_) => sleep(Duration::from_millis(5)),
                }
            }
        });
        return Self { address, routes: routes_shared, requests: requests_shared, stop, thread: Some(thread) };
    }

    /// The base URL,
    /// for example `http://127.0.0.1:41234`.
    pub fn url(&self) -> String {
        return self.address.clone();
    }

    /// Every request path served so far,
    /// in order.
    pub fn requests(&self) -> Vec<String> {
        return self.requests.lock().expect("requests lock").clone();
    }

    /// Add or replace a route while the server runs.
    pub fn set_route(&self, path: &str, status: u16, body: Vec<u8>) {
        self.routes.lock().expect("routes lock").insert(path.to_string(), (status, body));
    }
}

/// What:
///  `Drop` is Rust's destructor hook:
/// this code runs when the value goes out of scope.
/// Why:
///  The server thread is stopped and joined,
/// so no test leaves a thread or socket behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// afterEach(() => server.close());
/// ```
impl Drop for FakeRegistry {
    /// What:
    ///  The destructor body:
    /// stop the server thread and wait for it.
    /// Why:
    ///  A test must not leave a listening thread behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// server.close()
    /// ```
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
