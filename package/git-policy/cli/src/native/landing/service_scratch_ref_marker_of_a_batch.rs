//! What:
//!  The record a landing writes beside its batch before it puts the batch's commits on a reference,
//! naming the worktree whose scratch reference the
//!  landing is about to create,
//! and the sweep a restart runs over the records it finds.
//! Why:
//!  A landing protects the objects of its batch with `refs/worktree/cli-git-service/<seq>`,
//! the reference the import moves and the one a
//!  commit Git built is kept on,
//! and the branch write
//!  removes that reference in the same transaction that moves the branch.
//! A service killed between the import of the
//!  objects and the write of its intent leaves the reference behind with nothing that names it:
//! the intent is what a restart
//!  reads to decide the batch from the branch,
//! and this batch has none.
//! A service instance is a hash of the
//!  worktree's own Git directory,
//! so no path can be turned back out of it,
//! and the restart cannot look for that reference on its own.
//! The landing knows the worktree,
//! so it leaves this record before the reference can exist,
//!  and the restart's recovery keeps the commit the reference holds on `refs/cli-git/kept/<seq>` and only then removes the
//!  reference (`service_scratch_ref_of_a_dropped_batch.rs`).
//! A record that cannot be read,
//! a keep Git refused and a kept
//!  reference that names another commit all leave both the reference and this record where they are:
//! nothing is removed that cannot
//!  be proven safe,
//! which is the owner's rule "never discard user data".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await record(worktree, seq); /* the import moves the scratch ref */ await forget(seq); // and, at a restart: for (const record of records) await clearAfterDroppedBatch(record.repository, record.seq)
//! ```

use crate::service_instance::instance_of;
use crate::service_journal::{OPEN_EXTENSION, SCRATCH_EXTENSION, batch_path};
use crate::service_land_context::LandContext;
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::service_scratch_ref_of_a_dropped_batch::{ScratchRefRemoval, clear_after_a_failed_landing, clear_after_dropped_batch};
use crate::path_bytes_in_records as path_bytes;
use crate::transaction_git::GitContext;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// The record format version.
const VERSION: u64 = 1;


/// What:
///  The path of the record of batch `seq`'s scratch reference.
/// Why:
///   The batch number is the name's stem,
/// so a record is found from the batch and a batch from the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(journal, `${seq}.scratch`)
/// ```
pub(crate) fn record_path(journal: &Path, seq: u64) -> PathBuf {
    return batch_path(journal, seq, SCRATCH_EXTENSION);
}

/// What:
///  The one line a record holds:
/// the format version,
/// the worktree root,
/// and the worktree's own Git directory.
/// Why:
///  `clear_after_dropped_batch` needs the repository to ask Git about the reference,
/// and the Git directory is what
///  the service instance is a hash of,
/// so a journal that several worktrees' services share can be swept for one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ v: 1, repository, gitDir })
/// ```
fn encode(repository: &Path, git_dir: &Path) -> String {
    let mut fields: Map<String, Value> = Map::new();
    fields.insert(String::from("v"), Value::from(VERSION));
    // Each path goes in as plain text and as its bytes, so the sweep finds the worktree the reference is in.
    path_bytes::write_field(&mut fields, "repository", repository);
    path_bytes::write_field(&mut fields, "gitDir", git_dir);
    return format!("{}\n", Value::Object(fields));
}

/// What:
///  The worktree a record names,
/// or nothing when it names none.
/// Why:
///   A record of another version,
/// one that is not JSON or one without both paths is no proof of where a
///  reference is,
/// and the sweep that reads it removes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = (text) => { const value = JSON.parse(text); return value.v === 1 ? { repository: value.repository, gitDir: value.gitDir } : undefined }
/// ```
fn decode(text: &str) -> Option<(PathBuf, PathBuf)> {
    let value: Value = serde_json::from_str(text).ok()?;
    if value.get("v").and_then(Value::as_u64) != Some(VERSION) {
        return None;
    }
    return Some((path_bytes::read_field(&value, "repository").ok()?, path_bytes::read_field(&value, "gitDir").ok()?));
}

/// What:
///  Leaves the record of batch `seq`'s scratch reference,
/// naming the worktree it belongs to.
/// Why:
///   The batch is about to have its commits on `refs/worktree/cli-git-service/<seq>`,
/// and a kill after that instant leaves the reference
///  with nothing else that names the worktree it is in.
/// The record is published whole and never flushed:
/// a kill leaves it
///  whole or absent,
/// and a record that a power loss loses leaves the reference where it was,
/// which is the answer for anything
///  that cannot be proven safe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePrivateFileUnsynced(recordPath(journal, seq), encode(repository, gitDir)).catch(warn)
/// ```
pub(crate) fn record_scratch_ref(journal: &Path, seq: u64, repository: &Path, git_dir: &Path) {
    let path: PathBuf = record_path(journal, seq);
    match crate::private_storage::write_private_file_unsynced(path.as_path(), encode(repository, git_dir).as_bytes()) {
        Ok(()) => {}
        // A batch the branch moved under is built again under the same number, and the record of its first attempt already names
        //  the same worktree: nothing is written a second time.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => crate::service_warn!(
            "batch {seq}: the record of its scratch reference could not be written to {} ({error}), so a kill before its intent would leave that reference with nothing to find it by",
            path.display()
        ),
    }
}

/// What:
///  Removes the record of batch `seq`'s scratch reference,
/// when it is there.
/// Why:
///   A landing that ended settles its own reference:
/// the branch write removed it,
/// or the failed landing kept its commit and
///  removed it.
/// The record is only for a landing that ends without the chance to say what became of the reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(recordPath(journal, seq), { force: true })
/// ```
pub(crate) fn forget_scratch_ref(journal: &Path, seq: u64) {
    let _removed = std::fs::remove_file(record_path(journal, seq));
}

/// What:
///  Ends the record of batch `seq`'s scratch reference when the landing ends.
/// Why:
///   A landing that landed its commits had the reference removed by the same transaction that moved the branch,
/// and a
///  landing that failed keeps the commit and removes the reference itself
/// (`service_scratch_ref_of_a_dropped_batch.rs`),
/// so
///  the record goes with it.
/// A failed landing that had to leave the reference where it is keeps the record,
/// because the next
///  restart is what tries again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!failure || await clearAfterAFailedLanding(requests, seq) === 'removed') await forget(seq)
/// ```
pub(crate) fn end_scratch_ref_record(context: &LandContext, requests: &[CommitRequest], seq: u64, failure: Option<&FailureCode>) {
    if failure.is_some() && clear_after_a_failed_landing(context, requests, seq) == ScratchRefRemoval::Left {
        return;
    }
    forget_scratch_ref(context.journal.as_path(), seq);
}

/// What:
///  The records of scratch references in `journal`,
/// each with the batch it is named for.
/// Why:
///   The sweep walks these and nothing else;
/// a journal also holds intents,
/// claims and the marks of an import.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(journal)).filter((name) => name.endsWith('.scratch')).map((name) => [Number(name.split('.')[0]), join(journal, name)])
/// ```
fn records(journal: &Path) -> Vec<(u64, PathBuf)> {
    let Ok(listing) = std::fs::read_dir(journal) else {
        return Vec::new();
    };
    let mut found: Vec<(u64, PathBuf)> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(SCRATCH_EXTENSION) {
            continue;
        }
        let Some(seq) = path.file_stem().and_then(std::ffi::OsStr::to_str).and_then(batch_number_of_name) else {
            continue;
        };
        found.push((seq, path));
    }
    found.sort_by_key(batch_number_of_record);
    return found;
}

/// The batch number a record's name stem spells,
/// or nothing when it spells none.
fn batch_number_of_name(stem: &str) -> Option<u64> {
    return stem.parse::<u64>().ok();
}

/// The batch number of a record,
/// for the sort that puts the records in batch order.
fn batch_number_of_record(record: &(u64, PathBuf)) -> u64 {
    return record.0;
}

/// What:
///  Clears the scratch reference of every batch a record in `journal` names and no intent does,
/// keeping the commit it holds first,
/// then removes the record.
/// Why:
///   This is the one thing that can reach a reference whose batch was killed between its import and its intent:
///  the record names the worktree,
/// which the service instance's hash cannot be turned back into.
/// A batch with an intent is left to
///  the intent,
/// which names its worktree too and is settled from the branch.
/// A record that cannot be read,
/// and a
///  reference whose commit could not be proven safe,
/// both stay,
/// with the record beside them,
/// so a later restart tries again
///  and no commit is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [seq, path] of records(journal)) { if (openIntent(seq) || !mine(gitDir)) continue; const named = decode(await readText(path)); if (!named) continue; if (await clearAfterDroppedBatch(named.repository, seq) === 'removed') await rm(path) }
/// ```
pub(crate) fn clear_recorded_scratch_refs(context: &LandContext, instance: Option<&str>) {
    for (seq, path) in records(context.journal.as_path()) {
        // An intent names the batch's own worktree and its recovery settles the reference; a record beside it is that
        //  recovery's to remove.
        if batch_path(context.journal.as_path(), seq, OPEN_EXTENSION).exists() {
            continue;
        }
        let read: Option<String> = std::fs::read_to_string(path.as_path()).ok();
        let Some((repository, git_dir)) = read.as_deref().and_then(decode) else {
            crate::service_warn!("batch {seq}: the record {} of its scratch reference cannot be read, so nothing is removed", path.display());
            continue;
        };
        // A journal several worktrees' services share is swept only for the instance named.
        if let Some(named) = instance
            && instance_of(git_dir.as_path()) != named
        {
            continue;
        }
        let git: GitContext = context.git.clone();
        if clear_after_dropped_batch(&git, repository.as_path(), seq) == ScratchRefRemoval::Removed {
            let _removed = std::fs::remove_file(path.as_path());
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_scratch_ref_marker_of_a_batch_tests.rs"]
mod tests;
