//! What:
//!  Which directory the rollback deletes as the migration's stage,
//! and the refusal when that directory is not the subvolume the migration
//!  made.
//! Why:
//!  The rollback's removal falls back to deleting a plain directory when the name holds no subvolume,
//! so a directory
//!  somebody else put there would be deleted with whatever it holds;
//! the journal names the subvolume the migration made,
//!  and only that one is deleted,
//! whatever the name holds now.
//! The identity is compared where the deletion happens,
//!  because a preview answers from the same facts the run acts on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = stageRefusal(run); // undefined when the name holds the recorded subvolume
//! ```

use crate::subvolume_preflight::Refusal;
use crate::subvolume_run::Run;
use std::path::PathBuf;

/// What:
///  The directory that the rollback deletes as the stage.
/// Why:
///   An exchange that began and did not finish leaves the checkout holding the subvolume this migration made and the
///  stage's name holding the original directory;
/// the rollback renames the original aside,
/// swaps the trees back and deletes
///  the subvolume from the checkout's place,
/// so the object it will delete is the one the checkout holds now.
/// Every other state deletes what stands at the stage's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deleted = stageThatTheRollbackDeletes(run);
/// ```
fn stage_that_the_rollback_deletes(run: &Run<'_>) -> PathBuf {
    let root: PathBuf = run.layout.root.clone();
    if run.journal.has_begun("exchange") && !run.journal.is_done("exchange") && run.platform.is_subvolume_root(root.as_path()).unwrap_or(false) {
        return root;
    }
    return run.layout.new_path.clone();
}

/// The refusal of a stage that is not a subvolume:
/// the rollback deletes only the subvolume the migration created.
/// Why:
///   The removal falls back to deleting a plain directory when the name holds no subvolume,
/// so a directory that
///  replaced the stage would be deleted with whatever it holds;
/// and a journal that records no identity names no subvolume at all,
/// so nothing can be shown to be the one this rollback deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = stageIsNotASubvolume(run);
/// ```
pub(crate) fn stage_is_not_a_subvolume(run: &Run<'_>) -> Option<Refusal> {
    let stage: PathBuf = stage_that_the_rollback_deletes(run);
    if !stage.exists() {
        return None;
    }
    if !run.platform.is_subvolume_root(stage.as_path()).unwrap_or(false) {
        return Some(Refusal {
            cause: format!(
                "{} is not a subvolume, and this rollback deletes only the subvolume the migration created; nothing was deleted",
                stage.display()
            ),
            remedy: String::from(
                "If the directory is yours, move it away and run the command again; if it is the migration's own stage, delete it yourself and run the command again.",
            ),
        });
    }
    // The journal names the subvolume this migration made; a subvolume another process put at the same name is not it, and
    // an identity that cannot be read is treated the same way rather than deleting what it cannot name. A journal that
    // records none at all is one a kill left before the identity was written, or one an earlier release wrote.
    let Some(recorded) = run.journal.stage_identity() else {
        return Some(Refusal {
            cause: format!(
                "the journal records no identity for the subvolume this migration made, so {} cannot be shown to be the one this rollback deletes; nothing was deleted",
                stage.display()
            ),
            remedy: String::from(
                "If the subvolume is the migration's own stage, delete it yourself and run the command again; if it is yours, move it away and delete the journal, or run `--remove-old` after a finished migration.",
            ),
        });
    };
    let found: Option<crate::fs_beneath::ObjectId> = crate::worktree_convert_phases::identity_of_path(stage.as_path());
    if let Some(identity) = found
        && crate::worktree_convert_journal::is_recorded_object(&recorded, &identity)
    {
        return None;
    }
    return Some(Refusal {
        cause: format!(
            "{} is not the subvolume the migration created (the journal records another identity), and this rollback deletes only that one; nothing was deleted",
            stage.display()
        ),
        remedy: String::from(
            "If the subvolume is yours, move it away and run the command again; if it is the migration's own stage, delete it yourself and run the command again.",
        ),
    });
}
