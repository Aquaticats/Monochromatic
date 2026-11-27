//! What:
//!  What one step of the carry reads at the top of the worktree:
//!  the object and the index entry each side of one captured path holds,
//!  whether what stands at a path is one of the two the capture and the change explain,
//!  the context every read and write of a step names,
//!  and the runner the layer merge is given.
//! Why:
//!   The worktree's own shapes are read from the top of the worktree,
//!  where a relative directory option of the invocation does not reach:
//!  a `-C repo` applied to a request that already runs in the worktree would name the
//!  worktree twice,
//!  and every read of a path would answer about a place that stands nowhere.
//!  What a step does with these readings is
//!  `merge_isolation_carry_apply_clear_and_restore.rs`
//!  and `merge_isolation_carry_apply.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const here = rooted(context); const runner = runnerOf(here, root); // and what each side of one path holds
//! ```

/// The person's own work in the paths a merge changes,
/// as one step reads it of a single path.
use crate::merge_isolation_carry_parts::{
    CapturedFile,
    CapturedIndex,
    CapturedPath,
};
/// The change the merge makes at one path,
/// and the state the worktree holds there now.
use crate::merge_isolation_changes::{
    Change,
    WorktreeState,
};
/// The runner the word-level pass is given.
use crate::merge_second_pass::ContextRunner;
/// The Git context a read and a write take.
use crate::transaction_git::GitContext;
/// A place in the worktree.
use std::path::Path;

/// What:
///  Whether a worktree state is one the capture or the change explains.
/// Why:
///   Anything else was written after this command judged the path,
/// and replacing it would discard that
///  edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explained = (state, objects) => state.kind === 'missing' ? objects.includes(null) : objects.includes(state.oid);
/// ```
pub(super) fn explained(
    state: &WorktreeState,
    objects: &[Option<&str>],
) -> bool {
    return match state {
        WorktreeState::Missing => objects.contains(&None),
        WorktreeState::Content {
            object,
            ..
        } => objects.contains(&Some(object.as_str())),
        WorktreeState::Other => false,
    };
}

/// What:
///  Whether an index entry is one the capture or the change explains.
/// Why:
///   A staged entry nothing explains was staged after this command judged it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryExplained = (entry, allowed) => (entry === null && allowed.includes(null)) || allowed.some((a) => a !== null && a.mode === entry.mode && a.oid === entry.oid);
/// ```
pub(super) fn entry_explained(
    entry: &Option<(String, String)>,
    allowed: &[Option<(&str, &str)>],
) -> bool {
    let Some((mode, oid)) = entry else {
        return allowed.contains(&None);
    };
    for candidate in allowed {
        if *candidate == Some((mode.as_str(), oid.as_str())) {
            return true;
        }
    }
    return false;
}

/// What:
///  The change the merge makes to one captured path.
/// Why:
///   A path the merge does not change is not the carry's to write,
///  and every step of the carry asks this same question.
pub(super) fn change_for<'a>(
    changes: &'a [Change],
    entry: &CapturedPath,
) -> Option<&'a Change> {
    for change in changes {
        if change.path == entry.path {
            return Some(change);
        }
    }
    return None;
}

/// The object a captured worktree file holds,
/// as the filtered bytes Git records for it.
pub(super) fn blob_of(file: &CapturedFile) -> &str {
    return file.blob.as_str();
}

/// The object the path held before the merge,
/// when the path existed then.
pub(super) fn old_oid_of(change: &Change) -> Option<&str> {
    if !change.existed() {
        return None;
    }
    return Some(change.old_oid.as_str());
}

/// The object the merge makes at the path,
/// when the path exists after it.
pub(super) fn new_oid_of(change: &Change) -> Option<&str> {
    if !change.exists_after() {
        return None;
    }
    return Some(change.new_oid.as_str());
}

/// The index entry the path held before the merge,
/// when the path existed then.
pub(super) fn old_entry_of(change: &Change) -> Option<(&str, &str)> {
    if !change.existed() {
        return None;
    }
    return Some((change.old_mode.as_str(), change.old_oid.as_str()));
}

/// The index entry the merge makes at the path,
/// when the path exists after it.
pub(super) fn new_entry_of(change: &Change) -> Option<(&str, &str)> {
    if !change.exists_after() {
        return None;
    }
    return Some((change.new_mode.as_str(), change.new_oid.as_str()));
}

/// The mode and object of one captured index entry.
pub(super) fn index_pair(staged: &CapturedIndex) -> (&str, &str) {
    return (staged.mode.as_str(), staged.oid.as_str());
}

/// The runner that merges one layer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runner = { run: (args, input, index) => startGit(context, root, args, input, index) };
/// ```
pub(super) fn rooted(context: &GitContext) -> GitContext {
    // The worktree's own shapes are read from the top of the worktree, where the invocation's global options do not reach.
    return GitContext {
        global_prefix: Vec::new(),
        ..context.clone()
    };
}

/// The runner that merges one layer,
/// from the top of the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runner = { run: (args, input, index) => startGit(rooted(context), root, args, input, index) };
/// ```
pub(super) fn runner_of<'a>(
    context: &'a GitContext,
    root: &'a Path,
) -> ContextRunner<'a> {
    return ContextRunner {
        context,
        cwd: root,
        without_prefix: true,
        repository: Vec::new(),
        object_directory: None,
        environment: Vec::new(),
    };
}
