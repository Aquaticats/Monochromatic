//! What:
//!  The file-system primitives a direct fix installs with:
//!  the swap that puts a corrected file in
//!       place and leaves what it displaced readable,
//!  and the replacement written beside the file.
//! Why:
//!  `direct_fix_install.rs` decides the protocol and stays under the file-size limit;
//!  these two
//!       operations are the only places a worktree file is written or swapped,
//!  so a reader sees every way
//!       the worktree can change in one file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // exchange(destination, prepared); writeNew(prepared, bytes, kept);
//! ```

/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Swap the two files `first` and `second` name in one step.
/// Why:
///   A rename replaces a file without reading what it replaced,
///  so bytes another writer
///       put there after the fix read the file are lost;
///  `RENAME_EXCHANGE` puts the corrected
///       file in place and leaves the displaced one readable,
///  so the bytes that were really there are
///       compared after they can no longer be overwritten,
///  and the same call puts them back when they
///       differ.
///  Both names must exist and share a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // renameat2(AT_FDCWD, first, AT_FDCWD, second, RENAME_EXCHANGE)
/// ```
pub fn exchange(first: &Path, second: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        /// `CString` is a pathname as the operating system wants it:
        /// bytes and a terminator.
        use std::ffi::CString;
        /// `OsStrExt` lends a path's bytes.
        use std::os::unix::ffi::OsStrExt;
        // A pathname with a NUL byte in it cannot name a file, and the call would truncate it.
        let Ok(first_name) = CString::new(first.as_os_str().as_bytes()) else {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        };
        let Ok(second_name) = CString::new(second.as_os_str().as_bytes()) else {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        };
        // SAFETY: `renameat2` reads two NUL-terminated pathnames and writes nothing of this process;
        // both names outlive the call.
        let status: libc::c_int = unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                first_name.as_ptr(),
                libc::AT_FDCWD,
                second_name.as_ptr(),
                libc::RENAME_EXCHANGE,
            )
        };
        if status == 0 {
            return Ok(());
        }
        return Err(std::io::Error::last_os_error());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // Only Unix swaps two files in one step; elsewhere nothing is replaced.
        let _ = (first, second);
        return Err(std::io::Error::other(
            "this system has no way to swap two files in one step",
        ));
    }
}

/// What:
///  Write `bytes` to a new file at `path`,
///  then give it the permission bits and owner of the
///       file it will replace.
/// Why:
///   The file must not exist yet,
///  so no other file is overwritten,
///  and a replacement keeps what the
///       person's file had:
///  a replacement created at `0644` would publish a file kept at `0600` to
///       every local user,
///  and dropping the executable bits or the set-uid and set-gid bits would
///       change how the file runs.
///  The owner is restored only where the system lets this process set it
///       (a file created by the running user is already theirs),
///  and a failure to restore it is not a
///       failure of the write:
///  the bytes are what the fix was asked to change.
///  What a replacement does not
///       carry is named beside `InstallChange`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(path, bytes, { mode: kept.mode, flag: 'wx' }); await fchmod(fd, kept.mode); await fchown(fd, kept.uid, kept.gid);
/// ```
pub(crate) fn write_new(path: &Path, bytes: &[u8], kept: &Kept) -> std::io::Result<()> {
    /// `Write` adds `write_all` to the open file.
    use std::io::Write;
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        /// `OpenOptionsExt` adds `mode`,
        ///  the permissions a new file is created with.
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(kept.mode);
    }
    let mut file: std::fs::File = options.open(path)?;
    file.write_all(bytes)?;
    #[cfg(unix)]
    {
        keep_what_the_old_file_had(&file, kept);
    }
    return Ok(());
}

/// What:
///  What the file being replaced had,
///  which its replacement keeps.
/// Why:
///  A new file carries the umask's mode and the running user's identity;
///  both must be
///       replaced by the destination's,
///  or the fix publishes a private file and changes who may write it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Kept = { mode: number; uid: number; gid: number };
/// ```
pub(crate) struct Kept {
    /// The destination's permission bits,
    ///  the set-user-id,
    ///  set-group-id and sticky bits included.
    mode: u32,
    /// The destination's owner.
    uid: u32,
    /// The destination's group.
    gid: u32,
}

impl Kept {
    /// What:
    ///  The mode and owner of the file a replacement stands in for.
    /// Why:
    ///   Both are read from the name before anything is written,
    ///  so the replacement cannot be given the
    ///       umask's mode or the running user's identity by accident.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kept = { mode: st.mode & 0o7777, uid: st.uid, gid: st.gid };
    /// ```
    pub(crate) fn of(described: &std::fs::Metadata) -> Kept {
        #[cfg(unix)]
        {
            /// `MetadataExt` adds `uid` and `gid`;
            ///  `PermissionsExt` adds `mode`.
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            return Kept {
                mode: described.permissions().mode() & 0o7777,
                uid: described.uid(),
                gid: described.gid(),
            };
        }
        // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
        #[cfg(not(unix))]
        {
            // Other systems carry no mode or numeric owner a replacement could keep.
            let _ = described;
            return Kept {
                mode: 0,
                uid: 0,
                gid: 0,
            };
        }
    }
}

/// What:
///  Give the open replacement the permission bits and owner it keeps.
///  `&std::fs::File` borrows the written file.
/// Why:
///  `fchmod` and `fchown` act on the descriptor,
///  so no other process can swap the name between the
///       write and the change;
///  an owner this process may not give is left as the running user's,
///  because
///       only privilege can set another one and the bytes were still written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fchmodSync(fd, kept.mode); try { fchownSync(fd, kept.uid, kept.gid) } catch { /* not ours to give */ }
/// ```
#[cfg(unix)]
fn keep_what_the_old_file_had(file: &std::fs::File, kept: &Kept) {
    /// `AsRawFd` lends the descriptor the system calls take.
    use std::os::unix::io::AsRawFd;
    let descriptor: libc::c_int = file.as_raw_fd();
    // SAFETY: both calls take a descriptor this file owns and plain numbers;
    // neither reads or writes memory of this process.
    let _mode_set: libc::c_int = unsafe { libc::fchmod(descriptor, kept.mode as libc::mode_t) };
    // An owner this process may not set is not an error: the file already belongs to the running user.
    let _owner_set: libc::c_int =
        unsafe { libc::fchown(descriptor, kept.uid as libc::uid_t, kept.gid as libc::gid_t) };
}
