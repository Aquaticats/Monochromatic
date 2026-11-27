//! What:
//!  The stat data a landing may record for a path whose worktree file provably holds the bytes the commit stores,
//! so the entry the in-process reset writes describes the file instead of carrying all-zero stat data.
//! Why:
//!  The landing's in-process index reset writes the commit's paths as `git reset --no-refresh` does,
//! with all-zero stat data (`index_reset.rs`,
//! `StatData::default()`),
//! where native `git commit` leaves the worktree's.
//!  The next commit therefore finds those paths stale and refreshes them once,
//! through one Git start and one hash per path the
//!  previous commit landed and this one does not name,
//! which native Git pays nothing for
//! (`doc/handover/cli-git-rust-implementation.md`,
//! "Still to do").
//!  An entry may carry the file's stat data only when nothing the file holds is hidden by it,
//! because a reader that trusts the stat data
//!  (this wrapper's own staleness question,
//! and Git's own checks)
//! would otherwise report a changed file as unchanged,
//! and the person's
//!  edit would reach neither the next commit nor `git status`.
//!  So the stat data here is recorded only when all five hold:
//!
//! - The path is a regular file,
//! and every directory on the way is a real one on the worktree's device,
//! reached below a held root without following a link.
//!   A symbolic link is left to Git:
//! it is rare,
//! the
//!  zero value only costs a refresh,
//! and a link read across a swap of its parent directory cannot be proven to be the one examined.
//! - The file's raw bytes hash to the entry's object name,
//! read as a blob.
//! - No attribute Git applies converts these bytes,
//! and the bytes hold no carriage return.
//!   A clean filter,
//! `ident`
//!  or a `working-tree-encoding` can leave raw bytes equal to the blob while Git's view of the file is another blob,
//! and a
//!  line-ending rule may strip a carriage return on the way into the index,
//! so such a path keeps the zero stat data and costs a refresh.
//!   The attributes are read in
//!  process (`worktree_clean_content.rs`,
//! `conversion_is_the_identity`),
//! from the files Git itself would read,
//! and where their location cannot be established the
//!  path keeps the zero value.
//! - The state of the one inode that was read is the same before,
//! during and after the read and the path still names it,
//! and the
//!  file is older than the index file this entry is written into,
//! compared to the nanosecond as Git's `is_racy_stat` compares them.
//!   An entry as
//!  new as the index is what Git calls racily clean,
//! so it keeps the zero value and both readers hash the file;
//! a file written
//!  earlier in the index file's own second is older than the index on a file system that stores nanoseconds,
//! so the entries a
//!  person's edit just made record their stat data and the next commit starts no Git for them.
//! - The change time the file had when it was read has nanoseconds.
//!   A file system that stores whole seconds reports a change time with none,
//! and a same-size rewrite whose modification time is put back inside that second leaves every field this proof compares
//!  unchanged,
//! so such a file keeps the zero stat data and costs a refresh.
//!   The attributes reader refuses such a file for
//!  the same reason (`worktree_clean_content_attributes.rs`,
//! `attribute_bytes`).
//!
//! The read never waits and never passes a bound:
//! a path that is not a regular file is refused by its type,
//! the open does
//!  not block (a fifo renamed over the path is opened without waiting for a writer and refused by its type),
//! and the bytes are
//!  read at most up to what is left of `CONTENT_LIMIT` for the whole landing.
//! A read the system interrupts is retried at most `INTERRUPTED_TRIES` times,
//! so a source that interrupts every read cannot hold the landing's index lock:
//!  the path then keeps the zero stat data,
//! and the next command that needs it hashes the file through Git.
//! Every byte that reaches the disk is charged,
//! including a
//!  read the stability checks later refuse,
//! so a landing that lands a whole worktree reads at most `CONTENT_LIMIT` bytes of it inside the
//!  landing's index lock.
//!
//!  The proof is split across siblings because this file is over the line
//!  limit:
//!  this one is the two entry functions that prove one path,
//!  `index_reset_worktree_stat_request.rs` holds the bounds,
//!  the seams a control changes the file at and what one path asks,
//!  `index_reset_worktree_stat_proofs.rs` proves the bytes and the state of one file,
//!  and `index_reset_worktree_stat_landing.rs` holds the worktree a landing keeps open.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stat: StatData | undefined = recordedStat({ worktreeRoot, path, blob, mode, format, stamp, hashed });
//! ```

/// The source a landing names,
/// which proves many paths against one worktree held open.
pub use super::index_reset_worktree_stat_landing::LandingStat;
/// What one path asks the proof for,
/// and the seams a control changes the file at.
pub use super::index_reset_worktree_stat_request::{
    ProofSeams,
    WorktreeStatRequest,
};
/// The seams the release executable has nothing between.
use super::index_reset_worktree_stat_request::{
    do_nothing,
    read_some,
};
/// The stat fields an entry records.
use crate::index_stat::StatData;
/// The attributes Git would apply to one path,
/// and the directories whose attributes the index holds.
use crate::worktree_clean_tree_walk::ContentTarget;
// The proof a control reads through `super::`, which a control build is the only build that has.
#[cfg(test)]
use super::index_reset_worktree_stat_proofs::stamp_proves_one_version;

/// What:
///  The stat data one path's file provably has,
/// asked with the seams a control changes the file at.
/// Why:
///   The source a landing uses keeps the held root and the attribute files across paths,
/// and a caller that proves one path has nothing to keep,
///  so this reads the one file from the worktree root itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = recordedStatSeamed(request, seams);
/// ```
pub fn recorded_stat_seamed(
    request: &WorktreeStatRequest<'_>,
    seams: &mut ProofSeams<'_>,
) -> Option<StatData> {
    let root: std::fs::File = crate::fs_beneath::open_root(request.worktree_root).ok()?;
    let target: ContentTarget<'_> = ContentTarget {
        root: request.worktree_root,
        git_dir: request.git_dir,
        common_dir: request.common_dir,
        environment: request.environment,
        format: request.format,
    };
    let source: LandingStat<'_> = LandingStat::new(
        &root,
        &target,
        request.stamp,
        request.hashed,
        request.indexed_attributes,
    );
    return source.record(request.path, request.blob, request.mode, seams);
}

/// What:
///  `recorded_stat_seamed` with the seams that change nothing.
/// Why:
///  The release executable has no control under the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stat = recordedStat(request);
/// ```
pub fn recorded_stat(request: &WorktreeStatRequest<'_>) -> Option<StatData> {
    return recorded_stat_seamed(
        request,
        &mut ProofSeams {
            before_open: &mut do_nothing,
            after_read: &mut do_nothing,
            read: &mut read_some,
        },
    );
}

/// Controls for the recorded stat data stay out of the release executable.
#[cfg(test)]
#[path = "index_reset_worktree_stat_tests.rs"]
mod tests;
