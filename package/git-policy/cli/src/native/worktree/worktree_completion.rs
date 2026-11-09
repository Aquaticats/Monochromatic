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
//!      landed,
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await installAddedWorktreeFiles({ gitPath, cwd, repositoryRoot, records, objectDirectory });
//! ```

/// The blob batch.
pub use crate::worktree_completion_file_inspection::{InstallResult, WorktreeFileIdentity, WorktreeFileState, inspect_worktree_file, is_conflict_error};
use crate::blob_batch::load_blob_batch;
/// Diagnostics.
use crate::diagnostic_log::{debug, warn};
/// Exact path bytes as file names,
/// and as text for messages.
use crate::path_bytes::{os_from_path_bytes, path_message_text};
/// Unique temporary names.
use crate::random_id::random_uuid;
/// The fail-closed recovery failure.
use crate::recovery_error::{RecoveryError, io_failure};
/// Running real Git.
use crate::transaction_git::GitContext;
/// One added-path record.
use crate::transaction_journal::AddedPath;
/// `HashMap` maps object IDs to bytes.
use std::collections::HashMap;
/// The trait that gives files `.read_to_end` and `.write_all`.
use std::io::Write;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

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
    return crate::private_storage::protect_path(prepared, mode);
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
                        "Worktree copy of {path} changed or disappeared while monochromatic-git-policy-cli completed the commit; \
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
