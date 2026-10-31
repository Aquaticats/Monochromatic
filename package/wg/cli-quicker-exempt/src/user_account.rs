//! Looks up the account of the watched user in the system user database.
//! The home directory comes from the database,
//! never from a hardcoded path or the root watcher's own environment.

/// Syscall failures.
use std::io;
/// Raw Unix path bytes for the home directory.
use std::os::unix::ffi::OsStringExt;
/// Home directory path.
use std::path::PathBuf;

/// The facts about one account that the update traffic policy needs.
#[derive(Debug, Eq, PartialEq)]
pub struct UserAccount {
    /// Primary group id.
    pub gid: u32,
    /// Home directory.
    pub home: PathBuf,
}

/// Initial size of the buffer `getpwuid_r` fills.
const INITIAL_BUFFER_BYTES: usize = 1024;
/// Largest buffer tried before giving up.
const MAX_BUFFER_BYTES: usize = 1 << 20;

/// Looks up `uid` with `getpwuid_r`,
/// growing the buffer when the entry does not fit.
pub fn lookup_user(uid: u32) -> io::Result<UserAccount> {
    let mut size = INITIAL_BUFFER_BYTES;
    loop {
        let mut buffer: Vec<libc::c_char> = vec![0; size];
        // SAFETY: a zeroed passwd is valid output storage that getpwuid_r overwrites.
        let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
        let mut found: *mut libc::passwd = std::ptr::null_mut();
        // SAFETY: all pointers are valid for the lengths passed,
        // and the result pointer is read only after the call returns.
        let status = unsafe { libc::getpwuid_r(uid, &mut entry, buffer.as_mut_ptr(), buffer.len(), &mut found) };
        if status == libc::ERANGE && size < MAX_BUFFER_BYTES {
            size *= 2;
            continue;
        }
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status));
        }
        if found.is_null() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("uid {uid} has no account")));
        }
        // SAFETY: a successful lookup sets pw_dir to a NUL-terminated string inside the buffer.
        let directory = unsafe { std::ffi::CStr::from_ptr(entry.pw_dir) };
        let home = PathBuf::from(std::ffi::OsString::from_vec(directory.to_bytes().to_vec()));
        return Ok(UserAccount { gid: entry.pw_gid, home });
    }
}
