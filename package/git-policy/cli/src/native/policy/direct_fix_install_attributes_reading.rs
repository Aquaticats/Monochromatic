//! What:
//!  The calls that read what a file carries outside its bytes:
//!  the list of names,
//!  one value,
//!  and the
//!       retry that reads them until the length the kernel reports fits the buffer it was given.
//! Why:
//!  A reading answers with a length and then with bytes,
//!  and a value or a list that changes between those two
//!       calls would otherwise read as empty;
//!  the calls live apart from the rules about what a rewrite carries so
//!       each file says one thing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const names = (path) => llistxattr(path).split('\0').filter(Boolean); const value = (path, name) => lgetxattr(path, name);
//! ```

/// `CStr` is a NUL-terminated name for the system calls;
///  `CString` is one this code builds.
use std::ffi::{CStr, CString};
/// `OsStrExt` lends a path's bytes.
use std::os::unix::ffi::OsStrExt;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// How many times a list or a value is asked for before a changing file is refused.
use super::ATTRIBUTE_ATTEMPTS;

/// What:
///  A pathname as the system calls want it.
/// Why:
///   They take NUL-terminated bytes,
///  and a pathname holding a NUL byte cannot name a file at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (path) => Buffer.concat([path, Buffer.from([0])]);
/// ```
#[cfg(unix)]
fn name_of(path: &Path) -> Result<CString, std::io::Error> {
    let text: Result<CString, std::ffi::NulError> = CString::new(path.as_os_str().as_bytes());
    return match text {
        Ok(built) => Ok(built),
        Err(_invalid) => Err(std::io::Error::from(std::io::ErrorKind::InvalidInput)),
    };
}

/// Whether `byte` is the zero that separates two attribute names in a list.
/// Why:
///   The list is one buffer of names,
///  each one NUL-terminated,
///  so the separator is what splits them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isZero = (byte) => byte === 0;
/// ```
#[cfg(unix)]
fn is_zero(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether a listed name is one,
///  rather than the empty rest after a final separator.
/// Why:
///   The list ends with a zero,
///  which would otherwise read as an attribute with an empty name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = (name) => name.length > 0;
/// ```
#[cfg(unix)]
fn named(name: &&[u8]) -> bool {
    return !name.is_empty();
}

/// What:
///  A system call's answer as a length,
///  or the system's own error.
/// Why:
///  Every call here answers a byte count and a negative number as a failure,
///  so the length question and
///       the reading are read the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answerOf = (count) => count < 0 ? { error: lastError() } : { bytes: count };
/// ```
#[cfg(unix)]
fn answer_of(count: libc::ssize_t) -> std::io::Result<usize> {
    if count < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(usize::try_from(count).unwrap_or(usize::MAX));
}

/// What:
///  One two-call reading:
/// how many bytes the answer needs,
///  then the answer itself.
///  `length` asks,
///  `read` copies at most `buffer.len()` bytes and reports how many the answer needs.
/// Why:
///   The values,
/// the names and the descriptor's list are all read this way,
///  so the retry is written once;
///  the state travels as one argument because the calls are plain functions,
///  which keeps them out of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readUntilItFits(state, length, read) { for (let attempt = 0; attempt < ATTEMPTS; attempt++) { const buffer = Buffer.alloc(length(state)); const reported = read(state, buffer); if (reported > buffer.length) continue; return buffer.subarray(0, reported) } throw new Error('kept changing') }
/// ```
#[cfg(unix)]
pub(super) fn read_until_it_fits<State>(
    state: &mut State,
    changing: &str,
    length: fn(&mut State) -> std::io::Result<usize>,
    read: fn(&mut State, &mut [u8]) -> std::io::Result<usize>,
) -> std::io::Result<Vec<u8>> {
    for _attempt in 0..ATTRIBUTE_ATTEMPTS {
        let wanted: usize = length(state)?;
        let mut buffer: Vec<u8> = vec![0; wanted];
        let reported: usize = match read(state, buffer.as_mut_slice()) {
            Ok(count) => count,
            Err(error) if error.raw_os_error() == Some(libc::ERANGE) => continue,
            Err(error) => return Err(error),
        };
        // What: The kernel reads a zero-sized buffer as another length question, so an answer above the buffer means
        //       nothing was copied and the value or list has grown since the question was asked.
        // Why:  Truncating the empty buffer to that length cannot grow it, and the person's nonempty value would then be
        //       read as empty, which a displaced file compares equal on.
        //
        // In TS you'd write (pseudocode):
        // if (reported > buffer.length) continue;
        if reported > buffer.len() {
            continue;
        }
        buffer.truncate(reported);
        return Ok(buffer);
    }
    return Err(std::io::Error::other(changing));
}

/// What:
///  How many bytes the list of the file at `path` needs.
/// Why:
///  This is the length question of the two-call reading of a pathname's list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lengthOfList = (path) => llistxattr(path, null, 0);
/// ```
#[cfg(unix)]
fn length_of_list_on_path(path: &mut CString) -> std::io::Result<usize> {
    // SAFETY: a null buffer with size 0 asks the system for the length of the list.
    let length: libc::ssize_t = unsafe { libc::llistxattr(path.as_ptr(), std::ptr::null_mut(), 0) };
    return answer_of(length);
}

/// What:
///  Fill `buffer` with the list of the file at `path`,
///  and report how many bytes the list needs.
/// Why:
///  This is the reading itself;
///  a buffer too small for a list that grew answers `ERANGE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fillList = (path, buffer) => llistxattr(path, buffer, buffer.length);
/// ```
#[cfg(unix)]
fn fill_list_on_path(path: &mut CString, buffer: &mut [u8]) -> std::io::Result<usize> {
    // SAFETY: `buffer` holds `buffer.len()` writable bytes, and the pathname outlives the call.
    let filled: libc::ssize_t =
        unsafe { libc::llistxattr(path.as_ptr(), buffer.as_mut_ptr().cast(), buffer.len()) };
    return answer_of(filled);
}

/// What:
///  How many bytes the list of the open file at `descriptor` needs.
/// Why:
///  This is the length question of the two-call reading of the replacement's own list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lengthOfList = (fd) => flistxattr(fd, null, 0);
/// ```
#[cfg(unix)]
fn length_of_list_on_descriptor(descriptor: &mut libc::c_int) -> std::io::Result<usize> {
    // SAFETY: a null buffer with size 0 asks the system for the length of the list.
    let length: libc::ssize_t = unsafe { libc::flistxattr(*descriptor, std::ptr::null_mut(), 0) };
    return answer_of(length);
}

/// What:
///  Fill `buffer` with the list of the open file at `descriptor`,
///  and report how many bytes the list needs.
/// Why:
///  The descriptor cannot be redirected by a swap of the name,
///  so this is the replacement's own list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fillList = (fd, buffer) => flistxattr(fd, buffer, buffer.length);
/// ```
#[cfg(unix)]
fn fill_list_on_descriptor(descriptor: &mut libc::c_int, buffer: &mut [u8]) -> std::io::Result<usize> {
    // SAFETY: `buffer` holds `buffer.len()` writable bytes, and the descriptor is this file's.
    let filled: libc::ssize_t =
        unsafe { libc::flistxattr(*descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
    return answer_of(filled);
}

/// What:
///  The pathname and the name of one extended attribute,
///  as the two calls that read its bytes want them.
/// Why:
///  One reading takes its state as one value,
///  so the pathname and the attribute's name travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ValueOnDisk = { path: Buffer; name: Buffer };
/// ```
#[cfg(unix)]
struct ValueOnDisk {
    /// The pathname,
    ///  NUL-terminated.
    path: CString,
    /// The attribute's name,
    ///  NUL-terminated.
    name: CString,
}

/// What:
///  How many bytes the value of `asked`'s attribute needs.
/// Why:
///  This is the length question of the two-call reading of one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lengthOfValue = ({ path, name }) => lgetxattr(path, name, null, 0);
/// ```
#[cfg(unix)]
fn length_of_value(asked: &mut ValueOnDisk) -> std::io::Result<usize> {
    // SAFETY: a null buffer with size 0 asks the system for the length of the value.
    let length: libc::ssize_t = unsafe {
        libc::lgetxattr(asked.path.as_ptr(), asked.name.as_ptr(), std::ptr::null_mut(), 0)
    };
    return answer_of(length);
}

/// What:
///  Fill `buffer` with the value of `asked`'s attribute,
///  and report how many bytes it needs.
/// Why:
///  A buffer too small for a value that grew answers `ERANGE`,
///  and a zero-sized buffer makes
///       the call a length question again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fillValue = ({ path, name }, buffer) => lgetxattr(path, name, buffer, buffer.length);
/// ```
#[cfg(unix)]
fn fill_value(asked: &mut ValueOnDisk, buffer: &mut [u8]) -> std::io::Result<usize> {
    // SAFETY: `buffer` holds `buffer.len()` writable bytes, and both names outlive the call.
    let read: libc::ssize_t = unsafe {
        libc::lgetxattr(
            asked.path.as_ptr(),
            asked.name.as_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    return answer_of(read);
}

/// What:
///  The names of every extended attribute of the regular file at `path`,
///  in the order the kernel
///       lists them.
/// Why:
///   The list is what the replacement must carry;
///  a namespace the kernel does not report here is one this
///       process cannot see,
///  and `SPEC.md` records that limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (path) => llistxattr(path).split('\0').filter(Boolean);
/// ```
#[cfg(unix)]
pub(super) fn names_of_attributes(path: &Path) -> std::io::Result<Vec<Vec<u8>>> {
    let mut text: CString = name_of(path)?;
    let listed: Vec<u8> = read_until_it_fits(
        &mut text,
        "the list of its extended attributes kept changing while it was read",
        length_of_list_on_path,
        fill_list_on_path,
    )?;
    return Ok(listed
        .split(is_zero)
        .filter(named)
        .map(<[u8]>::to_vec)
        .collect());
}

/// What:
///  The names of every extended attribute of the open file at `descriptor`,
///  in the order the kernel
///       lists them.
/// Why:
///  The descriptor cannot be redirected by a swap of the name,
///  so the list is the replacement's own,
///  and the names are what a removal addresses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (fd) => flistxattr(fd).split('\0').filter(Boolean);
/// ```
#[cfg(unix)]
pub(super) fn names_on_descriptor(descriptor: libc::c_int) -> std::io::Result<Vec<Vec<u8>>> {
    let mut asked: libc::c_int = descriptor;
    let listed: Vec<u8> = read_until_it_fits(
        &mut asked,
        "the list of its extended attributes kept changing while it was read",
        length_of_list_on_descriptor,
        fill_list_on_descriptor,
    )?;
    return Ok(listed
        .split(is_zero)
        .filter(named)
        .map(<[u8]>::to_vec)
        .collect());
}

/// What:
///  The bytes the extended attribute `attribute` holds on the file at `path`.
/// Why:
///   The value is the person's data,
///  and a name whose value cannot be read is a refusal rather than an
///       attribute left behind on the replacement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = (path, attribute) => lgetxattr(path, attribute);
/// ```
#[cfg(unix)]
pub(super) fn value_of_attribute(path: &Path, name: &CStr) -> std::io::Result<Vec<u8>> {
    let mut asked: ValueOnDisk = ValueOnDisk {
        path: name_of(path)?,
        name: CString::from(name),
    };
    return read_until_it_fits(
        &mut asked,
        "its value kept changing while it was read",
        length_of_value,
        fill_value,
    );
}
