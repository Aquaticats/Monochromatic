//! What:
//!  Writing the files the word-level second pass merged after a forwarded command:
//! the stage that marks a path resolved,
//! the write to the worktree,
//! and the records that put the conflict back when that write fails.
//! Why:
//!  `merge_second_pass_worktree.rs` runs the pass over a stopped merge and stays under the package's
//!  file-size limit;
//! what it does with the files it merged needs nothing from it,
//! so it lives in a file of its own that the pass imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await install(runner, merged, oids); // update-index, then checkout-index, or the stages back
//! ```

use crate::merge_second_pass::{Conflict, GitReply, GitRunner, Merged, resolved_record};

/// The stages in the order `update-index --index-info` takes them back.
const STAGE_NUMBERS: [u8; 3] = [1, 2, 3];

/// The records that put a conflict's stages back into the index:
/// remove the path,
/// then each stage it had.
pub(super) fn restore_records(conflict: &Conflict) -> Vec<u8> {
    let mut width: usize = 0;
    if let Some(stage) = [&conflict.base, &conflict.ours, &conflict.theirs].into_iter().flatten().next() {
        width = stage.oid.len();
    }
    let mut records: Vec<u8> = format!("0 {}\t", "0".repeat(width)).into_bytes();
    records.extend_from_slice(conflict.path.as_slice());
    records.push(0);
    for (number, side) in STAGE_NUMBERS.iter().zip([&conflict.base, &conflict.ours, &conflict.theirs]) {
        if let Some(stage) = side {
            records.extend_from_slice(format!("{:06o} {} {number}\t", stage.mode, stage.oid).as_bytes());
            records.extend_from_slice(conflict.path.as_slice());
            records.push(0);
        }
    }
    return records;
}

/// The reply of a start that must have exited with status 0.
fn succeeded(started: Result<GitReply, String>, what: &str) -> Result<(), String> {
    let reply: GitReply = started?;
    if reply.code == Some(0) {
        return Ok(());
    }
    return Err(format!("{what} failed: {}", String::from_utf8_lossy(&reply.stderr).trim()));
}

/// What:
///  Stage the merged files as resolved and write them to the worktree;
/// put the conflicts back if the second step fails.
/// Why:
///   Staging first means a failed write leaves an index that says resolved over a file with markers,
/// so the stages are
///  restored then and the person finds the conflict exactly as Git left it.
/// The checkout applies the repository's own
///  filters (line endings),
/// which a plain file write would skip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function install(runner, merged: Merged[], oids: string[]): Promise<void>;
/// ```
pub(super) async fn install(runner: &dyn GitRunner, merged: &[Merged], oids: &[String]) -> Result<(), String> {
    let mut staging: Vec<u8> = Vec::new();
    let mut paths: Vec<u8> = Vec::new();
    for (file, oid) in merged.iter().zip(oids.iter()) {
        staging.extend_from_slice(resolved_record(file.mode, oid.as_str(), file.path.as_slice()).as_slice());
        paths.extend_from_slice(file.path.as_slice());
        paths.push(0);
    }
    succeeded(runner.run(&["update-index", "-z", "--index-info"], Some(staging.as_slice()), None).await, "git update-index")?;
    let written: Result<(), String> = succeeded(runner.run(&["checkout-index", "--force", "-z", "--stdin"], Some(paths.as_slice()), None).await, "git checkout-index");
    if let Err(reason) = written {
        let mut restore: Vec<u8> = Vec::new();
        for file in merged {
            restore.extend_from_slice(restore_records(&file.stages).as_slice());
        }
        if let Err(again) = succeeded(runner.run(&["update-index", "-z", "--index-info"], Some(restore.as_slice()), None).await, "git update-index") {
            return Err(format!("{reason}; putting the conflicts back also failed: {again}"));
        }
        return Err(reason);
    }
    return Ok(());
}
