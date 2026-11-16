//! The login a client makes on a D-Bus socket,
//! and the kernel's word on which process is on the other end.
//! A private socket of a service manager and a session bus's socket take the same login,
//! so it lives in one place:
//! the credentials byte,
//! `AUTH EXTERNAL` with the caller's user id,
//! the answer,
//! and `BEGIN`.
//! The peer check reads `SO_PEERCRED`,
//! which the kernel fills for the connected socket and no process can forge.

/// Reading the answer and writing the login.
use std::io::{self, Read, Write};
/// The socket the credentials are read from.
use std::os::fd::AsRawFd;
/// Unix stream sockets.
use std::os::unix::net::UnixStream;

/// Why a login did not finish.
#[derive(Debug)]
pub enum LoginError {
    /// The socket could not be written or read,
    /// or the peer refused the login.
    Failed(
        /// What happened.
        String,
    ),
    /// The peer did not answer in time.
    TimedOut(
        /// What was waited for.
        String,
    ),
}

/// Wording for reports.
impl std::fmt::Display for LoginError {
    /// Writes the plain text of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            LoginError::Failed(text) | LoginError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}

/// Most bytes accepted for the login's answer.
const MAX_AUTH_BYTES: usize = 512;
/// The longest line of the answer that a report keeps.
const MAX_AUTH_LINE: usize = 256;

/// The pid on the other end of a connected socket,
/// as the kernel reports it.
pub fn peer_pid(stream: &UnixStream) -> io::Result<i32> {
    let mut credentials = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: the descriptor is open for as long as `stream` lives, and the buffer is a `ucred` of the size passed.
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            std::ptr::from_mut(&mut credentials).cast(),
            std::ptr::from_mut(&mut length),
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(credentials.pid);
}

/// The login token of an EXTERNAL authentication:
/// the decimal spelling of the caller's user id,
/// written as the hexadecimal bytes that spell it.
pub fn external_token(uid: u32) -> String {
    let mut token = String::with_capacity(uid.to_string().len() * 2);
    for byte in uid.to_string().bytes() {
        token.push_str(&format!("{byte:02x}"));
    }
    return token;
}

/// Reads one line of the login's answer.
fn read_line(stream: &mut UnixStream) -> Result<String, LoginError> {
    let mut answer: Vec<u8> = Vec::new();
    let mut byte = [0u8; 1];
    while !answer.ends_with(b"\r\n") && answer.len() < MAX_AUTH_BYTES {
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_read) => answer.push(byte[0]),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock || error.kind() == io::ErrorKind::TimedOut => {
                return Err(LoginError::TimedOut("the socket did not answer the login in time".to_string()));
            }
            Err(error) => return Err(LoginError::Failed(format!("the socket could not be read: {error}"))),
        }
    }
    let text = String::from_utf8_lossy(&answer).into_owned();
    return Ok(text.lines().next().unwrap_or("").chars().take(MAX_AUTH_LINE).collect::<String>());
}

/// Logs in as the running process's user,
/// and finishes the login once the peer agrees.
pub fn login(stream: &mut UnixStream) -> Result<(), LoginError> {
    // SAFETY: geteuid has no preconditions.
    let uid = unsafe { libc::geteuid() };
    let request = format!("\0AUTH EXTERNAL {}\r\n", external_token(uid));
    if let Err(error) = stream.write_all(request.as_bytes()) {
        return Err(LoginError::Failed(format!("the login could not be written: {error}")));
    }
    if let Err(error) = stream.flush() {
        return Err(LoginError::Failed(format!("the login could not be written: {error}")));
    }
    let line = read_line(stream)?;
    if !line.starts_with("OK ") {
        return Err(LoginError::Failed(format!("the peer refused the login of uid {uid} ({line:?})")));
    }
    if let Err(error) = stream.write_all(b"BEGIN\r\n") {
        return Err(LoginError::Failed(format!("the login could not be finished: {error}")));
    }
    if let Err(error) = stream.flush() {
        return Err(LoginError::Failed(format!("the login could not be finished: {error}")));
    }
    return Ok(());
}
