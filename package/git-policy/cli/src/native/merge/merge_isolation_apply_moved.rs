//! What:
//!  The carry of a merge that landed,
//!  its lines,
//!  and what the branch does if it moved while that carry ran:
//!  the person's layers are then merged with the branch as it is now.
//! Why:
//!  Dozens of agents run Git in one worktree at once (`SPEC.md`,
//! "Merges and commits never fail").
//!  Another command lands a commit on this very branch in the window between this merge's own update and its
//!  carry,
//!  and the carry then finds the merge's paths holding that commit's content.
//!  The capture behind them holds every
//!  byte of the person's work,
//!  so dropping the record and the ref at that point leaves those bytes reachable nowhere,
//!  which is what `never discard user data` rules out.
//!  The recovery of a merge whose process died carries the capture onto the moved
//!  head (`merge_isolation_recovery_carry.rs`);
//!  this is the same answer while the command still runs,
//!  so the merge ends
//!  with the person's work in the files instead of leaving it to a later command.
//!  The carrier itself is
//!  `merge_isolation_carry_apply.rs`,
//!  and the merge that asks is `merge_isolation_apply.rs`,
//!  which is at its code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = carryAndSettle({ session, plan, reference, intent, changes, landed, transactionId });
//! ```

/// The entry of the process runtime that a synchronous caller takes.
use crate::lazy_runtime::run_on_runtime;
/// The words a carry that could not write now prints,
/// and the line a step of the merge that stopped prints.
use crate::merge_isolation_apply_findings::a_carry_that_stopped;
/// The writes of the carry itself.
use crate::merge_isolation_carry_apply::{CarryFailure, LockWait, carry};
/// The person's work as the capture recorded it.
use crate::merge_isolation_carry_parts::Capture;
/// The paths the merge changes,
/// and what Git's own update left at each of them.
use crate::merge_isolation_changes::Change;
use crate::merge_isolation_carry_landed::LandedFile;
/// The lines a carry prints about the paths it carried,
/// and everything one carry did.
use crate::merge_isolation_carry_notice::{Report, report_lines};
/// The record of the merge whose own path this is.
use crate::merge_isolation_journal::MergeIntent;
/// The merge as one round resolved it.
use crate::merge_isolation_run_types::Plan;
/// Whether the branch was built on this merge.
use crate::merge_isolation_recovery::is_ancestor_awaiting;
/// The person's work carried onto the branch as it is now.
use crate::merge_isolation_recovery_carry::carry_onto_awaiting;
/// The commit the branch names now.
use crate::merge_isolation_recovery_reads::resolve_commit_awaiting;
/// The place a recovery step runs in.
use crate::merge_isolation_recovery_steps::RecoveryPlace;
/// The questions an isolated merge asks Git.
use crate::merge_isolation_git_reads::Session;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  What the carry of a merge that landed comes to,
///  and whether the record and the ref that keep the person's bytes may go.
/// Why:
///   The lines are printed by the command that ran the merge,
/// and the two things the caller must know are whether the
///  person's work is in the files and what to print about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarryEnding = { lines: string; finished: boolean };
/// ```
pub(crate) struct CarryEnding {
    /// The lines to add to the command's standard error,
    /// in the order they are printed.
    pub(crate) lines: String,
    /// Whether every layer of the person's is in the files.
    pub(crate) finished: bool,
}

/// What:
///  Merge the person's layers with the merge,
/// print what became of each path,
/// and settle their work against the branch as it is now when it moved meanwhile.
/// Why:
///   A path this carry wrote nothing to holds bytes another command put there,
/// and that command may be one that landed
///  a commit on this very branch,
/// so the branch is read again before the record and the ref that keep the person's
///  bytes are dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function carryAndSettle({ session, plan, reference, intent, changes, landed, transactionId }): CarryEnding;
/// ```
pub(crate) fn carry_and_settle(
    session: &Session,
    plan: &Plan<'_>,
    reference: &str,
    intent: &MergeIntent,
    changes: &[Change],
    landed: &[LandedFile],
    transaction_id: &str,
) -> CarryEnding {
    let Some(capture): Option<&Capture> = plan.carried else { return CarryEnding { lines: String::new(), finished: true } };
    let (root, git_dir): (&Path, &Path) = (plan.root, plan.git_dir);
    let carried: Result<Report, CarryFailure> =
        carry(&session.context, root, git_dir, changes, capture, landed, transaction_id, LockWait::WhileFinishing, &mut std::io::stderr());
    let report: Report = match carried {
        Ok(found) => found,
        Err(failure) => return CarryEnding { lines: a_carry_that_stopped(reference, &failure, capture), finished: false },
    };
    let mut lines: String = String::new();
    for line in report_lines(reference, &report) {
        lines.push_str(line.as_str());
        lines.push('\n');
    }
    // Nothing was written to a path whose bytes changed under the carry, so the branch is read once more: a commit built on this merge
    //  is what the person's layers are merged with, exactly as the recovery of a killed merge carries them onto a moved head.
    if report.changed.is_empty() {
        return CarryEnding { lines, finished: true };
    }
    let place: RecoveryPlace = RecoveryPlace::settling(&session.context, git_dir, root);
    return match after_the_carry(&place, root, intent, reference) {
        AfterTheCarry::WhereTheMergePutIt => CarryEnding { lines, finished: true },
        AfterTheCarry::OntoTheMovedHead(moved_lines) => CarryEnding { lines: moved_lines, finished: true },
        AfterTheCarry::Waiting(line) => CarryEnding { lines: line, finished: false },
    };
}

/// What the carry of a merge that already landed comes to,
/// once the branch has been read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AfterTheCarry = 'where-the-merge-put-it' | { ontoTheMovedHead: string } | { waiting: string };
/// ```
enum AfterTheCarry {
    /// The branch is where this merge put it,
    /// so the carry's own lines stand as they are.
    WhereTheMergePutIt,
    /// The branch was built on this merge:
    /// the person's layers were merged with it,
    /// and these are the lines to print.
    OntoTheMovedHead(String),
    /// Nothing was written now and nothing may be removed:
    /// the record and the objects it names keep the person's work for the next command,
    /// and this line says so.
    Waiting(String),
}

/// What:
///  Settle the person's work of a merge that already landed against the branch as it is now.
/// Why:
///   The branch is read after the carry rather than before it,
/// because another command may land its commit in
///  any of the steps between the merge's own update and the record's removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function afterTheCarry({ place, root, intent, reference }): AfterTheCarry;
/// ```
fn after_the_carry(place: &RecoveryPlace, root: &Path, intent: &MergeIntent, reference: &str) -> AfterTheCarry {
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    // A read that fails and a runtime that refuses the task are the same answer: nothing written, nothing removed.
    return match run_on_runtime(after_the_carry_awaiting(place, root, intent, reference)) {
        Ok(Ok(settled)) => settled,
        Ok(Err(_)) | Err(_) => AfterTheCarry::Waiting(the_branch_could_not_be_read(reference, target)),
    };
}

/// What:
///  The same settling,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The reads and the carry start Git,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function afterTheCarry({ place, root, intent, reference }): Promise<AfterTheCarry>;
/// ```
async fn after_the_carry_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
    reference: &str,
) -> Result<AfterTheCarry, String> {
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    let named: Option<String> = match resolve_commit_awaiting(place, root, target).await {
        Ok(found) => found,
        Err(error) => return Err(error.0),
    };
    let Some(tip) = named else {
        return Ok(AfterTheCarry::Waiting(the_branch_could_not_be_read(reference, target)));
    };
    if tip == intent.new_oid {
        return Ok(AfterTheCarry::WhereTheMergePutIt);
    }
    // A branch that is not built on this merge is one this command may not write over: the record keeps the person's bytes and the
    //  command that owns the moved checkout puts them back.
    let built_on_it: bool =
        is_ancestor_awaiting(place, root, intent.new_oid.as_str(), tip.as_str()).await.map_err(|refusal| refusal.error.0)?;
    if !built_on_it {
        return Ok(AfterTheCarry::Waiting(the_branch_is_not_built_on_it(reference, target, tip.as_str())));
    }
    let mut carried: Vec<u8> = Vec::new();
    return match carry_onto_awaiting(place, root, intent, tip.as_str(), reference, &mut carried).await {
        Ok(true) => Ok(AfterTheCarry::OntoTheMovedHead(the_carry_onto_the_branch_as_it_is(reference, target, tip.as_str(), carried.as_slice()))),
        // The lock stays held, so the next command carries the same bytes.
        Ok(false) => Ok(AfterTheCarry::Waiting(the_index_kept_it(reference, target, tip.as_str()))),
        Err(refusal) => Ok(AfterTheCarry::Waiting(format!("{}\n", refusal.error.0.trim_end_matches('\n')))),
    };
}

/// What:
///  The line a merge prints when the branch was built on it while its own carry ran.
/// Why:
///   The carry wrote nothing to the paths the branch's newer commit holds,
/// so the person must read why the lines
///  that follow name the branch rather than their edit alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ${target} moved to ${head} while this merge carried your work, so ...`
/// ```
fn the_carry_onto_the_branch_as_it_is(reference: &str, target: &str, head: &str, carried: &[u8]) -> String {
    let mut lines: String = format!(
        "monochromatic-git-policy-cli: git merge {reference}: {target} moved to {head} while this merge carried your work, so your work in \
         the merge's paths was merged with the branch as it is now.\n"
    );
    lines.push_str(String::from_utf8_lossy(carried).as_ref());
    return lines;
}

/// What:
///  The line for a branch that moved away from the merge while its carry ran.
/// Why:
///   This merge is not in the branch any more,
/// so the person's work is put back by the command that owns the moved
///  checkout,
/// and this one names where their bytes are until then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ${target} moved to ${head} ... is not built on this merge ...`
/// ```
fn the_branch_is_not_built_on_it(reference: &str, target: &str, head: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: {target} moved to {head} while this merge carried your work, and that commit \
         is not built on this merge, so your work in the merge's paths is kept in the merge's record and in the objects that record names. \
         The next git command in this worktree puts it back; nothing of yours was removed.\n"
    );
}

/// What:
///  The line for a branch that moved onto the merge while another program held the index.
/// Why:
///   The carry onto the moved head waits for the index lock,
/// so the record and the ref stay and the next command
///  writes the same person's bytes;
/// the command that ends here says where they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ${target} moved to ${head} ... another program holds ... the index ...`
/// ```
fn the_index_kept_it(reference: &str, target: &str, head: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: {target} moved to {head} while this merge carried your work, and another \
         program holds this repository's index, so your work in the merge's paths is kept in the merge's record and in the objects that \
         record names. The next git command in this worktree merges it with the branch as it is then; nothing of yours was removed.\n"
    );
}

/// What:
///  The line for a branch this command could not read after the merge landed.
/// Why:
///   A branch read that fails cannot prove the checkout matches the merge,
/// so nothing is dropped and the next command
///  decides from a reading it makes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ${target} could not be read after this merge landed ...`
/// ```
fn the_branch_could_not_be_read(reference: &str, target: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: {target} could not be read after this merge landed, so your work in the \
         merge's paths is kept in the merge's record and in the objects that record names. The next git command in this worktree merges it \
         with the branch as it is then; nothing of yours was removed.\n"
    );
}

/// The lines and the endings a moved branch produces.
#[cfg(test)]
#[path = "merge_isolation_apply_moved_tests.rs"]
mod tests;
