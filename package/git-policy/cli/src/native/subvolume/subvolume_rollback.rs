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

use crate::subvolume_arguments::{MIGRATE_RISK_NOTICE, MigrateMode};
use crate::subvolume_error::MigrateError;
use crate::subvolume_index_files::IndexLock;
use crate::subvolume_index_refresh_of_every_worktree::refresh_all_indexes;
use crate::subvolume_lock::{release_all, token_of};
use crate::subvolume_preflight::{Refusal, remove_old_refusals, rollback_refusals};
use crate::subvolume_remove::remove_tree;
use crate::subvolume_run::Run;
use crate::subvolume_sync::SyncStats;
use std::path::{Path, PathBuf};

/// Remove a file,
/// treating "already gone" as done.
fn remove_if_present(path: &Path) -> Result<(), MigrateError> {
    return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(MigrateError::io("remove", path, &error)),
    };
}

/// What:
///  Every directory inside `top` (not `top` itself) that is a subvolume root,
/// deepest first.
/// Why:
///   Destroying them in this order never meets a subvolume that still holds another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nested = await nestedSubvolumes(top); // deepest first
/// ```
fn nested_subvolumes(run: &Run<'_>, top: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![top.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(listing) = std::fs::read_dir(&directory) else { continue };
        for item in listing.flatten() {
            let Ok(file_type) = item.file_type() else { continue };
            if !file_type.is_dir() {
                continue;
            }
            if run.platform.is_subvolume_root(item.path().as_path()).unwrap_or(false) {
                found.push(item.path());
            }
            stack.push(item.path());
        }
    }
    // Each path is paired with its depth so the sort compares numbers; the sort is stable, so equal depths keep their order.
    let mut with_depth: Vec<(usize, PathBuf)> = Vec::new();
    for path in found {
        with_depth.push((path.components().count(), path));
    }
    with_depth.sort_by(deeper_first);
    let mut ordered: Vec<PathBuf> = Vec::new();
    for (_depth, path) in with_depth {
        ordered.push(path);
    }
    return ordered;
}

/// Order two paths with the one that has more components first.
fn deeper_first(left: &(usize, PathBuf), right: &(usize, PathBuf)) -> std::cmp::Ordering {
    return right.0.cmp(&left.0);
}

/// What:
///  Remove one subvolume that holds no other:
/// by the destroy call where the mount allows it,
/// else by emptying it and
///  `rmdir`.
/// Why:
///   Both end with the directory gone.
/// A destroy that fails for any reason falls through to the emptying route,
/// whose
///  own failure is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeOneSubvolume(run, path);
/// ```
fn remove_one_subvolume(run: &mut Run<'_>, path: &Path) -> Result<(), MigrateError> {
    if run.platform.removal_allowed(path) && run.platform.destroy_subvolume(path).is_ok() {
        return Ok(());
    }
    MigrateError::from_io_result(remove_tree(path), "empty and remove the subvolume", path)?;
    return Ok(());
}

/// What:
///  Remove a subvolume and every subvolume nested in it,
/// deepest first.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeSubvolumeTree(run, path);
/// ```
pub fn remove_subvolume_tree(run: &mut Run<'_>, path: &Path) -> Result<(), MigrateError> {
    for nested in nested_subvolumes(run, path) {
        remove_one_subvolume(run, nested.as_path())?;
    }
    return remove_one_subvolume(run, path);
}

/// Whether an unfinished rollback has already exchanged the trees back:
/// the journal recorded `rollback-swap` beginning and the checkout is not a subvolume.
/// Why:
///   A rerun reads the checkout's own state rather than trusting the journal's last line,
/// and a preview must
///  read the same fact to say what the real run would do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const swapped = journal.hasBegun('rollback-swap') && !isSubvolumeRoot(root);
/// ```
fn trees_are_already_swapped_back(run: &Run<'_>) -> bool {
    return run.journal.has_begun("rollback-swap") && !run.platform.is_subvolume_root(run.layout.root.as_path()).unwrap_or(false);
}

/// The refusal of a rollback whose old tree is gone,
/// which nothing can restore.
fn old_tree_missing(run: &Run<'_>) -> Refusal {
    return Refusal {
        cause: format!("the migration is recorded as exchanged, but the old tree {} is missing, so there is nothing to restore", run.layout.old_path.display()),
        remedy: String::from(
            "If the old tree was moved, move it back to that path and run the command again; if it was deleted, the checkout cannot be rolled back and the new subvolume is the only copy.",
        ),
    };
}

/// The refusal of a deletion whose old tree is a symbolic link:
/// the command moves the tree it deletes and never follows a link.
fn old_tree_is_a_link(run: &Run<'_>) -> Option<Refusal> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(&run.layout.old_path).ok()?;
    if !metadata.file_type().is_symlink() {
        return None;
    }
    return Some(Refusal {
        cause: format!("{} is a symbolic link, and the command deletes only a directory it moved itself", run.layout.old_path.display()),
        remedy: String::from("Inspect it and delete it by hand."),
    });
}

/// The refusal of a deletion while files this user may not read exist only in the old tree.
fn unreadable_only_in_the_old_tree(run: &Run<'_>, discard_unreadable: bool) -> Option<Refusal> {
    if run.unreadable.is_empty() || discard_unreadable {
        return None;
    }
    return Some(Refusal {
        cause: format!(
            "{} file(s) that this user may not read exist only in {} (listed in {})",
            run.unreadable.len(),
            run.layout.old_path.display(),
            run.layout.unreadable_path.display()
        ),
        remedy: String::from("Copy them out with the privilege they need (for example `sudo cp --archive`), or pass --discard-unreadable together with --remove-old to delete them with the old tree."),
    });
}

/// What:
///  The refusals the chosen mode meets before it changes anything,
/// in the words the real run uses.
/// Why:
///   A preview that answers before the refusals are looked at promises steps the real run
/// refuses,
///  which is the one thing a preview must not do;
/// a preview takes no claim and must stay read-only,
/// so it
///  runs exactly the refusals the real run computes first.
/// The refusals a later read of the old tree can add (a file that turns out
///  unreadable while the carry-back copies it) are not among them,
/// because finding them writes files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusals = refusalsOfTheMode(run, mode, discardUnreadable);
/// ```
pub fn refusals_of_the_mode(run: &Run<'_>, mode: MigrateMode, discard_unreadable: bool) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = match mode {
        MigrateMode::Rollback => rollback_refusals(run.layout, &run.journal),
        MigrateMode::RemoveOld => remove_old_refusals(run.layout, &run.journal),
        MigrateMode::Migrate => Vec::new(),
    };
    if !found.is_empty() {
        return found;
    }
    match mode {
        MigrateMode::Rollback => {
            // The real rollback reaches `swap_back` only when the journal records the exchange as done; before it there is
            // no old tree yet, and that state is the one the rollback finishes by deleting the staged subvolume.
            if run.journal.is_done("exchange") && !trees_are_already_swapped_back(run) && !run.layout.old_path.exists() {
                found.push(old_tree_missing(run));
            }
        }
        MigrateMode::RemoveOld => {
            if let Some(refusal) = old_tree_is_a_link(run) {
                found.push(refusal);
            }
            if let Some(refusal) = unreadable_only_in_the_old_tree(run, discard_unreadable) {
                found.push(refusal);
            }
        }
        MigrateMode::Migrate => {}
    }
    return found;
}

/// What:
///  The swap of the rollback:
/// carry back what is newer in the new root,
/// exchange the trees once,
/// rename the new
///  subvolume to its own name,
/// and record it.
///  `Ok(Some(refusals))` means nothing was changed.
/// Why:
///   The exchange cannot be repeated:
/// after it the checkout path holds the original directory,
/// and a second exchange
///  would put the new subvolume back and then delete the original.
/// The journal step `rollback-swap` is recorded before
///  the call,
/// and a rerun reads the checkout path to learn whether the call took effect:
/// a subvolume root there means
///  it did not,
/// anything else means it did and only the rename and the record are left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = await swapBack(run);
/// ```
fn swap_back(run: &mut Run<'_>, discard_unreadable: bool) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let root: PathBuf = run.layout.root.clone();
    let new_path: PathBuf = run.layout.new_path.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    let swapped: bool = trees_are_already_swapped_back(run);
    if !swapped {
        if !old_path.exists() {
            return Ok(Some(vec![old_tree_missing(run)]));
        }
        let carried: SyncStats = run.pass_unrecorded("carry back (files newer in the new root)", root.as_path(), old_path.as_path(), false, true)?;
        if !carried.unreadable.is_empty() && !discard_unreadable {
            let mut shown: Vec<String> = Vec::new();
            for path in carried.unreadable.iter().take(10) {
                shown.push(path.display().to_string());
            }
            return Ok(Some(vec![Refusal {
                cause: format!("{} file(s) that this user may not read exist only in the new subvolume {} and cannot be carried back: {}", carried.unreadable.len(), root.display(), shown.join(", ")),
                remedy: String::from("Copy them out with the privilege they need (for example `sudo cp --archive`), or pass --discard-unreadable together with --rollback to delete them with the new subvolume."),
            }]));
        }
        run.checkpoint("rollback-before-exchange")?;
        run.journal.mark("rollback-swap", "started")?;
        run.platform.exchange(root.as_path(), old_path.as_path())?;
        run.checkpoint("rollback-after-swap")?;
    }
    if old_path.exists() && !new_path.exists() {
        MigrateError::from_io_result(std::fs::rename(&old_path, &new_path), "rename", old_path.as_path())?;
    }
    run.checkpoint("rollback-after-rename")?;
    run.journal.mark("exchange", "rolled-back")?;
    run.checkpoint("rollback-after-exchange")?;
    return Ok(None);
}

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
pub fn rollback(run: &mut Run<'_>, discard_unreadable: bool) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let refusals: Vec<Refusal> = rollback_refusals(run.layout, &run.journal);
    if !refusals.is_empty() {
        return Ok(Some(refusals));
    }
    // Said once, before the first step that changes the checkout; a refusal above changes nothing, so it stopped before here.
    run.line(MIGRATE_RISK_NOTICE);
    let root: PathBuf = run.layout.root.clone();
    let new_path: PathBuf = run.layout.new_path.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    if run.journal.has_begun("exchange") && !run.journal.is_done("exchange") && run.platform.is_subvolume_root(root.as_path()).unwrap_or(false) {
        if new_path.exists() && !old_path.exists() {
            MigrateError::from_io_result(std::fs::rename(&new_path, &old_path), "rename", new_path.as_path())?;
        }
        run.journal.mark("exchange", "done")?;
    }
    if run.journal.is_done("exchange")
        && let Some(blocked) = swap_back(run, discard_unreadable)?
    {
        return Ok(Some(blocked));
    }
    // The restored tree is the one the migration locked: a stop before the exchange leaves its locks in the checkout, and
    // a stop between the exchange and the removal of the locks from the old tree restores a tree that still holds them.
    // Git would refuse every index write while they stay. Only locks that hold this migration's token are removed.
    let token: String = token_of(&run.journal);
    release_all(root.as_path(), token.as_str())?;
    // The carried-back index describes the new tree's inodes; this runs after a stop at any step above, once.
    if run.journal.state("exchange") == Some("rolled-back") && !run.journal.is_done("rollback-indexes") {
        let mut lines: Vec<String> = Vec::new();
        // The refresh reports each index into the list the lines are collected in; they are said even when the refresh
        //  stops, because each of them is a fact about an index that was looked at.
        let refreshed: Result<usize, MigrateError> = refresh_all_indexes(root.as_path(), IndexLock::Acquire, std::slice::from_ref(&new_path), &mut lines);
        for text in &lines {
            run.line(text.as_str());
        }
        let _count: usize = refreshed?;
        run.journal.mark("rollback-indexes", "done")?;
    }
    // The stage is the subvolume the migration created; a directory that replaced it is somebody else's, and the removal's
    // fallback would delete it with whatever it holds.
    if new_path.exists() && !run.platform.is_subvolume_root(new_path.as_path()).unwrap_or(false) {
        return Ok(Some(vec![Refusal {
            cause: format!(
                "{} is not a subvolume, and this rollback deletes only the subvolume the migration created; nothing was deleted",
                new_path.display()
            ),
            remedy: String::from(
                "If the directory is yours, move it away and run the command again; if it is the migration's own stage, delete it yourself and run the command again.",
            ),
        }]));
    }
    if new_path.exists() {
        run.line(format!("deleting the new subvolume {} (nested subvolumes first)", new_path.display()).as_str());
        remove_subvolume_tree(run, new_path.as_path())?;
    }
    run.journal.delete()?;
    remove_if_present(run.layout.unreadable_path.as_path())?;
    run.line(format!("rolled back: {} is the original directory again, and the migration journal is deleted.", root.display()).as_str());
    return Ok(None);
}

/// What:
///  Carry into the checkout what was written to the old tree after the catch-up finished.
/// Why:
///   A process that kept a descriptor in the old tree can write there until it is stopped,
/// and the catch-up that was
///  recorded as done cannot see those writes.
/// The pass is update-only:
/// it copies a file that is newer than the
///  checkout's,
/// and a file the checkout lacks only when it is no older than the catch-up (older ones were deleted from
///  the checkout on purpose).
/// A journal without the catch-up's time carries only the newer files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await carryLateWrites(run);
/// ```
fn carry_late_writes(run: &mut Run<'_>) -> Result<(), MigrateError> {
    let (root, old_path): (PathBuf, PathBuf) = (run.layout.root.clone(), run.layout.old_path.clone());
    let since: i64 = run.journal.seconds_of("catchup").unwrap_or(i64::MAX);
    let stats: SyncStats = run.pass_late("late writes (files written to the old tree after the catch-up)", old_path.as_path(), root.as_path(), since)?;
    if stats.files_copied > 0 || !stats.conflicts.is_empty() {
        run.line(format!("{} file(s) written to the old tree after the catch-up were carried into {}", stats.files_copied, root.display()).as_str());
    }
    return Ok(());
}

/// What:
///  Delete the old tree of a finished migration.
///  `Ok(None)` means it ran;
/// `Ok(Some(refusals))` means nothing was changed.
/// Why:
///   The files this user could not read exist only there,
/// so a non-empty list refuses unless the person discards them
///  explicitly.
/// The deletion (`remove_tree`) unlinks entries,
/// makes directories the user owns writable on the way,
/// and never follows symbolic links.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = await removeOldTree(run, discardUnreadable);
/// ```
pub fn remove_old_tree(run: &mut Run<'_>, discard_unreadable: bool) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let mut refusals: Vec<Refusal> = remove_old_refusals(run.layout, &run.journal);
    let old_path: PathBuf = run.layout.old_path.clone();
    if refusals.is_empty() {
        // The examination of the old tree is what the preview also reads, so a path that cannot be examined is an error here.
        MigrateError::from_io_result(std::fs::symlink_metadata(&old_path), "examine", old_path.as_path())?;
        if let Some(refusal) = old_tree_is_a_link(run) {
            return Ok(Some(vec![refusal]));
        }
        // The carry is the first step that writes; the refusal below comes after it, so the risk is said from here on.
        run.line(MIGRATE_RISK_NOTICE);
        carry_late_writes(run)?;
    }
    if refusals.is_empty() && let Some(refusal) = unreadable_only_in_the_old_tree(run, discard_unreadable) {
        refusals.push(refusal);
    }
    if !refusals.is_empty() {
        return Ok(Some(refusals));
    }
    run.line(format!("deleting {} (every file is unlinked; this can take minutes)", old_path.display()).as_str());
    MigrateError::from_io_result(remove_tree(&old_path), "delete", old_path.as_path())?;
    run.journal.delete()?;
    remove_if_present(run.layout.unreadable_path.as_path())?;
    run.line("the old tree is deleted and the migration journal is removed.");
    return Ok(None);
}

/// Rollback controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_rollback_tests.rs"]
mod tests;
