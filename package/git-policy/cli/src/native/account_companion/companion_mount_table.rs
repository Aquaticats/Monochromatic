//! What:
//!  The mount facts the companion reads back:
//!  one descriptor's mount ID from the kernel's own `fdinfo`,
//!  `/proc/self/mountinfo` read into the buffer the parent built before the
//!  fork,
//!  and the two comparisons that prove a mount is where it says.
//! Why:
//!  The design asks for the mount facts read back rather than assumed,
//!  so the attach is proved by the kernel's table and the unmount is proved by
//!  its absence
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "What it runs").
//!  Nothing here allocates and nothing here formats text:
//!  the child's numbers are written into a stack array by hand,
//!  and a table longer than the buffer is a refusal rather than a partial read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const id = mountIdOf(fd); const table = readMountTable(MOUNT_TABLE_LIMIT);
//! expect(namesMount(table, id, slotPath)).toBe(true);
//! ```

use super::companion_calls::{CompanionCall, Failure, failed};
use std::ffi::CStr;
use std::os::fd::RawFd;

/// The largest `fdinfo` table one mount descriptor's IDs are read from.
const FDINFO_BYTES: usize = 512;

/// What:
///  The mount ID one descriptor is on,
///  read from the kernel's own `fdinfo`.
/// Why:
///  The mount's ID is what the read-back compares,
///  and the descriptor's own table is the only place before the move that
///  names it.
///
/// # Errors
/// The table could not be read or carries no mount ID.
pub fn mount_id_of(descriptor: RawFd) -> Result<u64, Failure> {
    let mut path: [u8; 32] = [0; 32];
    let prefix: &[u8] = b"/proc/self/fdinfo/";
    path[..prefix.len()].copy_from_slice(prefix);
    let written: usize = write_number(u64::try_from(descriptor).unwrap_or(0), path[prefix.len()..].as_mut());
    path[prefix.len() + written] = 0;
    let name: &CStr = CStr::from_bytes_until_nul(path.as_slice()).map_err(|_none| return Failure { call: CompanionCall::ReadFdinfo, errno: libc::EINVAL })?;
    let mut table: [u8; FDINFO_BYTES] = [0; FDINFO_BYTES];
    let read: usize = read_at_most(name, table.as_mut_slice())?;
    return found_mount_id(table[..read].as_ref());
}

/// What:
///  Read `/proc/self/mountinfo` into the buffer the parent built,
///  and answer how many bytes it holds.
/// Why:
///  The read-back needs the kernel's own facts,
///  and a table that fills the buffer is a refusal rather than a read of part
///  of it.
///
/// # Errors
/// The table could not be read or is longer than the buffer.
pub fn read_mount_table(path: &CStr, buffer: &mut [u8]) -> Result<usize, Failure> {
    let read: usize = read_at_most(path, buffer)?;
    if read == buffer.len() {
        return Err(Failure { call: CompanionCall::ReadMountinfo, errno: libc::E2BIG });
    }
    return Ok(read);
}

/// What:
///  Whether the mount table names one mount ID at one mount point,
///  as the read-back after the move must find it.
pub fn names_mount(table: &[u8], mount_id: u64, mount_point: &[u8]) -> bool {
    for line in table.split(|byte| return *byte == b'\n') {
        if !starts_with_number(line, mount_id) {
            continue;
        }
        if field_of(line, 4) == Some(mount_point) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether the mount table still names one mount ID anywhere,
///  which after the unmount it must not.
pub fn names_mount_id(table: &[u8], mount_id: u64) -> bool {
    return table.split(|byte| return *byte == b'\n').any(|line| return starts_with_number(line, mount_id));
}

/// Whether one mount-table line begins with this number and the space after it.
fn starts_with_number(line: &[u8], number: u64) -> bool {
    let mut digits: [u8; 24] = [0; 24];
    let written: usize = write_number(number, digits.as_mut_slice());
    return line.starts_with(&digits[..written]) && line.get(written) == Some(&b' ');
}

/// One space-separated field of a mount-table line,
/// counting from zero.
fn field_of(line: &[u8], index: usize) -> Option<&[u8]> {
    return line.split(|byte| return *byte == b' ').nth(index);
}

/// What:
///  The mount ID one `fdinfo` table names,
///  or the refusal that says its line is missing.
fn found_mount_id(table: &[u8]) -> Result<u64, Failure> {
    for line in table.split(|byte| return *byte == b'\n') {
        let Some(rest) = line.strip_prefix(b"mnt_id:".as_ref()) else { continue };
        // The kernel separates the key from its value with a tab, and a build that writes a space is read the same way.
        let digits: &[u8] = rest.strip_prefix(b"\t".as_ref()).or_else(|| return rest.strip_prefix(b" ".as_ref())).unwrap_or(rest);
        let number: u64 = parse_number(digits).ok_or(Failure { call: CompanionCall::ReadFdinfo, errno: libc::EINVAL })?;
        return Ok(number);
    }
    return Err(Failure { call: CompanionCall::ReadFdinfo, errno: libc::ENOENT });
}

/// What:
///  One decimal number as the digits a stack array holds.
/// Why:
///  A descriptor's number must reach a path without a formatter.
pub(super) fn write_number(number: u64, into: &mut [u8]) -> usize {
    let mut digits: [u8; 24] = [0; 24];
    let mut count: usize = 0;
    let mut left: u64 = number;
    loop {
        digits[count] = b'0' + u8::try_from(left % 10).unwrap_or(0);
        count += 1;
        left /= 10;
        if left == 0 {
            break;
        }
    }
    let mut written: usize = 0;
    while written < count && written < into.len() {
        into[written] = digits[count - 1 - written];
        written += 1;
    }
    return written;
}

/// What:
///  One decimal number from the bytes of a table,
///  or nothing when they hold a byte that is no digit.
pub(super) fn parse_number(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() {
        return None;
    }
    let mut value: u64 = 0;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte - b'0'))?;
    }
    return Some(value);
}

/// What:
///  Read one whole file by path into a buffer,
///  answering how many bytes it gave.
/// Why:
///  The child reads a `/proc` table and needs no allocating reader.
fn read_at_most(path: &CStr, buffer: &mut [u8]) -> Result<usize, Failure> {
    // SAFETY: the path outlives the call and the descriptor it answers is closed below, exactly once.
    let opened: libc::c_int = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
    if opened < 0 {
        return Err(failed(CompanionCall::ReadMountinfo));
    }
    let mut read: usize = 0;
    // A `seq_file` gives one page per read, so a table longer than a page is read until it ends or the buffer fills.
    while read < buffer.len() {
        // SAFETY: the buffer is the caller's and holds the room the call is given, from the offset it is read at.
        let taken: libc::ssize_t = unsafe { libc::read(opened, buffer.as_mut_ptr().add(read).cast::<libc::c_void>(), buffer.len() - read) };
        if taken < 0 {
            // SAFETY: the descriptor is this call's own and is closed exactly once.
            let _closed: libc::c_int = unsafe { libc::close(opened) };
            return Err(failed(CompanionCall::ReadMountinfo));
        }
        if taken == 0 {
            break;
        }
        read += usize::try_from(taken).unwrap_or(buffer.len() - read);
    }
    // SAFETY: the descriptor is this call's own and is closed exactly once.
    let _closed: libc::c_int = unsafe { libc::close(opened) };
    return Ok(read);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "companion_mount_table_tests.rs"]
mod tests;
