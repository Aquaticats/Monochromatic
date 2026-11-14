//! Controls for the look at a running service's start arguments,
//! and for the one-line request that follows it:
//!  a look whose read never answers ends at its ceiling,
//!  a queue nothing accepts ends the request at its bound,
//!  and a socket that accepts receives the client's warm line.

use super::*;
use std::io::{BufRead, BufReader};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::net::UnixListener;
use std::time::{Duration, Instant};
use crate::wait_ceilings::WARM_SCAN_CEILING;

/// The words of a start command line.
fn words(text: &[&str]) -> Vec<Vec<u8>> {
    return text.iter().map(|word| { return word.as_bytes().to_vec(); }).collect();
}

/// What:
///  A command line nothing writes:
///  a read of it answers only when a writer closes the file,
///  and no writer ever opens this one.
/// Why:
///  The look can be held by a process stuck in the kernel on this machine,
///  which no control may build here (`rules-every-pi-brief-carries.md`),
///  so a control holds the read itself instead:
///  the state the ceiling exists for is a read that does not answer,
///  and a pipe answers that without any process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(folder.join('4242/cmdline'), 0o600);
/// ```
fn a_command_line_nothing_writes(path: &Path) {
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("a path without a zero byte");
    // SAFETY: `mkfifo` reads the name it is given and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the control's pipe: {}", std::io::Error::last_os_error());
}

/// What:
///  A folder of this test process whose path one socket address holds,
///  and that the thread's own cleanup removes.
/// Why:
///  One socket address holds 107 bytes and the fixture root under the build directory is longer than that,
///  so a control of a socket keeps its own folder somewhere short
///  (`foundation/unix_socket_connect_within_a_deadline_tests.rs` does the same for the connect it controls).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = shortFolder('full-queue');
/// ```
fn short_folder(name: &str) -> PathBuf {
    let root: PathBuf = std::env::var_os("HOME").map(PathBuf::from).expect("a home directory").join("temp/agent").join(format!("warm{}", std::process::id()));
    let folder: PathBuf = root.join(name);
    std::fs::create_dir_all(folder.as_path()).expect("the short folder");
    crate::test_support::track_for_this_thread(root.as_path());
    return folder;
}

/// What:
///  A listening socket of this test process at `path`,
///  with the queue depth it is given,
///  which nothing in the control accepts from.
/// Why:
///  `std::os::unix::net::UnixListener::bind` asks the kernel for the largest queue it allows,
///  so a control that fills a queue makes its own listener.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listener = aListenerThatNeverAccepts(path, 0);
/// ```
fn a_listener_that_never_accepts(path: &Path, backlog: libc::c_int) -> OwnedFd {
    use std::os::unix::ffi::OsStrExt as _;
    let bytes: &[u8] = path.as_os_str().as_bytes();
    // SAFETY: `sockaddr_un` is plain data, and all zero bytes of it are a valid value.
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    assert!(bytes.len() < address.sun_path.len(), "the control's socket path fits one socket address");
    address.sun_family = libc::AF_UNIX as libc::sa_family_t;
    for (slot, byte) in address.sun_path.iter_mut().zip(bytes) {
        *slot = *byte as libc::c_char;
    }
    let length: libc::socklen_t = (size_of::<libc::sa_family_t>() + bytes.len() + 1) as libc::socklen_t;
    // SAFETY: `socket` creates a descriptor of the family and type it is given and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) };
    assert!(made >= 0, "the control's socket: {}", std::io::Error::last_os_error());
    // SAFETY: `made` is an open descriptor of this process and nothing else owns it.
    let listener: OwnedFd = unsafe { OwnedFd::from_raw_fd(made) };
    // SAFETY: `bind` reads the address it is given and the length says how much of it; the descriptor is this process's.
    let bound: libc::c_int = unsafe { libc::bind(listener.as_raw_fd(), std::ptr::from_ref(&address).cast::<libc::sockaddr>(), length) };
    assert_eq!(bound, 0, "the control's socket is bound: {}", std::io::Error::last_os_error());
    // SAFETY: `listen` reads the descriptor and the number it is given and touches no memory.
    let listening: libc::c_int = unsafe { libc::listen(listener.as_raw_fd(), backlog) };
    assert_eq!(listening, 0, "the control's socket listens: {}", std::io::Error::last_os_error());
    return listener;
}

/// A service is the one of an instance only by `service run` and `--instance <that instance>`.
#[test]
fn a_service_is_told_by_its_start_arguments_and_its_instance() {
    let own: Vec<Vec<u8>> = words(&["/bin/monochromatic-git-policy-cli", "monochromatic-git-policy-cli", "service", "run", "--journal", "/j", "--instance", "abc"]);
    let borrowed: Vec<&[u8]> = own.iter().map(Vec::as_slice).collect();
    assert!(is_service_of(borrowed.as_slice(), "abc"));
    assert!(!is_service_of(borrowed.as_slice(), "abd"));
    let status: Vec<Vec<u8>> = words(&["monochromatic-git-policy-cli", "service", "status", "--instance", "abc"]);
    let status_words: Vec<&[u8]> = status.iter().map(Vec::as_slice).collect();
    assert!(!is_service_of(status_words.as_slice(), "abc"), "a status command is not the service");
    let other: Vec<Vec<u8>> = words(&["git", "commit", "--instance", "abc"]);
    let other_words: Vec<&[u8]> = other.iter().map(Vec::as_slice).collect();
    assert!(!is_service_of(other_words.as_slice(), "abc"));
}

/// A process table with no service of the instance says no,
/// and one that cannot be read says no.
#[test]
fn an_empty_or_missing_process_table_says_no() {
    let root: PathBuf = crate::test_support::fixture("warm-proc-table");
    assert!(!service_process_runs("abc", root.as_path()));
    assert!(!service_process_runs("abc", root.join("nothing").as_path()));
    let own: PathBuf = root.join("4242");
    std::fs::create_dir_all(&own).expect("process directory");
    std::fs::write(own.join("cmdline"), b"x\0service\0run\0--instance\0abc\0").expect("cmdline");
    assert!(service_process_runs("abc", root.as_path()));
    crate::test_support::remove(root.as_path());
}

/// What:
///  A look whose read never answers gives the request up at `WARM_SCAN_CEILING`,
///  and says that no service runs,
///  so the command compiles the rules itself.
/// Why:
///  This is the case finding 1 of the review of 2026-11-13 reproduced with a process stuck in the kernel:
///  a commit with cold rules waited in the look for as long as that process stayed stuck.
///  The look is made for a request the command does without,
///  so its own ceiling ends it (`wait_ceilings.rs`,
///  `WARM_SCAN_CEILING`).
#[test]
fn a_look_whose_read_never_answers_gives_the_request_up_at_its_ceiling() {
    let root: PathBuf = crate::test_support::fixture("warm-scan-held");
    let process: PathBuf = root.join("4242");
    std::fs::create_dir_all(process.as_path()).expect("a process folder");
    a_command_line_nothing_writes(process.join("cmdline").as_path());
    let (told, heard) = std::sync::mpsc::channel();
    let looked_at: PathBuf = root.clone();
    std::thread::spawn(move || {
        let _sent = told.send(service_process_runs("abc", looked_at.as_path()));
    });
    let began: Instant = Instant::now();
    let answered: Result<bool, std::sync::mpsc::RecvTimeoutError> = heard.recv_timeout(WARM_SCAN_CEILING * 4);
    let waited: Duration = began.elapsed();
    assert_eq!(answered, Ok(false), "the look gives up at its ceiling and says no service runs, and it took {waited:?}");
    assert!(waited >= WARM_SCAN_CEILING, "the ceiling is what ended the look, and it took {waited:?}");
    assert!(waited < WARM_SCAN_CEILING * 4, "the wait is the ceiling and not a wait of its own: {waited:?}");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A socket whose listener is behind answers the request's own bound:
///  nothing is sent, and the wait is the ceiling instead of the kernel's.
/// Why:
///  This is the case a plain connect waits in for ever:
///  a listener whose program is stopped accepts nothing and refuses nothing,
///  so the commit that asks a stopped service to compile its rules waited with no end
///  (`wait_ceilings.rs`,
///  `WARM_REQUEST_CONNECT_CEILING`).
#[test]
fn a_request_into_a_queue_nothing_accepts_ends_at_its_bound() {
    let socket: PathBuf = short_folder("full-queue").join("full-queue.sock");
    let _listener: OwnedFd = a_listener_that_never_accepts(socket.as_path(), 0);
    // A queue of no depth takes one connection, and every one after it waits for an accept nothing does.
    let _filling: std::os::unix::net::UnixStream = std::os::unix::net::UnixStream::connect(socket.as_path()).expect("the queue takes one connection");
    let began: Instant = Instant::now();
    let sent: bool = ask_the_service_at(socket.as_path(), Path::new("/a/repository/.git"), Path::new("/a/repository"));
    let waited: Duration = began.elapsed();
    assert!(!sent, "nothing was sent into a queue nothing accepts");
    assert!(waited >= WARM_REQUEST_CONNECT_CEILING, "the ceiling is what ended the wait, and it took {waited:?}");
    assert!(waited < WARM_REQUEST_CONNECT_CEILING * 10, "the wait is the ceiling and not a wait of its own: {waited:?}");
}

/// A socket that accepts receives the client's own warm line,
/// framing newline included.
#[test]
fn a_request_into_a_socket_that_accepts_is_the_clients_warm_line() {
    let socket: PathBuf = short_folder("accepting").join("accepting.sock");
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("a listener");
    let (told, heard) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let Ok((accepted, _address)) = listener.accept() else { return };
        let mut line: String = String::new();
        let _read = BufReader::new(accepted).read_line(&mut line);
        let _sent = told.send(line);
    });
    assert!(ask_the_service_at(socket.as_path(), Path::new("/a/repository/.git"), Path::new("/a/repository")), "the request is sent");
    let line: String = heard.recv_timeout(WARM_REQUEST_CONNECT_CEILING * 10).expect("the service takes the line");
    let mut expected: String = encode_client(&ClientMessage::Warm { git_dir: PathBuf::from("/a/repository/.git"), repository: PathBuf::from("/a/repository") });
    expected.push('\n');
    assert_eq!(line, expected);
}
