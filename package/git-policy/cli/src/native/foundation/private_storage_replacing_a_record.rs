//! What:
//!  Publishing a record over the one the same name already holds,
//! in one rename,
//! with or without making it durable.
//! Why:
//!  A record this program rewrites is read by a later run as a whole,
//! and a truncating write leaves it empty
//!  for a kill that lands between the truncation and the bytes;
//! staging the bytes beside the name and publishing them by one rename
//!  is what makes that impossible,
//! and whether the bytes are flushed is the caller's decision,
//! because a record written on a
//!  command's own path may not make that command wait for the disk.
//! Kept in a file of its own because the primitives file reached the repository's line limit
//!  (`rust/max-lines`);
//! it is a module of `private_storage`,
//! which re-exports both writers,
//! so no caller changed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export async function replacePrivateFile({ path, bytes, sync }) { const staged = stage(path); await write(staged, bytes); if (sync) await syncIt(staged); await rename(staged, path); if (sync) await syncDirectory(dirname(path)); }
//! ```

use super::{staging_beside, sync_directory, sync_when_durable};
use std::io::Write;
use std::path::Path;

/// The operating system's answer from a publish that kept its staged file,
/// which is the failure the caller reports.
fn without_the_staged_file(error: tempfile::PersistError) -> std::io::Error {
    return error.error;
}

/// What:
///  Publish `bytes` at `path` in one rename,
///  replacing whatever the name held.
/// Why:
///   A record this program rewrites (the migration's list of files this user may not read) is read
/// by a later run as a whole;
/// a truncating write leaves it empty for a kill that lands between the truncation and the
///  bytes,
/// and the next run reads an empty record as nothing to report.
/// The bytes are staged beside the
///  name and made durable before the rename,
/// so a kill leaves either the record that was there or the complete
///  new one,
/// and the staging file is created exclusively with mode `0600` and never follows a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function replacePrivateFile({ path, bytes }): Promise<void>;
/// ```
pub fn replace_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut staged: tempfile::NamedTempFile = staging_beside(path)?;
    staged.as_file_mut().write_all(bytes)?;
    sync_when_durable(staged.as_file(), path, true)?;
    staged.persist(path).map_err(without_the_staged_file)?;
    return match path.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => sync_directory(directory),
        _ => sync_directory(Path::new(".")),
    };
}

/// What:
///  Publish `bytes` at `path` in one rename that replaces whatever the name held,
/// without flushing the record or its directory.
/// Why:
///   A record a command rewrites on a command's own path (the verdict a commit leaves in its held
///  push record) may not make that command wait for the disk:
/// native Git's commit makes no sync,
/// so a commit's answer waits on none of the wrapper's own
///  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
/// The bytes are staged beside the name and
///  published whole,
/// so a kill leaves either the record that was there or the complete new one,
/// and a machine crash
///  leaves the record as it was,
/// which every reader of a rewritten record treats as the state before the rewrite
///  (`work_item_holding.rs`,
/// `held_push_holder_and_verdict.rs`).
///   A record whose meaning a machine crash must keep stays on `replace_private_file`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function replacePrivateFile({ path, bytes, sync: false }): Promise<void>;
/// ```
pub fn replace_private_file_unsynced(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut staged: tempfile::NamedTempFile = staging_beside(path)?;
    staged.as_file_mut().write_all(bytes)?;
    staged.persist(path).map_err(without_the_staged_file)?;
    return Ok(());
}
