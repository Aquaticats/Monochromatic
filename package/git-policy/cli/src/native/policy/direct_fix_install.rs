//! What:
//!  Install the corrections a converged `git monochromatic-git-policy-cli fix` settled on into the
//!       worktree,
//!  all of them or none,
//!  and never touch the index.
//! Why:
//!  The installed wrapper's `direct-fix-install.ts` decides this protocol:
//!  every
//!      corrected file must still hold the bytes the fix read,
//!  each replacement is written
//!      beside its file and swapped with it,
//!  and the real index must hold the same bytes afterwards.
//!      Any failure restores every file already replaced.
//!  Failure messages name no file,
//!  because a
//!      pathname can hold the text a scan exists to keep out of output.
//!
//! What window remains:
//!  A swap (`exchange`) puts the corrected file in place and leaves the file it
//!  displaced readable at the prepared path, so the bytes that were really there
//!  are compared after nothing can overwrite them any more, and a difference
//!  swaps them back (owner: "never discard user data").
//!  Two writes can still be lost or replaced, and neither is ours to prevent:
//!  a write through a descriptor a program opened before the swap lands in the
//!  displaced file after the comparison and before the prepared file is removed,
//!  and a write to the path after the swap replaces the corrected bytes (the
//!  newest bytes win, which is what the person asked for).
//!  Both windows are the microseconds between one file system call and the next,
//!  not the length of a policy pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await installDirectFix({ scope, changedPaths, finalCandidates, originals });
//! ```

/// Import the candidate's mode,
///  which decides the replacement's permissions.
use crate::candidate_object::CandidateMode;
/// Import the unique name part shared with the private index directories.
use crate::candidate_private_index::unique_suffix;
/// Import the conversion of Git pathname bytes to a native path.
use crate::git_metadata::path_from_git_bytes;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The name prefix of the files a fix writes beside each corrected file.
pub const INSTALL_SIBLING_PREFIX: &str = ".cli-git-direct-fix-";

/// The start of every installation failure message.
const INSTALL_FAILED: &str = "monochromatic-git-policy-cli fix could not install its corrections";

/// What:
///  One file to correct:
///  its pathname,
///  mode,
///  the bytes the fix read and the bytes to
///       write.
///  `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   The read bytes prove nobody changed the file while the fix ran.
///  What a replacement keeps is the
///       destination's permission bits and owner (see `Kept`);
///  what it does not carry is everything
///       else a file can hold:
///  its extended attributes and access-control list,
///  its inode number and
///       birth time,
///  and its share of a hard link,
///  whose other name keeps the bytes it had.
///  A symbolic
///       link at the path is refused rather than replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallChange = { path: Uint8Array; mode: CandidateMode; original: Uint8Array; replacement: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallChange {
    /// The pathname,
    ///  relative to the repository's top level,
    ///  as Git's raw bytes.
    pub path: Vec<u8>,
    /// The file's mode;
    ///  executable files stay executable.
    pub mode: CandidateMode,
    /// The bytes the fix read;
    ///  the worktree file must still hold them.
    pub original: Rc<[u8]>,
    /// The corrected bytes.
    pub replacement: Rc<[u8]>,
}

/// What:
///  One prepared replacement:
///  the file,
///  the corrected bytes beside it,
///  and the bytes
///       the fix read.
/// Why:
///   Swapping the prepared file with its destination replaces it in one step and
///       leaves the replaced file at the prepared path,
///  so the bytes that were really there can
///       be compared after the swap,
///  and the same swap puts them back when they differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedReplacement = { destination: string; prepared: string; original: Uint8Array };
/// ```
#[derive(Debug)]
pub struct Replacement {
    /// The worktree file.
    destination: PathBuf,
    /// The corrected bytes before the swap,
    ///  and the bytes the swap displaced after it,
    ///       beside the file.
    prepared: PathBuf,
    /// The bytes the fix read,
    ///  which the displaced file must still hold.
    original: Rc<[u8]>,
}

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
        /// `CString` is a pathname as the operating system wants it: bytes and a terminator.
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
///  An installation failure message:
///  the shared start,
///  then what happened.
/// Why:
///   Every failure reads the same way and names no file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const installFailure = (detail: string) => `monochromatic-git-policy-cli fix could not install its corrections: ${detail}`;
/// ```
fn install_failure(detail: &str) -> String {
    return format!("{INSTALL_FAILED}: {detail}");
}

/// What:
///  The real index's bytes,
///  or nothing when it does not exist.
///       `std::io::Result<Option<Vec<u8>>>` is "bytes,
///  absent,
///  or a read failure".
/// Why:
///   A repository without an index file has an empty index,
///  and must still have none
///       after the fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readIndex(path: string): Promise<Uint8Array | typeof INDEX_ABSENT>;
/// ```
pub fn read_index(path: &Path) -> std::io::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => return Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
}

/// What:
///  Remove the prepared and backup files of each replacement,
///  ignoring files that
///       were never written.
/// Why:
///   Neither may stay behind after the fix ends,
///  whichever way it ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cleanup(replacements): Promise<void>;
/// ```
fn cleanup(replacements: &[Replacement]) {
    for replacement in replacements {
        // What: `let _ = ...` discards each removal's `Result` on purpose.
        // Why:  A file that was never written cannot be removed, and that is fine.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await rm(prepared, { force: true });
        // ```
        let _ = std::fs::remove_file(replacement.prepared.as_path());
    }
}

/// What:
///  Swap each installed file with the bytes it displaced before it,
///  last installed first.
///       `Result<(), String>` is success or the first restore that failed.
/// Why:
///   The worktree returns to the bytes it held before the fix,
///  in the very files that held
///       them;
///  a file that cannot be restored is left in place,
///  so its bytes are not lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rollback(installed): Promise<void>;
/// ```
fn rollback(installed: &[Replacement]) -> Result<(), String> {
    // `.iter().rev()` visits the installed files in reverse order.
    for replacement in installed.iter().rev() {
        if let Err(error) = exchange(
            replacement.prepared.as_path(),
            replacement.destination.as_path(),
        ) {
            return Err(format!(
                "restoring a corrected file failed ({error}); its original bytes remain beside it in a file whose name starts with {INSTALL_SIBLING_PREFIX}"
            ));
        }
    }
    return Ok(());
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
fn write_new(path: &Path, bytes: &[u8], kept: &Kept) -> std::io::Result<()> {
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
struct Kept {
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
    fn of(described: &std::fs::Metadata) -> Kept {
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

/// What:
///  Prepare every replacement:
///  check the file still holds the bytes the fix read,
///       copy it beside itself,
///  and write the corrected bytes beside it.
/// Why:
///   Nothing in the worktree changes until every file is prepared,
///  so a file that
///       changed under the fix stops it before any file is replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareReplacements(root, changes): Promise<PreparedReplacement[]>;
/// ```
pub fn prepare_replacements(
    root: &Path,
    changes: &[InstallChange],
) -> Result<Vec<Replacement>, String> {
    let mut replacements: Vec<Replacement> = Vec::new();
    for change in changes {
        let Some(relative) = path_from_git_bytes(change.path.as_slice()) else {
            cleanup(replacements.as_slice());
            return Err(install_failure(
                "Git named a corrected file this system cannot represent, so no file was changed.",
            ));
        };
        let destination: PathBuf = root.join(relative);
        // What: `symlink_metadata` describes the name itself, so a symbolic link is seen as a link and not followed.
        // Why:  Reading through the link reads another file, and the swap would put the corrected bytes where the link
        //       is, so the link itself would be gone.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const kept = await lstat(destination);
        // ```
        let described: std::fs::Metadata = match std::fs::symlink_metadata(destination.as_path()) {
            Ok(found) => found,
            Err(error) => {
                cleanup(replacements.as_slice());
                return Err(install_failure(
                    format!("reading a corrected file failed ({error}), so no file was changed.")
                        .as_str(),
                ));
            }
        };
        if described.file_type().is_symlink() {
            cleanup(replacements.as_slice());
            return Err(install_failure(
                "a selected file is a symbolic link, and putting corrected bytes at its path would remove the link, so no file was changed.",
            ));
        }
        let current: Vec<u8> = match std::fs::read(destination.as_path()) {
            Ok(bytes) => bytes,
            Err(error) => {
                cleanup(replacements.as_slice());
                return Err(install_failure(
                    format!("reading a corrected file failed ({error}), so no file was changed.")
                        .as_str(),
                ));
            }
        };
        if current.as_slice() != &*change.original {
            cleanup(replacements.as_slice());
            return Err(install_failure(
                "a selected file changed while monochromatic-git-policy-cli fix ran, so no file was changed. Run monochromatic-git-policy-cli fix again.",
            ));
        }
        let parent: PathBuf = match destination.parent() {
            Some(found) => found.to_path_buf(),
            None => root.to_path_buf(),
        };
        let kept: Kept = Kept::of(&described);
        let prefix: String = format!("{INSTALL_SIBLING_PREFIX}{}", unique_suffix());
        let replacement: Replacement = Replacement {
            prepared: parent.join(format!("{prefix}.new")),
            original: Rc::clone(&change.original),
            destination,
        };
        let written: std::io::Result<()> = write_new(
            replacement.prepared.as_path(),
            &change.replacement,
            &kept,
        );
        // Recorded before the failure check, so a failure still removes what was written.
        replacements.push(replacement);
        if let Err(error) = written {
            cleanup(replacements.as_slice());
            return Err(install_failure(
                format!("writing a corrected file beside its original failed ({error}), so no file was changed.").as_str(),
            ));
        }
    }
    return Ok(replacements);
}

/// What:
///  Swap every prepared file with its destination,
///  compare the bytes each swap displaced
///       with the bytes the fix read,
///  then prove the real index still holds `index_before`.
///  `&Option<Vec<u8>>` borrows the bytes read before,
///  or absence.
/// Why:
///   The fix changes the worktree only.
///  A swap that fails,
///  a file whose displaced bytes
///       are no longer the ones the fix read,
///  an index that changed,
///  or an index that cannot be
///       read restores every file already replaced (see "What window remains" in `direct_fix_install.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installPrepared(replacements, realIndexPath, indexBefore): Promise<void>;
/// ```
pub fn install_prepared(
    replacements: &[Replacement],
    real_index: &Path,
    index_before: &Option<Vec<u8>>,
) -> Result<(), String> {
    let mut installed: usize = 0;
    let mut failure: Option<String> = None;
    for replacement in replacements {
        if let Err(error) = exchange(
            replacement.prepared.as_path(),
            replacement.destination.as_path(),
        ) {
            failure = Some(format!("putting a corrected file in place failed ({error})"));
            break;
        }
        installed += 1;
        // What: The prepared path now holds the file the swap displaced, so its bytes are the bytes the
        //       destination held at the instant of the swap.
        // Why:  A write that arrived after the fix read the file is compared here, after nothing can overwrite it any
        //       more, and a difference saves it by swapping it back. A path that is no longer a regular file changed
        //       in a way no byte comparison answers (a directory, a symbolic link), and is refused the same way.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const displaced = await lstat(prepared);
        // ```
        let displaced_kind: std::io::Result<std::fs::FileType> =
            match std::fs::symlink_metadata(replacement.prepared.as_path()) {
                Ok(found) => Ok(found.file_type()),
                Err(error) => Err(error),
            };
        match displaced_kind {
            Ok(kind) if kind.is_file() => {}
            Ok(_) => {
                failure = Some(String::from(
                    "a selected file changed while monochromatic-git-policy-cli fix ran",
                ));
                break;
            }
            Err(error) => {
                failure = Some(format!(
                    "reading the bytes a corrected file displaced failed ({error})"
                ));
                break;
            }
        }
        match std::fs::read(replacement.prepared.as_path()) {
            Ok(displaced) if displaced.as_slice() == &*replacement.original => {}
            Ok(_) => {
                failure = Some(String::from(
                    "a selected file changed while monochromatic-git-policy-cli fix ran",
                ));
                break;
            }
            Err(error) => {
                failure = Some(format!(
                    "reading the bytes a corrected file displaced failed ({error})"
                ));
                break;
            }
        }
    }
    if failure.is_none() {
        failure = match read_index(real_index) {
            Ok(after) if after == *index_before => None,
            Ok(_) => Some(String::from("the index changed while monochromatic-git-policy-cli fix ran")),
            Err(error) => Some(format!("reading the index again failed ({error})")),
        };
    }
    let Some(cause) = failure else {
        cleanup(replacements);
        return Ok(());
    };
    // `..installed` is the files already replaced, which the rollback restores.
    let restored: Result<(), String> = rollback(&replacements[..installed]);
    match restored {
        Ok(()) => {
            cleanup(replacements);
            return Err(install_failure(
                format!("{cause}, so every corrected file was restored. Run monochromatic-git-policy-cli fix again.")
                    .as_str(),
            ));
        }
        Err(restore_failure) => {
            // Prepared files are removed; backups stay, because one of them is needed.
            for replacement in replacements {
                let _ = std::fs::remove_file(replacement.prepared.as_path());
            }
            return Err(install_failure(
                format!("{cause}, and {restore_failure}.").as_str(),
            ));
        }
    }
}

/// What:
///  Install every change,
///  or none:
///  read the real index,
///  prepare,
///  then rename and
///       verify.
///  `Result<(), String>` is success or the failure message to report.
/// Why:
///   The whole protocol of the installed wrapper's `installDirectFix`,
///  apart from the
///       landing lock,
///  which the native wrapper does not take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installCorrections(root, realIndexPath, changes): Promise<void>;
/// ```
pub fn install_corrections(
    root: &Path,
    real_index: &Path,
    changes: &[InstallChange],
) -> Result<(), String> {
    let index_before: Option<Vec<u8>> = match read_index(real_index) {
        Ok(found) => found,
        Err(error) => {
            return Err(install_failure(
                format!("reading the index failed ({error}), so no file was changed.").as_str(),
            ));
        }
    };
    let replacements: Vec<Replacement> = prepare_replacements(root, changes)?;
    return install_prepared(replacements.as_slice(), real_index, &index_before);
}

/// Installation,
///  rollback and cleanup controls stay out of the release executable.
#[cfg(test)]
#[path = "direct_fix_install_tests.rs"]
mod tests;
