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
use crate::subvolume_arguments::{MIGRATE_RISK_NOTICE, MigrateMode};
use crate::subvolume_error::MigrateError;
use crate::subvolume_index_files::IndexLock;
use crate::subvolume_index_refresh_of_every_worktree::refresh_all_indexes;
use crate::subvolume_lock::{release_all, token_of};
use crate::subvolume_preflight::{INSTALLED_ROOT, Refusal, remove_old_refusals, rollback_refusals};
use crate::subvolume_tree_proof::is_this_programs_own;
use crate::subvolume_rollback_identity::{
    PairLook, PinnedRoot, installed_identity_refusal, object_form, pair_refusal, pin_root, pinned_root_moved, resumed_old_tree_refusal, shown_left_out, uncarried_cause, uncarried_remedy,
    unproved_cause, unproved_remedy,
};
use crate::subvolume_rollback_stage::stage_is_not_a_subvolume;
use crate::subvolume_rollback_subvolume_removal::{remove_if_present, remove_subvolume_tree};
use crate::subvolume_remove::remove_tree;
use crate::subvolume_run::Run;
use crate::subvolume_sync::SyncStats;
use std::path::{Path, PathBuf};

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

/// What:
///  The paths a person has already accepted may go,
///  which the proof of the deletion does not name.
/// Why:
///  Each flag answers for its own record and nothing else,
///  so an entry nobody recorded is still named even when a flag was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const accepted = new Set([...(discardUnreadable ? run.unreadable : []), ...(discardLeftOut ? run.uncarried() : [])]);
/// ```
fn accepted_paths(run: &mut Run<'_>, discard_unreadable: bool, discard_left_out: bool) -> Result<std::collections::BTreeSet<PathBuf>, MigrateError> {
    let mut accepted: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    if discard_unreadable {
        accepted.extend(run.unreadable.iter().cloned());
    }
    if discard_left_out {
        accepted.extend(run.uncarried()?);
    }
    return Ok(accepted);
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
/// The refusals a later read of the old tree can add are not among them:
/// a file that turns out unreadable while the carry-back copies it,
/// and an entry the carry-back keeps because the original tree holds another kind of entry at its path.
/// Finding either writes files.
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
            if let Some(refusal) = stage_is_not_a_subvolume(run) {
                found.push(refusal);
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

/// The refusal of a rollback whose carry-back left entries of the new subvolume behind,
/// or nothing when it left none or the person accepted the loss.
/// Why:
///   The carry-back keeps the original tree's entry where the two trees hold different kinds of entry at one path,
/// so the new subvolume's directory and everything in it is the only copy of those files,
/// and this rollback deletes that subvolume:
/// the same rule the deletion of the old tree follows,
/// and the same flag accepts the loss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = carryBackLeftOut(run, carried, discardLeftOut);
/// ```
fn carry_back_left_out(run: &Run<'_>, carried: &SyncStats, discard_left_out: bool) -> Option<Refusal> {
    if discard_left_out
        || (carried.left_out.is_empty() && carried.conflicts.is_empty() && carried.kept_at_the_destination.is_empty() && carried.changed_while_passing.is_empty())
    {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    if !carried.left_out.is_empty() {
        parts.push(format!("{} entr(ies) that {} lacks ({})", carried.left_out.len(), run.layout.old_path.display(), shown_left_out(carried.left_out.iter())));
    }
    if !carried.conflicts.is_empty() {
        parts.push(format!(
            "{} entr(ies) whose path in {} holds another kind of entry, kept as the newer one ({})",
            carried.conflicts.len(),
            run.layout.old_path.display(),
            shown_left_out(carried.conflicts.iter())
        ));
    }
    if !carried.kept_at_the_destination.is_empty() {
        parts.push(format!(
            "{} entr(ies) the carry did not place because {} came to hold another entry at their path while it ran, and that entry stays ({})",
            carried.kept_at_the_destination.len(),
            run.layout.old_path.display(),
            shown_left_out(carried.kept_at_the_destination.iter())
        ));
    }
    if !carried.changed_while_passing.is_empty() {
        parts.push(format!(
            "{} entr(ies) the carry did not place because they were another kind of entry when it reached them ({})",
            carried.changed_while_passing.len(),
            shown_left_out(carried.changed_while_passing.iter())
        ));
    }
    return Some(Refusal {
        cause: format!("{} did not reach {}: {} (this rollback deletes the new subvolume with them)", run.layout.root.display(), run.layout.old_path.display(), parts.join("; ")),
        remedy: String::from(
            "Copy anything you want to keep out of the new subvolume first. If what the original tree keeps there is there on purpose, pass --discard-left-out together with --rollback to delete the new subvolume with it; you can also delete the new subvolume yourself.",
        ),
    });
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
fn swap_back(run: &mut Run<'_>, discard_unreadable: bool, discard_left_out: bool) -> Result<Option<Vec<Refusal>>, MigrateError> {
    return swap_back_seamed(run, discard_unreadable, discard_left_out, &mut nothing_at_all);
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
fn swap_back_seamed(run: &mut Run<'_>, discard_unreadable: bool, discard_left_out: bool, before_exchange: &mut dyn FnMut()) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let root: PathBuf = run.layout.root.clone();
    let new_path: PathBuf = run.layout.new_path.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    let swapped: bool = trees_are_already_swapped_back(run);
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
            return Ok(Some(vec![old_tree_missing(run)]));
        }
        // The carry back brings what is newer in the new root, as it always did; what it cannot bring is not deleted on a
        //  modification time, because the proof against the subvolume this deletes reads every byte it holds.
        // The carry back replaces nothing: the tree it writes into is the one the person keeps, so a modification time
        //  is no reason to delete a byte of it. What differs is named, and the proof against the subvolume this deletes
        //  refuses over it unless the person accepts it.
        let mut carried: SyncStats = run.pass_carry_back("carry back (files newer in the new root)", root.as_path(), old_path.as_path())?;
        // What this program writes for itself is nobody's bytes to report: the index it rewrote, the locks a Git holds,
        //  the partial copy it was making. The proof leaves the same paths out, so the two agree.
        let root_for_the_filter: PathBuf = root.clone();
        let the_roots_own_kind = || return std::fs::symlink_metadata(root_for_the_filter.as_path()).expect("the root").file_type();
        carried.kept_at_the_destination.retain(|path| {
            return !is_this_programs_own(root_for_the_filter.as_path(), path, std::fs::symlink_metadata(root_for_the_filter.join(path)).map(|found| return found.file_type()).unwrap_or_else(|_gone| return the_roots_own_kind()));
        });
        carried.changed_while_passing.retain(|path| {
            return !is_this_programs_own(root_for_the_filter.as_path(), path, std::fs::symlink_metadata(root_for_the_filter.join(path)).map(|found| return found.file_type()).unwrap_or_else(|_gone| return the_roots_own_kind()));
        });
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
        if let Some(refusal) = carry_back_left_out(run, &carried, discard_left_out) {
            return Ok(Some(vec![refusal]));
        }
        // What the carry back kept is what the proof against the subvolume names,
        //  so the person's flag answers for exactly those paths.
        for path in carried.kept_at_the_destination.iter() {
            run.kept_at_the_destination.insert(path.clone());
        }
        for path in carried.changed_while_passing.iter() {
            run.changed_while_passing.insert(path.clone());
        }
        for path in carried.left_out.iter() {
            run.left_out.insert(path.clone());
        }
        for path in carried.conflicts.iter() {
            run.conflicts.insert(path.clone());
        }
        for path in carried.unreadable.iter() {
            run.unreadable.insert(path.clone());
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
pub fn rollback(run: &mut Run<'_>, discard_unreadable: bool, discard_left_out: bool) -> Result<Option<Vec<Refusal>>, MigrateError> {
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
    if run.journal.state("exchange") == Some("rolled-back") && !run.journal.is_done("rollback-indexes") {
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
        // The same rule in the other direction: every byte of the subvolume this deletes is read against the restored
        //  original directory, and what the person accepted with the flag is what the carry back recorded and no more.
        let accepted: std::collections::BTreeSet<PathBuf> = accepted_paths(run, discard_unreadable, discard_left_out)?;
        let unproved: Vec<PathBuf> = crate::subvolume_tree_proof::entries_not_in(new_path.as_path(), root.as_path(), &accepted)?;
        if !unproved.is_empty() {
            return Ok(Some(vec![Refusal { cause: unproved_cause(run, &unproved), remedy: unproved_remedy() }]));
        }
        run.line(format!("deleting the new subvolume {} (nested subvolumes first)", new_path.display()).as_str());
        remove_subvolume_tree(run, new_path.as_path())?;
    }
    run.journal.delete()?;
    remove_if_present(run.layout.unreadable_path.as_path())?;
    // The record describes entries of a migration that this rollback undid:
    //  the old tree is the checkout again, so nothing in that record is still true,
    //  and a later migration must not report it.
    remove_if_present(run.layout.left_out_path.as_path())?;
    run.line(format!("rolled back: {} is the original directory again, and the migration journal is deleted.", root.display()).as_str());
    let parked_copies: Vec<(PathBuf, PathBuf)> = run.parked.iter().map(|(parked, original)| return (parked.clone(), original.clone())).collect();
    for (parked, original) in parked_copies {
        run.line(format!("{} holds the parked copy {parked}, a copy of {original} that this command parked when a replacement was interrupted, carried back with the rest; look at it, and delete it when nothing in it is yours", root.display(), parked = parked.display(), original = original.display()).as_str());
    }
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
        // The examination of the old tree is what the preview also reads, so a path that cannot be examined is an error here.
        MigrateError::from_io_result(std::fs::symlink_metadata(&old_path), "examine", old_path.as_path())?;
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
    // The files this user may not read are asked first: they have their own flag, and their record is the unreadable
    //  list rather than the list of entries no pass carried.
    if refusals.is_empty() && let Some(refusal) = unreadable_only_in_the_old_tree(run, discard_unreadable) {
        refusals.push(refusal);
    }
    // The one question the deletion asks of the rest: every entry this run did not carry and every path the record
    //  beside the checkout holds, read now, so a run that has passed nothing refuses as the run that learned it would.
    if refusals.is_empty() && !discard_left_out {
        let uncarried: std::collections::BTreeSet<PathBuf> = run.uncarried()?;
        if !uncarried.is_empty() {
            refusals.push(Refusal { cause: uncarried_cause(run, &uncarried), remedy: uncarried_remedy() });
        }
    }
    if !refusals.is_empty() {
        return Ok(Some(refusals));
    }
    // The rule this deletion follows, and the same one for every way a name reached the checkout:
    //  every byte the old tree holds is read against the checkout at this moment,
    //  and a modification time proves nothing,
    //  because a person can set one with `touch`, `cp -p`, `tar -x` or `rsync -a`.
    //  What a person accepted with `--discard-left-out` is the recorded paths and no others.
    let accepted: std::collections::BTreeSet<PathBuf> = accepted_paths(run, discard_unreadable, discard_left_out)?;
    let unproved: Vec<PathBuf> = crate::subvolume_tree_proof::entries_not_in(old_path.as_path(), run.layout.root.as_path(), &accepted)?;
    if !unproved.is_empty() {
        return Ok(Some(vec![Refusal { cause: unproved_cause(run, &unproved), remedy: unproved_remedy() }]));
    }
    // A parked copy stands in the checkout and is no entry of the old tree, so it never holds this deletion back;
    //  it is named, because nothing here deletes it and the person has to look at it.
    let parked_copies: Vec<(PathBuf, PathBuf)> = run.parked.iter().map(|(parked, original)| return (parked.clone(), original.clone())).collect();
    for (parked, original) in parked_copies {
        run.line(format!("{} keeps the parked copy {parked}, a copy of {original} that this command parked when a replacement was interrupted; look at it, and delete it when nothing in it is yours", run.layout.root.display(), parked = parked.display(), original = original.display()).as_str());
    }
    run.line(format!("deleting {} (every file is unlinked; this can take minutes)", old_path.display()).as_str());
    MigrateError::from_io_result(remove_tree(&old_path), "delete", old_path.as_path())?;
    run.journal.delete()?;
    remove_if_present(run.layout.unreadable_path.as_path())?;
    remove_if_present(run.layout.left_out_path.as_path())?;
    // The record of parked copies stays while it names one: what it names is in the checkout, and a later command tells
    //  the person about it again.
    if run.parked.is_empty() {
        remove_if_present(run.layout.parked_path.as_path())?;
    }
    run.line("the old tree is deleted and the migration journal is removed.");
    return Ok(None);
}

/// Rollback controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_rollback_tests.rs"]
mod tests;
