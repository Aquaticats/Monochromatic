//! Controls for the bound on a connection to the commit service:
//! a socket that accepts comes back at once,
//! a socket whose queue of unaccepted connections is full ends the wait at the bound,
//! and a failure names the address the service was looked for at.

use super::*;
use std::os::unix::net::UnixListener;
use std::time::Instant;

/// The longest a control waits for anything,
/// so a broken bound fails it instead of holding the run.
const PATIENCE: Duration = Duration::from_secs(60);

/// A directory whose path is short enough for a socket name.
fn short_directory(name: &str) -> std::path::PathBuf {
    let home: std::path::PathBuf = std::path::PathBuf::from(std::env::var_os("HOME").expect("a home directory"));
    let directory: std::path::PathBuf = home.join("temp/agent").join(format!("cw-{}-{name}", std::process::id()));
    let _stale = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("short directory");
    return directory;
}

/// What:
///  A listening socket nothing accepts on,
///  with two connections already waiting in its queue of one.
/// Why:
///   A connect waits only for room in that queue,
/// so a full queue is the one state in which the bound has anything to end.
/// The listener and the connections that fill the queue are kept alive by the value that returns them: a closed listener
///  would make the connect refuse instead of wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const full = aFullQueue('full-queue'); // two connections went in, a third waits
/// ```
struct FullQueue {
    /// The directory the socket is in,
    /// removed at the end.
    directory: std::path::PathBuf,
    /// The socket name.
    socket: std::path::PathBuf,
    /// What keeps the connections of the queue alive.
    _held: Vec<std::os::unix::net::UnixStream>,
    /// The listener itself,
    /// which is what the queue belongs to.
    _listener: UnixListener,
}

impl Drop for FullQueue {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(self.directory.as_path());
    }
}

/// What:
///  A listening socket whose queue of unaccepted connections holds one,
///  which the standard library's bind does not let a caller choose.
/// Why:
///   Every other listener in this file takes the queue's default depth,
/// and the wait this file bounds is reached at a depth a control fills in a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listener = listenerWithAQueueOfOne(path); // socket, bind, listen(1)
/// ```
fn listener_with_a_queue_of_one(socket: &std::path::Path) -> UnixListener {
    use std::os::fd::FromRawFd as _;
    use std::os::unix::ffi::OsStrExt as _;
    let bytes: &[u8] = socket.as_os_str().as_bytes();
    // SAFETY: `socket` takes three integers and touches no memory of this process.
    let descriptor: libc::c_int = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
    assert!(descriptor >= 0, "a descriptor for the listener: {}", std::io::Error::last_os_error());
    // SAFETY: `sockaddr_un` is a plain structure of integers and a byte array, for which all zero bytes are a valid value.
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    address.sun_family = libc::sa_family_t::try_from(libc::AF_UNIX).expect("the address family fits");
    for (slot, byte) in address.sun_path.iter_mut().zip(bytes.iter()) {
        *slot = libc::c_char::try_from(*byte).unwrap_or(0);
    }
    let length: libc::socklen_t = libc::socklen_t::try_from(size_of::<libc::sa_family_t>() + bytes.len() + 1).expect("the address length fits");
    // SAFETY: the descriptor is open, and the address with its length is one this function built and the call only reads.
    let bound: libc::c_int = unsafe { libc::bind(descriptor, (&raw const address).cast(), length) };
    assert_eq!(bound, 0, "the listener is bound: {}", std::io::Error::last_os_error());
    // SAFETY: `listen` takes the open descriptor and a depth.
    let listening: libc::c_int = unsafe { libc::listen(descriptor, 1) };
    assert_eq!(listening, 0, "the listener listens: {}", std::io::Error::last_os_error());
    // SAFETY: the descriptor is a listening socket this function opened, and it is handed to the listener alone.
    return unsafe { UnixListener::from_raw_fd(descriptor) };
}

/// A socket whose queue of one already holds the two connections the kernel lets in.
fn a_full_queue(name: &str) -> FullQueue {
    let directory: std::path::PathBuf = short_directory(name);
    let socket: std::path::PathBuf = directory.join("s");
    let listener: UnixListener = listener_with_a_queue_of_one(socket.as_path());
    // The kernel lets one more connection into a queue than the depth asked for, so the third one is the one that waits.
    let mut held: Vec<std::os::unix::net::UnixStream> = Vec::new();
    for _ in 0..2 {
        held.push(std::os::unix::net::UnixStream::connect(socket.as_path()).expect("a connection into the queue"));
    }
    return FullQueue { directory, socket, _held: held, _listener: listener };
}

/// A connect to a socket whose listener accepts comes back well within the bound and gives a usable socket.
#[test]
fn a_socket_that_accepts_takes_the_connection() {
    let directory: std::path::PathBuf = short_directory("accepts");
    let socket: std::path::PathBuf = directory.join("s");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind");
    let accepted = std::thread::spawn(move || {
        let _taken = listener.accept();
    });
    let began: Instant = Instant::now();
    let connected: std::io::Result<std::os::unix::net::UnixStream> = connect_within_for(&Bind::Path(socket.clone()), PATIENCE);
    assert!(connected.is_ok(), "a socket whose listener accepts takes the connection");
    assert!(began.elapsed() < CONNECT_CEILING, "it comes back within the bound");
    accepted.join().expect("the accept thread ends");
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// What:
///  A connect into a queue nothing accepts waits no longer than the bound it was given.
/// Why:
///   This is the wait that has no end in the standard library's connect:
/// measured on this host on 2026-11-12,
/// a blocking connect into a queue of one that already held two connections was still waiting after 2,000 ms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => connectWithinFor(address, 200)).toThrow(/took no connection within/)
/// ```
#[test]
fn a_queue_nothing_accepts_ends_the_wait_at_its_bound() {
    let full: FullQueue = a_full_queue("full-queue");
    let given: Duration = Duration::from_millis(200);
    let began: Instant = Instant::now();
    let refused: std::io::Result<std::os::unix::net::UnixStream> = connect_within_for(&Bind::Path(full.socket.clone()), given);
    let took: Duration = began.elapsed();
    let error: std::io::Error = refused.expect_err("a queue nothing accepts takes no connection");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut, "the wait ends at the bound: {error}");
    assert!(took >= given, "the wait is what it was given, and not less: {took:?}");
    assert!(took < given + CONNECT_CEILING, "the wait ends near its bound: {took:?}");
    let words: String = error.to_string();
    assert!(words.contains(full.socket.display().to_string().as_str()), "the failure names the address: {words}");
    assert!(words.contains("took no connection within 200 ms"), "the failure names the wait: {words}");
}

/// A connect to a name nothing listens at fails at once and names the address.
#[test]
fn a_name_nothing_listens_at_fails_and_names_the_address() {
    let directory: std::path::PathBuf = short_directory("nothing");
    let socket: std::path::PathBuf = directory.join("nothing.sock");
    let refused: std::io::Result<std::os::unix::net::UnixStream> = connect_within_for(&Bind::Path(socket.clone()), PATIENCE);
    let error: std::io::Error = refused.expect_err("a name nothing listens at takes no connection");
    let words: String = error.to_string();
    assert!(words.contains(socket.display().to_string().as_str()), "the failure names the address: {words}");
    let _removed = std::fs::remove_dir_all(directory.as_path());
}

/// What:
///  A socket the kernel names rather than a file is connected to the same way.
/// Why:
///   A service started for a development or a control run listens at such a name,
/// and the one bound covers both kinds of
///  address.
#[cfg(target_os = "linux")]
#[test]
fn a_name_the_kernel_holds_takes_the_connection() {
    use std::os::linux::net::SocketAddrExt as _;
    let name: String = format!("cwa-{}-{}", std::process::id(), "bound");
    let listener: UnixListener = UnixListener::bind_addr(&std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("an abstract name")).expect("bind");
    let accepted = std::thread::spawn(move || {
        let _taken = listener.accept();
    });
    let connected: std::io::Result<std::os::unix::net::UnixStream> = connect_within_for(&Bind::Abstract(name), PATIENCE);
    assert!(connected.is_ok(), "an abstract name whose listener accepts takes the connection: {:?}", connected.err());
    accepted.join().expect("the accept thread ends");
}

/// What:
///  The bound of one connect is the silence a service that is working may keep.
/// Why:
///   A socket that has taken no connection for five beats is one nothing accepts on,
/// and a connect costs a few microseconds when
///  something does accept it,
/// so the bound costs a working commit nothing.
#[test]
fn the_connect_bound_is_the_silence_a_working_service_keeps() {
    assert_eq!(CONNECT_CEILING, Duration::from_secs(5));
    assert_eq!(CONNECT_CEILING, crate::service_protocol::HEARTBEAT_SILENCE);
    assert_eq!(connect_within_for(&Bind::Path(std::path::PathBuf::from("/nonexistent")), PATIENCE).is_err(), true);
}
