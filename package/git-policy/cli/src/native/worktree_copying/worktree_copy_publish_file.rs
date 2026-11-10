//! What:
//!  The publication of one staged regular file at its destination name in one step:
//!  its bytes are written to a file the filesystem has given no name,
//!  the file takes the entry's exact mode,
//!  and one link gives it the destination name only once it holds every byte.
//! Why:
//!  A file created under its final name and filled afterwards exists,
//! for as long as the copy runs,
//!  with a prefix of the bytes and the mode the process's umask chose,
//! so a kill in that window leaves a file at that name that is not what the source holds,
//!  a credential at a mode wider than its source,
//! and a resumed copy refuses the destination and erases its own record.
//!  A file with no name cannot leave either:
//!  the kernel gives its blocks back when the last descriptor closes,
//! so a kill leaves the destination as it was,
//! and the resumed copy installs the entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unnamed = await openUnnamed(directory, mode); // O_TMPFILE, or a private name renamed into place
//! await writeBytes(stagePath, unnamed);
//! await unnamed.chmod(mode);
//! await linkUnnamed(unnamed, destinationPath); // fails when anything is there
//! ```

use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure, raw_failure_of};
use std::ffi::{CString, OsStr, OsString};
use std::io::Write;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// What:
///  The name a fallback temporary gets while it is being filled.
/// Why:
///   The fallback is taken only on a filesystem that has no unnamed files,
/// so the name is visible until the publication renames it;
///  the reader must be able to tell it from the person's own files by its name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_PREFIX = '.monochromatic-git-policy-cli-partial-';
/// ```
const TEMPORARY_PREFIX: &str = ".monochromatic-git-policy-cli-partial-";

/// What:
///  How many characters of the temporary's digest are kept.
/// Why:
///   A name is at most `NAME_MAX` bytes and the destination's own name can use all of them,
/// so the temporary's name may not grow with it;
///  sixteen hexadecimal characters name the stage and the entry together,
/// which is enough that two entries of one stage never share one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_DIGEST_CHARACTERS = 16;
/// ```
const TEMPORARY_DIGEST_CHARACTERS: usize = 16;

/// What:
///  The events of the first prime of the digest:
/// the one FNV-1a multiplies by.
/// Why:
///   A name that a later release computes again must be the same name,
/// and a hasher chosen by the standard library may change with the compiler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FNV_PRIME = 0x100000001b3n;
/// ```
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// What:
///  The offset basis FNV-1a starts from.
/// Why:
///   The same as the prime:
/// the digest is written out here rather than taken from a library whose
///  value may change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FNV_OFFSET = 0xcbf29ce484222325n;
/// ```
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// What:
///  The operating-system errors that mean the filesystem cannot hold an unnamed file.
/// Why:
///   Only those refusals fall back to a temporary that has a name;
/// a full disk or a missing directory does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNNAMED_UNSUPPORTED_CODES = new Set(['EOPNOTSUPP', 'ENOSYS', 'EINVAL']);
/// ```
const UNNAMED_UNSUPPORTED_CODES: [i32; 3] = [libc::EOPNOTSUPP, libc::ENOSYS, libc::EINVAL];

/// What:
///  The operating-system errors that mean the filesystem cannot publish a temporary
///  by a rename that does not replace.
/// Why:
///   Only those refusals fall back to a hard link;
/// a name already taken is a refusal of the publication,
/// not of the filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RENAME_UNSUPPORTED_CODES = new Set(['EINVAL', 'ENOSYS', 'EOPNOTSUPP']);
/// ```
const RENAME_UNSUPPORTED_CODES: [i32; 3] = [libc::EINVAL, libc::ENOSYS, libc::EOPNOTSUPP];

/// What:
///  Whether a failure means the filesystem has no such operation.
/// Why:
///   The fallbacks exist for that one answer,
/// and a refusal for any other reason is reported as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unsupported = (error, codes) => codes.includes(error.errno);
/// ```
fn is_unsupported(error: &std::io::Error, codes: &[i32]) -> bool {
    return match error.raw_os_error() {
        Some(code) => codes.contains(&code),
        None => false,
    };
}

/// What:
///  The error a name this release cannot hand the kernel stands for.
/// Why:
///   Every name and path here becomes NUL-terminated text,
/// and a NUL byte inside one is not a name the kernel can take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invalidName = () => Object.assign(new Error('invalid'), { code: 'EINVAL' });
/// ```
fn invalid_name<T>(_refused: T) -> std::io::Error {
    return std::io::Error::from_raw_os_error(libc::EINVAL);
}

/// What:
///  The directory a name below it is created in,
/// held open for the calls that create it.
/// Why:
///   Every call below names one component of that directory,
/// so a path a rename swapped between two calls cannot send the
///  bytes somewhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = await open(parent, O_DIRECTORY);
/// ```
fn open_directory(path: &Path) -> Result<std::fs::File, WorktreeCopyError> {
    // `match` on the open names the directory the caller could not use.
    return match std::fs::File::open(path) {
        Ok(directory) => Ok(directory),
        Err(error) => Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: the directory {} holding an ignored-state entry could not be opened.",
                crate::json_record::quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            Some(&error),
        )),
    };
}

/// What:
///  The final component of a path,
/// which is the one name every call here takes.
/// Why:
///   The directory is held open,
/// so only the last component is resolved,
/// and a path with no component names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastName = (path) => path.basename(path);
/// ```
fn last_name(path: &Path) -> Result<&OsStr, WorktreeCopyError> {
    return match path.file_name() {
        Some(name) => Ok(name),
        None => Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: {} names no entry to install.",
                crate::json_record::quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        )),
    };
}

/// What:
///  A file with no name in `directory`,
/// created with `mode`,
/// or the refusal the filesystem gave.
/// Why:
///   This is the preferred home of a file being filled:
/// nothing can read it before its bytes and mode are whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const descriptor = openat(dir, '.', O_TMPFILE | O_WRONLY, mode);
/// ```
fn open_unnamed(directory: &std::fs::File, mode: u32) -> std::io::Result<std::fs::File> {
    let dot: CString = match CString::new(".") {
        Ok(text) => text,
        Err(_nul) => return Err(std::io::Error::from_raw_os_error(libc::EINVAL)),
    };
    // SAFETY: `dot` is a NUL-terminated string that outlives the call, and `directory` is an open descriptor.
    let descriptor: libc::c_int = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            dot.as_ptr(),
            libc::O_TMPFILE | libc::O_WRONLY | libc::O_CLOEXEC,
            mode as libc::mode_t,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// What:
///  Create a file under `name` in `directory`,
/// failing when anything is there,
/// a link included.
/// Why:
///   The fallback's temporary is created exclusively,
/// so it never takes a name a person's file already has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const descriptor = openat(dir, name, O_CREAT | O_EXCL | O_WRONLY, mode);
/// ```
fn create_named(directory: &std::fs::File, name: &OsStr, mode: u32) -> std::io::Result<std::fs::File> {
    let text: CString = CString::new(name.as_bytes()).map_err(invalid_name)?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `directory` is an open descriptor.
    let descriptor: libc::c_int = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            text.as_ptr(),
            libc::O_CREAT | libc::O_EXCL | libc::O_WRONLY | libc::O_CLOEXEC,
            mode as libc::mode_t,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// What:
///  Give the open file the destination name,
/// and never another object's name up for it.
/// Why:
///   A link is one step:
/// the name either names this file with every byte and its mode,
///  or it names what was there before,
///  so a reader never sees a partial file and nothing is ever replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkat(unnamed, destination); // EEXIST when anything is there
/// ```
fn link_unnamed(file: &std::fs::File, destination: &Path) -> std::io::Result<()> {
    let text: CString = CString::new(destination.as_os_str().as_bytes()).map_err(invalid_name)?;
    // The descriptor's path is the file itself, so the link names the object and not a directory entry.
    let through_descriptor: CString = CString::new(format!("/proc/self/fd/{}", file.as_raw_fd())).map_err(invalid_name)?;
    // SAFETY: both strings are NUL-terminated and outlive the call.
    let linked: libc::c_int = unsafe {
        libc::linkat(
            libc::AT_FDCWD,
            through_descriptor.as_ptr(),
            libc::AT_FDCWD,
            text.as_ptr(),
            libc::AT_SYMLINK_FOLLOW,
        )
    };
    if linked == 0 {
        return Ok(());
    }
    let refusal: std::io::Error = std::io::Error::last_os_error();
    // A system without the descriptor directory still links the file by the empty name that names it.
    if refusal.raw_os_error() != Some(libc::ENOENT) && refusal.raw_os_error() != Some(libc::EPERM) {
        return Err(refusal);
    }
    // SAFETY: the empty string is a NUL-terminated string that outlives the call, and `file` is an open descriptor.
    let empty_after_descriptor: libc::c_int = unsafe {
        libc::linkat(
            file.as_raw_fd(),
            c"".as_ptr(),
            libc::AT_FDCWD,
            text.as_ptr(),
            libc::AT_EMPTY_PATH,
        )
    };
    if empty_after_descriptor == 0 {
        return Ok(());
    }
    return Err(std::io::Error::last_os_error());
}

/// What:
///  Move `temporary` to `destination`,
/// and never over anything already there.
/// Why:
///   A rename that does not replace is one step,
/// so the destination name either still holds what it held or holds the whole file;
///  a filesystem without that flag publishes by a hard link whose staging name is removed after,
/// which is one step as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await renameat2(dir, temporary, dir, destination, RENAME_NOREPLACE) } catch (e) { if (e.code === 'EEXIST') throw e; await link(temporary, destination); await unlink(temporary); }
/// ```
fn move_temporary_into_place(
    directory: &std::fs::File,
    temporary: &OsStr,
    destination: &OsStr,
) -> std::io::Result<()> {
    let from: CString = CString::new(temporary.as_bytes()).map_err(invalid_name)?;
    let to: CString = CString::new(destination.as_bytes()).map_err(invalid_name)?;
    // SAFETY: both strings are NUL-terminated and outlive the call, and `directory` is an open descriptor.
    let renamed: libc::c_int = unsafe {
        libc::renameat2(
            directory.as_raw_fd(),
            from.as_ptr(),
            directory.as_raw_fd(),
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if renamed == 0 {
        return Ok(());
    }
    let refusal: std::io::Error = std::io::Error::last_os_error();
    if is_unsupported(&refusal, &RENAME_UNSUPPORTED_CODES) {
        // SAFETY: both strings are NUL-terminated and outlive the call, and `directory` is an open descriptor.
        let linked: libc::c_int = unsafe {
            libc::linkat(
                directory.as_raw_fd(),
                from.as_ptr(),
                directory.as_raw_fd(),
                to.as_ptr(),
                0,
            )
        };
        if linked != 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: the string is NUL-terminated and outlives the call, and `directory` is an open descriptor.
        let _unlinked: libc::c_int = unsafe { libc::unlinkat(directory.as_raw_fd(), from.as_ptr(), 0) };
        return Ok(());
    }
    return Err(refusal);
}

/// What:
///  Remove the entry of this name in `directory`,
/// accepting an absent one.
/// Why:
///   The fallback's temporary is named after the stage and the entry,
/// so a name left by an interrupted run of this very copy is the one that must go before it is made again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await unlinkat(dir, name).catch((e) => { if (e.code !== 'ENOENT') throw e; });
/// ```
fn remove_present(directory: &std::fs::File, name: &OsStr) -> std::io::Result<()> {
    let text: CString = CString::new(name.as_bytes()).map_err(invalid_name)?;
    // SAFETY: the string is NUL-terminated and outlives the call, and `directory` is an open descriptor.
    let removed: libc::c_int = unsafe { libc::unlinkat(directory.as_raw_fd(), text.as_ptr(), 0) };
    if removed == 0 {
        return Ok(());
    }
    let refusal: std::io::Error = std::io::Error::last_os_error();
    if refusal.kind() == std::io::ErrorKind::NotFound {
        return Ok(());
    }
    return Err(refusal);
}

/// What:
///  A stable sixteen-character name for one entry of one stage.
/// Why:
///   The fallback needs a name that a later run of the same copy computes again,
/// so the run that resumes an interrupted one removes that
///  run's own temporary and no other file;
///  the digest is written out here,
///  so a compiler change cannot rename them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const temporaryName = (token, relativePath) => TEMPORARY_PREFIX + fnv1a(token + '\0' + relativePath).toString(16).padStart(16, '0');
/// ```
pub fn temporary_name_of(token: &str, relative_path: &[u8]) -> OsString {
    let mut digest: u64 = FNV_OFFSET;
    for byte in token
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0_u8))
        .chain(relative_path.iter().copied())
    {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(FNV_PRIME);
    }
    let text: String = format!("{TEMPORARY_PREFIX}{digest:016x}");
    debug_assert_eq!(text.len(), TEMPORARY_PREFIX.len() + TEMPORARY_DIGEST_CHARACTERS);
    return OsString::from_vec(text.into_bytes());
}

/// What:
///  Publish `stage_path`'s bytes at `destination_path` with `mode`,
/// through a file that never holds a name until it is whole.
/// Why:
///   Neither the bytes nor the mode are ever readable at the destination's name before both are complete,
/// so a
///  credential is never at a wider mode than its source,
/// and a kill leaves nothing at that name for a resumed copy to refuse.
///  `temporary` names the file the fallback fills where the filesystem has no unnamed files;
/// it is this copy's own name for this entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function publishCopyOfFile({ stagePath, destinationPath, mode, temporary }): Promise<void>;
/// ```
pub fn publish_copy_of_file(
    stage_path: &Path,
    destination_path: &Path,
    mode: u32,
    temporary: &OsStr,
) -> Result<(), WorktreeCopyError> {
    return publish_copy_of_file_with(stage_path, destination_path, mode, temporary, open_unnamed);
}

/// What:
///  `publish_copy_of_file` with the request that makes the file being filled,
/// so a control can make the filesystem refuse an unnamed one and see both paths on every machine.
/// Why:
///   Whether a filesystem has unnamed files is its choice,
/// and the fallback must be exercised where the
///  preferred path exists too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function publishCopyOfFileWith({ stagePath, destinationPath, mode, temporary, unnamed }): Promise<void>;
/// ```
pub fn publish_copy_of_file_with(
    stage_path: &Path,
    destination_path: &Path,
    mode: u32,
    temporary: &OsStr,
    unnamed: fn(&std::fs::File, u32) -> std::io::Result<std::fs::File>,
) -> Result<(), WorktreeCopyError> {
    let parent: PathBuf = match destination_path.parent() {
        Some(found) => found.to_path_buf(),
        None => {
            return Err(failure_with_cause(
                format!(
                    "monochromatic-git-policy-cli: {} names no directory to install into.",
                    crate::json_record::quote(destination_path.to_string_lossy().as_ref())
                )
                .as_str(),
                None,
            ));
        }
    };
    let directory: std::fs::File = open_directory(parent.as_path())?;
    let destination_name: OsString = last_name(destination_path)?.to_os_string();
    // `match` on the unnamed file: where the filesystem has none, the entry is filled under this copy's own name.
    let (mut file, named): (std::fs::File, bool) = match unnamed(&directory, mode) {
        Ok(held) => (held, false),
        Err(error) if is_unsupported(&error, &UNNAMED_UNSUPPORTED_CODES) => {
            remove_present(&directory, temporary).map_err(raw_failure_of)?;
            (create_named(&directory, temporary, mode).map_err(raw_failure_of)?, true)
        }
        Err(error) => return Err(raw_failure(&error)),
    };
    let written: Result<(), WorktreeCopyError> = fill_and_publish(Filling {
        stage_path,
        file: &mut file,
        mode,
        directory: &directory,
        destination_path,
        destination_name: destination_name.as_os_str(),
        temporary,
        named,
    });
    if let Err(error) = written {
        // A file that was not published is not the destination's, so the name it may hold goes with it.
        if named {
            let _removed: std::io::Result<()> = remove_present(&directory, temporary);
        }
        return Err(error);
    }
    return Ok(());
}

/// What:
///  Everything one fill needs to end with the file published,
/// or the failure that stopped it.
/// Why:
///   The steps read the same five values,
/// and a control can read them in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Filling = { stagePath; file; mode; directory; destinationPath; destinationName; temporary; named };
/// ```
struct Filling<'a> {
    /// The staged file whose bytes are copied.
    stage_path: &'a Path,
    /// The file being filled.
    file: &'a mut std::fs::File,
    /// The entry's permission bits.
    mode: u32,
    /// The destination's directory,
    /// held open.
    directory: &'a std::fs::File,
    /// The destination the file is published at.
    destination_path: &'a Path,
    /// The destination's one name.
    destination_name: &'a OsStr,
    /// The private name the fallback fills.
    temporary: &'a OsStr,
    /// Whether the file being filled holds a name.
    named: bool,
}

/// What:
///  Fill the file,
/// give it its mode,
/// and publish it,
/// stopping at the first failure.
/// Why:
///   Each step must happen on the same object in this order,
/// and the caller removes a name the fallback made when one of them fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fillAndPublish(filling): void { fillFromStage(...); filling.file.chmod(...); publish(...); }
/// ```
fn fill_and_publish(filling: Filling<'_>) -> Result<(), WorktreeCopyError> {
    fill_from_stage(filling.stage_path, filling.file)?;
    filling
        .file
        .set_permissions(std::os::unix::fs::PermissionsExt::from_mode(filling.mode))
        .map_err(raw_failure_of)?;
    return publish(
        filling.directory,
        filling.file,
        filling.destination_path,
        filling.destination_name,
        filling.temporary,
        filling.named,
    );
}

/// What:
///  Copy every byte of the staged file into the file being filled.
/// Why:
///   The bytes are the staged entry's own,
/// so the destination holds what was compared before the install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pipeline(createReadStream(stagePath), unnamed);
/// ```
fn fill_from_stage(stage_path: &Path, output: &mut std::fs::File) -> Result<(), WorktreeCopyError> {
    let mut input: std::fs::File = std::fs::File::open(stage_path).map_err(raw_failure_of)?;
    std::io::copy(&mut input, output).map_err(raw_failure_of)?;
    output.flush().map_err(raw_failure_of)?;
    return Ok(());
}

/// What:
///  Give the filled file the destination name,
/// by the link an unnamed file takes or the rename a named one takes.
/// Why:
///   Both are one step and neither replaces an entry that is already there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// named ? renameNoReplace(temporary, destination) : linkUnnamed(unnamed, destination);
/// ```
fn publish(
    directory: &std::fs::File,
    file: &std::fs::File,
    destination_path: &Path,
    destination_name: &OsStr,
    temporary: &OsStr,
    named: bool,
) -> Result<(), WorktreeCopyError> {
    if named {
        move_temporary_into_place(directory, temporary, destination_name).map_err(raw_failure_of)?;
        return Ok(());
    }
    link_unnamed(file, destination_path).map_err(raw_failure_of)?;
    return Ok(());
}

/// Publication controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_publish_file_tests.rs"]
mod tests;
