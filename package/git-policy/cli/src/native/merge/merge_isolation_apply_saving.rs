//! What:
//!  The merges a command keeps for a later finish:
//!  one that met another merge installing in this worktree,
//!  and one that met a live `index.lock`.
//! Why:
//!   Both end the same way,
//! and both are the answer the owner's rules give instead of a failure:
//! a merge that cannot install now keeps its
//!  commit on a side ref,
//! records that the checkout is still to be finished,
//! exits 0,
//! and the next command in the worktree
//!  finishes it.
//! The steps live here so `merge_isolation_apply.rs` stays readable and inside the package's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ended = keepTheMergeForALaterFinish({ session, plan, intent, reference, heldFor }); // exit 0, side ref named
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The merge record,
/// and the ref that keeps a deferred merge's commit.
use crate::merge_isolation_journal::{MergeIntent, write_merge_intent};
/// The person's own work in the paths the merge changes,
/// and the ref that keeps it.
use crate::merge_isolation_carry::drop_capture_ref;
/// The words a merge that could not be finished now prints.
use crate::merge_isolation_apply_findings::{a_deferred_save_behind_a_merge, a_deferred_save_finding, a_merge_that_could_not_be_saved};
/// The mark of an install running in this worktree,
/// which is where a merge that is saved for a later finish names what blocked it.
use crate::merge_install_guard::install_guard_path;
/// The values an isolated merge passes between its steps.
use crate::merge_isolation_run_types::{Bail, IsolatedMerge, Plan, TAG, refused};
/// The queries of a merge's own session.
use crate::merge_isolation_git_reads::Session;
/// Real Git's answer.
use crate::transaction_git::GitOutput;
/// Unique identifiers.
use crate::random_id::random_uuid;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// A span of time a wait lasted.
use std::time::Duration;
/// The clock reading a side ref's number starts from.
use std::time::SystemTime;

/// What:
///  Save a merge that met another merge installing in the same worktree:
/// keep its commit on a side ref,
/// record that the checkout is still to be finished,
/// say so,
///  and end with status 0.
/// Why:
///   Two installs beside each other leave an index the merge commit does not explain and a checkout neither install
///  can finish (`SPEC.md`,
/// "One install in a worktree at a time").
/// A merge that finds the mark of a live install is
///  kept the way a merge that met a live `index.lock` is kept:
/// its commit on a side ref,
/// a record that names it,
/// and the next
///  command in this worktree finishes it.
/// Nothing of the merge is in the checkout,
/// so the person's own work is
///  untouched,
/// and the record names the capture,
/// so a finish carries it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function saveBehindAMergeThatInstalls({ session, plan, intent, reference, holderPid }): Promise<IsolatedMerge>;
/// ```
pub(crate) fn save_behind_a_merge_that_installs(
    session: &Session,
    plan: &Plan<'_>,
    intent: &MergeIntent,
    reference: &str,
    holder_pid: i64,
) -> Result<IsolatedMerge, Bail> {
    let Some(side_ref) = keep_on_a_side_ref(session, plan.new_oid) else {
        return Err(refused(2, a_merge_that_could_not_be_saved(reference, install_guard_path(plan.git_dir).as_path())));
    };
    let mut later: MergeIntent = intent.clone();
    later.owner.transaction_id = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(Bail::from(error.0)),
    };
    later.side_ref = Some(side_ref.clone());
    // Nothing of this merge is in the checkout, so nothing of the person's is to be put back later either: a capture
    //  restored against a branch another merge has moved would write their old index entries over the merge's, which is
    //  the half-applied result this mark exists to prevent. The worktree still holds their bytes, so the capture is
    //  dropped here and the record names none (`merge_isolation_recovery.rs`, the settling of a saved merge).
    later.carried = None;
    if let Some(capture) = plan.carried {
        drop_capture_ref(&session.context, plan.git_dir, plan.root, capture);
    }
    if let Err(error) = write_merge_intent(plan.git_dir, &later) {
        return Err(refused(
            2,
            format!(
                "monochromatic-git-policy-cli: git merge {reference}: another merge is installing its result in this worktree (process {holder_pid}), and \
                 this merge could not be recorded for a later finish ({error}); its commit is kept on {side_ref}. Nothing was changed. Run the \
                 merge again once that merge is done.\n"
            ),
        ));
    }
    debug(TAG, format!("the merge is saved on {side_ref} because the merge of process {holder_pid} is installing in this worktree").as_str());
    return Ok(IsolatedMerge::Finished { code: 0, stdout: String::new(), stderr: a_deferred_save_behind_a_merge(reference, holder_pid, side_ref.as_str()) });
}

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
/// names the side ref and the merge,
/// so the next command finishes this very merge even though nothing of it is in the
///  checkout yet.
/// A merge whose commit cannot be kept that way is refused instead,
/// with nothing changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function saveForALaterFinish({ session, plan, intent, record, reference, heldFor }): Promise<IsolatedMerge>;
/// ```
pub(crate) fn save_for_a_later_finish(
    session: &Session,
    plan: &Plan<'_>,
    intent: &MergeIntent,
    record: &Path,
    reference: &str,
    held_for: Option<Duration>,
) -> Result<IsolatedMerge, Bail> {
    let lock: PathBuf = plan.git_dir.join("index.lock");
    let Some(side_ref) = keep_on_a_side_ref(session, plan.new_oid) else {
        return Err(refused(2, a_merge_that_could_not_be_saved(reference, lock.as_path())));
    };
    let mut later: MergeIntent = intent.clone();
    later.owner.transaction_id = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(Bail::from(error.0)),
    };
    later.side_ref = Some(side_ref.clone());
    if let Err(error) = write_merge_intent(plan.git_dir, &later) {
        return Err(refused(
            2,
            format!(
                "monochromatic-git-policy-cli: git merge {reference}: another program holds this repository's index ({}), and the merge \
                 could not be recorded for a later finish ({error}); its commit is kept on {side_ref}. Nothing was changed. Run the merge \
                 again once that program is done.\n",
                lock.display()
            ),
        ));
    }
    debug(TAG, format!("the merge is saved on {side_ref} beside its first record {}", record.display()).as_str());
    return Ok(IsolatedMerge::Finished { code: 0, stdout: String::new(), stderr: a_deferred_save_finding(reference, lock.as_path(), side_ref.as_str(), held_for) });
}
