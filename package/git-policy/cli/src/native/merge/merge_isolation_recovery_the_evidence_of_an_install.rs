//! What:
//!  What this worktree already holds of a merge,
//!  read once before anything can refuse:
//!  its changed paths,
//!  the index standings of those paths,
//!  how the worktree stands at each of them,
//!  and whether that evidence says the merge began.
//! Why:
//!   Every refusal after the reading reports from it,
//! so no path can say nothing is installed while the index holds the merge.
//!  The index is read first and the worktree second,
//! because Git installs every index entry of a merge in one step,
//!  while an independent commit of the same content leaves the branch that content
//! and no index entry of the merge at all.
//!  A read that fails refuses the merge with whatever the reading already proved,
//! so a command is never let past an index that may hold the merge.
//!  The reading is a file of its own so the recovery that decides with it
//! stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { changes, indexStanding, judgements, anyInIndex, anyInWorktree, began } = await theEvidenceOfAnInstall(event);
//! ```

/// The raw differences of an aggregate merge.
use crate::merge_isolation_changes::Change;
use crate::merge_isolation_journal::MergeIntent;
/// Where recovery runs Git.
use crate::merge_isolation_recovery_place::RecoveryPlace;
/// Whether the index and the worktree hold the merge's own entry at one path.
use crate::merge_isolation_recovery_reads::{
    holds_the_merge,
    holds_the_merges_entry,
    paths_the_branch_does_not_hold_awaiting,
};
/// What one recovery came to,
/// and the refusal that carries whether the merge began.
use crate::merge_isolation_recovery_record::MergeRefusal;
use crate::merge_isolation_recovery_steps::{
    index_standings_awaiting,
    judge_changes_awaiting,
};
/// The paths a merge changes,
/// the putting back of what a killed take left beside them,
/// the index standings,
/// and the judgement of each changed path.
use crate::merge_isolation_recovery_steps::{
    merge_changes_awaiting,
    put_back_stranded_takes_awaiting,
};
use crate::merge_isolation_the_merges_paths_judged::{
    Judged,
    Standing,
};
/// Writing the notices of what recovery did.
use std::io::Write;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  What this worktree already holds of the merge,
///  read once before anything can refuse.
/// Why:
///   Every refusal after this point reports from this one reading,
/// so no path can say nothing is installed while the index holds the merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheEvidence = { changes: Change[]; indexStanding: Standing[]; judgements: Judged[]; anyInIndex: boolean; anyInWorktree: boolean; began: boolean };
/// ```
pub(crate) struct TheEvidence {
    /// The paths the merge changes.
    pub changes: Vec<Change>,
    /// The index's standing at each of those paths.
    pub index_standing: Vec<Standing>,
    /// How the worktree stands at each of those paths.
    pub judgements: Vec<Judged>,
    /// Whether anything of the merge is in the index at all,
    /// which is what keeps a record from being only discarded.
    pub any_in_index: bool,
    /// Whether anything of the merge is in the worktree at all.
    pub any_in_worktree: bool,
    /// Whether a path the branch does not hold the merge's content for holds it here,
    /// which is what an install leaves and an independent commit does not.
    pub began: bool,
}

/// What:
///  Read what this worktree already holds of the merge,
///  and say whether that evidence means the merge began.
/// Why:
///   A merge whose own commits cannot be read may still be half installed,
/// so a command is not let past this:
/// the refusal that follows says the merge is installed even where nothing was read,
///  which is the safe answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function theEvidenceOfAnInstall({ place, root, intent, current, out }): Promise<TheEvidence>; // throws MergeRefusal
/// ```
pub(crate) async fn the_evidence_of_an_install_awaiting<W: Write + ?Sized>(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
    current: Option<&str>,
    out: &mut W,
) -> Result<TheEvidence, MergeRefusal> {
    // A merge whose own commits cannot be read may still be half installed, so a command is not let past this: the refusal
    //  that follows says the merge is installed even where nothing was read, which is the safe answer.
    let changes: Vec<Change> = match merge_changes_awaiting(place, root, intent).await {
        Ok(found) => found,
        Err(error) => {
            return Err(MergeRefusal {
                error,
                began: true,
            });
        }
    };
    // A file a command that stopped left under its private name is put back before the paths are judged: the path is empty
    //  while the file waits beside it, and would otherwise be read as someone else's.
    put_back_stranded_takes_awaiting(
        &place.git,
        root,
        place.git_dir.as_path(),
        changes.as_slice(),
        intent.carried.as_ref(),
        intent.owner.transaction_id.as_str(),
        out,
    )
    .await;
    // What this worktree already holds of the merge is read once, the index first and the worktree second, before anything can
    //  refuse; every refusal after this point reports from this one reading, so no path can say nothing is installed while the
    //  index holds the merge.
    let index_standing: Vec<Standing> =
        match index_standings_awaiting(place, root, intent.old_oid.as_str(), changes.as_slice())
            .await
        {
            Ok(found) => found,
            // An index that cannot be read may hold the merge, so a command must not read the checkout past this.
            Err(error) => {
                return Err(MergeRefusal {
                    error,
                    began: true,
                });
            }
        };
    // Whether anything of the merge is in the index at all, which is what keeps a record from being only discarded; and whether
    //  every path the merge changes holds its side in the index, which is what an install leaves because Git writes the entries
    //  in one step.
    let any_in_index: bool = index_standing.iter().any(holds_the_merges_entry);
    let every_entry_is_the_merges: bool =
        !index_standing.is_empty() && index_standing.iter().all(holds_the_merges_entry);
    let judgements: Vec<Judged> =
        match judge_changes_awaiting(place, root, changes.as_slice()).await {
            Ok(found) => found,
            // The worktree cannot be read; the index is the evidence of the install that is still there.
            Err(error) => {
                return Err(MergeRefusal {
                    error,
                    began: every_entry_is_the_merges,
                });
            }
        };
    let any_in_worktree: bool = judgements.iter().any(holds_the_merge);
    let every_file_is_the_merges: bool =
        !judgements.is_empty() && judgements.iter().all(holds_the_merge);
    // Every path holding the merge is what an install leaves.
    let every_path_is_the_merges: bool = every_entry_is_the_merges || every_file_is_the_merges;
    // Evidence of an install is a path the branch does not already hold the merge's content for and the checkout does:
    //  an independent commit of the same content gives the branch that content, while an install leaves the branch as it was,
    //  whatever another command does to the checkout afterwards.
    let branch_lacks: Vec<bool> = match current.as_deref() {
        Some(tip) => match paths_the_branch_does_not_hold_awaiting(
            place,
            root,
            tip,
            intent.new_oid.as_str(),
            changes.as_slice(),
        )
        .await
        {
            Ok(found) => found,
            Err(error) => {
                return Err(MergeRefusal {
                    error,
                    began: every_path_is_the_merges,
                });
            }
        },
        None => Vec::new(),
    };
    let mut evidence: bool = false;
    for (index, lacks) in branch_lacks.iter().enumerate() {
        let held: bool = index_standing.get(index) == Some(&Standing::New)
            || judgements.get(index).is_some_and(holds_the_merge);
        if *lacks && held {
            evidence = true;
            break;
        }
    }
    let began: bool = evidence;
    return Ok(TheEvidence {
        changes,
        index_standing,
        judgements,
        any_in_index,
        any_in_worktree,
        began,
    });
}
