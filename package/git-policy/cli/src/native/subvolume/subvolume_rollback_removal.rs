//! What:
//!  Deleting the old tree once the person asks for it:
//!  the refusals first,
//!  then the late-write carry into the checkout,
//!  then the removal of the tree and of the subvolume,
//!  and its seamed form a control takes over at the moment before either.
//! Why:
//!  The old tree is the only copy of the files this user could not read,
//!  and of anything not yet carried over,
//!  so it is deleted only after a verified migration,
//!  only when the person asks,
//!  and only after what is newer in it has been carried into the checkout.
//!  A remove that cannot prove a subvolume is the migration's leaves it and says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeOldTree(run, { discardUnreadable });
//! ```

use crate::fs_beneath::handle_path;
use crate::subvolume_arguments::MIGRATE_RISK_NOTICE;
use crate::subvolume_error::MigrateError;
use crate::subvolume_preflight::{
    Refusal,
    remove_old_refusals,
};
use crate::subvolume_remove::remove_tree;
use crate::subvolume_rollback_carry_back::carry_late_writes;
use crate::subvolume_rollback_identity::{
    PinnedRoot,
    pin_root,
    pinned_root_moved,
    uncarried_cause,
    uncarried_remedy,
};
use crate::subvolume_rollback_refusals::{
    old_tree_is_a_link,
    unreadable_only_in_the_old_tree,
};
use crate::subvolume_rollback_subvolume_removal::remove_if_present;
use crate::subvolume_run::Run;
use std::path::PathBuf;

/// What:
///  Delete the old tree of a finished migration.
///  `Ok(None)` means it ran;
/// `Ok(Some(refusals))` means nothing was changed.
/// Why:
///   Two kinds of entry exist only there and are never deleted without the person's word:
/// the files this user may not read,
///  and the entries the late-write carry left out because the checkout lacks them and they are older than the
///  catch-up,
/// which may be files restored into the old tree or files deleted from the checkout on purpose.
/// The refusal names both kinds,
///  and each is deleted only with its own flag.
/// The deletion (`remove_tree`) unlinks entries,
/// makes directories the user owns writable on the way,
/// and never follows symbolic links.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = await removeOldTree(run, { discardUnreadable, discardLeftOut });
/// ```
pub fn remove_old_tree(
    run: &mut Run<'_>,
    discard_unreadable: bool,
    discard_left_out: bool,
) -> Result<Option<Vec<Refusal>>, MigrateError> {
    return remove_old_tree_seamed(
        run,
        discard_unreadable,
        discard_left_out,
        &mut nothing_at_all,
    );
}

/// A step that does nothing,
/// for the run between the checkout being pinned and the carry.
pub(super) fn nothing_at_all() {}

/// What:
///  `remove_old_tree`,
/// with a call right after the checkout was opened and judged to be the installed one,
/// where a control
///  moves it aside and puts another directory at its name.
/// Why:
///   The window between the identity check and the writes is the defect the pin closes,
/// and a test of it can only
///  enter that window through a seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeOldTreeSeamed(run, flags, afterPinned);
/// ```
pub fn remove_old_tree_seamed(
    run: &mut Run<'_>,
    discard_unreadable: bool,
    discard_left_out: bool,
    after_pinned: &mut dyn FnMut(),
) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let mut refusals: Vec<Refusal> = remove_old_refusals(run.layout, &run.journal);
    let old_path: PathBuf = run.layout.old_path.clone();
    let mut pinned: Option<PinnedRoot> = None;
    if refusals.is_empty() {
        // The examination of the old tree is what the preview also reads, so a path that cannot be examined is an error here.
        MigrateError::from_io_result(
            std::fs::symlink_metadata(&old_path),
            "examine",
            old_path.as_path(),
        )?;
        if let Some(refusal) = old_tree_is_a_link(run) {
            return Ok(Some(vec![refusal]));
        }
        // The carry is the first step that writes; the refusal below comes after it, so the risk is said from here on.
        run.line(MIGRATE_RISK_NOTICE);
        match pin_root(run, after_pinned)? {
            Ok(found) => {
                let destination: PathBuf = handle_path(&found.directory);
                carry_late_writes(run, destination.as_path(), run.layout.root.as_path())?;
                pinned = Some(found);
            }
            Err(refusal) => return Ok(Some(vec![refusal])),
        }
    }
    // The files went into the directory that was pinned; the old tree is deleted only while the name still leads to it.
    if let Some(found) = pinned.as_ref()
        && let Some(refusal) = pinned_root_moved(run, found)
    {
        return Ok(Some(vec![refusal]));
    }
    if refusals.is_empty()
        && (!run.left_out.is_empty() || !run.conflicts.is_empty())
        && !discard_left_out
    {
        refusals.push(Refusal {
            cause: uncarried_cause(run),
            remedy: uncarried_remedy(),
        });
    }
    if refusals.is_empty()
        && let Some(refusal) = unreadable_only_in_the_old_tree(run, discard_unreadable)
    {
        refusals.push(refusal);
    }
    if !refusals.is_empty() {
        return Ok(Some(refusals));
    }
    run.line(
        format!(
            "deleting {} (every file is unlinked; this can take minutes)",
            old_path.display()
        )
        .as_str(),
    );
    MigrateError::from_io_result(remove_tree(&old_path), "delete", old_path.as_path())?;
    run.journal.delete()?;
    remove_if_present(run.layout.unreadable_path.as_path())?;
    remove_if_present(run.layout.left_out_path.as_path())?;
    run.line("the old tree is deleted and the migration journal is removed.");
    return Ok(None);
}
