//! What:
//!  The entries of a held directory:
//! creating a file,
//! a directory and a symbolic link,
//! reading a link's target,
//! reading an entry's metadata,
//! setting its access and modification times,
//! and removing,
//! renaming or exchanging a name,
//! each as one call on one plain name below the descriptor that was opened.
//! Why:
//!  A tree other processes can rename into is only safe to change through the descriptor of the directory that was
//!  looked at,
//! and every call here names one component of that directory,
//! so nothing a swap puts at a path redirects the call.
//! The one removal that compares an identity first is in `fs_beneath_owned_removal`,
//! because a name can be taken between the look and the unlink and only a lock this program holds open makes that
//!  risk acceptable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await createFileAt(dir, 'index.lock', 0o644); // O_EXCL, a link or a file in the way is an error
//! await renameAt(dir, 'index.lock', dir, 'index', false); // RENAME_NOREPLACE
//! ```

use super::fs_beneath_names::{
    c_string_of,
    checked,
    handle_path,
    name_of,
};
use super::fs_beneath_open::{
    open_beneath,
    open_directory_at,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// Create the file `name` in `dir` with `mode`,
/// failing when anything is there,
/// a link included.
pub fn create_file_at(
    dir: &File,
    name: &OsStr,
    mode: u32,
) -> std::io::Result<File> {
    return open_beneath(
        dir,
        name,
        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
        mode,
    );
}

/// Create the directory `name` in `dir` with `mode`;
/// `Ok(true)` when it was made,
/// `Ok(false)` when a directory was
///  already there (a link or a file in the way is an error).
pub fn make_directory_at(
    dir: &File,
    name: &OsStr,
    mode: u32,
) -> std::io::Result<bool> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call and `dir` is an open descriptor.
    let result: libc::c_int =
        unsafe { libc::mkdirat(dir.as_raw_fd(), c_name.as_ptr(), mode as libc::mode_t) };
    if result == 0 {
        return Ok(true);
    }
    let error: std::io::Error = std::io::Error::last_os_error();
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        drop(open_directory_at(dir, name)?);
        return Ok(false);
    }
    return Err(error);
}

/// Create the link `name` in `dir` pointing at `target`.
pub fn symlink_at(
    target: &Path,
    dir: &File,
    name: &OsStr,
) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    let c_target = c_string_of(target.as_os_str().as_bytes(), "a link target")?;
    // SAFETY: both strings outlive the call and `dir` is an open descriptor.
    checked(unsafe { libc::symlinkat(c_target.as_ptr(), dir.as_raw_fd(), c_name.as_ptr()) })?;
    return Ok(());
}

/// The target of the link `name` in `dir`.
pub fn read_link_at(
    dir: &File,
    name: &OsStr,
) -> std::io::Result<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    let c_name = name_of(name)?;
    let mut buffer: Vec<u8> = vec![0; 4096];
    // SAFETY: `c_name` outlives the call, `buffer` has the length passed, and `dir` is an open descriptor.
    let length: libc::ssize_t = unsafe {
        libc::readlinkat(
            dir.as_raw_fd(),
            c_name.as_ptr(),
            buffer.as_mut_ptr().cast::<libc::c_char>(),
            buffer.len(),
        )
    };
    if length < 0 {
        return Err(std::io::Error::last_os_error());
    }
    buffer.truncate(usize::try_from(length).unwrap_or(0));
    return Ok(PathBuf::from(OsString::from_vec(buffer)));
}

/// The metadata of the entry `name` in `dir`,
/// without following a link.
pub fn entry_metadata_at(
    dir: &File,
    name: &OsStr,
) -> std::io::Result<std::fs::Metadata> {
    // The name is one plain component, so the lookup below the descriptor's magic link resolves nothing else.
    name_of(name)?;
    return std::fs::symlink_metadata(handle_path(dir).join(name));
}

/// Set the access and modification times of the entry `name` in `dir`,
/// without following a link.
pub fn set_times_at(
    dir: &File,
    name: &OsStr,
    times: [libc::timespec; 2],
) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    // SAFETY: `c_name` outlives the call, `times` holds the two timestamps the call reads, and `dir` is an open descriptor.
    checked(unsafe {
        libc::utimensat(
            dir.as_raw_fd(),
            c_name.as_ptr(),
            times.as_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    })?;
    return Ok(());
}

/// Remove the entry `name` from `dir`:
/// a directory when `directory` is set (it must be empty),
/// else a file or a link.
///
/// Why:
/// this removes whatever the name holds now;
/// use `remove_owned_at` for an object that was made earlier.
pub fn remove_at(
    dir: &File,
    name: &OsStr,
    directory: bool,
) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    let flags: libc::c_int = if directory { libc::AT_REMOVEDIR } else { 0 };
    // SAFETY: `c_name` outlives the call and `dir` is an open descriptor.
    checked(unsafe { libc::unlinkat(dir.as_raw_fd(), c_name.as_ptr(), flags) })?;
    return Ok(());
}

/// Rename `from` of `from_dir` to `to` of `to_dir`;
/// with `replace` false an existing `to` is an error (`RENAME_NOREPLACE`).
pub fn rename_at(
    from_dir: &File,
    from: &OsStr,
    to_dir: &File,
    to: &OsStr,
    replace: bool,
) -> std::io::Result<()> {
    let c_from = name_of(from)?;
    let c_to = name_of(to)?;
    let flags: libc::c_uint = if replace { 0 } else { libc::RENAME_NOREPLACE };
    // SAFETY: both names outlive the call and both directories are open descriptors.
    checked(unsafe {
        libc::renameat2(
            from_dir.as_raw_fd(),
            c_from.as_ptr(),
            to_dir.as_raw_fd(),
            c_to.as_ptr(),
            flags,
        )
    })?;
    return Ok(());
}

/// Exchange the entries `first` and `second` of `dir` atomically (`RENAME_EXCHANGE`).
pub fn exchange_at(
    dir: &File,
    first: &OsStr,
    second: &OsStr,
) -> std::io::Result<()> {
    let c_first = name_of(first)?;
    let c_second = name_of(second)?;
    // SAFETY: both names outlive the call and `dir` is an open descriptor.
    checked(unsafe {
        libc::renameat2(
            dir.as_raw_fd(),
            c_first.as_ptr(),
            dir.as_raw_fd(),
            c_second.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    })?;
    return Ok(());
}
