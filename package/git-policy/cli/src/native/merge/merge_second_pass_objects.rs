//! What:
//!  The objects the word-level second pass makes from merges:
//!  one merged file written as a blob,
//!  the index record that stages a blob at a path as resolved,
//!  and the tree the merge would have with those files in place.
//! Why:
//!  A merge computed from objects has no worktree to write into,
//!  so its result is carried by object names alone:
//!  the blob of each merged file,
//!  and a tree built in a scratch index from Git's own commands.
//!  `--no-filters` keeps the bytes exactly as merged;
//!  the checkout filters apply when a worktree file is made,
//!  never here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const oid = await writeBlob(runner, bytes);
//! const tree = await treeWithFiles(runner, tree, [{ path, mode, oid }], scratchIndex);
//! ```

use crate::merge_second_pass_git_runner::{
    GitReply,
    GitRunner,
};
use std::path::Path;
/// What:
///  Write one merged file as a blob and return its object name.
/// Why:
///   `--no-filters` keeps the bytes exactly as merged;
/// the checkout filters apply when a worktree file is made,
/// not here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeBlob(runner: GitRunner, bytes: Uint8Array): Promise<string>;
/// ```
pub async fn write_blob(
    runner: &dyn GitRunner,
    bytes: &[u8],
) -> Result<String, String> {
    let reply: GitReply = runner
        .run(
            &["hash-object", "-w", "--no-filters", "--stdin"],
            Some(bytes),
            None,
        )
        .await?;
    if reply.code != Some(0) {
        return Err(format!(
            "git hash-object failed: {}",
            String::from_utf8_lossy(&reply.stderr).trim()
        ));
    }
    return Ok(String::from_utf8_lossy(&reply.stdout).trim().to_owned());
}

/// The `update-index --index-info` record that puts a blob at a path as a resolved entry.
pub fn resolved_record(
    mode: u32,
    oid: &str,
    path: &[u8],
) -> Vec<u8> {
    let mut record: Vec<u8> = format!("{mode:06o} {oid}\t").into_bytes();
    record.extend_from_slice(path);
    record.push(0);
    return record;
}

/// What:
///  The tree that is `tree` with the named files replaced,
/// from a scratch index.
/// Why:
///   The tree `git merge-tree` wrote holds the conflicted file with markers;
/// replacing it by the merged blob makes the tree
///  of the merge as if Git had merged cleanly.
/// A scratch index is the way to edit a tree with Git's own commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function treeWithFiles(runner, tree: string, files: { path, mode, oid }[], scratchIndex: string): Promise<string>;
/// ```
pub async fn tree_with_files(
    runner: &dyn GitRunner,
    tree: &str,
    files: &[(&[u8], u32, &str)],
    scratch_index: &Path,
) -> Result<String, String> {
    let read: GitReply = runner
        .run(&["read-tree", tree], None, Some(scratch_index))
        .await?;
    if read.code != Some(0) {
        return Err(format!(
            "git read-tree failed: {}",
            String::from_utf8_lossy(&read.stderr).trim()
        ));
    }
    let mut records: Vec<u8> = Vec::new();
    for (path, mode, oid) in files {
        records.extend_from_slice(resolved_record(*mode, oid, path).as_slice());
    }
    let updated: GitReply = runner
        .run(
            &["update-index", "-z", "--index-info"],
            Some(records.as_slice()),
            Some(scratch_index),
        )
        .await?;
    if updated.code != Some(0) {
        return Err(format!(
            "git update-index failed: {}",
            String::from_utf8_lossy(&updated.stderr).trim()
        ));
    }
    let written: GitReply = runner
        .run(&["write-tree"], None, Some(scratch_index))
        .await?;
    if written.code != Some(0) {
        return Err(format!(
            "git write-tree failed: {}",
            String::from_utf8_lossy(&written.stderr).trim()
        ));
    }
    return Ok(String::from_utf8_lossy(&written.stdout).trim().to_owned());
}
