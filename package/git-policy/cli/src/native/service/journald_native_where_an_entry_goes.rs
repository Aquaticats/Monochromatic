//! What:
//!  Where an entry may go:
//!  the check on `$JOURNAL_STREAM`,
//!  the device and inode of standard error,
//!  and the C address a datagram is sent to.
//! Why:
//!  A redirection leaves `$JOURNAL_STREAM` set,
//!  so the variable alone never says standard error is the journal;
//!  and a signal handler may not build the address itself,
//!  because the standard library allocates,
//!  so it is made once,
//!  ahead of the signal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! rawAddress(address) // { family: AF_UNIX, path: '\0name' }
//! ```

use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::SocketAddr;
/// Whether `$JOURNAL_STREAM` names the file with this device and inode,
/// which is how `systemd.exec(5)` says to tell that
/// standard error is still the journal:
/// the variable alone is not enough,
/// because a redirection leaves it set.
pub fn stream_matches(
    variable: Option<&str>,
    device: u64,
    inode: u64,
) -> bool {
    let Some(text) = variable else {
        return false;
    };
    let Some((named_device, named_inode)) = text.split_once(':') else {
        return false;
    };
    return named_device.parse::<u64>().ok() == Some(device)
        && named_inode.parse::<u64>().ok() == Some(inode);
}

/// The device and inode of standard error,
/// or nothing when it cannot be read.
pub fn stderr_identity() -> Option<(u64, u64)> {
    let mut status: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: `status` is valid for the call to write one `stat`.
    let result: libc::c_int = unsafe { libc::fstat(libc::STDERR_FILENO, &raw mut status) };
    if result != 0 {
        return None;
    }
    return Some((u64::from(status.st_dev), u64::from(status.st_ino)));
}

/// What:
///  The address as the C structure `sendto` takes,
/// and its length,
/// or nothing when the name does not fit.
/// Why:
///   A signal handler may not build it (the standard library allocates),
/// so it is made once,
/// ahead of the signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rawAddress(address) // { family: AF_UNIX, path: '\0name' }
/// ```
pub fn raw_address(address: &SocketAddr) -> Option<(libc::sockaddr_un, libc::socklen_t)> {
    use std::os::unix::ffi::OsStrExt;
    // SAFETY: `sockaddr_un` is a plain C structure for which all zero bytes is a valid value.
    let mut raw: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    raw.sun_family = libc::sa_family_t::try_from(libc::AF_UNIX).ok()?;
    let (offset, name): (usize, &[u8]) = match (address.as_pathname(), address.as_abstract_name()) {
        (Some(path), _) => (0, path.as_os_str().as_bytes()),
        (None, Some(name)) => (1, name),
        (None, None) => return None,
    };
    // A path needs room for the zero byte that ends it; an abstract name starts with one, so it may fill the whole field.
    if offset + name.len() + usize::from(offset == 0) > raw.sun_path.len() {
        return None;
    }
    for (slot, byte) in raw.sun_path[offset..].iter_mut().zip(name) {
        *slot = libc::c_char::from_ne_bytes([*byte]);
    }
    let length: usize =
        std::mem::size_of::<libc::sa_family_t>() + offset + name.len() + usize::from(offset == 0);
    return Some((raw, libc::socklen_t::try_from(length).ok()?));
}
