//! What:
//!  The conversion fence's one `git ls-tree` size read:
//! the map every written path is proved by,
//! the thread that can start it before the source check on rung 1,
//! and the join that takes its answer at the fence.
//! Why:
//!  On rung 1 (the snapshot) the read sat on the add's waiting path after `checkSource` and after `git check-attr`.
//! The read needs only what the ref files answer in process (the worktree root,
//! the commit `HEAD` names and the object format),
//!  so the ladder starts it before the check and the fence takes its answer when it listed the commit the check proved
//! (`doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md`,
//! its section "The overlap,
//! answered 2026-11-22").
//! The read never fetches from a server,
//!  and it is joined with a deadline,
//!  so one that never answers refuses the rung with a failed read's own sentence rather than holding the add.
//! It runs on a thread of its own and not on the process's runtime,
//!  because every Git run registers its child's pipes with that runtime
//!  (`step_output_read_whole_through_pipes::attach_pipes`),
//!  which refuses a caller already inside it:
//! a task and a job of the blocking pool both are,
//!  and both failed this read with that refusal when it was built that way.
//! Rung 2 is untouched:
//!  its fence reads beside the copies (`worktree_clone_conversion_beside_the_clone.rs`).
//! This read is split across siblings because this file is over the line limit:
//! this one holds the join at the fence,
//! `worktree_clone_conversion_size_read_start.rs` starts the read beside the check,
//! `worktree_clone_conversion_size_read_answer.rs` holds the value one add carries,
//! `worktree_clone_conversion_size_read_git_read.rs` makes the one Git run,
//! `worktree_clone_conversion_size_read_seams.rs` holds the seams a control arms,
//! `worktree_clone_conversion_size_read_deadline.rs` the join's deadline,
//! and `worktree_clone_conversion_size_read_free.rs` the free of a map nobody uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = startBesideTheCheck(context, identity); // rung 1 only, null when it cannot start
//! const fence = fenceTheCloneWithTheReadBesideTheCheck(source, context, started); // takes it, or reads here
//! ```

use crate::diagnostic_log::debug;
use crate::transaction_git::GitContext;
#[cfg(test)]
use crate::transaction_git::GitRequest;
use crate::worktree_clone_conversion_fence::{
    AttributeFence,
    Fence,
    decide_the_paths_with_sizes,
    fence_the_attributes,
};
use crate::worktree_clone_conversion_git_reads::{
    BlobSizes,
    blob_sizes,
};
pub use crate::worktree_clone_conversion_size_read_answer::SizeReadBesideTheCheck;
use crate::worktree_clone_conversion_size_read_answer::TAG;
#[cfg(test)]
use crate::worktree_clone_conversion_size_read_answer::{
    ReadAnswer,
    WORK,
};
#[cfg(test)]
use crate::worktree_clone_conversion_size_read_git_read::the_early_read_request;
pub use crate::worktree_clone_conversion_size_read_start::start_beside_the_check;
#[cfg(test)]
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_snapshot_source::SnapshotSource;
#[cfg(test)]
use std::ffi::OsString;
#[cfg(test)]
use std::time::Duration;

/// What:
///  Whether this read listed the commit the fence needs.
/// Why:
///   The guard is the whole of the proof the answer is used on:
/// it compares the commit the read listed with the commit the source check proved,
///  and the map is a function of that commit and of nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lists = (read, commit) => read.commit === commit;
/// ```
fn lists(
    read: &SizeReadBesideTheCheck,
    commit: &str,
) -> bool {
    return read.commit == commit;
}

/// What:
///  Both halves of the fence for the snapshot rung,
/// with the sizes taken from the read the ladder started when it listed the proved commit,
///  and read here when it did not.
/// Why:
///   The attributes and the decline are decided first,
/// as everywhere,
/// and the sizes are wanted only after a decline was refused;
/// a read of another commit is ended here and the fence reads the proved commit itself,
/// which is today's serial order,
///  because a read nobody will use must not keep running to the process's end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fenceTheCloneWithTheReadBesideTheCheck(source, context, started): Fence;
/// ```
pub fn fence_the_clone_with_the_read_beside_the_check(
    source: &SnapshotSource,
    context: &GitContext,
    started: &mut Option<SizeReadBesideTheCheck>,
) -> Result<Fence, String> {
    let read: AttributeFence = fence_the_attributes(source, context)?;
    if !read.needs_sizes() {
        // No path needs a size, so this read has nothing to answer: an add that leaves it running pays it beside
        //  Git's own checkout to the process's end (the integration recheck's Low 4 of 2026-11-22).
        end_the_read_the_fence_will_not_use(started);
        return decide_the_paths_with_sizes(read, None);
    }
    let sizes: BlobSizes = sizes_of_the_paths(source, context, started)?;
    return decide_the_paths_with_sizes(read, Some(sizes));
}

/// What:
///  End the read this fence will not use,
///  stating it as work when it has already answered and as started and not joined otherwise.
/// Why:
///   A fence with no path to size,
///  or one whose started read listed another commit,
///  has no answer to take;
/// the read is still a child that would compete with Git's own checkout,
///  so the add ends it where it decides not to use it
///  (the integration recheck's Low 4 of 2026-11-22).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endTheReadTheFenceWillNotUse = (started) => started.take()?.abandon();
/// ```
fn end_the_read_the_fence_will_not_use(started: &mut Option<SizeReadBesideTheCheck>) {
    if let Some(read) = started.take() {
        read.abandon();
    }
}

/// What:
///  The sizes the fence decides with:
/// the started read's answer when it listed the proved commit,
///  and one `git ls-tree` of that commit otherwise.
/// Why:
///   A started read that listed another commit is not this fence's answer,
/// and it is not waited for here:
///  it is ended where the fence learns it cannot use it,
///  so a settled `HEAD` is the only case that spends the overlap
///  and an ended one pays no read beside Git's checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sizesOfThePaths(source, context, started): BlobSizes;
/// ```
fn sizes_of_the_paths(
    source: &SnapshotSource,
    context: &GitContext,
    started: &mut Option<SizeReadBesideTheCheck>,
) -> Result<BlobSizes, String> {
    let commit: &str = source.clean.head_commit.as_str();
    match started.as_ref() {
        Some(proved) if lists(proved, commit) => {
            let taken: SizeReadBesideTheCheck = started
                .take()
                .expect("the read that lists the proved commit is the one just matched");
            return taken.join();
        }
        Some(read) => {
            debug(TAG, format!("the read started beside the check listed {} and the fence needs {commit}, so it is ended here and the fence reads for itself", read.commit).as_str());
            end_the_read_the_fence_will_not_use(started);
        }
        None => {}
    }
    return blob_sizes(context, source.root.as_path(), commit);
}

#[cfg(test)]
pub(crate) use crate::worktree_clone_conversion_size_read_answer::deadline::a_control_shortens_the_deadline_to;
#[cfg(test)]
pub(crate) use crate::worktree_clone_conversion_size_read_answer::free::{
    answered_maps_freed_for_a_control,
    the_answer_was_sent_for_a_control,
};
#[cfg(test)]
pub(crate) use crate::worktree_clone_conversion_size_read_seams::{
    a_control_holds_the_read,
    a_control_holds_the_sizes_after_the_wait,
    a_control_makes_the_read_die,
    the_sizes_hold_was_reached,
};

/// Size-read controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_conversion_size_read_tests.rs"]
mod tests;
