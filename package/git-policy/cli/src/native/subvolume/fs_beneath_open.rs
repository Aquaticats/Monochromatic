//! What:
//!  Opening a directory,
//! a file or a path handle below a held directory handle,
//! one plain name at a time,
//! through `openat2` and the `RESOLVE_*` bits that keep the lookup inside that
//!  directory and off every link and mount.
//! Why:
//!  A path string is resolved again by every call that takes it,
//! so a directory swapped for a link between two calls
//!  redirects the second one.
//! Holding the handle of each directory and naming one component at a time keeps every call on the object that
//!  was looked at,
//! and the resolution bits are written once here instead of at every call.
//! Two callers must cross exactly one mount on their last hop (the staging folder under the account's cache,
//! and the slot a clone is unmounted from),
//! so those two opens state their own bits,
//! and the rest of the folder keeps the strict open.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const root = await openRoot('/canonical/path'); // throws when an ancestor is a link
//! const dir = await openDirectoryAt(root, 'a');   // throws on a link, a mount, '.' or '..'
//! ```

use super::fs_beneath_names::{
    c_string_of,
    checked,
    handle_path,
    name_of,
};
use std::ffi::OsStr;
use std::fs::File;
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// `RESOLVE_NO_XDEV`:
/// no mount point,
/// bind mounts included,
/// is crossed.
pub(super) const RESOLVE_NO_XDEV: u64 = 0x01;
/// `RESOLVE_NO_MAGICLINKS`:
/// no `/proc` style link is followed.
pub(super) const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
/// `RESOLVE_NO_SYMLINKS`:
/// no symbolic link is followed,
/// the last component included.
pub(super) const RESOLVE_NO_SYMLINKS: u64 = 0x04;
/// `RESOLVE_BENEATH`:
/// the lookup stays below the directory it starts from.
pub(super) const RESOLVE_BENEATH: u64 = 0x08;

/// The arguments of `openat2`,
/// laid out as the kernel's `struct open_how`.
#[repr(C)]
struct OpenHow {
    /// The `open` flags.
    flags: u64,
    /// The mode of a created file.
    mode: u64,
    /// The `RESOLVE_*` bits.
    resolve: u64,
}

/// `openat2` of one plain name below `dir`,
/// resolving nothing but the name itself and refusing a mount point at it.
pub(super) fn open_beneath(
    dir: &File,
    name: &OsStr,
    flags: libc::c_int,
    mode: u32,
) -> std::io::Result<File> {
    return open_beneath_by(
        dir,
        name,
        flags,
        mode,
        RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_XDEV | RESOLVE_NO_MAGICLINKS,
    );
}

/// `openat2` of one plain name below `dir`,
/// with the `resolve` bits passed as they are.
pub(super) fn open_beneath_by(
    dir: &File,
    name: &OsStr,
    flags: libc::c_int,
    mode: u32,
    resolve: u64,
) -> std::io::Result<File> {
    let c_name = name_of(name)?;
    let how: OpenHow = OpenHow {
        flags: (flags | libc::O_CLOEXEC) as u64,
        mode: u64::from(mode),
        resolve,
    };
    // SAFETY: `c_name` is a NUL-terminated string and `how` a live `open_how` of the size passed, both outliving the call;
    // `dir` is an open descriptor.
    let result: libc::c_long = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            dir.as_raw_fd(),
            c_name.as_ptr(),
            &how as *const OpenHow,
            std::mem::size_of::<OpenHow>(),
        )
    };
    if result < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat2` returned a new descriptor that nothing else owns.
    return Ok(File::from(unsafe {
        OwnedFd::from_raw_fd(result as libc::c_int)
    }));
}

/// Open the directory at the canonical absolute path `path` as the root of a walk.
///
/// Why:
/// the ancestors of a path are resolved by the kernel and may be links,
/// so the open directory's own path,
/// read back
/// from the descriptor,
/// must equal the path asked for;
/// a path with a link among its ancestors is refused.
pub fn open_root(path: &Path) -> std::io::Result<File> {
    if !path.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{} is not absolute", path.display()),
        ));
    }
    let name = c_string_of(path.as_os_str().as_bytes(), "a path")?;
    // SAFETY: `name` is a NUL-terminated string that outlives the call.
    let descriptor: libc::c_int = checked(unsafe {
        libc::open(
            name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    })?;
    // SAFETY: `open` returned a new descriptor that nothing else owns.
    let held: File = File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
    let real: PathBuf = std::fs::read_link(handle_path(&held))?;
    if real.as_os_str() != path.as_os_str() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "{} is reached through a link: it is {}",
                path.display(),
                real.display()
            ),
        ));
    }
    return Ok(held);
}

/// Open the subdirectory `name` of `parent`,
/// refusing a link,
/// a mount point and `.` or `..`.
pub fn open_directory_at(
    parent: &File,
    name: &OsStr,
) -> std::io::Result<File> {
    return open_beneath(parent, name, libc::O_RDONLY | libc::O_DIRECTORY, 0);
}

/// What:
///  A handle naming the subdirectory `name` of `parent`,
///  which asks what the entry is without following a link,
///  crossing a mount or needing any permission on the entry itself.
/// Why:
///  A caller that must not change what it judges needs the entry's kind before it opens the entry for reading,
/// and reading a directory needs access that widening would give by changing its mode.
/// A mount point is refused with `EXDEV` here exactly as a reading open refuses it,
/// because the resolution refuses to cross a mount whatever the access asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = handleDirectoryAt(parent, name); // throws EXDEV at a mount point
/// ```
pub fn handle_directory_at(
    parent: &File,
    name: &OsStr,
) -> std::io::Result<File> {
    return open_beneath(parent, name, libc::O_PATH | libc::O_DIRECTORY, 0);
}

/// What:
///  Open the directory `name` of `parent` for reading,
///  refusing a link,
///  a magic link and a `..` escape,
///  but crossing the one mount at that name.
/// Why:
///  The staging folder lives under the account's `~/.cache`,
///  which is its own dataset mount,
///  so the deliberate hop from the home into it must cross exactly that mount
///  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
///  "The staging target and who creates it").
///  Every hop below the held cache descriptor refuses a crossing again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cache = openDirectoryCrossingOneMount(home, '.cache'); // crosses the user-cache mount only
/// ```
pub fn open_directory_crossing_one_mount(
    parent: &File,
    name: &OsStr,
) -> std::io::Result<File> {
    return open_beneath_by(
        parent,
        name,
        libc::O_RDONLY | libc::O_DIRECTORY,
        0,
        RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS,
    );
}

/// What:
///  A path handle naming the directory `name` below `parent`,
///  refusing a link,
///  a magic link and a `..` escape,
///  but crossing the one mount at that name.
/// Why:
///  The unmount's final slot hop differs from the attach's:
///  the clone's own mount stands at the slot,
///  so that one hop must cross it,
///  and every earlier hop stayed on the checkout's own filesystems
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slot = openHandleAtAMount(worktrees, 'one'); // crosses the clone's mount only
/// ```
pub fn open_handle_at_a_mount(
    parent: &File,
    name: &OsStr,
) -> std::io::Result<File> {
    return open_beneath_by(
        parent,
        name,
        libc::O_PATH | libc::O_DIRECTORY,
        0,
        RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS | RESOLVE_NO_MAGICLINKS,
    );
}

/// Open the regular file `name` of `dir` for reading,
/// refusing a link;
/// a fifo or a device is refused later by its type.
pub fn open_file_at(
    dir: &File,
    name: &OsStr,
) -> std::io::Result<File> {
    return open_beneath(dir, name, libc::O_RDONLY | libc::O_NONBLOCK, 0);
}
