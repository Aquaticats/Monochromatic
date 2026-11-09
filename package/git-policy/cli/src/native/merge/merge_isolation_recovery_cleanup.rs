//! What:
//!  What recovery writes to the records and side refs of a merge it settled,
//!  and the refusal that says whether anything of the merge is installed.
//! Why:
//!  A record and its side ref go together,
//! a refusal must carry whether the merge began,
//! and a saved merge whose branch
//!  advanced drops only its record;
//! the wording of all three lives beside the decision in a file the decision's own file does not
//!  carry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await dropRecordAndSideRef({ place, root, record, intent }); return partlyInstalled(error);
//! ```

use crate::diagnostic_log::debug;
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent};
use crate::merge_isolation_recovery_steps::remove_record;
use crate::merge_isolation_recovery_steps::{MergeRefusal, RecoveryPlace};
use crate::recovery_error::RecoveryError;
use crate::transaction_git::{GitRequest, run_git_checked};
use std::path::Path;

/// What:
///  The refusal of a step that ran after the index was written,
///  so the merge is partly installed.
/// Why:
///   The branch write and the record's removal come after the index and the files,
/// and a failure there leaves a
///  merge a command must not read past.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const partlyInstalled = (error: RecoveryError): MergeRefusal => ({ error, began: true });
/// ```
pub fn partly_installed(error: RecoveryError) -> MergeRefusal {
    return MergeRefusal { error, began: true };
}

/// What:
///  A refusal about a merge that wrote nothing,
/// so a command that meets it may go on when the merge was saved.
/// Why:
///  Every refusal before the index write leaves the checkout as it was,
/// and the flag says so once for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const untouched = (error: RecoveryError): MergeRefusal => ({ error, began: false });
/// ```
pub fn untouched(error: RecoveryError) -> MergeRefusal {
    return MergeRefusal { error, began: false };
}

/// What:
///  Drop a record and,
/// when it named one,
/// the side ref that keeps its merge commit from being collected.
/// Why:
///   The side ref exists for the window between the save and the finish,
/// so the merge's landing is what removes it,
/// and only
///  while the ref still holds the merge's commit:
/// another command may have put its own commit on the name,
/// and that
///  commit may have no other.
/// A ref that is not deleted stays,
/// and the log says why,
/// because nothing of the merge is lost either
///  way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropRecordAndSideRef({ place, root, record, intent }): Promise<void>;
/// ```
pub fn drop_record_and_side_ref(place: &RecoveryPlace, root: &Path, record: &Path, intent: &MergeIntent) -> Result<(), RecoveryError> {
    remove_record(record)?;
    drop_side_ref(place, root, intent);
    return Ok(());
}

/// What:
///  Delete the side ref a record names,
/// saying in the log when that fails.
/// Why:
///   A merge that lands must not leave a name pointing at its commit,
/// but a deletion that fails is not a reason to
///  keep the record or stop the command:
/// the commit stays,
/// and the name is what a person can delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropSideRef({ place, root, intent }): Promise<void>;
/// ```
pub fn drop_side_ref(place: &RecoveryPlace, root: &Path, intent: &MergeIntent) {
    let Some(name) = intent.side_ref.as_deref() else {
        return;
    };
    // The ref is deleted only while it still holds this merge's commit: another command may have put its own commit on the
    //  name, and that commit may have no other.
    let arguments: [&str; 5] = ["update-ref", "--no-deref", "-d", name, intent.new_oid.as_str()];
    if let Err(error) = run_git_checked(&place.git, &GitRequest::new(root, &arguments)) {
        debug(
            "settleFailedMerge",
            format!("the side ref {name} no longer holds the landed merge's commit, so it was left where it is: {error}").as_str(),
        );
    }
}

/// What:
///  Remove a record,
/// naming it when that fails.
/// Why:
///  A saved merge whose branch advanced keeps its side ref and drops only its record,
/// so no later command is
///  stopped for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(record, { force: true });
/// ```
pub fn drop_record(record: &Path) -> Result<(), RecoveryError> {
    return match remove_merge_intent(record) {
        Ok(()) => Ok(()),
        Err(error) => Err(RecoveryError(format!("removing {} failed: {error}", record.display()))),
    };
}
