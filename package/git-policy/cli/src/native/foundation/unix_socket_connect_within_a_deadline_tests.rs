//! Controls for a connection that is made within one deadline:
//!  a socket a program listens on is connected to and read through,
//!  a socket file that is gone answers its own failure,
//!  a socket whose listener is behind answers the deadline and not a wait of its own,
//!  and a path that cannot fit one socket address is refused.

use super::*;
use std::io::{BufRead, BufReader, Write};
use std::os::fd::OwnedFd;
use std::path::PathBuf;

/// The span these controls give a connection,
/// which is short enough to keep a control that must time out quick and long enough for a host under load.
const WAIT: Duration = Duration::from_millis(200);

/// What:
///  A folder of this test process whose path one socket address holds,
///  and that the thread's own cleanup removes.
/// Why:
///  One socket address holds 107 bytes and the fixture root under the build directory is longer than that,
///  so a control of a socket keeps its own folder somewhere short,
///  which the test support tracks for this thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = shortFolder('a-control');
/// ```
fn short_folder(name: &str) -> PathBuf {
    let root: PathBuf = std::env::var_os("HOME").map(PathBuf::from).expect("a home directory").join("temp/agent").join(format!("sock{}", std::process::id()));
    let folder: PathBuf = root.join(name);
    std::fs::create_dir_all(folder.as_path()).expect("the short folder");
    crate::test_support::track_for_this_thread(root.as_path());
    return folder;
}

/// What:
///  A listening socket of this test process at `path`,
///  with the backlog it is given,
///  which nothing in the control accepts from.
/// Why:
///  A backlog is what decides how many connections a socket takes before the next one waits,
///  and `std::os::unix::net::UnixListener::bind` asks for a large one,
///  so a control of the waiting connection makes its own listener.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listener = aListenerThatNeverAccepts(path, 0);
/// ```
fn a_listener_that_never_accepts(path: &Path, backlog: libc::c_int) -> OwnedFd {
    let (address, length): (libc::sockaddr_un, libc::socklen_t) = address_of(path).expect("an address of the fixture path");
    // SAFETY: `socket` creates a descriptor of the family and type it is given and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
    assert!(made >= 0, "a socket for the control: {}", io::Error::last_os_error());
    // SAFETY: `made` is an open descriptor of this process and nothing else owns it.
    let listener: OwnedFd = unsafe { OwnedFd::from_raw_fd(made) };
    // SAFETY: `bind` reads the address it is given and the length says how much of it; the descriptor is this process's.
    let bound: libc::c_int = unsafe { libc::bind(listener.as_raw_fd(), std::ptr::from_ref(&address).cast::<libc::sockaddr>(), length) };
    assert_eq!(bound, 0, "the control's socket is bound: {}", io::Error::last_os_error());
    // SAFETY: `listen` reads the descriptor and the number it is given and touches no memory.
    let listening: libc::c_int = unsafe { libc::listen(listener.as_raw_fd(), backlog) };
    assert_eq!(listening, 0, "the control's socket listens: {}", io::Error::last_os_error());
    return listener;
}

/// What:
///  A socket a program listens on is connected to within the deadline,
///  and the stream it answers is a blocking stream a caller reads a line from.
/// Why:
///  Everything above is written as if the connected stream blocks;
///  this is the control that the flag is cleared and that a timeout on the read finds a line.
#[test]
fn a_socket_a_program_listens_on_is_connected_to() {
    let root: PathBuf = short_folder("connected");
    let path: PathBuf = root.join("listening.sock");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind(path.as_path()).expect("a listener");
    let answering = std::thread::spawn(move || {
        let (mut accepted, _address) = listener.accept().expect("the control's connection");
        accepted.write_all(b"one line\n").expect("an answer");
    });
    let stream: UnixStream = connect_within(path.as_path(), WAIT).expect("a connection");
    stream.set_read_timeout(Some(WAIT)).expect("a read timeout");
    let mut answer: String = String::new();
    BufReader::new(stream).read_line(&mut answer).expect("the answer");
    assert_eq!(answer, String::from("one line\n"));
    answering.join().expect("the answering thread ends");
}

/// What:
///  A socket path nothing listens on answers the operating system's own failure,
///  and the wait is not spent on it.
#[test]
fn a_socket_file_that_is_gone_answers_its_own_failure() {
    let path: PathBuf = short_folder("gone").join("nothing-here.sock");
    let failed: io::Error = connect_within(path.as_path(), WAIT).expect_err("no socket to connect to");
    assert_eq!(failed.kind(), io::ErrorKind::NotFound, "{failed}");
}

/// What:
///  A socket whose listener takes no more connections answers the deadline:
///  the connection is refused nothing and made nothing,
///  and only the wait ends it.
/// Why:
///  This is the case a plain `UnixStream::connect` waits for ever in:
///  a listener whose program is stopped accepts nothing and refuses nothing.
#[test]
fn a_socket_whose_listener_is_behind_answers_the_deadline() {
    let path: PathBuf = short_folder("behind").join("behind.sock");
    let _listener: OwnedFd = a_listener_that_never_accepts(path.as_path(), 0);
    // The listener's backlog is zero, so the first connection fills it and every one after it waits for an accept nothing does.
    let _filling: UnixStream = UnixStream::connect(path.as_path()).expect("the connection the backlog takes");
    let began: Instant = Instant::now();
    let failed: io::Error = connect_within(path.as_path(), WAIT).expect_err("the listener takes no more connections");
    let waited: Duration = began.elapsed();
    assert_eq!(failed.kind(), io::ErrorKind::TimedOut, "{failed}");
    assert!(waited >= WAIT, "the deadline is what ended the wait, and it took {waited:?}");
    assert!(waited < WAIT * 10, "the wait is the deadline and not a wait of its own: {waited:?}");
}

/// What:
///  A path that cannot fit one socket address is refused,
///  and nothing is connected to a name it was cut down to.
#[test]
fn a_path_longer_than_one_socket_address_is_refused() {
    let path: PathBuf = short_folder("long").join("a".repeat(120));
    let failed: io::Error = connect_within(path.as_path(), WAIT).expect_err("a path no socket address holds");
    assert_eq!(failed.kind(), io::ErrorKind::InvalidInput, "{failed}");
}
