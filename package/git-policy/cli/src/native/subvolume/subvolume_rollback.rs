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

use crate::fs_beneath::handle_path;
use crate::subvolume_arguments::MIGRATE_RISK_NOTICE;
use crate::subvolume_error::MigrateError;
use crate::subvolume_index_files::IndexLock;
use crate::subvolume_index_refresh_of_every_worktree::refresh_all_indexes;
use crate::subvolume_lock::{release_all, token_of};
use crate::subvolume_preflight::{INSTALLED_ROOT, Refusal, remove_old_refusals, rollback_refusals};
use crate::subvolume_rollback_identity::{PairLook, PinnedRoot, object_form, pair_refusal, pin_root, pinned_root_moved, resumed_old_tree_refusal};
use crate::subvolume_remove::remove_tree;
use crate::subvolume_run::Run;
use crate::subvolume_sync::SyncStats;
use std::path::{Path, PathBuf};

/// How many left-out entries a refusal spells out before it names the file that lists them all.
const SHOWN_LEFT_OUT: usize = 10;

/// The first few left-out entries,
/// joined for a refusal,
/// and how many more there are.
fn shown_left_out<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> String {
    let mut shown: Vec<String> = Vec::new();
    let mut more: usize = 0;
    for path in paths {
        if shown.len() < SHOWN_LEFT_OUT {
            shown.push(path.display().to_string());
        } else {
            more += 1;
        }
    }
    if more > 0 {
        shown.push(format!("and {more} more"));
    }
    return shown.join(", ");
}

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
    return swap_back_seamed(run, discard_unreadable, &mut nothing_at_all);
}

/// What:
///  `swap_back`,
/// with a call after the carry-back and before the last look at the two directories,
/// where a control
///  moves the checkout aside and puts another directory at its path.
/// Why:
///   The exchange takes two names,
/// so nothing binds it to the directories that were checked;
/// the last look
///  immediately before it keeps a replacement from being exchanged and deleted,
/// and a test can only enter that window
///  through a seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// swapBackSeamed(run, discardUnreadable, beforeExchange);
/// ```
fn swap_back_seamed(run: &mut Run<'_>, discard_unreadable: bool, before_exchange: &mut dyn FnMut()) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let root: PathBuf = run.layout.root.clone();
    let new_path: PathBuf = run.layout.new_path.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    let swapped: bool = run.journal.has_begun("rollback-swap") && !run.platform.is_subvolume_root(root.as_path()).unwrap_or(false);
    if swapped {
        // A resumed rollback renames what the old tree's name holds to the new subvolume's name and then deletes it, so that
        // object must be the one the journal recorded installing: a kill alone leaves this state, and a directory the person
        // put there while the command was down is theirs.
        if let Some(refusal) = resumed_old_tree_refusal(run, old_path.as_path(), new_path.as_path()) {
            return Ok(Some(vec![refusal]));
        }
    }
    if !swapped {
        if !old_path.exists() {
            return Ok(Some(vec![Refusal {
                cause: format!("the migration is recorded as exchanged, but the old tree {} is missing, so there is nothing to restore", old_path.display()),
                remedy: String::from("If the old tree was moved, move it back to that path and run the command again; if it was deleted, the checkout cannot be rolled back and the new subvolume is the only copy."),
            }]));
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
        before_exchange();
        // The last look before the exchange, and the identities it reads are what the exchange must produce.
        let before: (Option<String>, Option<String>) = (object_form(root.as_path()), object_form(old_path.as_path()));
        let installed: Option<&str> = run.journal.value(INSTALLED_ROOT);
        if let Some(refusal) = pair_refusal(PairLook {
            root: root.as_path(),
            old_path: old_path.as_path(),
            at_root: before.0.clone(),
            at_old: before.1.clone(),
            installed,
            what: "before",
        }) {
            return Ok(Some(vec![refusal]));
        }
        run.journal.mark("rollback-swap", "started")?;
        run.platform.exchange(root.as_path(), old_path.as_path())?;
        run.checkpoint("rollback-after-swap")?;
        // An exchange of names can take a directory put there in the moment before it, so what went where is read back: a
        // wrong one is exchanged back and nothing is deleted.
        let after: (Option<String>, Option<String>) = (object_form(root.as_path()), object_form(old_path.as_path()));
        if after.0 != before.1 || after.1 != before.0 {
            let undone: Result<(), crate::fs_primitive_error::PrimitiveError> = run.platform.exchange(root.as_path(), old_path.as_path());
            let note: &str = if undone.is_ok() { "The exchange was undone" } else { "The exchange could not be undone, so read the paths named here before you run anything else" };
            return Ok(Some(vec![Refusal {
                cause: format!(
                    "{} and {} did not exchange their directories: they held {} and {}, and after the exchange they hold {} and {}. {note}.",
                    root.display(),
                    old_path.display(),
                    before.0.as_deref().unwrap_or("a name that leads nowhere"),
                    before.1.as_deref().unwrap_or("a name that leads nowhere"),
                    after.0.as_deref().unwrap_or("a name that leads nowhere"),
                    after.1.as_deref().unwrap_or("a name that leads nowhere")
                ),
                remedy: String::from("Check both paths, move the directory the migration installed back to the checkout's path, and run the command again. Nothing was deleted."),
            }]));
        }
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
        // The refresh reports each index into the list the lines are collected in.
        refresh_all_indexes(root.as_path(), IndexLock::Acquire, std::slice::from_ref(&new_path), &mut lines);
        for text in &lines {
            run.line(text.as_str());
        }
        run.journal.mark("rollback-indexes", "done")?;
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
/// await carryLateWrites(run, pinned, shown);
/// ```
fn carry_late_writes(run: &mut Run<'_>, destination: &Path, shown: &Path) -> Result<(), MigrateError> {
    let old_path: PathBuf = run.layout.old_path.clone();
    let since: i64 = run.journal.seconds_of("catchup").unwrap_or(i64::MAX);
    let stats: SyncStats = run.pass_late("late writes (files written to the old tree after the catch-up)", old_path.as_path(), destination, since)?;
    if stats.files_copied > 0 || !stats.conflicts.is_empty() {
        run.line(format!("{} file(s) written to the old tree after the catch-up were carried into {}", stats.files_copied, shown.display()).as_str());
    }
    return Ok(());
}

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
pub fn remove_old_tree(run: &mut Run<'_>, discard_unreadable: bool, discard_left_out: bool) -> Result<Option<Vec<Refusal>>, MigrateError> {
    return remove_old_tree_seamed(run, discard_unreadable, discard_left_out, &mut nothing_at_all);
}

/// A step that does nothing,
/// for the run between the checkout being pinned and the carry.
fn nothing_at_all() {}

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
pub fn remove_old_tree_seamed(run: &mut Run<'_>, discard_unreadable: bool, discard_left_out: bool, after_pinned: &mut dyn FnMut()) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let mut refusals: Vec<Refusal> = remove_old_refusals(run.layout, &run.journal);
    let old_path: PathBuf = run.layout.old_path.clone();
    let mut pinned: Option<PinnedRoot> = None;
    if refusals.is_empty() {
        let metadata: std::fs::Metadata = MigrateError::from_io_result(std::fs::symlink_metadata(&old_path), "examine", old_path.as_path())?;
        if metadata.file_type().is_symlink() {
            return Ok(Some(vec![Refusal {
                cause: format!("{} is a symbolic link, and the command deletes only a directory it moved itself", old_path.display()),
                remedy: String::from("Inspect it and delete it by hand."),
            }]));
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
    if refusals.is_empty() && !run.left_out.is_empty() && !discard_left_out {
        refusals.push(Refusal {
            cause: format!(
                "{} entr(ies) of {} are absent from {} and older than the catch-up, so they were left out of the carry: {} (the whole list is in {})",
                run.left_out.len(),
                run.layout.old_path.display(),
                run.layout.root.display(),
                shown_left_out(run.left_out.iter()),
                run.layout.left_out_path.display()
            ),
            remedy: String::from(
                "Copy anything you want to keep out of the old tree first. If they are files you deleted from the checkout on purpose, pass --discard-left-out together with --remove-old to delete the old tree with them; you can also delete the old tree yourself.",
            ),
        });
    }
    if refusals.is_empty() && !run.unreadable.is_empty() && !discard_unreadable {
        refusals.push(Refusal {
            cause: format!("{} file(s) that this user may not read exist only in {} (listed in {})", run.unreadable.len(), run.layout.old_path.display(), run.layout.unreadable_path.display()),
            remedy: String::from("Copy them out with the privilege they need (for example `sudo cp --archive`), or pass --discard-unreadable together with --remove-old to delete them with the old tree."),
        });
    }
    if !refusals.is_empty() {
        return Ok(Some(refusals));
    }
    run.line(format!("deleting {} (every file is unlinked; this can take minutes)", old_path.display()).as_str());
    MigrateError::from_io_result(remove_tree(&old_path), "delete", old_path.as_path())?;
    run.journal.delete()?;
    remove_if_present(run.layout.unreadable_path.as_path())?;
    remove_if_present(run.layout.left_out_path.as_path())?;
    run.line("the old tree is deleted and the migration journal is removed.");
    return Ok(None);
}

/// Rollback controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_rollback_tests.rs"]
mod tests;
