//! What:
//!  What a first preparation captured,
//!  kept for the retry of a commit the service sent back,
//!  and the records a replay speaks in.
//! Why:
//!   The bytes a commit lands are the bytes that were there when the person ran the command,
//!  so the change is captured once and carried,
//!  and a replay merges it onto the branch as it is now instead of reading the worktree.
//!  The stamp of the capture is kept for the same reason: a retry keeps the first capture's stamp,
//!  because the merged bytes are not a new capture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const captured = retryOf(ready);           // { base, unborn, tree, captured, landed, conclusion }
//! const again = retryOfAPreparedTree(tree);  // the same, over a tree this command built
//! ```

use crate::commit_transaction::{
    TransactionStop,
    failed,
};
use crate::merge_second_pass::Merged as MergedFile;
use crate::private_index_request::indexed_request;
use crate::service_messages::NOTHING_CHANGED;
use crate::service_prepare_ready_shapes::Ready;
use crate::service_protocol::LandedCapture;
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git_checked,
};
use std::path::Path;

/// What the first preparation captured,
/// kept for the retry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captured {
    /// The revision the replay merges from:
    /// the commit the change was prepared against,
    /// or the empty tree for a branch that did not exist yet,
    /// because such a commit's change is a whole root tree and there is no commit to merge from.
    pub base: String,
    /// Whether the branch did not exist when the change was captured,
    /// so no history stands between the base and the branch as it is now.
    pub unborn: bool,
    /// The tree of that base with the captured change applied.
    pub tree: String,
    /// When the worktree's bytes were captured (`CommitRequest::captured`);
    /// zero when unknown.
    /// A retry keeps the first
    ///  capture's stamp,
    /// because the merged bytes are not a new capture.
    pub captured: u64,
    /// The commits the service landed since,
    /// with the stamps of their captures,
    /// as its last answer told.
    pub landed: Vec<LandedCapture>,
    /// What the merge,
    /// cherry-pick or revert state named when the change was captured,
    /// so a replay concludes the same operation;
    /// nothing when the command concluded none.
    /// Why:
    ///   A conclusion's parents are the state's,
    /// and a replay over another state would make a commit the person never asked for.
    pub conclusion: Option<String>,
}

/// What:
///  The stamp of a capture finished now:
/// nanoseconds of `CLOCK_MONOTONIC`,
/// zero where the platform has none.
/// Why:
///   That clock is the host's,
/// one for every process,
/// and never steps back,
/// so a capture made later by another wrapper
///  process gets a larger stamp (`service_captures.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.hrtime.bigint()
/// ```
pub fn capture_stamp() -> u64 {
    #[cfg(unix)]
    {
        let mut now = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: `clock_gettime` writes one `timespec` through a valid pointer.
        let read: i32 = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &raw mut now) };
        if read != 0 {
            return 0;
        }
        let seconds: u64 = u64::try_from(now.tv_sec).unwrap_or(0);
        let nanoseconds: u64 = u64::try_from(now.tv_nsec).unwrap_or(0);
        return seconds
            .saturating_mul(1_000_000_000)
            .saturating_add(nanoseconds)
            .max(1);
    }
    #[cfg(not(unix))]
    {
        return 0;
    }
}

/// A retry:
/// the change the first preparation captured,
/// and which retry this is.
pub struct Retry<'a> {
    /// What the first preparation captured.
    pub captured: &'a Captured,
    /// Which retry this is,
    /// from 1,
    /// which names the retry's scratch files.
    pub attempt: u64,
}

/// What merging the captured change onto the branch came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Merged {
    /// The change applies;
    /// the merged tree exists in the repository.
    Tree(String),
    /// The change applies after the word-level second pass merged every file Git's line merge left in conflict;
    /// the tree
    ///  exists in the repository,
    /// and the files are listed to the person and in the journal.
    Combined {
        /// The merged tree.
        tree: String,
        /// The files the word pass merged,
        /// with their parts.
        files: Vec<MergedFile>,
    },
    /// The change overlaps what landed.
    Conflict(Vec<Vec<u8>>),
}

/// What the merge needs to know about where it runs.
pub struct MergeSite<'a> {
    /// How to run Git.
    pub context: &'a GitContext,
    /// The directory the commit was run from.
    pub cwd: &'a Path,
    /// The common Git directory,
    /// which holds the objects and which the object-only steps run against.
    pub common_dir: &'a Path,
    /// The object store receiving what the steps write.
    pub object_directory: &'a Path,
    /// The commit's private directory,
    /// where the subsumption keeps its scratch index.
    pub scratch: &'a Path,
}

/// What:
///  What a retry replays,
/// from the preparation that was sent back:
/// its tree on its base,
/// its capture stamp,
/// and the commits
///  the service says landed since.
/// Why:
///   Both the commit the service builds and the commit Git builds are replayed from the tree of their private index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryOf(context, cwd, ready, landed)
/// ```
pub fn retry_of(
    context: &GitContext,
    cwd: &Path,
    ready: &Ready,
    landed: Vec<LandedCapture>,
) -> Result<Captured, TransactionStop> {
    let tree: String = captured_tree(
        context,
        cwd,
        ready.inputs.commit_index.as_path(),
        ready.inputs.capture.layout.object_directory.as_path(),
    )?;
    return Ok(capture_of(ready, landed, tree));
}

/// What:
///  What a retry replays when the change is the tree a prepared commit already holds,
///  and not the private index.
/// Why:
///   A line whose own commit takes the index out of it (`--fixup=reword:<commit>`) did not capture the private index:
///  the change such a commit stands for is the branch's own tree,
///  and replaying that tree onto the branch as it is now lands an empty change there,
///  which is the commit the person asked for
///  (`service_commit.rs`,
///  the arm for a branch that moved).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// retryOfAPreparedTree(ready, landed, tree)
/// ```
pub fn retry_of_a_prepared_tree(
    ready: &Ready,
    landed: Vec<LandedCapture>,
    tree: String,
) -> Captured {
    return capture_of(ready, landed, tree);
}

/// What:
///  The capture of one retry:
///  its change on its base,
///  its capture stamp,
///  and the commits the service says landed since.
/// Why:
///   Both ways into a retry share every field but the tree,
///  and the base is decided once here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// captureOf(ready, landed, tree)
/// ```
fn capture_of(
    ready: &Ready,
    landed: Vec<LandedCapture>,
    tree: String,
) -> Captured {
    // A branch that did not exist yet named the all-zero object as its base, which no Git step reads as a tree; the empty tree
    //  is what that base holds, and the change captured against it is the whole root tree the person staged.
    let unborn: bool = matches!(
        ready.inputs.capture.base,
        crate::transaction_journal::Base::Unborn
    );
    let base: String = if unborn {
        ready.inputs.capture.layout.empty_tree_oid.clone()
    } else {
        ready.base.clone()
    };
    // The state the preparation saw, not a reading now: another command may have concluded the operation while this commit
    //  waited, and the retry must find the same one.
    return Captured {
        base,
        unborn,
        tree,
        captured: ready.captured_at,
        landed,
        conclusion: ready.conclusion_state.clone(),
    };
}

/// What:
///  The tree a prepared commit holds,
///  which is the change a line that takes the index out of its commit captured.
/// Why:
///   Such a commit's tree is the one Git wrote into it,
///  and it is the record of what the line stands for,
///  so a replay reads it from the object rather than from the private index that Git ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await treeOfCommit(context, cwd, commit)
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` when the commit cannot be read.
pub fn tree_of_commit(
    context: &GitContext,
    cwd: &Path,
    commit: &str,
) -> Result<String, TransactionStop> {
    let spec: String = format!("{commit}^{{tree}}");
    let read = run_git_checked(
        context,
        &GitRequest::new(cwd, &["rev-parse", "--verify", spec.as_str()]),
    )?;
    return Ok(String::from_utf8_lossy(read.stdout.as_slice())
        .trim()
        .to_string());
}

/// What:
///  The tree the private index holds,
/// which is the captured change on its base.
/// Why:
///   The retry merges this tree,
/// so it is written once,
/// when the service sends the commit back.
///  The index file is looked at first:
///  `git write-tree` answers with the empty tree for an index that is gone,
///  so a read that found nothing would rebuild the commit as a deletion of every file and report a conflict naming
///  paths the commit never touched.
///  A private index that is gone is named instead,
///  and nothing is replayed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await capturedTree(context, cwd, commitIndex, objectDirectory)
/// ```
pub fn captured_tree(
    context: &GitContext,
    cwd: &Path,
    commit_index: &Path,
    object_directory: &Path,
) -> Result<String, TransactionStop> {
    let present: bool = match std::fs::symlink_metadata(commit_index) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    };
    if !present {
        return Err(failed(format!(
            "The change this commit captured cannot be read back: the private index of the attempt that captured it, {}, is not there, \
             so this commit was not made. Run the commit again. {NOTHING_CHANGED}",
            commit_index.display()
        )));
    }
    let written = run_git_checked(
        context,
        &indexed_request(
            context,
            cwd,
            &["write-tree"],
            Some(commit_index),
            Some(object_directory),
        ),
    )?;
    return Ok(String::from_utf8_lossy(written.stdout.as_slice())
        .trim()
        .to_string());
}
