//! What:
//!  What recovery writes to the records and side refs of a merge it settled:
//!  the dropping of the ref that kept a capture,
//!  the dropping of a side ref and then of the record,
//!  and the refusal that says whether anything of the merge is installed.
//!  The words a refusal is spelled in are `merge_isolation_recovery_refusals.rs`.
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
use crate::merge_isolation_carry_ref::drop_capture_ref_awaiting;
use crate::merge_isolation_journal::{
    MergeIntent,
    remove_merge_intent,
};
use crate::merge_isolation_recovery_place::RecoveryPlace;
use crate::merge_isolation_recovery_reads::resolve_commit_awaiting;
use crate::merge_isolation_recovery_record::MergeRefusal;
/// The words a recovery refuses a merge with,
/// named here so a caller of the recovery keeps one path.
pub use crate::merge_isolation_recovery_refusals::{
    a_branch_that_could_not_be_put_back,
    a_landed_merge_in_another_branchs_worktree,
    a_worktree_that_switched,
    an_installed_merge_behind_a_held_index,
    edits_on_the_merges_own_paths,
    no_commit_names_the_merges_parent,
    staged_work_on_the_merges_paths,
};
use crate::recovery_error::RecoveryError;
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use std::path::Path;

/// The prefix of the names that keep a merge commit whose saved name no longer holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIDE_REF_PREFIX = 'refs/cli-git/kept/';
/// ```
const SIDE_REF_PREFIX: &str = "refs/cli-git/kept/";

/// How many names are tried when a merge commit is kept on one of this command's own.
/// Why:
///   A name a landing or a merge already kept is never written over;
/// the next number is tried instead,
///  and a host that took all of them is a refusal rather than a loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIDE_REF_ATTEMPTS = 8;
/// ```
const SIDE_REF_ATTEMPTS: u128 = 8;

/// What:
///  Drop a record and,
/// when it named them,
/// the refs that keep its capture's bytes and its merge commit from being collected,
///  and say whether the record could be removed.
/// Why:
///   A ref exists for the window between the save and the finish,
/// so the merge's landing is what removes it,
/// and only
///  while the ref still holds the merge's own commit:
/// another command may have put its own commit on the name,
/// and that
///  commit may have no other.
///  The record is removed last,
/// because it is the only name that still needs those refs:
/// a deletion
///  that fails keeps the record,
/// and the next command's recovery tries again
/// (a cross-vendor audit of alpha.12,
///  "Capture-ref cleanup is never retried after a transient failure").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropRecordAndSideRef({ place, root, record, intent }): Promise<boolean>;
/// ```
pub async fn drop_record_and_side_ref_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    record: &Path,
    intent: &MergeIntent,
) -> Result<bool, RecoveryError> {
    return drop_the_records_refs_then_the_record(place, root, record, intent, SideRef::Drop).await;
}

/// What:
///  Delete the side ref a record names,
/// saying in the log when that fails,
/// and answering whether the name is gone.
/// Why:
///   A merge that lands must not leave a name pointing at its commit,
/// and the caller keeps the record when the
///  name stayed,
/// so a later command's recovery deletes it instead of it being orphaned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropSideRef({ place, root, intent }): Promise<boolean>;
/// ```
pub async fn drop_side_ref_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
) -> bool {
    let Some(name) = intent.side_ref.as_deref() else {
        return true;
    };
    // The ref is deleted only while it still holds this merge's commit: another command may have put its own commit on the
    //  name, and that commit may have no other.
    let arguments: [&str; 5] = [
        "update-ref",
        "--no-deref",
        "-d",
        name,
        intent.new_oid.as_str(),
    ];
    return match run_git_checked_awaiting(&place.git, &GitRequest::new(root, &arguments)).await {
        Ok(_dropped) => true,
        Err(error) => {
            // A name already gone is what this deletion wanted: a kill between dropping a record's refs and removing the
            //  record leaves exactly it, and a retry that refused it kept the record for ever (the second review of this
            //  branch, its first finding, whose shape this shares with the capture's ref).
            if the_name_is_gone(place, root, name).await {
                debug(
                    "settleFailedMerge",
                    format!("the side ref {name} was already gone, which is what its deletion wanted: {error}").as_str(),
                );
                return true;
            }
            debug(
                "settleFailedMerge",
                format!("the side ref {name} no longer holds the landed merge's commit, so it was left where it is: {error}").as_str(),
            );
            false
        }
    };
}

/// What:
///  Whether a ref names nothing at all.
/// Why:
///   The answer comes from Git rather than from the words of a refusal,
/// and a read that fails is not proof of absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function theNameIsGone({ place, root, name }): Promise<boolean>; // for-each-ref --count=1 --format=%(objectname) <name>
/// ```
async fn the_name_is_gone(
    place: &RecoveryPlace,
    root: &Path,
    name: &str,
) -> bool {
    let arguments: [&str; 4] = ["for-each-ref", "--count=1", "--format=%(objectname)", name];
    return match run_git_checked_awaiting(&place.git, &GitRequest::new(root, &arguments)).await {
        Ok(output) => output.stdout.iter().all(|byte| {
            return byte.is_ascii_whitespace();
        }),
        Err(_) => false,
    };
}

/// What:
///  Settle a merge saved behind an index lock whose paths hold the person's own edits newer than the capture it keeps,
///  by
///  removing the record and naming the ref that keeps the merge.
/// Why:
///   Nothing of the merge is in the index or the checkout,
/// so it cannot be installed without writing over bytes only the
///  person holds;
/// the record must go or every command meets it again,
/// and the merge's commit must stay where they can reach it,
///  which the side ref does and the record did.
/// The words name the ref and the command that makes the merge on what the files hold now,
/// so the way forward is one they
///  can type rather than a decision only the wrapper could make
/// (`merge_isolation_recovery.rs`,
/// the paths that are nobody's).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settled = await settleASavedMergeOverNewerEdits({ place, root, record, intent, foreign, stderr }); // drops the record, keeps the side ref
/// ```
pub async fn settle_a_saved_merge_over_newer_edits<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    root: &Path,
    record: &Path,
    intent: &MergeIntent,
    foreign: &[String],
    out: &mut W,
) -> Result<crate::merge_isolation_recovery_record::MergeRecoveryAction, MergeRefusal> {
    let saved_ref: &str = intent.side_ref.as_deref().unwrap_or("its side ref");
    // The line promises a ref keeps the merge commit, so the ref is read before the record goes. Another command may have
    //  repointed the name, and a commit whose record and ref are both gone is collected by `git gc`; the commit is kept on a
    //  name of this command's own then, and the line names that name.
    let keeping: String = match resolve_commit_awaiting(place, root, saved_ref).await {
        Ok(Some(found)) if found == intent.new_oid => String::from(saved_ref),
        Ok(_) => match keep_the_merge_on_a_side_ref_awaiting(place, root, intent.new_oid.as_str())
            .await
        {
            Some(name) => name,
            // No name could be written, so the record stays and the next command tries again: the commit's only other name
            //  is the one this command could not keep.
            None => {
                return Err(MergeRefusal {
                    error: RecoveryError(format!(
                        "The merge saved on {saved_ref} was settled over your own edits, and its commit {} is named on no ref, so the \
                         record {} was kept: keeping it on a ref of its own failed. Nothing was changed. Run `git update-ref \
                         refs/cli-git/kept/<name> {}` to keep the merge commit, then any git command again.",
                        intent.new_oid,
                        record.display(),
                        intent.new_oid
                    )),
                    began: false,
                });
            }
        },
        // The ref could not be read, so nothing proves a ref keeps the commit and the record stays for the next command.
        Err(error) => {
            return Err(MergeRefusal {
                error,
                began: false,
            });
        }
    };
    let moved: String = match keeping.as_str() == saved_ref {
        true => String::new(),
        false => format!(
            " The ref this merge was saved on ({saved_ref}) no longer names it, so the commit was kept on {keeping}."
        ),
    };
    // The capture's bytes are the person's files' own, so the ref that kept them goes before the record that names it: a
    //  deletion that fails keeps the record, and the next command's recovery tries again.
    if !drop_capture_awaiting(place, root, record, intent).await {
        let _noted = out.write_all(
            format!(
                "monochromatic-git-policy-cli: the merge saved on {saved_ref} was not made, and the keep of the work it carries could not be \
                 removed yet, so the record {} was kept and the next git command here tries again. Nothing of that merge is in the checkout, \
                 and the merge commit {} is kept on {keeping}.\n",
                record.display(),
                intent.new_oid
            )
            .as_bytes(),
        );
        return Ok(crate::merge_isolation_recovery_record::MergeRecoveryAction::DeferredRefused);
    }
    if let Err(error) = remove_merge_intent(record) {
        return Err(MergeRefusal {
            error: RecoveryError(format!(
                "{} could not be removed: {error}",
                record.display()
            )),
            began: true,
        });
    }
    let _noted = out.write_all(
        format!(
            "monochromatic-git-policy-cli: the merge saved on {saved_ref} was not made: {} hold your own edits, which were saved after \
             that merge saved the work it carries, so installing it would write over them. Nothing was changed and the record {} was \
             removed; the merge commit {} is kept on {keeping}, which nothing removes.{moved} Run `git merge {}` to make the merge on the \
             bytes your files hold now, or delete the ref once you have decided.\n",
            foreign.join(", "),
            record.display(),
            intent.new_oid,
            intent.new_oid
        )
        .as_bytes(),
    );
    return Ok(crate::merge_isolation_recovery_record::MergeRecoveryAction::DeferredRefused);
}

/// What:
///  Keep one merge commit on a name of its own under `refs/cli-git/kept/`,
///  and say which name it took.
/// Why:
///  A merge commit whose record and ref are both gone is collected by `git gc`,
/// and the line that settles a saved merge
///  promises the person a ref keeps it.
/// The write is an `update-ref` whose expected old value is all zeros,
/// so a name another
///  command kept is never written over,
/// and the walk to the next name is bounded rather than a loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function keepTheMergeOnASideRef({ place, root, oid }): Promise<string | undefined>; // update-ref --no-deref refs/cli-git/kept/<ms+n> <oid> 0000...
/// ```
async fn keep_the_merge_on_a_side_ref_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    merge_oid: &str,
) -> Option<String> {
    let zeros: String = "0".repeat(merge_oid.len());
    let first: u128 = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .ok()?
        .as_millis();
    for step in 0..SIDE_REF_ATTEMPTS {
        let name: String = format!("{SIDE_REF_PREFIX}{}", first + step);
        let arguments: [&str; 5] = [
            "update-ref",
            "--no-deref",
            name.as_str(),
            merge_oid,
            zeros.as_str(),
        ];
        let made = run_git_checked_awaiting(&place.git, &GitRequest::new(root, &arguments)).await;
        if made.is_ok() {
            return Some(name);
        }
    }
    return None;
}

/// What:
///  What becomes of the side ref a record names when the record goes.
/// Why:
///   A merge that landed has no use for the ref any more,
/// while one a command reported as saved keeps its commit
///  there and names it to the person,
/// so each caller says which it means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SideRef = 'drop' | 'keep';
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SideRef {
    /// The side ref goes with the record:
    /// the merge's commit is in the branch.
    Drop,
    /// The side ref stays:
    /// it is the only name that still keeps the merge's commit reachable.
    Keep,
}

/// What:
///  Drop the ref that kept a capture,
/// once its bytes are back in the checkout,
///  and say whether the name is gone.
/// Why:
///   The record is the only thing that names the ref,
/// so a caller that removed the record first left a name
///  nothing ever retried when the deletion met another command's lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dropCapture({ place, root, intent }): boolean;
/// ```
pub(crate) async fn drop_capture_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    record: &Path,
    intent: &MergeIntent,
) -> bool {
    let Some(capture) = intent.carried.as_ref() else {
        return true;
    };
    return drop_capture_ref_awaiting(
        &place.git,
        place.git_dir.as_path(),
        root,
        capture,
        Some(record),
    )
    .await;
}

/// What:
///  Take away what a record keeps,
/// and only then the record itself,
///  answering whether it could be removed.
/// Why:
///   The record is the only name that still needs the capture's ref and a side ref,
/// so a command that removed
///  it first and then failed to delete a ref left a name nothing ever retried
/// (a cross-vendor audit of
///  alpha.12,
/// "Capture-ref cleanup is never retried after a transient failure").
///  Keeping the record when a ref stayed leaves the
///  retry to the next command,
/// whose recovery finishes the same merge and tries the refs again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropTheRecordsRefsThenTheRecord({ place, root, record, intent, sideRef }): Promise<boolean>;
/// ```
pub(crate) async fn drop_the_records_refs_then_the_record(
    place: &RecoveryPlace,
    root: &Path,
    record: &Path,
    intent: &MergeIntent,
    side_ref: SideRef,
) -> Result<bool, RecoveryError> {
    // The capture's bytes are in the checkout or back in the files by the time a caller gets here, and that is written beside
    //  the record before its ref is decided: a kill between the two steps then leaves a record whose next settle needs nothing
    //  from the objects (`merge_isolation_journal.rs`, `mark_carry_finished`).
    if intent.carried.is_some()
        && let Err(error) = crate::merge_isolation_journal::mark_carry_finished(record)
    {
        return Err(RecoveryError(format!(
            "the finish of the carried work could not be recorded: {error}"
        )));
    }
    if !drop_capture_awaiting(place, root, record, intent).await {
        return Ok(false);
    }
    if side_ref == SideRef::Drop
        && !crate::merge_isolation_recovery_cleanup::drop_side_ref_awaiting(place, root, intent)
            .await
    {
        return Ok(false);
    }
    crate::merge_isolation_recovery_steps::remove_record(record)?;
    return Ok(true);
}
