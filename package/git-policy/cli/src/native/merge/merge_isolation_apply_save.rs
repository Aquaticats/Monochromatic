//! What:
//!  The merge a command saves for a later one:
//!  its commit kept on a side ref,
//!  the record that names that ref and the person's captured bytes,
//!  and the line the person reads instead of a refusal.
//! Why:
//!  A merge that cannot reach the checkout now is never a failed merge
//!  (`SPEC.md`,
//!  "Merges and commits never fail"):
//!  the branch does not move,
//!  the command exits 0,
//!  and the next git command in the worktree finishes the merge once the index is free.
//!  Three instants end here,
//!  and each one is named by the `why` it passes,
//!  and a save that cannot be named ends with the truth about the checkout rather than a refusal:
//!  a lock the merge found when it looked,
//!  a lock another program took after that look,
//!  and a lock that stopped both Git's own update and the putting back of the person's own bytes.
//!  The writing half that calls this is `merge_isolation_apply.rs`,
//!  and the step ending that also reaches it is `merge_isolation_apply_stops.rs`;
//!  both are at their code-line limit,
//!  and the words are `merge_isolation_apply_findings.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const saved = await saveForALaterFinish({ session, plan, intent, record, reference, why });
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The words a saved merge prints,
/// and the lines for a save that could not be made or recorded,
/// built from the bytes the settle left where they are.
use crate::merge_isolation_apply_findings::{
    BytesAfterAFailedMerge, WhyTheMergeIsSaved, a_deferred_save_finding, a_merge_that_could_not_be_kept,
    a_merge_whose_save_could_not_be_recorded,
};
/// The intent record,
/// written once per saved merge.
use crate::merge_isolation_journal::{MergeIntent, write_merge_intent};
/// The command's own ending.
use crate::merge_isolation_run_types::{Bail, IsolatedMerge, Plan, TAG};
/// Real Git,
/// which keeps the commit on the side ref.
use crate::merge_isolation_git_reads::Session;
/// One Git command's output.
use crate::transaction_git::GitOutput;
/// A transaction identifier of the record that names the save.
use crate::random_id::random_uuid;
/// `Path` is a borrowed filesystem path,
/// and `PathBuf` an owned one.
use std::path::{Path, PathBuf};
/// The clock a side ref's number starts from.
use std::time::SystemTime;

/// What:
///  How many names are tried when the merge is kept on a side ref.
/// Why:
///   A name a landing already kept is never written over;
/// the next number is tried instead,
/// and a host that
///  took all of them is a refusal rather than a loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIDE_REF_ATTEMPTS = 8;
/// ```
const SIDE_REF_ATTEMPTS: u128 = 8;

/// What:
///  Keep the merge commit on `refs/cli-git/kept/<number>`,
/// which only a deletion removes,
/// and return the name it took.
/// Why:
///   The record that names the merge is not enough:
/// `git gc` collects a commit nothing points at,
/// and a
///  deferred merge must still be finishable after the lock frees.
/// The write is an `update-ref` whose old value is all zeros,
/// so a name
///  another command kept is never written over,
/// and the walk to the next number is bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepOnASideRef(session, mergeOid): string | undefined; // update-ref --no-deref refs/cli-git/kept/<ms+n> <oid> 0000...
/// ```
fn keep_on_a_side_ref(session: &Session, merge_oid: &str) -> Option<String> {
    let zeros: String = "0".repeat(merge_oid.len());
    let first: u128 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok()?.as_millis();
    for step in 0..SIDE_REF_ATTEMPTS {
        let name: String = format!("refs/cli-git/kept/{}", first + step);
        let made: GitOutput = session.run(&["update-ref", "--no-deref", name.as_str(), merge_oid, zeros.as_str()]).ok()?;
        if made.succeeded() {
            return Some(name);
        }
    }
    return None;
}

/// What:
///  Save a merge that met an index lock it may not clear:
/// keep its commit on a side ref,
/// record that the checkout is still to be finished,
/// say so,
/// and end
///  with status 0.
/// Why:
///   The owner's answer for a live foreign `index.lock` (`SPEC.md`,
/// "Merges and commits never fail") is that a merge
///  saves its result on a side ref,
/// exits 0,
/// and finishes the checkout when the lock frees.
/// A second record,
///  created once like every other,
/// names the side ref,
/// the merge,
/// and the same capture of the person's bytes the first record
///  names,
/// so the next command finishes this very merge even though nothing of it is in the checkout yet.
/// A merge whose commit cannot be kept that way is not saved,
/// and the line says so and what the next command does with the person's own bytes,
/// rather than refusing a checkout the clear already changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function saveForALaterFinish({ session, plan, intent, record, reference, why, thePathsWereCleared }): Promise<IsolatedMerge>;
/// ```
pub(crate) fn save_for_a_later_finish(
    session: &Session,
    plan: &Plan<'_>,
    intent: &MergeIntent,
    record: &Path,
    reference: &str,
    why: WhyTheMergeIsSaved,
    the_paths_were_cleared: bool,
) -> Result<IsolatedMerge, Bail> {
    let lock: PathBuf = plan.git_dir.join("index.lock");
    // The ref the captured bytes are kept on, when this merge captured any: every line here names it, because a restore the
    //  lock stopped leaves those bytes only there and the person has to be told where they are.
    let mut kept: Option<&str> = None;
    if let Some(capture) = plan.carried.as_ref() {
        kept = capture.reference.as_deref();
    }
    let bytes: BytesAfterAFailedMerge<'_> = BytesAfterAFailedMerge { the_paths_were_cleared, kept_on: kept };
    let mut later: MergeIntent = intent.clone();
    // The record's own identifier is made before anything is written, so a source that cannot answer leaves nothing behind.
    later.owner.transaction_id = match random_uuid() {
        Ok(id) => id,
        // A save that cannot be named is not made: the record this command wrote still puts the person's bytes back, and the
        //  merge is theirs to run again.
        Err(error) => {
            debug(TAG, format!("the merge could not be given an identifier for a later finish: {}", error.0).as_str());
            return Ok(the_merge_is_not_saved(reference, lock.as_path(), &bytes));
        }
    };
    let Some(side_ref) = keep_on_a_side_ref(session, plan.new_oid) else {
        return Ok(the_merge_is_not_saved(reference, lock.as_path(), &bytes));
    };
    later.side_ref = Some(side_ref.clone());
    if let Err(error) = write_merge_intent(plan.git_dir, &later) {
        debug(TAG, format!("the save of the merge could not be recorded: {error}").as_str());
        return Ok(IsolatedMerge::Finished {
            code: 0,
            stdout: String::new(),
            stderr: a_merge_whose_save_could_not_be_recorded(reference, lock.as_path(), side_ref.as_str(), plan.new_oid, &bytes),
        });
    }
    debug(TAG, format!("the merge is saved on {side_ref} beside its first record {}", record.display()).as_str());
    return Ok(IsolatedMerge::Finished { code: 0, stdout: String::new(), stderr: a_deferred_save_finding(reference, lock.as_path(), side_ref.as_str(), &why, &bytes) });
}

/// What:
///  The ending of a merge whose save could not be named,
///  which says what the checkout holds and what the next command does with it.
/// Why:
///   A merge that fails on another program's lock is never a failed merge (`SPEC.md`,
/// "Merges and commits never fail"),
/// so this exits 0 with the truth rather than refusing a checkout the clear changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notSaved = (reference, lock, bytes) => finished(0, aMergeThatCouldNotBeKept(reference, lock, bytes));
/// ```
fn the_merge_is_not_saved(reference: &str, lock: &Path, bytes: &BytesAfterAFailedMerge<'_>) -> IsolatedMerge {
    return IsolatedMerge::Finished { code: 0, stdout: String::new(), stderr: a_merge_that_could_not_be_kept(reference, lock, bytes) };
}
