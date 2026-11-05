//! What:
//!  The descriptor-bound look at one added path's worktree copy: its identity, whether it still holds the original bytes, the
//!  landed bytes or something else, and the result of installing every added path.
//! Why:
//!  A copy is replaced only while it still holds the original bytes, so every replacement starts from an inspection that
//!  reads through an open descriptor and never follows a link.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const state = inspectWorktreeFile({ path, original, landed }); // 'original' | 'landed' | 'changed' | 'missing'
//! ```

use std::io::Read;
use std::path::Path;

/// What:
///  The identity of an unchanged original worktree file.
/// Why:
///   The replacement is installed only while the file still has exactly this identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeFileIdentity = { device; inode; modified; changed; size; mode };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorktreeFileIdentity {
    /// Device number.
    pub device: u64,
    /// Inode number.
    pub inode: u64,
    /// Modification time in nanoseconds.
    pub modified: i128,
    /// Status change time in nanoseconds.
    pub changed: i128,
    /// Size in bytes.
    pub size: u64,
    /// Permission bits.
    pub mode: u32,
}

/// What:
///  The exact post-commit state of one worktree copy.
/// Why:
///   `WorktreeFileState`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeFileState = { kind: 'original'; identity } | { kind: 'intended' } | { kind: 'conflict' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorktreeFileState {
    /// The original bytes,
    ///  unchanged since the check began.
    Original(WorktreeFileIdentity),
    /// Already the landed bytes.
    Intended,
    /// Missing,
    ///  linked,
    ///  replaced,
    ///  edited or of the wrong kind.
    Conflict,
}

/// What:
///  Rewritten and kept paths of one completion.
/// Why:
///   `AddedWorktreeInstallResult`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddedWorktreeInstallResult = { rewritten: Uint8Array[]; conflicted: Uint8Array[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstallResult {
    /// Paths whose copy now holds the landed bytes because this call wrote them.
    pub rewritten: Vec<Vec<u8>>,
    /// Paths left alone because their copy changed after the check.
    pub conflicted: Vec<Vec<u8>>,
}

/// What:
///  Whether an error means "the worktree changed under us" rather than a failure.
/// Why:
///   `WORKTREE_CONFLICT_CODES`:
///  `ENOENT`,
///  `ENOTDIR`,
///  `ELOOP`,
///  `EACCES`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// WORKTREE_CONFLICT_CODES.has(error.code)
/// ```
pub fn is_conflict_error(error: &std::io::Error) -> bool {
    if matches!(
        error.kind(),
        std::io::ErrorKind::NotFound
            | std::io::ErrorKind::NotADirectory
            | std::io::ErrorKind::PermissionDenied
    ) {
        return true;
    }
    #[cfg(unix)]
    {
        return error.raw_os_error() == Some(libc::ELOOP);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return false;
    }
}

/// What:
///  The identity fields of metadata.
/// Why:
///   Compared before and after the read,
///  and again before the replacement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { device: m.dev, inode: m.ino, modified: m.mtimeNs, changed: m.ctimeNs, size: m.size, mode: m.mode & 0o777 }
/// ```
pub fn identity_of(metadata: &std::fs::Metadata) -> WorktreeFileIdentity {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return WorktreeFileIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
            modified: i128::from(metadata.mtime()) * 1_000_000_000
                + i128::from(metadata.mtime_nsec()),
            changed: i128::from(metadata.ctime()) * 1_000_000_000
                + i128::from(metadata.ctime_nsec()),
            size: metadata.size(),
            mode: metadata.mode() & 0o777,
        };
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let modified: i128 = match metadata.modified() {
            Ok(time) => match time.duration_since(std::time::UNIX_EPOCH) {
                Ok(elapsed) => i128::try_from(elapsed.as_nanos()).unwrap_or(i128::MAX),
                Err(_) => 0,
            },
            Err(_) => 0,
        };
        return WorktreeFileIdentity {
            device: 0,
            inode: 0,
            modified,
            changed: modified,
            size: metadata.len(),
            mode: if metadata.permissions().readonly() {
                0o444
            } else {
                0o666
            },
        };
    }
}

/// What:
///  The link count of metadata;
///  1 where the platform does not report one.
/// Why:
///   A hard-linked copy is shared with another path and is never rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// metadata.nlink
/// ```
pub fn link_count(metadata: &std::fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return metadata.nlink();
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = metadata;
        return 1;
    }
}

/// What:
///  Whether the directory of `destination` is its own canonical path.
/// Why:
///   A directory replaced by a link after the check would redirect the write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await realpath(dirname(destination))) === dirname(destination)
/// ```
pub fn directory_is_canonical(destination: &Path) -> std::io::Result<bool> {
    let Some(parent) = destination.parent() else {
        return Ok(false);
    };
    return Ok(std::fs::canonicalize(parent)? == parent);
}

/// What:
///  Open a file for reading without following a link or blocking on a FIFO.
/// Why:
///   A FIFO swapped in after the `lstat` must not hang the wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await open(destination, O_RDONLY | O_NOFOLLOW | O_NONBLOCK)
/// ```
pub fn open_nonblocking(destination: &Path) -> std::io::Result<std::fs::File> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    return options.open(destination);
}

/// What:
///  Classify a worktree copy as original,
///  intended or conflict.
/// Why:
///   `inspectWorktreeFile`:
///  missing,
///  linked and concurrently replaced paths are conflicts,
///       not recovery failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function inspectWorktreeFile({ destination, gitMode, original, intended }): Promise<WorktreeFileState>;
/// ```
pub fn inspect_worktree_file(
    destination: &Path,
    git_mode: &str,
    original: &[u8],
    intended: &[u8],
) -> std::io::Result<WorktreeFileState> {
    match inspect_or_fail(destination, git_mode, original, intended) {
        Ok(state) => return Ok(state),
        Err(error) if is_conflict_error(&error) => return Ok(WorktreeFileState::Conflict),
        Err(error) => return Err(error),
    }
}

/// What:
///  The inspection steps,
///  with every error returned.
/// Why:
///   `inspect_worktree_file` turns the conflict errors into a conflict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // body of inspectWorktreeFile inside its try
/// ```
pub fn inspect_or_fail(
    destination: &Path,
    git_mode: &str,
    original: &[u8],
    intended: &[u8],
) -> std::io::Result<WorktreeFileState> {
    if !directory_is_canonical(destination)? {
        return Ok(WorktreeFileState::Conflict);
    }
    let entry: std::fs::Metadata = std::fs::symlink_metadata(destination)?;
    if !entry.file_type().is_file() || link_count(&entry) != 1 {
        return Ok(WorktreeFileState::Conflict);
    }
    let mut handle: std::fs::File = open_nonblocking(destination)?;
    let opened: std::fs::Metadata = handle.metadata()?;
    let opened_identity: WorktreeFileIdentity = identity_of(&opened);
    let entry_identity: WorktreeFileIdentity = identity_of(&entry);
    if !opened.file_type().is_file()
        || link_count(&opened) != 1
        || entry_identity.device != opened_identity.device
        || entry_identity.inode != opened_identity.inode
    {
        return Ok(WorktreeFileState::Conflict);
    }
    if (opened_identity.mode & 0o100 != 0) != (git_mode == "100755") {
        return Ok(WorktreeFileState::Conflict);
    }
    // `mut` allows reading the whole content into the buffer.
    let mut bytes: Vec<u8> = Vec::new();
    handle.read_to_end(&mut bytes)?;
    let after: WorktreeFileIdentity = identity_of(&std::fs::symlink_metadata(destination)?);
    if after.device != opened_identity.device
        || after.inode != opened_identity.inode
        || after.modified != opened_identity.modified
        || after.changed != opened_identity.changed
        || after.size != opened_identity.size
        || !directory_is_canonical(destination)?
    {
        return Ok(WorktreeFileState::Conflict);
    }
    if bytes == intended {
        return Ok(WorktreeFileState::Intended);
    }
    if bytes != original {
        return Ok(WorktreeFileState::Conflict);
    }
    return Ok(WorktreeFileState::Original(opened_identity));
}
