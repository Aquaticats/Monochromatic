//! What:
//!  Carrying the person's work back out of the new subvolume:
//!  the entries the copy-back left out,
//!  and the writes that landed after the copy-back was planned.
//! Why:
//!  A rollback deletes the directory that stood as the checkout,
//!  so a file written there after the carry-back would be lost;
//!  the late writes are carried once more before the exchange,
//!  and what could not be carried is reported with both ways forward rather than dropped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! carryBackLeftOut(run); carryLateWrites(run);
//! ```

use crate::subvolume_error::MigrateError;
use crate::subvolume_preflight::Refusal;
use crate::subvolume_rollback_identity::shown_left_out;
use crate::subvolume_run::Run;
use crate::subvolume_sync::SyncStats;
use std::path::{
    Path,
    PathBuf,
};

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
pub(super) fn carry_back_left_out(
    run: &Run<'_>,
    carried: &SyncStats,
    discard_left_out: bool,
) -> Option<Refusal> {
    if discard_left_out || (carried.left_out.is_empty() && carried.conflicts.is_empty()) {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    if !carried.left_out.is_empty() {
        parts.push(format!(
            "{} entr(ies) that {} lacks ({})",
            carried.left_out.len(),
            run.layout.old_path.display(),
            shown_left_out(carried.left_out.iter())
        ));
    }
    if !carried.conflicts.is_empty() {
        parts.push(format!(
            "{} entr(ies) whose path in {} holds another kind of entry, kept as the newer one ({})",
            carried.conflicts.len(),
            run.layout.old_path.display(),
            shown_left_out(carried.conflicts.iter())
        ));
    }
    return Some(Refusal {
        cause: format!(
            "{} did not reach {}: {} (this rollback deletes the new subvolume with them)",
            run.layout.root.display(),
            run.layout.old_path.display(),
            parts.join("; ")
        ),
        remedy: String::from(
            "Copy anything you want to keep out of the new subvolume first. If what the original tree keeps there is there on purpose, pass --discard-left-out together with --rollback to delete the new subvolume with it; you can also delete the new subvolume yourself.",
        ),
    });
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
pub(super) fn carry_late_writes(
    run: &mut Run<'_>,
    destination: &Path,
    shown: &Path,
) -> Result<(), MigrateError> {
    let old_path: PathBuf = run.layout.old_path.clone();
    let since: i64 = run.journal.seconds_of("catchup").unwrap_or(i64::MAX);
    let stats: SyncStats = run.pass_late(
        "late writes (files written to the old tree after the catch-up)",
        old_path.as_path(),
        destination,
        since,
    )?;
    if stats.files_copied > 0 || !stats.conflicts.is_empty() {
        run.line(
            format!(
                "{} file(s) written to the old tree after the catch-up were carried into {}",
                stats.files_copied,
                shown.display()
            )
            .as_str(),
        );
    }
    return Ok(());
}
