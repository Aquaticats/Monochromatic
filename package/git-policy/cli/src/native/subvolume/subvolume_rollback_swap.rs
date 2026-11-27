//! What:
//!  The exchange that turns the checkout back into the original tree,
//!  and its seamed form a control takes over at the moment before the exchange.
//! Why:
//!  The swap is the one step that cannot be undone by reading files afterwards,
//!  so the two directories are proved to be the pair the journal recorded first,
//!  the carry-back is finished before it,
//!  and the journal records it beginning and ending.
//!  The name the exchange works under is the one the pair was proved at,
//!  so a path swapped in between cannot redirect it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! swapBack(run); // the seamed form lets a control act right before the exchange
//! ```

use crate::subvolume_error::MigrateError;
use crate::subvolume_preflight::{
    INSTALLED_ROOT,
    Refusal,
};
use crate::subvolume_rollback_carry_back::carry_back_left_out;
use crate::subvolume_rollback_identity::{
    PairLook,
    object_form,
    pair_refusal,
    resumed_old_tree_refusal,
};
use crate::subvolume_rollback_refusals::{
    old_tree_missing,
    trees_are_already_swapped_back,
};
use crate::subvolume_rollback_removal::nothing_at_all;
use crate::subvolume_run::Run;
use crate::subvolume_sync::SyncStats;
use std::path::PathBuf;

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
pub(super) fn swap_back(
    run: &mut Run<'_>,
    discard_unreadable: bool,
    discard_left_out: bool,
) -> Result<Option<Vec<Refusal>>, MigrateError> {
    return swap_back_seamed(
        run,
        discard_unreadable,
        discard_left_out,
        &mut nothing_at_all,
    );
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
pub(super) fn swap_back_seamed(
    run: &mut Run<'_>,
    discard_unreadable: bool,
    discard_left_out: bool,
    before_exchange: &mut dyn FnMut(),
) -> Result<Option<Vec<Refusal>>, MigrateError> {
    let root: PathBuf = run.layout.root.clone();
    let new_path: PathBuf = run.layout.new_path.clone();
    let old_path: PathBuf = run.layout.old_path.clone();
    let swapped: bool = trees_are_already_swapped_back(run);
    if swapped {
        // A resumed rollback renames what the old tree's name holds to the new subvolume's name and then deletes it, so that
        // object must be the one the journal recorded installing: a kill alone leaves this state, and a directory the person
        // put there while the command was down is theirs.
        if let Some(refusal) = resumed_old_tree_refusal(run, old_path.as_path(), new_path.as_path())
        {
            return Ok(Some(vec![refusal]));
        }
    }
    if !swapped {
        if !old_path.exists() {
            return Ok(Some(vec![old_tree_missing(run)]));
        }
        let carried: SyncStats = run.pass_unrecorded(
            "carry back (files newer in the new root)",
            root.as_path(),
            old_path.as_path(),
            false,
            true,
        )?;
        if !carried.unreadable.is_empty() && !discard_unreadable {
            let mut shown: Vec<String> = Vec::new();
            for path in carried.unreadable.iter().take(10) {
                shown.push(path.display().to_string());
            }
            return Ok(Some(vec![Refusal {
                cause: format!(
                    "{} file(s) that this user may not read exist only in the new subvolume {} and cannot be carried back: {}",
                    carried.unreadable.len(),
                    root.display(),
                    shown.join(", ")
                ),
                remedy: String::from(
                    "Copy them out with the privilege they need (for example `sudo cp --archive`), or pass --discard-unreadable together with --rollback to delete them with the new subvolume.",
                ),
            }]));
        }
        if let Some(refusal) = carry_back_left_out(run, &carried, discard_left_out) {
            return Ok(Some(vec![refusal]));
        }
        run.checkpoint("rollback-before-exchange")?;
        before_exchange();
        // The last look before the exchange, and the identities it reads are what the exchange must produce.
        let before: (Option<String>, Option<String>) =
            (object_form(root.as_path()), object_form(old_path.as_path()));
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
        let after: (Option<String>, Option<String>) =
            (object_form(root.as_path()), object_form(old_path.as_path()));
        if after.0 != before.1 || after.1 != before.0 {
            let undone: Result<(), crate::fs_primitive_error::PrimitiveError> =
                run.platform.exchange(root.as_path(), old_path.as_path());
            let note: &str = if undone.is_ok() {
                "The exchange was undone"
            } else {
                "The exchange could not be undone, so read the paths named here before you run anything else"
            };
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
                remedy: String::from(
                    "Check both paths, move the directory the migration installed back to the checkout's path, and run the command again. Nothing was deleted.",
                ),
            }]));
        }
    }
    if old_path.exists() && !new_path.exists() {
        MigrateError::from_io_result(
            std::fs::rename(&old_path, &new_path),
            "rename",
            old_path.as_path(),
        )?;
    }
    run.checkpoint("rollback-after-rename")?;
    run.journal.mark("exchange", "rolled-back")?;
    run.checkpoint("rollback-after-exchange")?;
    return Ok(None);
}
