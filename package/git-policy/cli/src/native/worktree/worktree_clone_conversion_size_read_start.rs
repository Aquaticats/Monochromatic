//! What:
//!  The start of the fence's size read beside the add's source check:
//!  the worktree root and the two Git directories it reads,
//!  the thread that carries the read,
//!  and the answer it sends once.
//! Why:
//!   On rung 1 the read sat on the add's waiting path after the source check and after `git check-attr`,
//!  and it needs only what the ref files answer in process,
//!  so the ladder starts it before the check and the fence takes its answer when it listed the commit the
//!  check proved.
//!  It runs on a thread of its own and not on the process's runtime,
//!  because every Git run registers its child's pipes with that runtime and refuses a caller already inside it,
//!  which a task and a job of the blocking pool both are.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = startBesideTheCheck(context, identity); // undefined when it cannot start
//! ```

use crate::diagnostic_log::debug;
use crate::repo_read_layout::ObjectFormat;
use crate::scoped_start::{
    refused_thread_sentence,
    spawn_named,
};
use crate::subvolume_index_files::repository_hash_length;
use crate::transaction_git::GitContext;
use crate::worktree_clean::head_commit;
use crate::worktree_clone_conversion_git_reads::BlobSizes;
#[cfg(test)]
use crate::worktree_clone_conversion_size_read_answer::free::note_the_answer_was_sent;
use crate::worktree_clone_conversion_size_read_answer::{
    ReadAnswer,
    SizeReadBesideTheCheck,
    TAG,
};
use crate::worktree_clone_conversion_size_read_git_read::sizes_of_the_commit_where_nothing_is_fetched;
use crate::worktree_identity::WorktreeIdentity;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::mpsc::Sender;
use std::time::Instant;

/// The name the read's thread carries in the process.
/// Why:
///   The read is the one step of an add that runs on a thread of its own,
/// so a diagnostic that lists threads names it for what it does.
const THREAD: &str = "clone-size-read-beside-the-check";

/// What:
///  The worktree root and its two Git directories,
/// or nothing for an identity without a worktree.
/// Why:
///   The read lists the commit `HEAD` names in the source's own worktree,
/// and the ref files and the configuration are read from its directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directoriesOf = (identity) => identity.kind === 'worktree' ? [identity.root, identity.gitDir, identity.commonDir] : undefined;
/// ```
fn directories_of(identity: &WorktreeIdentity) -> Option<(&Path, &Path, &Path)> {
    return match identity {
        WorktreeIdentity::MainWorktree {
            worktree_root,
            git_dir,
            common_dir,
        }
        | WorktreeIdentity::LinkedWorktree {
            worktree_root,
            git_dir,
            common_dir,
        } => Some((
            worktree_root.as_path(),
            git_dir.as_path(),
            common_dir.as_path(),
        )),
        WorktreeIdentity::BareRepository {
            ..
        }
        | WorktreeIdentity::OutsideWorktree => None,
    };
}

/// What:
///  Start the fence's size read now,
///  before the source check the ladder runs next.
/// Why:
///   The ladder starts it on rung 1 alone,
///  where the read would otherwise run after the check and after `git check-attr`;
///  the read needs only the worktree root,
///  the commit `HEAD` names and the object format,
///  all of which are answered here without a Git run.
/// It is a plain thread and not a task of the process's runtime,
///  nor a job of its blocking pool:
/// a Git run registers its child's pipes with that runtime and refuses a caller already inside it
///  (`step_output_read_whole_through_pipes::attach_pipes`),
///  and both of those callers are inside it,
/// so neither can run this read at all.
/// A start that cannot happen (no worktree,
///  a ref store this reader does not know,
///  a system that refuses the thread)
///  loses only the overlap:
/// the fence then reads where it stands,
/// as it always has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startBesideTheCheck(context, identity): SizeReadBesideTheCheck | undefined;
/// ```
pub fn start_beside_the_check(
    context: &GitContext,
    identity: &WorktreeIdentity,
) -> Option<SizeReadBesideTheCheck> {
    let (root, git_dir, common_dir): (&Path, &Path, &Path) = directories_of(identity)?;
    let hash_length: usize = repository_hash_length(common_dir)?;
    let format: ObjectFormat = if hash_length == 32 {
        ObjectFormat::Sha256
    } else {
        ObjectFormat::Sha1
    };
    let Some(commit): Option<String> = head_commit(git_dir, common_dir, format) else {
        debug(
            TAG,
            "the commit HEAD names is not a plain ref, so the size read does not start beside the check",
        );
        return None;
    };
    let child: std::sync::Arc<std::sync::atomic::AtomicI32> =
        std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let (sender, answer): (Sender<ReadAnswer>, std::sync::mpsc::Receiver<ReadAnswer>) =
        std::sync::mpsc::channel();
    let state: (
        GitContext,
        PathBuf,
        String,
        Sender<ReadAnswer>,
        std::sync::Arc<std::sync::atomic::AtomicI32>,
    ) = (
        context.clone(),
        root.to_path_buf(),
        commit.clone(),
        sender,
        child.clone(),
    );
    if let Err(error) = spawn_named(THREAD, move || {
        return read_the_sizes_where_nothing_is_fetched(state);
    }) {
        debug(
            TAG,
            refused_thread_sentence(
                "read the fence's blob sizes beside the source check",
                &error,
            )
            .as_str(),
        );
        return None;
    }
    return Some(SizeReadBesideTheCheck {
        commit,
        answer,
        child,
    });
}

/// What:
///  The read's thread:
/// the sizes of the commit's tree with the time they took,
///  sent once to the add that started it.
/// Why:
///   The thread owns what it reads,
/// so it takes the context,
///  the worktree root and the commit by value and answers both the cost and the map;
/// a send nobody waits for any more is dropped,
///  because the add moved past its deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTheSizesWhereNothingIsFetched([context, root, commit, sender]) { const began = performance.now(); sender.send([performance.now() - began, sizesOf(context, root, commit)]); }
/// ```
fn read_the_sizes_where_nothing_is_fetched(
    (context, root, commit, sender, child): (
        GitContext,
        PathBuf,
        String,
        Sender<ReadAnswer>,
        std::sync::Arc<std::sync::atomic::AtomicI32>,
    )
) {
    let began: Instant = Instant::now();
    let sizes: Result<BlobSizes, String> = sizes_of_the_commit_where_nothing_is_fetched(
        &context,
        root.as_path(),
        commit.as_str(),
        &child,
    );
    // The cell is not cleared here: the runner clears it with a compare-exchange when the child it published is
    //  gone (`child_watch_one_shot_run.rs`, the `StepBound::EndAt` arm), so a store here would erase an abandon
    //  marker for nothing, and it can name no process the runner has not already dealt with.
    #[cfg(test)]
    note_the_answer_was_sent();
    let _sent: Result<(), std::sync::mpsc::SendError<ReadAnswer>> =
        sender.send((began.elapsed(), sizes));
}
