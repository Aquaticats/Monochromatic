//! What:
//!  The refusals a rollback and a deletion of the old tree answer with:
//!  the old tree gone or turned into a symbolic link,
//!  files this user may not read that exist only in the old tree,
//!  a carry-back that left entries of the new subvolume behind,
//!  and whether an unfinished rollback has already exchanged the trees back.
//! Why:
//!  Either step moves the person's work,
//!  so each one is refused before anything changes when the two directories are not the pair the journal records,
//!  and every refusal names its cause and every way forward.
//!  A rerun reads the checkout's own state rather than trusting the journal's last line,
//!  and a preview reads the same fact to say what the real run would do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const swapped = journal.hasBegun('rollback-swap') && !isSubvolumeRoot(root);
//! const refusals = refusalsOfTheMode(run, mode);
//! ```

use crate::subvolume_arguments::MigrateMode;
use crate::subvolume_preflight::{
    Refusal,
    remove_old_refusals,
    rollback_refusals,
};
use crate::subvolume_rollback_stage::stage_is_not_a_subvolume;
use crate::subvolume_run::Run;

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
pub(super) fn trees_are_already_swapped_back(run: &Run<'_>) -> bool {
    return run.journal.has_begun("rollback-swap")
        && !run
            .platform
            .is_subvolume_root(run.layout.root.as_path())
            .unwrap_or(false);
}

/// The refusal of a rollback whose old tree is gone,
/// which nothing can restore.
pub(super) fn old_tree_missing(run: &Run<'_>) -> Refusal {
    return Refusal {
        cause: format!(
            "the migration is recorded as exchanged, but the old tree {} is missing, so there is nothing to restore",
            run.layout.old_path.display()
        ),
        remedy: String::from(
            "If the old tree was moved, move it back to that path and run the command again; if it was deleted, the checkout cannot be rolled back and the new subvolume is the only copy.",
        ),
    };
}

/// The refusal of a deletion whose old tree is a symbolic link:
/// the command moves the tree it deletes and never follows a link.
pub(super) fn old_tree_is_a_link(run: &Run<'_>) -> Option<Refusal> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(&run.layout.old_path).ok()?;
    if !metadata.file_type().is_symlink() {
        return None;
    }
    return Some(Refusal {
        cause: format!(
            "{} is a symbolic link, and the command deletes only a directory it moved itself",
            run.layout.old_path.display()
        ),
        remedy: String::from("Inspect it and delete it by hand."),
    });
}

/// The refusal of a deletion while files this user may not read exist only in the old tree.
pub(super) fn unreadable_only_in_the_old_tree(
    run: &Run<'_>,
    discard_unreadable: bool,
) -> Option<Refusal> {
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
        remedy: String::from(
            "Copy them out with the privilege they need (for example `sudo cp --archive`), or pass --discard-unreadable together with --remove-old to delete them with the old tree.",
        ),
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
pub fn refusals_of_the_mode(
    run: &Run<'_>,
    mode: MigrateMode,
    discard_unreadable: bool,
) -> Vec<Refusal> {
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
            if run.journal.is_done("exchange")
                && !trees_are_already_swapped_back(run)
                && !run.layout.old_path.exists()
            {
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
