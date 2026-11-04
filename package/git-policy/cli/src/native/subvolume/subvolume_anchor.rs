//! What:
//!  An open directory used as the fixed point of every operation inside it: the directory is opened one component at
//!  a time without following symbolic links, and names inside it are then addressed through its descriptor.
//! Why:
//!  The tree passes write into a checkout that other programs may be changing (the catch-up and the carry-back run
//!  against a tree in use). A path such as `destination/d/file` is resolved again for every call, so a directory `d`
//!  replaced by a symbolic link between two calls would send the write, the removal or the permission change outside
//!  the tree. Opening each directory without following links and addressing its children through
//!  `/proc/self/fd/<descriptor>/<name>` makes every call land in the directory that was checked, whatever happens to
//!  its path afterwards. Only the last component of a child path is still looked up by name, and the operations on it
//!  (exclusive create, rename, unlink, `lstat`) never follow a link there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const anchor = await openBelow(root, 'a/b'); // throws when a or b is a symbolic link
//! await writeFile(anchor.child('file'), bytes);
//! ```

use std::ffi::{CString, OsStr};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::path::{Component, Path, PathBuf};

/// What:
///  A directory held open.
///  `File` owns the descriptor and closes it when the value is dropped.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Anchor = { directory: FileHandle };
/// ```
#[derive(Debug)]
pub struct Anchor {
    /// The directory's descriptor.
    directory: File,
}

/// The path text of a name given as bytes, or the error a name with an interior NUL deserves.
fn c_name(name: &OsStr) -> std::io::Result<CString> {
    return CString::new(name.as_bytes()).map_err(|_| { return std::io::Error::from_raw_os_error(libc::EINVAL); });
}

/// `openat` for a directory, with the given extra flags, returning the descriptor as a file.
fn open_directory(parent: RawFd, name: &OsStr, flags: libc::c_int) -> std::io::Result<File> {
    let text: CString = c_name(name)?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `parent` is `AT_FDCWD` or an open descriptor.
    let descriptor: libc::c_int = unsafe { libc::openat(parent, text.as_ptr(), flags | libc::O_DIRECTORY | libc::O_CLOEXEC) };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(unsafe { File::from_raw_fd(descriptor) });
}

/// What:
///  Open the directory `name` inside `parent`, never following a link, and make it searchable by its owner when the
///  mode forbids reading it.
/// Why:
///   A copied directory can be mode 000 or 555; the pass has to enter it to fill it. The owner's bits are changed
///  through the descriptor of the directory just opened, so a swapped path cannot redirect the change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = await openDirectoryNoFollow(parent, name);
/// ```
fn open_component(parent: RawFd, name: &OsStr, follow: bool) -> std::io::Result<File> {
    let nofollow: libc::c_int = if follow { 0 } else { libc::O_NOFOLLOW };
    // One flag set for the first open and the retry after widening, so the retry cannot follow a link the first open refused.
    // `O_RDONLY` is 0, so `|` and `^` give the same flags here.
    let reading: libc::c_int = libc::O_RDONLY | nofollow;
    match open_directory(parent, name, reading) {
        Err(error) if error.raw_os_error() == Some(libc::EACCES) => {
            let handle: File = open_directory(parent, name, libc::O_PATH | nofollow)?;
            let metadata: std::fs::Metadata = handle.metadata()?;
            let widened: std::fs::Permissions = std::fs::Permissions::from_mode(metadata.mode() | 0o700);
            if std::fs::set_permissions(Anchor::descriptor_path(handle.as_raw_fd()), widened).is_err() {
                return Err(error);
            }
            return open_directory(parent, name, reading);
        }
        other => return other,
    }
}

impl Anchor {
    /// The `/proc` path that names the directory open on a descriptor.
    fn descriptor_path(descriptor: RawFd) -> PathBuf {
        return PathBuf::from(format!("/proc/self/fd/{descriptor}"));
    }

    /// What:
    ///  Open `root` (links in `root` itself are followed, because the caller names it) and then each component of
    ///  `relative` below it without following a link.
    /// Why:
    ///   A component that is a symbolic link, a file or gone is an error (`ELOOP`, `ENOTDIR`, `ENOENT`), so the caller
    ///  can tell a replaced directory from a failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const anchor = await Anchor.openBelow(root, relative);
    /// ```
    pub fn open_below(root: &Path, relative: &Path) -> std::io::Result<Anchor> {
        let mut current: File = open_component(libc::AT_FDCWD, root.as_os_str(), true)?;
        for component in relative.components() {
            let Component::Normal(name) = component else { return Err(std::io::Error::from_raw_os_error(libc::EINVAL)) };
            current = open_component(current.as_raw_fd(), name, false)?;
        }
        return Ok(Anchor { directory: current });
    }

    /// The open directory, for the operations that take a descriptor.
    pub fn file(&self) -> &File {
        return &self.directory;
    }

    /// The path that names the directory itself.
    pub fn path(&self) -> PathBuf {
        return Anchor::descriptor_path(self.directory.as_raw_fd());
    }

    /// The path of `name` inside the directory.
    pub fn child(&self, name: &OsStr) -> PathBuf {
        return Anchor::descriptor_path(self.directory.as_raw_fd()).join(name);
    }
}

/// Anchor controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_anchor_tests.rs"]
mod tests;
