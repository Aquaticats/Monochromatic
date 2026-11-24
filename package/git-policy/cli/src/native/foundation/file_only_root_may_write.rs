//! What:
//!  Reading one file this program's own account alone may write:
//!  opened without following a link,
//!  then refused unless the account that runs this program owns it and no group
//!  or other write bit is set.
//! Why:
//!  The handler's configuration names the datasets root acts on,
//!  so a file another account could rewrite would make every entry advisory,
//!  and a link at the path would let the account point root at a file of its
//!  choosing
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!  The store folder and the `/run` directories already prove the same two
//!  terms through their own descriptors
//!  (`slot_identity_store_control.rs`,
//!  `root_owned_directory.rs`),
//!  and the configuration file is the one path the root-run modes read that had
//!  no such proof.
//!  The owner is the account the kernel reports for this process,
//!  which is root for every installed caller of the root modes;
//!  a control that runs as an account reads its own fixture file,
//!  so both arms are reachable on a host without root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = readFileOnlyRootMayWrite('/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc'); // string | undefined
//! ```

use rustix::process::geteuid;
use std::io::Read;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The permission bits that let an account other than the owner write the file.
pub const WRITABLE_BY_OTHERS: u32 = 0o022;

/// What:
///  Why a file this program alone may write was refused,
///  or why it could not be read.
/// Why:
///   The two terms a caller has to fix are its owner and its write bits,
///  and telling them apart from a read error says which command a person runs
///  next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileControlRefusal = { kind: 'link' | 'open' | 'owner' | 'writable-by-others'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileControlRefusal {
    /// The path is a link,
    /// which this reader never follows.
    Link {
        /// The path as written.
        path: PathBuf,
    },
    /// The file could not be opened,
    /// read or examined.
    Open {
        /// The path as written.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
    /// The file is owned by another account than the one this program runs as.
    Owner {
        /// The path as written.
        path: PathBuf,
        /// The owner found.
        found: u32,
        /// The owner this program runs as.
        wanted: u32,
    },
    /// A permission bit lets an account other than the owner write the file.
    WritableByOthers {
        /// The path as written.
        path: PathBuf,
        /// The permission bits found.
        mode: u32,
    },
}

impl FileControlRefusal {
    /// The path that was refused.
    pub fn path(&self) -> &Path {
        return match self {
            FileControlRefusal::Link { path }
            | FileControlRefusal::Open { path, .. }
            | FileControlRefusal::Owner { path, .. }
            | FileControlRefusal::WritableByOthers { path, .. } => path,
        };
    }

    /// Why the file is not one this program alone may write,
    /// with no sentence about the route and no remedy:
    /// each caller appends what settles the case for it.
    ///
    /// A file that could not be opened or read at all is a read failure and
    /// not a control one,
    /// and this says what the kernel answered so each caller keeps its own
    /// sentence for that case.
    pub fn reason(&self) -> String {
        return match self {
            FileControlRefusal::Link { .. } => String::from("the path is a link, and this reader opens the file itself and never follows one"),
            FileControlRefusal::Open { reason, .. } => reason.clone(),
            FileControlRefusal::Owner { found, wanted, .. } => format!(
                "its owner is user {found} and this program runs as user {wanted}, so the account that owns it could rewrite the names this route acts on"
            ),
            FileControlRefusal::WritableByOthers { mode, .. } => {
                format!("its permission bits are {mode:04o}, and {WRITABLE_BY_OTHERS:04o} would let another account write it")
            }
        };
    }
}

/// What:
///  The file's text,
///  or nothing when no file is there,
///  requiring the owner the kernel reports for this process.
/// Why:
///   A missing configuration file is the answer "names nothing" rather than a
///  refusal,
///  which is the project's answer of 2026-11-22,
///  while a file that is there must be one this program alone may write.
///  The descriptor is opened once and both the owner and the bytes are read
///  from it,
///  so a name replaced between the two reads cannot change what was judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = readFileOnlyRootMayWrite(path); // undefined when absent, throws FileControlRefusal
/// ```
///
/// # Errors
/// A path that is a link,
/// a file that cannot be opened or read,
/// an owner other than this program's account,
/// or a group or other write bit.
pub fn read_file_only_root_may_write(path: &Path) -> Result<Option<String>, FileControlRefusal> {
    return read_file_only_root_may_write_owned_by(path, geteuid().as_raw());
}

/// What:
///  The file's text,
///  or nothing when no file is there,
///  requiring the owner it is given.
/// Why:
///  The owner this program runs as is the one fact a control cannot set,
///  so it is an argument here:
///  a control drives the owner refusal against its own fixture file,
///  and the release path passes the account the kernel reports.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = readFileOnlyRootMayWriteOwnedBy(path, 0);
/// ```
///
/// # Errors
/// Everything `read_file_only_root_may_write` refuses,
/// including an owner other than `owner`.
pub fn read_file_only_root_may_write_owned_by(path: &Path, owner: u32) -> Result<Option<String>, FileControlRefusal> {
    let held: std::fs::File = match open_without_following_a_link(path)? {
        Some(file) => file,
        None => return Ok(None),
    };
    let facts: std::fs::Metadata = held.metadata().map_err(|error| return FileControlRefusal::Open { path: path.to_path_buf(), reason: error.to_string() })?;
    if facts.uid() != owner {
        return Err(FileControlRefusal::Owner { path: path.to_path_buf(), found: facts.uid(), wanted: owner });
    }
    let permission_bits: u32 = facts.mode() & 0o7777;
    if permission_bits & WRITABLE_BY_OTHERS != 0 {
        return Err(FileControlRefusal::WritableByOthers { path: path.to_path_buf(), mode: permission_bits });
    }
    let mut text: String = String::new();
    let mut text_reader: std::fs::File = held;
    text_reader.read_to_string(&mut text).map_err(|error| return FileControlRefusal::Open { path: path.to_path_buf(), reason: error.to_string() })?;
    return Ok(Some(text));
}

/// What:
///  The descriptor of the file at `path`,
///  opened read-only with no link followed,
///  or nothing when no file is there.
/// Why:
///  `O_NOFOLLOW` refuses a link at the last component rather than resolving it,
///  so the kernel's own answer is the check;
///  a missing file answers `ENOENT` and is the one absence this reader accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openWithoutFollowingALink(path); // undefined when absent
/// ```
fn open_without_following_a_link(path: &Path) -> Result<Option<std::fs::File>, FileControlRefusal> {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_nul| {
        return FileControlRefusal::Open { path: path.to_path_buf(), reason: String::from("the path holds a NUL byte") };
    })?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `open` takes no other pointer.
    let descriptor: libc::c_int = unsafe { libc::open(text.as_ptr(), libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
    if descriptor < 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        return match (error.raw_os_error(), error.kind()) {
            (Some(libc::ENOENT), _) | (_, std::io::ErrorKind::NotFound) => Ok(None),
            (Some(libc::ELOOP), _) => Err(FileControlRefusal::Link { path: path.to_path_buf() }),
            _ => Err(FileControlRefusal::Open { path: path.to_path_buf(), reason: error.to_string() }),
        };
    }
    // SAFETY: `open` returned a new descriptor that nothing else owns.
    return Ok(Some(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) })));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "file_only_root_may_write_tests.rs"]
mod tests;
