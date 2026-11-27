//! What:
//!  The named pipes (FIFOs) of a copied tree:
//!  one opened without blocking so the attribute calls can use its descriptor,
//!  and one created with the permission bits of the original.
//! Why:
//!  A pipe is part of the tree and has an owner,
//!  times and attributes like any other entry,
//!  and opening it read-only with `O_NONBLOCK` succeeds whether or not a writer exists,
//!  while `O_NOFOLLOW` refuses a link put in its place.
//!  Sockets and device nodes are not recreated (the report lists them).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const handle = await open(path, constants.O_RDONLY | constants.O_NONBLOCK | constants.O_NOFOLLOW);
//! await mkfifo(path, mode);
//! ```

use crate::fs_primitive_error::c_path;
use std::fs::File;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// What:
///  Open a named pipe without blocking,
/// so attribute calls can use its descriptor.
/// Why:
///   Opening a pipe read-only with `O_NONBLOCK` succeeds at once whether or not a writer exists,
/// and `O_NOFOLLOW`
///  refuses a link put in its place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await open(path, constants.O_RDONLY | constants.O_NONBLOCK | constants.O_NOFOLLOW);
/// ```
pub fn open_fifo(path: &Path) -> std::io::Result<File> {
    return std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path);
}

/// What:
///  Create a named pipe (FIFO) with the permission bits of the original.
///  `mkfifo(3)` is the call;
/// the umask applies,
/// and `preserve_on_descriptor` sets the exact bits afterwards.
/// Why:
///   A FIFO is part of the tree;
/// sockets and device nodes are not recreated (the report lists them).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkfifo(path, mode);
/// ```
pub fn make_fifo(
    path: &Path,
    mode: u32,
) -> std::io::Result<()> {
    let name = match c_path("mkfifo", path) {
        Ok(text) => text,
        Err(error) => return Err(std::io::Error::from_raw_os_error(error.errno)),
    };
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let result: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), mode & 0o7777) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}
