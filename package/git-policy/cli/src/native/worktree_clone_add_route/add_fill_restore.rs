//! What:
//!  The fill's restore:
//! whether it still has to run,
//! and the reset of the clone
//!  to the commit the caller asked for.
//! Why:
//!  The clone is a snapshot of a checkout that may hold uncommitted edits and
//!  whose `HEAD` may not be the commit this add checks out,
//! so the tree must be
//!  made equal to Git's own checkout of that commit before any name is published
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore" step 6).
//!  A resume must not run it twice:
//! the publication takes the private index's
//!  name away,
//! so a Git run pointed at the gone private name would read an empty
//!  index
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "What a resume finds").
//!  `add_fill.rs` runs the fill in order and would go over the file-size limit
//!  carrying the restore,
//!  and the restore runs through the btrfs rung's own `restore_dirty`,
//! so nothing
//!  here re-implements a reset.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (theRestoreStillRuns(adminPath, cloneRoot, alreadyExchanged)) restoreTheClone(input);
//! ```

use super::add_fill_input_and_result::FillInput;
use super::add_index::{
    PENDING_INDEX_NAME,
    pending_stands,
    published_stands,
};
use super::add_route_git::RouteGit;
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_clean_content_attribute_sources::for_context;
use crate::worktree_dirty_restore::restore_dirty;
use crate::worktree_ladder_target::Target;
use std::path::Path;

/// What:
///  Whether the fill's restore still has to run.
/// Why:
///  The restore edits the private index and the tree,
///  and the publication takes that index's name away,
///  so a resume that finds the worktree's own index already standing was
///  past the restore and must not run it again:
///  a restored tree needs no second restore,
/// and a Git run pointed at the
///  gone private name would read an empty index
///  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
///  "What a resume finds",
///  the rows for a mounted clone with the real-name index and for the saved
///  pointer).
///  A run resumed after the exchange is past the restore for the same reason.
pub(super) fn the_restore_still_runs(
    admin_path: &Path,
    clone_root: &Path,
    already_exchanged: bool,
) -> bool {
    if already_exchanged || published_stands(admin_path) {
        return false;
    }
    // The restore only runs against a private name that stands.
    return pending_stands(admin_path) || clone_root.join(".git").is_dir();
}

/// What:
///  Reset the clone to the commit the caller asked for,
///  through the btrfs rung's own restore against the private index.
/// Why:
///  The clone is a snapshot of a checkout that may hold uncommitted edits and
///  whose `HEAD` may not be the commit this add checks out,
/// so the tree must be
///  made equal to Git's own checkout of that commit before any name is published
///  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
///  "The restore" step 6).
///  The restore runs only once the private index stands,
/// and it reads and writes
///  that file rather than the worktree's own,
/// so no run touches a name the
///  worktree does not hold yet.
///
/// # Errors
/// The restore refused:
/// the clean,
/// the reset or its own proof.
pub(super) fn restore_the_clone(input: &FillInput<'_>) -> Result<(), String> {
    let target: Target = Target {
        admin: input.admin_path.to_path_buf(),
        index: Some(input.admin_path.join(PENDING_INDEX_NAME)),
        pointer: input.pointer.to_vec(),
        commit: input.capture.head.clone(),
        differs: false,
        line: String::new(),
    };
    let conversion: ConversionEnvironment = for_context(input.context);
    let mut runner: RouteGit<'_> = RouteGit {
        real_git: input.real_git,
        overlay: input.overlay,
    };
    return restore_dirty(
        input.source,
        &target,
        input.clone_root,
        &mut runner,
        input.context,
        &conversion,
    );
}
