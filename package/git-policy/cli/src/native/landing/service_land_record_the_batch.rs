//! What:
//!  The record a landing puts in the journal before it moves a branch:
//!  what the batch is,
//!  where its branch was and goes,
//!  the commits it holds,
//!  and the paths it changes,
//!  with the copy of the conclusion state its cleanup compares
//!  and the reflog row its branch move writes.
//! Why:
//!  A kill between the branch move and the index install leaves this record,
//!  and a restart reads it to decide the batch from the branch
//!  (`service_land_finish.rs`).
//!  The batch's own record,
//! its conclusion copy and its row are one value and one write,
//!  so they stand in a file of their own and the landing's body stays short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { intent, reflog } = recordTheBatch({ context, timing, seq, first, batch, commits, old, new, unborn });
//! ```

use crate::service_built_checks::commit_content;
use crate::service_journal::{Intent, write_intent};
use crate::service_land_context::LandContext;
use crate::service_land_follow::storage_failure;
use crate::service_land_git_steps_and_import_stream::ids_of;
use crate::service_land_reflog::{Committed, conclusion_copy, kind, message};
use crate::service_protocol::{CommitRequest, FailureCode, is_zero_object};
use crate::service_timing::LandTiming;
use std::path::PathBuf;
use std::time::Instant;

/// What one landing records about its batch.
pub(crate) struct TheBatch<'a> {
    /// The journal,
    /// the Git and the lock ceiling of the landing.
    pub context: &'a LandContext,
    /// The landing's clock,
    /// which states what each stretch of the work cost.
    pub timing: &'a mut LandTiming,
    /// The batch number.
    pub seq: u64,
    /// The first request of the batch,
    /// which names the worktree,
    /// the branch and the index.
    pub first: &'a CommitRequest,
    /// The requests that land,
    /// in the order their commits are made.
    pub batch: &'a [&'a CommitRequest],
    /// The commits the batch holds,
    /// in the same order.
    pub commits: &'a [String],
    /// The value the branch named when the batch was built.
    pub old: &'a str,
    /// The commit the branch is about to name.
    pub new: &'a str,
    /// Whether the branch did not exist yet,
    /// which the row and a recovery both read.
    pub unborn: bool,
}

/// What:
///  Writes the batch's record and answers it with the reflog row of its branch move.
/// Why:
///  The row is written by the branch move and the kind it names depends on what the batch concluded,
///  so both are made here,
///  and the record is on the disk before the move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reflog = rowOf(theBatch); const intent = theRecord(theBatch, reflog); writeIntent(intent); return { intent, reflog };
/// ```
///
/// # Errors
///
/// The storage failure of the journal the record could not be written in.
pub(crate) fn of(the: TheBatch<'_>) -> Result<(Intent, std::ffi::OsString), FailureCode> {
    let TheBatch { context, timing, seq, first, batch, commits, old, new, unborn } = the;
    // What the reflog row says: the kind of commit and the first line of its message, as native Git words both
    // (`builtin/commit.c`, `update_head_with_reflog` in `sequencer.c`). A landing answers a batch with one branch move, so
    // the row names the commit that landed last, which is the value the branch now holds.
    let last: &CommitRequest = batch.last().copied().unwrap_or(first);
    let content: Option<crate::service_built_checks::CommitContent> = commit_content(first.common_dir.as_path(), new);
    let parents: usize = match &content {
        Some(read) => read.parents.len(),
        None => 0,
    };
    let copy: Option<PathBuf> = conclusion_copy(context.journal.as_path(), seq, first.cleanup.is_some());
    let initial: bool = unborn || is_zero_object(old);
    let committed: Committed = kind(last.amend, initial, parents, copy.as_deref());
    let subject: Vec<u8> = match content {
        Some(read) => read.message,
        None => last.message.clone().into_bytes(),
    };
    let reflog: std::ffi::OsString = message(committed, subject.as_slice());
    // A commit Git built carries no list of changes: its index entries are the paths that differ from the old branch.
    let derived: bool = first.commit.is_some();
    let paths: Vec<Vec<u8>> = if derived { Vec::new() } else { changed_paths(batch) };
    let intent: Intent = Intent {
        seq,
        repository: first.repository.clone(),
        common_dir: first.common_dir.clone(),
        reference: first.reference.clone(),
        old: old.to_string(),
        new: new.to_string(),
        ids: ids_of(batch),
        commits: commits.to_vec(),
        paths,
        derived,
        cleanup: first.cleanup.clone(),
        index: first.index.clone(),
        git_dir: first.git_dir.clone(),
    };
    // What the conclusion's cleanup compares is the client's to lose: a copy is kept in the journal before the branch moves,
    // so a cleanup that fails once and is finished by a restart still removes the state (`conclusion_state_copy.rs`).
    let conclusion_began: Instant = Instant::now();
    crate::conclusion_state_copy::keep_of_intent(intent.cleanup.as_ref(), context.journal.as_path(), seq);
    timing.reads_split.conclusion += conclusion_began.elapsed();
    let journal_began: Instant = Instant::now();
    let written = write_intent(context.journal.as_path(), &intent);
    timing.journal += journal_began.elapsed();
    if let Err(error) = written {
        return Err(storage_failure(&error));
    }
    return Ok((intent, reflog));
}

/// What:
///  The paths the batch's own commits change,
///  as bytes,
///  in the order they are named.
/// Why:
///  A name is bytes,
///  and this list is what a commit prepared meanwhile leaves out of itself
///  (`owed_index_repair.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// batch.flatMap((request) => request.changes.map((change) => pathBytes(change.path)))
/// ```
fn changed_paths(batch: &[&CommitRequest]) -> Vec<Vec<u8>> {
    let mut named: Vec<Vec<u8>> = Vec::new();
    for request in batch {
        for change in &request.changes {
            named.push(crate::path_bytes::path_bytes_of(change.path.as_path()).to_vec());
        }
    }
    return named;
}
