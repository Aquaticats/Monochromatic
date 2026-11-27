//! What:
//!  Undo a migration (`--rollback`) and delete the old tree after a finished one (`--remove-old`),
//! plus the removal of
//!  a subvolume tree that both depend on.
//! Why:
//!  Rollback carries back what was written to the new root since the exchange (files newer there win),
//! swaps the trees
//!  again,
//! rewrites the restored indexes' stat fields (the carried-back index describes the new tree's inodes),
//! and
//!  deletes the new subvolume deepest nested subvolume first,
//! because the kernel refuses to destroy a subvolume that
//!  holds another (`ENOTEMPTY`).
//! Where the mount does not allow an unprivileged destroy,
//! an emptied subvolume is
//!  removed with `rmdir`,
//! which the kernel allows for an empty subvolume root.
//! The old tree is an ordinary directory,
//!  so deleting it is a file walk,
//! never a subvolume destroy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await rollback(run); await removeOldTree(run, { discardUnreadable });
//! ```
//!
//! The code of this module is split by role,
//! and this file re-exports every name those files hold,
//! so that a caller writes `subvolume_rollback::rollback` as it did before:
//! `subvolume_rollback_refusals` holds the refusals,
//! `subvolume_rollback_carry_back` the carry of the person's work out of the new subvolume,
//! `subvolume_rollback_swap` the exchange back,
//! and `subvolume_rollback_removal` the deletion of the old tree.

use crate::subvolume_arguments::MIGRATE_RISK_NOTICE;
use crate::subvolume_error::MigrateError;
use crate::subvolume_index_files::IndexLock;
use crate::subvolume_index_refresh_of_every_worktree::refresh_all_indexes;
use crate::subvolume_lock::{
    release_all,
    token_of,
};
use crate::subvolume_preflight::{
    Refusal,
    rollback_refusals,
};
use crate::subvolume_rollback_identity::installed_identity_refusal;
use crate::subvolume_rollback_stage::stage_is_not_a_subvolume;
use crate::subvolume_rollback_subvolume_removal::{
    remove_if_present,
    remove_subvolume_tree,
};
use crate::subvolume_rollback_swap::swap_back;
use crate::subvolume_run::Run;
use std::path::PathBuf;

pub use crate::subvolume_rollback_refusals::refusals_of_the_mode;
pub use crate::subvolume_rollback_removal::{
    remove_old_tree,
    remove_old_tree_seamed,
};
#[cfg(test)]
pub(super) use crate::subvolume_rollback_swap::swap_back_seamed;

/// What:
///  The rollback.
///  `Ok(None)` means it ran;
/// `Ok(Some(refusals))` means nothing was changed.
/// Why:
///   Three states can be undone:
/// before the exchange (delete the new subvolume and the lock),
/// after it (carry back,
///  swap again,
/// delete the new subvolume),
/// and in between (finish the bookkeeping,
/// then the second case).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = await rollback(run);
/// ```
pub fn rollback(
    run: &mut Run<'_>,
    discard_unreadable: bool,
    discard_left_out: bool,
) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let refusals: Vec<Refusal> = rollback_refusals(run.layout, &run.journal);
    if !refusals.is_empty() {
        return Ok(Some(refusals));
    }
    // Said once, before the first step that changes the checkout; a refusal above changes nothing, so it stopped before here.
    run.line(MIGRATE_RISK_NOTICE);
    let root: PathBuf = run.layout.root.clone();
    let new_path: PathBuf = run.layout.new_path.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    if run.journal.has_begun("exchange")
        && !run.journal.is_done("exchange")
        && run
            .platform
            .is_subvolume_root(root.as_path())
            .unwrap_or(false)
    {
        if new_path.exists() && !old_path.exists() {
            MigrateError::from_io_result(
                std::fs::rename(&new_path, &old_path),
                "rename",
                new_path.as_path(),
            )?;
        }
        run.journal.mark("exchange", "done")?;
    }
    if run.journal.is_done("exchange")
        && let Some(blocked) = swap_back(run, discard_unreadable, discard_left_out)?
    {
        return Ok(Some(blocked));
    }
    // The restored tree is the one the migration locked: a stop before the exchange leaves its locks in the checkout, and
    // a stop between the exchange and the removal of the locks from the old tree restores a tree that still holds them.
    // Git would refuse every index write while they stay. Only locks that hold this migration's token are removed.
    let token: String = token_of(&run.journal);
    release_all(root.as_path(), token.as_str())?;
    // The carried-back index describes the new tree's inodes; this runs after a stop at any step above, once.
    if run.journal.state("exchange") == Some("rolled-back")
        && !run.journal.is_done("rollback-indexes")
    {
        let mut lines: Vec<String> = Vec::new();
        // The refresh reports each index into the list the lines are collected in; they are said even when the refresh
        //  stops, because each of them is a fact about an index that was looked at.
        let refreshed: Result<usize, MigrateError> = refresh_all_indexes(
            root.as_path(),
            IndexLock::Acquire,
            std::slice::from_ref(&new_path),
            run.platform.flushes_to_disk(),
            &mut lines,
        );
        for text in &lines {
            run.line(text.as_str());
        }
        let _count: usize = refreshed?;
        run.journal.mark("rollback-indexes", "done")?;
    }
    // The subvolume this deletes is the one the migration installed, compared at every path that reaches the deletion: a
    // run resumed after the exchange was recorded skips the swap, and with it the pair check that guards the exchange. This
    // check comes first because a name that holds something other than the installed checkout says so more exactly than a
    // stage identity that no longer answers.
    if new_path.exists()
        && let Some(refusal) = installed_identity_refusal(run, new_path.as_path())
    {
        return Ok(Some(vec![refusal]));
    }
    // The stage is the subvolume the migration created; a directory that replaced it is somebody else's, and the removal's
    // fallback would delete it with whatever it holds.
    if let Some(refusal) = stage_is_not_a_subvolume(run) {
        return Ok(Some(vec![refusal]));
    }
    if new_path.exists() {
        run.line(
            format!(
                "deleting the new subvolume {} (nested subvolumes first)",
                new_path.display()
            )
            .as_str(),
        );
        remove_subvolume_tree(run, new_path.as_path())?;
    }
    run.journal.delete()?;
    remove_if_present(run.layout.unreadable_path.as_path())?;
    run.line(format!("rolled back: {} is the original directory again, and the migration journal is deleted.", root.display()).as_str());
    return Ok(None);
}

/// Rollback controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_rollback_tests.rs"]
mod tests;
