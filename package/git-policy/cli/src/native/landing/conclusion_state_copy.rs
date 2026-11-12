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
use crate::service_journal::{CONCLUSION_COPY_EXTENSION, batch_path};
use crate::service_protocol::Cleanup;
use std::path::{Path, PathBuf};

/// The extension of the folder that holds one batch's copy.
pub const COPY_EXTENSION: &str = CONCLUSION_COPY_EXTENSION;

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
    // The copy is built beside its name and renamed onto it, so a copy that failed half way never takes the name the cleanup
    //  reads, and a landing that could not keep everything is a landing that kept nothing.
    let kept: PathBuf = path_of(journal, seq);
    let staging: PathBuf = journal.join(format!("{seq}.{COPY_EXTENSION}.tmp"));
    let _stale: std::io::Result<()> = remove_tree(staging.as_path());
    if let Some(failed) = copy_into(cleanup, journal, seq, staging.as_path()) {
        let _gone: std::io::Result<()> = remove_tree(staging.as_path());
        return failed;
    }
    if let Err(error) = std::fs::rename(staging.as_path(), kept.as_path()) {
        crate::service_error!("batch {seq}: the copy of the conclusion state cannot be published as {} ({error})", kept.display());
        let _gone: std::io::Result<()> = remove_tree(staging.as_path());
        return None;
    }
    return Some(kept);
}

/// What:
///  Copies what the cleanup compares into `kept`,
/// answering the failure that stops it.
/// Why:
///   A copy that cannot be made says which part it lost,
/// and the caller publishes nothing of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyInto(cleanup, kept) // null when everything was copied
/// ```
fn copy_into(cleanup: &Cleanup, journal: &Path, seq: u64, kept: &Path) -> Option<Option<PathBuf>> {
    let _ = journal;
    let copies: PathBuf = cleanup.scratch.join(CONCLUSION_COPY_DIRECTORY);
    if copies.exists()
        && let Err(error) = copy_tree(copies.as_path(), kept.join(CONCLUSION_COPY_DIRECTORY).as_path())
    {
        crate::service_error!("batch {seq}: the conclusion state it must remove cannot be copied into {} ({error})", kept.display());
        return Some(None);
    }
    let shadow: PathBuf = cleanup.scratch.join(SHADOW_FOLDER);
    if let Err(error) = create_private_directory_all(kept.join(SHADOW_FOLDER).as_path()) {
        crate::service_error!("batch {seq}: the conclusion state it must remove cannot be copied into {} ({error})", kept.display());
        return Some(None);
    }
    for name in CONCLUSION_STATE_FILES {
        let source: PathBuf = shadow.join(name);
        if !source.exists() {
            continue;
        }
        if let Err(error) = std::fs::copy(source.as_path(), kept.join(SHADOW_FOLDER).join(name)) {
            crate::service_error!("batch {seq}: the conclusion state {name} cannot be copied into {} ({error})", kept.display());
            return Some(None);
        }
    }
    return None;
}

/// What:
///  Keeps the copy of a batch's conclusion state,
/// when the batch has one.
/// Why:
///   The landing calls this before the branch moves,
/// so no other caller restates the record it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepOf(cleanup, journal, seq)
/// ```
pub fn keep_of_intent(cleanup: Option<&Cleanup>, journal: &Path, seq: u64) -> Option<PathBuf> {
    let named: &Cleanup = cleanup?;
    return keep(named, journal, seq);
}

/// Removes one batch's copy;
/// a copy that cannot be removed stays where it is.
pub fn remove(journal: &Path, seq: u64) {
    let _removed = remove_tree(path_of(journal, seq).as_path());
}
