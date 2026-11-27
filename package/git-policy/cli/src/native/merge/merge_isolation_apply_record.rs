//! What:
//!  What an isolated merge writes before any byte of the checkout changes:
//! the intent record that lets a later command finish or discard this merge,
//! the look at `index.lock` that decides whether it is saved for a later command instead,
//! and the marker phase a crash control arms.
//! Why:
//!   The record must be on disk before Git starts,
//! so a kill at any instant leaves recovery something to read,
//! and the look must come after it,
//! so a merge that saves itself leaves the next command the work of finishing this very merge.
//!  `merge_isolation_apply.rs` writes the merge and moves the checkout,
//! and this is a file of its own so that one stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { transactionId, intent, record, notNow } = await writeTheRecordAndLookAtTheIndex({ session, plan });
//! ```

/// Waiting for any program's hold on the index lock before the checkout changes,
/// and what the wait proved.
use crate::index_holder_wait::{
    IndexWait,
    look_at_the_index_before_a_merge,
};
/// Scanning every readable process for a descriptor on that lock.
use crate::index_lock_holders::scan_platform_holders_bounded;
/// The clock the merge's owner is named with.
use crate::iso_time::now_iso;
/// Why a merge is saved for a later command instead of applied now.
use crate::merge_isolation_apply_findings::WhyTheMergeIsSaved;
/// The dropping of the ref that keeps the person's captured bytes.
use crate::merge_isolation_carry_ref::drop_capture_ref;
/// The questions an isolated merge asks Git.
use crate::merge_isolation_git_reads::Session;
/// The intent record.
use crate::merge_isolation_journal::{
    MergeIntent,
    write_merge_intent,
};
/// The values an isolated merge passes between its steps,
/// and the ending every early stop of one takes.
use crate::merge_isolation_run_types::{
    Bail,
    Plan,
    refused,
};
/// A transaction identifier of the record.
use crate::random_id::random_uuid;
/// The platform whose unlink removes the lock.
use crate::service_index_placement::SYSTEM;
/// Removing an index lock a gone process left,
/// on the commit service's proof of disuse.
use crate::service_index_unheld_removal::{
    Unheld,
    remove_if_the_owner_is_gone,
};
/// One Git command's output.
use crate::transaction_git::GitOutput;
/// The owner of the intent record,
/// and its identifier.
use crate::transaction_owner::current_transaction_owner;
/// Marker phases for the crash controls.
use crate::transaction_phase::{
    Phase,
    reach_transaction_phase,
};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The record an isolated merge wrote,
/// the transaction it is written under,
/// and why the merge must wait for a later command,
/// or nothing when the index is free.
/// Why:
///   Every later step of the merge names the record and the transaction,
/// and the look's answer decides whether this command applies the merge or saves it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRecordAndTheLockLook = { transactionId: string; intent: MergeIntent; record: string; notNow?: WhyTheMergeIsSaved };
/// ```
pub(crate) struct TheRecordAndTheLockLook {
    /// The transaction the record and the landed record are written under.
    pub transaction_id: String,
    /// What the record says about this merge.
    pub intent: MergeIntent,
    /// The record's own file.
    pub record: PathBuf,
    /// Why the merge is saved for a later command instead of applied now,
    /// or nothing when nothing holds the lock.
    pub not_now: Option<WhyTheMergeIsSaved>,
}

/// What:
///  Write the intent record,
/// then look at `index.lock` and say whether this command may apply the merge now.
/// Why:
///   A record written first is what a later command reads when this one is killed,
/// and a lock another program holds is answered by a save rather than a wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeTheRecordAndLookAtTheIndex({ session, plan }): Promise<TheRecordAndTheLockLook>; // throws Bail
/// ```
pub(crate) fn write_the_record_and_look_at_the_index(
    session: &Session,
    plan: &Plan<'_>,
) -> Result<TheRecordAndTheLockLook, Bail> {
    let (git_dir, root): (&Path, &Path) = (plan.git_dir, plan.root);
    let (old_oid, new_oid): (&str, &str) = (plan.old_oid, plan.new_oid);
    let target_ref: Option<String> = {
        let named: GitOutput = session.run(&["symbolic-ref", "--quiet", "HEAD"])?;
        if named.succeeded() {
            Some(String::from_utf8_lossy(&named.stdout).trim().to_owned())
        } else {
            None
        }
    };
    let transaction_id: String = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(Bail::from(error.0)),
    };
    let owner = match current_transaction_owner(transaction_id.as_str(), now_iso().as_str()) {
        Ok(found) => found,
        Err(error) => return Err(Bail::from(error.to_string())),
    };
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: root.to_path_buf(),
        target_ref,
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
        side_ref: None,
        carried: plan.carried.cloned(),
    };
    let record: PathBuf = match write_merge_intent(git_dir, &intent) {
        Ok(written) => written,
        Err(error) => {
            // A capture whose record could not be written names nothing a later command would settle, so it goes; a capture whose
            //  ref stays names itself in the refusal, because nothing else does.
            let kept: String = match plan.carried {
                Some(capture)
                    if !drop_capture_ref(&session.context, git_dir, root, capture, None) =>
                {
                    match capture.reference.as_deref() {
                        Some(kept) => format!(
                            ", and the keep of your edits is still on {kept}, which nothing removes: `git update-ref -d {kept}` deletes it"
                        ),
                        None => String::new(),
                    }
                }
                _ => String::new(),
            };
            return Err(Bail::from(format!(
                "the merge record could not be written: {error}{kept}"
            )));
        }
    };
    // The bytes the person's paths hold are objects on a ref of their own now, and the record names them; nothing of the
    //  checkout has changed yet, which is the instant a kill leaves the command exactly as it was.
    reach_phase(Phase::MergeCarried)?;
    reach_phase(Phase::MergeIntent)?;
    // Git's fast-forward takes `index.lock` before it changes anything and dies when another program holds it. The look names a
    //  dead owner whose lock this command may clear, and a live one it does not wait for: waiting on another agent's command is
    //  what the owner forbids, and the save below is the answer that needs no wait. The record is written first, so a merge that
    //  saves itself leaves the next command the work of finishing this very merge.
    let looked: IndexWait =
        look_at_the_index_before_a_merge(git_dir.join("index").as_path(), &mut std::io::stderr());
    let mut why: Option<WhyTheMergeIsSaved> = None;
    match looked {
        IndexWait::NotHeld | IndexWait::Released(_) => {}
        IndexWait::GaveUp(waited_for) => why = Some(WhyTheMergeIsSaved::TheWaitEnded(waited_for)),
        // The look never waits, so a holder it met is one this merge leaves for the next command rather than wait for.
        IndexWait::HeldByAnotherCommand(holder) => {
            why = Some(WhyTheMergeIsSaved::AnotherProgramHoldsIt(holder))
        }
        IndexWait::OwnerGone(_) => {
            // A lock a gone process left is removed on the proof the commit service acts on, so a merge repairs what a commit
            //  repairs (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`, finding 3). A lock that proof does
            //  not cover (it holds content, it stands behind a service's marker, a process has it open) stays where it is.
            if !matches!(
                remove_if_the_owner_is_gone(
                    &SYSTEM,
                    git_dir.join("index").as_path(),
                    &scan_platform_holders_bounded
                ),
                Unheld::Removed(_)
            ) {
                why = Some(WhyTheMergeIsSaved::NotTheCommandsToClear);
            }
        }
    }
    return Ok(TheRecordAndTheLockLook {
        transaction_id,
        intent,
        record,
        not_now: why,
    });
}

/// What:
///  Reach a marker phase,
/// failing the invocation when the marker variable is malformed.
/// Why:
///   The crash controls arm a phase by environment;
/// a malformed value must not be ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachTransactionPhase({ phase });
/// ```
pub fn reach_phase(phase: Phase) -> Result<(), Bail> {
    return match reach_transaction_phase(phase, None) {
        Ok(()) => Ok(()),
        Err(error) => Err(refused(
            2,
            format!("monochromatic-git-policy-cli: {error}\n"),
        )),
    };
}
