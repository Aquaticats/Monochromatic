//! What:
//!  The identity of a linked worktree's administrative directory, and the registration token published into exactly that
//!  directory.
//! Why:
//!  Git reuses an administrative name after a worktree is removed, so a name says nothing about which registration is
//!  meant. The wrapper reads a worktree's state, and only later writes its token; the token is written through a handle to
//!  the directory that was read, so a directory that was removed and created again under the same name is never tagged
//!  with the earlier registration's identity. The token appears whole or not at all: it is written to a private
//!  temporary name and published with a link, so a process killed in the middle leaves no empty token.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = await adminIdentity(adminRoot, id); const token = await ensureRegistrationToken(adminRoot, id, identity);
//! ```

use super::random_id::random_uuid;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_registry::REGISTRATION_NAME;
use crate::json_record::quote;
use std::ffi::CString;
use std::fs::{File, Metadata};
use std::io::Write;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

/// What:
///  The identity text of a directory: device, inode and, where the file system records it, the birth time.
/// Why:
///   A directory that is removed and created again under the same name is another directory; the birth time tells apart
///  two directories that reused one inode number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${stat.dev}:${stat.ino}:${stat.birthtimeNs}`
/// ```
pub fn identity_of(metadata: &Metadata) -> String {
    let born: String = metadata
        .created()
        .ok()
        .and_then(|time| return time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or_else(|| return String::from("-"), |elapsed| return elapsed.as_nanos().to_string());
    return format!("{}:{}:{born}", metadata.dev(), metadata.ino());
}

fn inspect_failure(admin_root: &Path, error: &std::io::Error) -> WorktreeCopyError {
    let message: String = format!(
        "cli-git: could not inspect linked-worktree administration at {}.",
        quote(admin_root.to_string_lossy().as_ref())
    );
    return failure_with_cause(message.as_str(), Some(error));
}

/// Whether an open failure means that the directory is not there (any more) as a directory.
fn is_gone(error: &std::io::Error) -> bool {
    return matches!(
        error.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
    ) || error.raw_os_error() == Some(libc::ELOOP);
}

/// Opens the administrative directory itself, refusing a link in its place; `None` when it is gone.
fn open_directory(admin_root: &Path, admin_id: &str) -> Result<Option<File>, WorktreeCopyError> {
    let opened: std::io::Result<File> = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(admin_root.join(admin_id));
    return match opened {
        Ok(directory) => Ok(Some(directory)),
        Err(error) if is_gone(&error) => Ok(None),
        Err(error) => Err(inspect_failure(admin_root, &error)),
    };
}

/// What:
///  The identity of the administrative directory now at this name, or nothing when there is none.
/// Why:
///   Recorded when a worktree is observed, so that the token is later written into that directory and no other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await adminIdentity(adminRoot, id)
/// ```
pub fn admin_identity(admin_root: &Path, admin_id: &str) -> Result<Option<String>, WorktreeCopyError> {
    let Some(directory) = open_directory(admin_root, admin_id)? else {
        return Ok(None);
    };
    let metadata: Metadata = directory.metadata().map_err(|error| return inspect_failure(admin_root, &error))?;
    return Ok(Some(identity_of(&metadata)));
}

fn c_name(name: &str) -> std::io::Result<CString> {
    return CString::new(name).map_err(|error| return std::io::Error::new(std::io::ErrorKind::InvalidInput, error));
}

fn open_at(directory: &File, name: &str, flags: i32) -> std::io::Result<File> {
    let c_text: CString = c_name(name)?;
    // SAFETY: the descriptor is open for the call, the name is NUL-terminated, and a returned descriptor is owned here.
    let descriptor: i32 = unsafe {
        libc::openat(directory.as_raw_fd(), c_text.as_ptr(), flags | libc::O_CLOEXEC, 0o600 as libc::c_uint)
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `descriptor` was just returned by `openat` and nothing else owns it.
    return Ok(unsafe { File::from_raw_fd(descriptor) });
}

fn link_at(directory: &File, from: &str, to: &str) -> std::io::Result<()> {
    let from_text: CString = c_name(from)?;
    let to_text: CString = c_name(to)?;
    let fd: i32 = directory.as_raw_fd();
    // SAFETY: the descriptor is open for the call and both names are NUL-terminated.
    let status: i32 = unsafe { libc::linkat(fd, from_text.as_ptr(), fd, to_text.as_ptr(), 0) };
    return if status == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

fn unlink_at(directory: &File, name: &str) -> std::io::Result<()> {
    let c_text: CString = c_name(name)?;
    // SAFETY: the descriptor is open for the call and the name is NUL-terminated.
    let status: i32 = unsafe { libc::unlinkat(directory.as_raw_fd(), c_text.as_ptr(), 0) };
    return if status == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

/// The token in the directory, `None` for an absent or empty file.
fn read_token_at(directory: &File) -> std::io::Result<Option<String>> {
    return match open_at(directory, REGISTRATION_NAME, libc::O_RDONLY | libc::O_NOFOLLOW) {
        Ok(mut file) => {
            let mut text: String = String::new();
            std::io::Read::read_to_string(&mut file, &mut text)?;
            Ok(Some(text.trim().to_string()).filter(|token| return !token.is_empty()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    };
}

/// Writes the token to a private temporary name in the directory and publishes it whole under its final name.
fn publish_token_at(directory: &File, token: &str, unique: &str) -> std::io::Result<()> {
    let temporary: String = format!("{REGISTRATION_NAME}.{unique}.tmp");
    let mut file: File = open_at(directory, temporary.as_str(), libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL)?;
    let written: std::io::Result<()> = file.write_all(token.as_bytes()).and_then(|()| return file.sync_all());
    let linked: std::io::Result<()> = written.and_then(|()| return link_at(directory, temporary.as_str(), REGISTRATION_NAME));
    let _removed: std::io::Result<()> = unlink_at(directory, temporary.as_str());
    linked?;
    return directory.sync_all();
}

/// What:
///  The registration token of the administrative directory with this identity, written if the directory has none.
/// Why:
///   Written through a handle to the directory that was observed, so a replacement under the same name is refused
///  (`None`) rather than tagged; published whole, so a kill leaves no empty file that would read as no token forever.
///  An existing token is kept: the first publisher wins and the others read what it wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ensureRegistrationToken(adminRoot, id, expectedIdentity)
/// ```
pub fn ensure_registration_token(
    admin_root: &Path,
    admin_id: &str,
    expected_identity: Option<&str>,
) -> Result<Option<String>, WorktreeCopyError> {
    let Some(directory) = open_directory(admin_root, admin_id)? else {
        return Ok(None);
    };
    let metadata: Metadata = directory.metadata().map_err(|error| return inspect_failure(admin_root, &error))?;
    if expected_identity.is_some_and(|expected| return expected != identity_of(&metadata)) {
        return Ok(None);
    }
    if let Some(known) = read_token_at(&directory).map_err(|error| return inspect_failure(admin_root, &error))? {
        return Ok(Some(known));
    }
    let token: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    let unique: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    // `match` on the publication: published now, published first by another command, or the directory is gone.
    match publish_token_at(&directory, token.as_str(), unique.as_str()) {
        Ok(()) => return Ok(Some(token)),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let first: Option<String> =
                read_token_at(&directory).map_err(|read| return inspect_failure(admin_root, &read))?;
            if first.is_some() {
                return Ok(first);
            }
            // A file with no token in it is what an older release left when it was killed between creating the file and
            // writing it. This release never publishes one, so it is replaced.
            let _removed = unlink_at(&directory, REGISTRATION_NAME);
            let retry: String = random_uuid().map_err(|failure| return raw_failure(&failure.0))?;
            return match publish_token_at(&directory, token.as_str(), retry.as_str()) {
                Ok(()) => Ok(Some(token)),
                Err(again) if again.kind() == std::io::ErrorKind::AlreadyExists => {
                    read_token_at(&directory).map_err(|read| return inspect_failure(admin_root, &read))
                }
                Err(again) if again.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(again) => Err(inspect_failure(admin_root, &again)),
            };
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(inspect_failure(admin_root, &error)),
    }
}

#[cfg(test)]
#[path = "worktree_copy_identity_tests.rs"]
mod tests;
