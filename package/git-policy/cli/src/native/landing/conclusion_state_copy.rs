//! What:
//!  The copy the service keeps of the conclusion state a landed commit removes,
//! made when it writes the batch's intent,
//! and
//!  thrown away with the batch.
//! Why:
//!   A merge,
//! cherry-pick or revert conclusion removes the state files the client's own scratch directory
//!  holds copies of,
//! and each entry goes only while it still holds the bytes copied at preparation,
//! so a later command's
//!  merge state is never removed.
//! That comparison cannot be made once the client is gone:
//! the service removes its scratch
//!  directory when its watch connection closes,
//! and a cleanup that failed once then removed nothing and answered success,
//!  so the worktree kept `MERGE_HEAD` and the next commit concluded the merge again
//! (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
//! finding 4).
//! The service
//!  knows the state before the client can disappear,
//! so it copies what the comparison reads into the batch's journal,
//!  and the comparison is made against that copy when the client's directory is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copy = await keepConclusionState(cleanup, journal, seq); await dropConclusionState(journal, seq);
//! ```

use crate::conclusion_cleanup::{CONCLUSION_COPY_DIRECTORY, CONCLUSION_STATE_FILES};
use crate::private_storage::{create_private_directory_all, remove_tree};
use crate::service_journal::batch_path;
use crate::service_protocol::Cleanup;
use std::path::{Path, PathBuf};

/// The extension of the folder that holds one batch's copy.
pub const COPY_EXTENSION: &str = "cleanup";

/// The folder of the shadow's state files inside a copy.
const SHADOW_FOLDER: &str = "shadow";

/// The copy folder of a batch.
pub fn path_of(journal: &Path, seq: u64) -> PathBuf {
    return batch_path(journal, seq, COPY_EXTENSION);
}

/// What:
///  Copies every file of one directory tree into another,
/// keeping the names of the files and nothing else.
/// Why:
///   The conclusion copies are a few small text files and the sequencer's directory;
/// what a copy holds is
///  what the comparison reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyTree(from, to)
/// ```
fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    create_private_directory_all(to)?;
    for entry in std::fs::read_dir(from)?.flatten() {
        let source: PathBuf = entry.path();
        let target: PathBuf = to.join(entry.file_name());
        let held = entry.file_type()?;
        if held.is_dir() {
            copy_tree(source.as_path(), target.as_path())?;
            continue;
        }
        if held.is_file() {
            let _copied = std::fs::copy(source.as_path(), target.as_path())?;
        }
    }
    return Ok(());
}

/// What:
///  Keeps what the cleanup compares,
/// in the batch's journal,
/// before the branch moves.
/// Why:
///   The client's scratch directory is the service's to remove when that client is gone,
/// so a copy is the only thing a
///  later recovery can compare the state files with.
/// A cleanup without a scratch directory,
/// and a copy that cannot be made,
/// are
///  both nothing the landing may fail on:
/// the commit itself is not in question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepConclusionState(cleanup, journal, seq) // the folder, or nothing
/// ```
pub fn keep(cleanup: &Cleanup, journal: &Path, seq: u64) -> Option<PathBuf> {
    if !cleanup.scratch.exists() {
        return None;
    }
    let kept: PathBuf = path_of(journal, seq);
    let copies: PathBuf = cleanup.scratch.join(CONCLUSION_COPY_DIRECTORY);
    if copies.exists()
        && let Err(error) = copy_tree(copies.as_path(), kept.join(CONCLUSION_COPY_DIRECTORY).as_path())
    {
        crate::service_error!("batch {seq}: the conclusion state it must remove cannot be copied into {} ({error})", kept.display());
        return None;
    }
    let shadow: PathBuf = cleanup.scratch.join(SHADOW_FOLDER);
    if let Err(error) = create_private_directory_all(kept.join(SHADOW_FOLDER).as_path()) {
        crate::service_error!("batch {seq}: the conclusion state it must remove cannot be copied into {} ({error})", kept.display());
        return None;
    }
    for name in CONCLUSION_STATE_FILES {
        let source: PathBuf = shadow.join(name);
        if !source.exists() {
            continue;
        }
        if let Err(error) = std::fs::copy(source.as_path(), kept.join(SHADOW_FOLDER).join(name)) {
            crate::service_error!("batch {seq}: the conclusion state {name} cannot be copied into {} ({error})", kept.display());
            return None;
        }
    }
    return Some(kept);
}

/// Removes one batch's copy;
/// a copy that cannot be removed stays where it is.
pub fn remove(journal: &Path, seq: u64) {
    let _removed = remove_tree(path_of(journal, seq).as_path());
}
