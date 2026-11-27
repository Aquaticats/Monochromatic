//! What:
//!  The plain names and the descriptor paths every call in this folder is written with:
//! one relative path split into its names,
//! a name refused unless it is one plain component,
//! a NUL-terminated copy of either,
//! the check that turns a C result into an error,
//! and the `/proc/self/fd` path that names a held descriptor.
//! Why:
//!  Every call below a held directory names one component of it and nothing else,
//! and one place refusing `.`,
//! `..`,
//! a slash and a NUL byte is what keeps that true.
//! The path of a descriptor is how a tool that takes a path,
//! and the `l`-call form of an attribute operation,
//! reaches the object that was opened rather than whatever a name holds now.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const names = plainComponents('a/b'); // ['a', 'b'], refusing '..', '.' and '/'
//! const path = handlePath(directory);   // '/proc/self/fd/12'
//! ```

use std::ffi::{
    CString,
    OsStr,
    OsString,
};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Component,
    Path,
    PathBuf,
};
/// The components of a relative path,
/// each refused unless it is a plain name.
///
/// Why:
/// `..`,
/// `.`,
/// a root or a prefix would leave the held directory or name it twice.
pub fn plain_components(relative: &Path) -> std::io::Result<Vec<OsString>> {
    let mut names: Vec<OsString> = Vec::new();
    for part in relative.components() {
        match part {
            Component::Normal(name) => names.push(name.to_os_string()),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{} is not a plain relative path", relative.display()),
                ));
            }
        }
    }
    return Ok(names);
}

/// A NUL-terminated copy of `bytes`,
/// or an invalid-input error saying `what` holds a NUL byte.
pub(super) fn c_string_of(
    bytes: &[u8],
    what: &str,
) -> std::io::Result<CString> {
    return match CString::new(bytes) {
        Ok(text) => Ok(text),
        Err(_error) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{what} holds a NUL byte"),
        )),
    };
}

/// A NUL-terminated copy of one name,
/// refused when it is empty,
/// `.`,
/// `..`,
/// holds a slash or a NUL byte.
pub(super) fn name_of(name: &OsStr) -> std::io::Result<CString> {
    let bytes: &[u8] = name.as_bytes();
    if bytes.is_empty() || bytes == b"." || bytes == b".." || bytes.contains(&b'/') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "a name must be one plain component",
        ));
    }
    return c_string_of(bytes, "a name");
}

/// Turn a C result that is `-1` on failure into the thread's error.
pub(super) fn checked(result: libc::c_int) -> std::io::Result<libc::c_int> {
    return if result < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(result)
    };
}

/// The path through which a held directory handle names that directory,
/// for tools that take a path.
pub fn handle_path(directory: &File) -> PathBuf {
    return Path::new("/proc/self/fd").join(directory.as_raw_fd().to_string());
}
