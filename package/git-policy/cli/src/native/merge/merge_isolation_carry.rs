//! What:
//!  Reading the person's own work in the paths a merge changes:
//! what stands at one worktree path,
//! the bytes and the mode Git records for it,
//! the permission bits a person set on it,
//! and one capture of every layer of every such path.
//! Why:
//!  A merge replaces exactly these paths,
//! so their bytes must exist somewhere recovery can find them before the first one changes
//! (`package/git-policy/cli/SPEC.md`,
//! "Merges and commits never fail";
//! owner's rule "never discard user data").
//!  A path whose worktree entry is a directory,
//! or whose index entry is unmerged,
//! is not one edit anybody can merge,
//!  so the capture refuses it and the command falls back to native Git's own answer.
//! The objects the capture writes,
//! and the ref that keeps them from `git gc`,
//! are `merge_isolation_carry_objects.rs` and `merge_isolation_carry_ref.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const capture = captureWork({ context, root, paths }); // then keepCapture() on a ref of its own
//! ```

/// Real Git,
/// on the runtime this process may already have entered.
use crate::lazy_runtime::run_on_runtime;
/// One object from the bytes of a path,
/// and the index's own entries.
use crate::merge_isolation_carry_objects::{
    Entry,
    captured_index_of,
    entry_at,
    object_for_awaiting,
    staged_entries_awaiting,
};
/// The captured work.
use crate::merge_isolation_carry_parts::{
    Capture,
    CapturedFile,
    CapturedIndex,
    CapturedPath,
};
/// The worktree path of one path Git names.
use crate::merge_isolation_changes::worktree_path;
/// Real Git.
use crate::transaction_git::GitContext;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What stands at one worktree path,
/// as a merge reads it,
/// in the file that reads it.
pub(crate) use crate::merge_isolation_worktree_entry::{
    WorktreeEntry,
    not_one_edit_a_merge_can_carry,
    worktree_entry_at,
};

/// The Git mode a symbolic link is recorded with,
/// which a capture compares a path's mode with.
use crate::merge_isolation_carry_place_write::SYMBOLIC_LINK_MODE;

/// What:
///  The bytes the worktree holds at one path:
/// a file's content,
/// or a link's target text,
/// with the mode Git records for it and every permission bit a person set on it.
/// Why:
///   These are the bytes written back,
/// so they are read exactly and never through a filter,
/// and the permission bits are read beside them because Git records the
///  executable bit alone:
/// a capture that kept only that bit gave a `chmod 0600` file back as `0644`
///  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
/// its second finding).
///  A name that holds a named pipe,
/// a socket,
/// a device or a directory holds no bytes a merge can carry,
/// so a capture refuses it rather than holding part of what stands there,
///  and opening a named pipe for reading waits for a writer that may never come,
/// so everything but a plain file is refused from its type,
///  and the bytes of a plain file are read from a handle that can only be a plain file
/// (`crate::plain_file`,
///  owner's rule "nothing should ever wait forever or indefinitely").
///  A link carries no permission bits of a person's,
/// so its own are not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bytesAt(location: string): { bytes: Uint8Array; mode: string; permissions?: number } | null;
/// ```
pub(crate) fn bytes_at(
    location: &Path
) -> Result<Option<(Vec<u8>, &'static str, Option<u32>)>, String> {
    return match worktree_entry_at(location)? {
        WorktreeEntry::Holds {
            bytes,
            mode,
            permissions,
        } => Ok(Some((bytes, mode, permissions))),
        WorktreeEntry::Absent => Ok(None),
        WorktreeEntry::NoBytes(kind) => Err(not_one_edit_a_merge_can_carry(location, &kind)),
    };
}

/// What:
///  The person's own work in the paths the merge changes,
/// read once.
/// Why:
///   A path the merge changes and the person has not touched is no work to carry,
/// so it is left out and the merge
///  takes it as it always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureWork({ context, root, paths }): Promise<Capture>;
/// ```
pub fn capture_work(
    context: &GitContext,
    root: &Path,
    paths: &[Vec<u8>],
) -> Result<Capture, String> {
    return match run_on_runtime(capture_work_awaiting(context, root, paths)) {
        Ok(found) => found,
        Err(refusal) => Err(format!("{refusal}")),
    };
}

/// What:
///  The same capture of the person's work,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Every byte is written as an object through Git,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureWork({ context, root, paths }): Promise<Capture>;
/// ```
pub async fn capture_work_awaiting(
    context: &GitContext,
    root: &Path,
    paths: &[Vec<u8>],
) -> Result<Capture, String> {
    if paths.is_empty() {
        return Err(String::from("no path holds work to carry"));
    }
    let staged: Vec<Entry> = staged_entries_awaiting(context, root).await?;
    let mut captured: Vec<CapturedPath> = Vec::new();
    for path in paths {
        let location = worktree_path(root, path.as_slice());
        let worktree: Option<CapturedFile> = match bytes_at(location.as_path())? {
            Some((bytes, mode, permissions)) => {
                let raw: String =
                    object_for_awaiting(context, root, path.as_slice(), bytes.as_slice(), false)
                        .await?;
                let blob: String = if mode == SYMBOLIC_LINK_MODE {
                    raw.clone()
                } else {
                    object_for_awaiting(context, root, path.as_slice(), bytes.as_slice(), true)
                        .await?
                };
                Some(CapturedFile {
                    mode: String::from(mode),
                    raw,
                    blob,
                    permissions,
                })
            }
            None => None,
        };
        let index: Option<CapturedIndex> =
            entry_at(staged.as_slice(), path.as_slice()).map(captured_index_of);
        // Both layers absent is the person's own deletion of a path the merge changes, which is work to carry as much as an
        //  edit is: the merge would put a file back they removed.
        captured.push(CapturedPath {
            path: path.clone(),
            worktree,
            index,
        });
    }
    if captured.is_empty() {
        return Err(String::from("no path holds work to carry"));
    }
    return Ok(Capture {
        commit: String::new(),
        reference: None,
        paths: captured,
    });
}

/// Reading what the person's work is,
/// and keeping it outside the checkout.
#[cfg(test)]
#[path = "merge_isolation_carry_tests.rs"]
mod tests;
