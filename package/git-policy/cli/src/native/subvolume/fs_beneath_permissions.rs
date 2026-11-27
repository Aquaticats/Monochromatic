//! What:
//!  Giving the owner read,
//! write and search permission on a directory this program must empty,
//! through the descriptor that was judged rather than through the name it was opened at.
//! Why:
//!  A directory of mode 555 or 000 cannot be emptied until its owner can write and search it,
//! and the change has to land on the object that was judged:
//! a name the kernel resolves again may lead to another directory,
//! so a removal that widened through the name would change the mode of whatever stands there by then.
//! A handle opened without read access (`O_PATH`) cannot be changed with `fchmod`,
//! so the last open retries through the path of the descriptor that was judged,
//! and proves from the descriptor it reaches that it is still the same directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! widenThroughAHandle(handle);            // chmod /proc/self/fd/<n> 0700 for the owner
//! const dir = await openDirectoryWidening(parent, 'a'); // retries after giving the owner access
//! ```

use super::fs_beneath_names::handle_path;
use super::fs_beneath_open::{
    open_beneath,
    open_directory_at,
};
use std::ffi::OsStr;
use std::fs::File;

/// Give the owner read,
/// write and search permission on the open directory,
/// through its descriptor.
///
/// Why:
/// a directory of mode 555 or 000 cannot be emptied until its owner can write and search it;
/// the change lands on the
/// directory that is held,
/// whatever its path becomes.
/// A directory the user does not own keeps its mode and the later
/// removal reports the operating system's refusal.
pub fn widen_directory(directory: &File) -> std::io::Result<()> {
    use std::os::unix::fs::{
        MetadataExt,
        PermissionsExt,
    };
    let mode: u32 = directory.metadata()?.mode();
    if mode & 0o700 == 0o700 {
        return Ok(());
    }
    return directory.set_permissions(std::fs::Permissions::from_mode(mode | 0o700));
}

/// What:
///  Give the owner read,
///  write and search permission on the directory a handle names,
///  through that handle and never through the name it was opened at.
/// Why:
///   A handle opened without read access (`O_PATH`) cannot be changed with `fchmod`,
/// and the mode has to be given to the
///  object that was judged:
/// a name the kernel resolves again may lead to another directory,
///  and a removal that widened
/// through the name would change the mode of whatever stands there by then.
///  A directory this user does not own keeps the mode the kernel gives it,
/// and the caller reads the refusal where its
///  own opening fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// widenThroughAHandle(handle); // chmod /proc/self/fd/<n> 0700 for the owner
/// ```
pub fn widen_through_a_handle(handle: &File) -> std::io::Result<()> {
    use std::os::unix::fs::{
        MetadataExt,
        PermissionsExt,
    };
    let mode: u32 = handle.metadata()?.mode();
    if mode & 0o700 == 0o700 {
        return Ok(());
    }
    return std::fs::set_permissions(
        handle_path(handle),
        std::fs::Permissions::from_mode(mode | 0o700),
    );
}

/// Open the subdirectory `name` of `parent` as `open_directory_at` does,
/// giving the owner access to a directory whose mode
///  forbids opening it.
///
/// Why:
/// the retry names the very directory the first open found (a handle that only names it),
/// so a swapped name cannot
/// redirect the change of mode.
pub fn open_directory_widening(
    parent: &File,
    name: &OsStr,
) -> std::io::Result<File> {
    use std::os::unix::fs::MetadataExt;
    match open_directory_at(parent, name) {
        Err(error) if error.raw_os_error() == Some(libc::EACCES) => {
            let handle: File = open_beneath(parent, name, libc::O_PATH | libc::O_DIRECTORY, 0)?;
            let judged: std::fs::Metadata = handle.metadata()?;
            if widen_through_a_handle(&handle).is_err() {
                return Err(error);
            }
            let reached: File = open_directory_at(parent, name)?;
            // The name is resolved again here, so it is proved to lead to the directory whose mode was given: another
            // object at that name is not this command's to read.
            let now: std::fs::Metadata = reached.metadata()?;
            if now.dev() != judged.dev() || now.ino() != judged.ino() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "{} is not the directory that was opened here any more",
                        handle_path(&handle).display()
                    ),
                ));
            }
            return Ok(reached);
        }
        other => return other,
    }
}
