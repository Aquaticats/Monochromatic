//! What:
//!  Bringing each added path's worktree copy to the landed bytes after a commit landed,
//!  with
//!       descriptor-bound checks that never overwrite a concurrent edit.
//! Why:
//!  An added path's committed bytes differ from its worktree copy (a policy corrected it);
//!  the
//!      copy is replaced only while it still holds the original bytes,
//!  through a same-directory
//!      temporary file.
//!  A copy holding neither is kept and reported,
//!  since the commit already
//!      landed (`src/policy-engine/commit-transaction-added-paths.ts`,
//!  `-worktree-check.ts`,
//!      `-worktree-replace.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await installAddedWorktreeFiles({ gitPath, cwd, repositoryRoot, records, objectDirectory });
//! ```

/// The blob batch.
use super::blob_batch::load_blob_batch;
/// Diagnostics.
use super::diagnostic_log::{debug, warn};
/// Exact path bytes as file names, and as text for messages.
use super::path_bytes::{os_from_path_bytes, path_message_text};
/// Unique temporary names.
use super::random_id::random_uuid;
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// Running real Git.
use super::transaction_git::GitContext;
/// One added-path record.
use super::transaction_journal::AddedPath;
/// `HashMap` maps object IDs to bytes.
use std::collections::HashMap;
/// The trait that gives files `.read_to_end` and `.write_all`.
use std::io::{Read, Write};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

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
fn identity_of(metadata: &std::fs::Metadata) -> WorktreeFileIdentity {
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
fn link_count(metadata: &std::fs::Metadata) -> u64 {
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
fn directory_is_canonical(destination: &Path) -> std::io::Result<bool> {
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
fn open_nonblocking(destination: &Path) -> std::io::Result<std::fs::File> {
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
fn inspect_or_fail(
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

/// What:
///  Write `bytes` with `mode` into a fresh same-directory file.
/// Why:
///   The replacement is prepared completely before the destination is checked again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(prepared, bytes, { mode, flag: 'wx' }); await chmod(prepared, mode);
/// ```
fn write_prepared(prepared: &Path, bytes: &[u8], mode: u32) -> std::io::Result<()> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    let mut file: std::fs::File = options.open(prepared)?;
    file.write_all(bytes)?;
    drop(file);
    return super::private_storage::protect_path(prepared, mode);
}

/// What:
///  Replace one unchanged worktree file through a same-directory temporary file;
///  returns
///       whether it was installed without an observed conflict.
/// Why:
///   `replaceWorktreeFile`:
///  the destination is checked again after the replacement is ready.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function replaceWorktreeFile({ destination, bytes, mode, gitMode, original, identity }): Promise<boolean>;
/// ```
pub fn replace_worktree_file(
    destination: &Path,
    bytes: &[u8],
    git_mode: &str,
    original: &[u8],
    identity: &WorktreeFileIdentity,
) -> Result<bool, RecoveryError> {
    let unique: String = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(RecoveryError(error.0)),
    };
    let directory: &Path = destination.parent().unwrap_or(destination);
    let prepared: PathBuf = directory.join(format!(".cli-git-added-{unique}"));
    let outcome: Result<bool, RecoveryError> = replace_through(
        prepared.as_path(),
        destination,
        bytes,
        git_mode,
        original,
        identity,
    );
    match &outcome {
        Ok(true) => {}
        Ok(false) => {
            let _ = std::fs::remove_file(&prepared);
        }
        Err(error) => {
            warn(
                "replaceWorktreeFile",
                format!(
                    "worktree completion failed for {}: {error}",
                    destination.display()
                )
                .as_str(),
            );
            let _ = std::fs::remove_file(&prepared);
        }
    }
    return outcome;
}

/// What:
///  The replacement steps for a prepared name.
/// Why:
///   `replace_worktree_file` removes the prepared file on every outcome but success.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // body of replaceWorktreeFile inside its try
/// ```
fn replace_through(
    prepared: &Path,
    destination: &Path,
    bytes: &[u8],
    git_mode: &str,
    original: &[u8],
    identity: &WorktreeFileIdentity,
) -> Result<bool, RecoveryError> {
    if let Err(error) = write_prepared(prepared, bytes, identity.mode) {
        return Err(io_failure(
            "preparing the worktree replacement",
            prepared,
            &error,
        ));
    }
    let current: WorktreeFileState =
        match inspect_worktree_file(destination, git_mode, original, bytes) {
            Ok(state) => state,
            Err(error) => {
                return Err(io_failure(
                    "checking the worktree file",
                    destination,
                    &error,
                ));
            }
        };
    let WorktreeFileState::Original(again) = current else {
        return Ok(false);
    };
    if again != *identity {
        return Ok(false);
    }
    if let Err(error) = std::fs::rename(prepared, destination) {
        return Err(io_failure(
            "installing the worktree replacement",
            destination,
            &error,
        ));
    }
    return Ok(true);
}

/// What:
///  Bring every added path's worktree copy to its landed bytes.
/// Why:
///   `installAddedWorktreeFiles`:
///  a copy already holding the landed bytes is left alone,
///       which makes recovery idempotent;
///  a changed copy is kept and reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installAddedWorktreeFiles({ gitPath, cwd, repositoryRoot, records, objectDirectory }): Promise<AddedWorktreeInstallResult>;
/// ```
pub fn install_added_worktree_files(
    context: &GitContext,
    cwd: &Path,
    repository_root: &Path,
    records: &[AddedPath],
    object_directory: Option<&Path>,
) -> Result<InstallResult, RecoveryError> {
    // `mut` allows collecting outcomes per path.
    let mut result: InstallResult = InstallResult::default();
    if records.is_empty() {
        return Ok(result);
    }
    let mut oids: Vec<String> = Vec::with_capacity(records.len() * 2);
    for record in records {
        oids.push(record.original_oid.clone());
        oids.push(record.intended_oid.clone());
    }
    let blobs: HashMap<String, Vec<u8>> =
        match load_blob_batch(context, cwd, oids.as_slice(), object_directory) {
            Ok(found) => found,
            Err(message) => return Err(RecoveryError(message)),
        };
    for record in records {
        let destination: PathBuf = repository_root.join(os_from_path_bytes(record.path.as_slice()));
        let shown: String = path_message_text(record.path.as_slice());
        let (Some(intended), Some(original)) = (
            blobs.get(&record.intended_oid),
            blobs.get(&record.original_oid),
        ) else {
            return Err(RecoveryError(format!(
                "Git blob batch omitted a worktree completion object for {}.",
                shown
            )));
        };
        let state: WorktreeFileState = match inspect_worktree_file(
            destination.as_path(),
            record.git_mode.as_str(),
            original.as_slice(),
            intended.as_slice(),
        ) {
            Ok(found) => found,
            Err(error) => {
                return Err(io_failure(
                    "checking the worktree file",
                    destination.as_path(),
                    &error,
                ));
            }
        };
        let identity: WorktreeFileIdentity = match state {
            WorktreeFileState::Intended => continue,
            WorktreeFileState::Conflict => {
                warn(
                    "installAddedWorktreeFiles",
                    format!(
                        "Worktree copy of {path} changed or disappeared while cli-git completed the commit; \
                         the committed bytes remain in HEAD and your worktree state was kept. \
                         Compare it with HEAD (git diff HEAD -- {path}).",
                        path = shown
                    )
                    .as_str(),
                );
                result.conflicted.push(record.path.clone());
                continue;
            }
            WorktreeFileState::Original(found) => found,
        };
        if !replace_worktree_file(
            destination.as_path(),
            intended.as_slice(),
            record.git_mode.as_str(),
            original.as_slice(),
            &identity,
        )? {
            warn(
                "installAddedWorktreeFiles",
                format!(
                    "Worktree copy of {path} changed while its replacement was prepared; \
                     the committed bytes remain in HEAD and your edit was kept. \
                     Compare it with HEAD (git diff HEAD -- {path}).",
                    path = shown
                )
                .as_str(),
            );
            result.conflicted.push(record.path.clone());
            continue;
        }
        debug(
            "installAddedWorktreeFiles",
            format!("rewrote {shown} with its landed bytes").as_str(),
        );
        result.rewritten.push(record.path.clone());
    }
    return Ok(result);
}

/// Worktree completion controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_completion_tests.rs"]
mod tests;
