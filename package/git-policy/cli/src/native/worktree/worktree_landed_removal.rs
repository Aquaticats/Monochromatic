//! What:
//!  The removal of every wrapper-made worktree whose branch a merge just landed on the branch it ran in.
//! Why:
//!  On btrfs each worktree the wrapper adds is a snapshot,
//!  and a snapshot keeps every file of the worktree it was taken from as it was at the add,
//!  ignored build output included,
//!  until the snapshot itself goes.
//!  A finished worktree that nobody removes therefore pins a generation of that tree's files,
//!  and the space shows up nowhere until all of them are gone.
//!  Nothing removed them:
//!  `git worktree remove` was the only route,
//!  and a person had to run it.
//!  The event that ends a worktree's use is its branch landing,
//!  and the merge that lands the branch sees it,
//!  so this runs after that merge and answers with what it removed and what it left.
//!  A worktree is removed only when all of these hold:
//!  the wrapper recorded that it made it (`worktree_added_by_the_wrapper.rs`),
//!  the branch it has checked out holds a commit of its own,
//!  it is not the worktree the command runs in,
//!  it is not locked,
//!  no merge,
//!  rebase,
//!  cherry-pick,
//!  revert or bisect is in progress,
//!  no process works inside it,
//!  every ignored file of it is also in the worktree the merge ran in,
//!  and Git removes it without `--force`.
//!  The ignored files are the one hole Git leaves:
//! `git worktree remove` deletes what Git ignores
//!  without a word,
//! and the btrfs destroy takes the whole directory,
//!  so a file the person wrote there after the add would go with it.
//!  The source is the proof available (`worktree_landed_ignored_files.rs`):
//!  an ignored file this repository's worktree holds with the same kind,
//! mode and bytes is state the repository already has,
//!  and one it does not hold that way may be the person's,
//! so the worktree stays and the file is named.
//!  A branch that has never moved since it was made holds no commit of its own,
//!  and no merge brought any of its work in:
//!  a merge that brings in another branch's commits brings every branch sitting on those commits along in the same set,
//!  so `git merge main` inside a feature worktree removed a worktree a person had just added
//!  (a cross-vendor audit of `574063773d`,
//! finding F4).
//!  The oldest entry of the branch's own reflog names the commit it was made at,
//!  and a branch whose tip is that commit has no commit of its own;
//!  a repository that keeps no reflog for it proves nothing,
//!  and the branch stays a candidate as it did before
//!  (`worktree_branch_reflog.rs`).
//!  A branch that has moved holds no commit of its own either when another branch's own reflog names the commit it holds:
//!  its own reflog records a move there from another commit,
//!  while a branch created at a commit records no commit as the value before it and proves nothing.
//!  A person who runs `git reset --hard main` in a task worktree moves its branch onto main's commit and makes nothing,
//!  and the tip differs from the commit the branch was made at,
//!  so movement alone read it as the branch's own work and the merge that brought main in took the worktree
//!  (a cross-vendor review of the landed sweep,
//!  round 208).
//!  The branch whose reflog is asked need not still sit on that commit:
//!  one that committed again names the same commit in an older line,
//!  and a rule that demanded the tip equal the candidate's found no witness and took the worktree anyway.
//!  The one move this merge wrote in the branch it ran in is no witness,
//!  because a fast-forward records exactly that line and it would otherwise read as another branch's work.
//!  A branch moved onto a commit this one made is the mirror of that and keeps the worktree,
//!  since Git records no owner of a commit,
//!  and a landed worktree whose tip another branch's history passed through stays the same way:
//!  a worktree this rule cannot prove it may remove stays.
//!  Ignored directories are the build state this sweep exists to give back and go with the worktree.
//!  The process the sweep runs in counts as one working in a worktree:
//!  `-C` names the repository a command works on,
//!  never its own working folder,
//!  so excluding the worktree the command runs in by its path did not cover
//!  a wrapper a program started in a landed one,
//!  and the sweep removed the folder that program had set the command loose in
//!  (a cross-vendor audit of the landed sweep).
//!  The worktree then stays,
//! the notice says this command was started in it,
//!  and a person removes it when nothing there is theirs.
//!  What is left is Git's own decision in Git's own words:
//!  the removal runs real `git worktree remove` unless the btrfs destroy applies,
//!  so a worktree holding a modified or untracked file is refused by Git exactly as it is for a person,
//!  and what the person reads here is Git's sentence.
//!  Everything else is named with its reason and left alone.
//!  What the removal deletes beside the files Git keeps is read again inside the removal,
//!  after its proof of cleanliness and immediately before the destroy:
//!  a worktree a command added inside the candidate while the sweep ran is in the registrations only after that command
//!  finishes,
//!  so the list the sweep read at its start proves nothing about it,
//!  and the removal took the nested worktree's changed files and its own record with the parent
//!  (a cross-vendor review of the landed sweep,
//!  round 208);
//!  a push failure recorded in the candidate's own Git directory after the one removal snapshot the sweep takes went
//!  with that directory,
//!  so the snapshot is retaken there too,
//!  and a worktree whose records could not all be copied stays and says where they are;
//!  what no look can keep is an add or a record that arrives between that last look and the destroy,
//!  which is the window `worktree_remove.rs` takes for every write and lock in the tree it deletes.
//!  A commit a reference lock holds in a landed worktree lives in that worktree's own Git directory
//!  and on its per-worktree reference,
//! and both go with the removal,
//!  so every held commit reaches `refs/cli-git/kept/<batch>` of the common Git directory before a candidate is examined
//!  (`worktree_removal_held_commits.rs`),
//!  and the person reads where the commit is before reading that the worktree went.
//!  A removal that the fast destroy cannot take is handed to the commit service instead of run here:
//!  removing a worktree Git removes file by file reads every tracked file of it and can take minutes,
//!  and it is a step that may not be run twice,
//!  so a merge that ran it kept the person's command open for as long as the deletion took
//!  (the coordinator's integration passes on 2026-11-11 waited more than twenty seconds on one,
//!  `still waiting for git worktree remove ..., a bulk step ..., it is a step that may not be run twice`).
//!  The record that names the removal outlives the command,
//!  the repository's commit service performs it,
//!  and Git's own words for a refusal settle there as the record's outcome.
//!  Everything proved here is proved again by that service immediately before it destroys anything
//!  (`worktree_removal_second_look.rs`),
//!  because this proof is spent when the record is written and the destruction happens later:
//!  a worktree added inside the candidate,
//!  an ignored file written in it,
//!  a push failure its own Git directory records,
//!  a process that starts working in it and a commit a reference lock then holds are all read there again,
//!  and a worktree that no longer meets a condition stays and is named in the repository's kept-worktree notices.
//!  Nothing here can fail the merge that woke it.
//!  The sweep is split across siblings because this file is over the line limit:
//!  this one decides which candidates this merge landed and walks them,
//!  `worktree_landed_removal_records.rs` holds what it is asked with and answers with,
//!  `worktree_landed_removal_landed_branches.rs` decides which branches landed,
//!  and `worktree_landed_removal_one_candidate.rs` examines and removes one of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sweep = sweepLandedWorktrees({ context, commonDir, currentRoot, procRoot }, before, now, branch);
//! ```

use crate::transaction_git::GitContext;
use crate::worktree_landed_registrations::{
    Registration,
    registrations_of,
};
use crate::worktree_landed_removal_landed_branches::{
    Landing,
    ThisMerge,
    another_branch_took_the_tip,
    branches_merged_into,
    holds_its_own_commits,
    reference_of,
};
use crate::worktree_landed_removal_one_candidate::{
    Examined,
    examine_and_remove,
};
pub use crate::worktree_landed_removal_records::{
    HandedOverWorktree,
    KeptWorktree,
    LandedPlace,
    LandedSweep,
    RemovedAroundWorktree,
    RemovedWorktree,
    UnfinishedWorktree,
    UnprovenWorktree,
};
use crate::worktree_removal_held_commits::{
    KeptCommits,
    keep_held_commits,
};
use crate::worktree_removal_rescue::snapshot_linked_worktrees;
use crate::worktree_remove::LinuxOperations;
use std::collections::HashSet;
use std::path::Path;

/// What:
///  The worktrees whose branch this merge landed,
///  removed when every condition holds and left with their reason otherwise.
/// Why:
///   The event that ends a worktree's use is its branch landing,
///  so the set is the branches contained in the new head and not those already contained in the head before the merge:
///  a branch that merely sat on an older commit of the landed branch is not work this merge brought in,
///  and neither is one that has never moved since it was made:
///  the commits it points at came from another branch,
///  which the merge brought in on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepLandedWorktrees(place, before, now, landedBranch)
/// ```
pub fn sweep_landed_worktrees(
    place: &LandedPlace<'_>,
    before: &str,
    now: &str,
    landed_branch: &str,
) -> LandedSweep {
    let mut sweep: LandedSweep = LandedSweep {
        landed_branch: String::from(landed_branch),
        ..LandedSweep::default()
    };
    let landed: Vec<Landing> = match branches_merged_into(place, now) {
        Ok(found) => found,
        Err(reason) => {
            sweep.failed = Some(reason);
            return sweep;
        }
    };
    let earlier: HashSet<String> = match branches_merged_into(place, before) {
        Ok(found) => found.into_iter().map(reference_of).collect(),
        Err(reason) => {
            sweep.failed = Some(reason);
            return sweep;
        }
    };
    let mut candidates: HashSet<String> = HashSet::new();
    for one in &landed {
        if earlier.contains(one.reference.as_str()) {
            continue;
        }
        if !holds_its_own_commits(place.common_dir, one.reference.as_str(), one.tip.as_str()) {
            continue;
        }
        if another_branch_took_the_tip(
            landed.as_slice(),
            place.common_dir,
            one.reference.as_str(),
            one.tip.as_str(),
            ThisMerge {
                branch: landed_branch,
                before,
                now,
            },
        ) {
            continue;
        }
        candidates.insert(one.reference.clone());
    }
    if candidates.is_empty() {
        return sweep;
    }
    // A held commit lives on the worktree's own reference and in its own Git directory, and both go with the removal, so
    // every one of them reaches a reference of this repository first: the ordinary `git worktree remove` keeps them the same
    // way (`worktree_removal_held_commits.rs`). Nothing here can fail the merge, so the lines are kept for the notice.
    // A held commit that no reference could keep lives only in the worktree a removal would delete, so then nothing is
    // removed: the refusal's paragraphs say which commit and what to do, and the sweep reports them as its reason.
    let kept: KeptCommits = keep_held_commits(place.common_dir, place.context);
    sweep.kept_commits = kept.said;
    if !kept.refused.is_empty() {
        sweep.refused_commits = true;
        sweep.failed = Some(kept.refused.join("\n"));
        return sweep;
    }
    // The unshown push failures and unfinished pushes of every linked worktree live in its own Git directory, which the
    // removal deletes, so they are copied into the removal snapshot first, exactly as the person's `git worktree remove`
    // path does (`snapshot_before_worktree_removal`): the sweep runs no such command, and without this its removals
    // destroyed those records with no word to anyone (a cross-vendor audit of `574063773d`, finding F3).
    sweep.rescue_problems = snapshot_linked_worktrees(place.common_dir);
    let registered: Vec<Registration> = match registrations_of(place.common_dir) {
        Ok(found) => found,
        Err(reason) => {
            sweep.failed = Some(reason);
            return sweep;
        }
    };
    // A service that does not answer costs this merge one wait and not one per candidate: the record is what outlives
    // the command, and every later command of the repository wakes the same service
    // (`HAND_OVER_GREETING_WAIT`, `wait_ceilings.rs`). A branch named for a merge that never waits on a removal had the
    // merge wait about two seconds for each of four candidates against a stopped service
    // (an audit of the release changeover, finding 7).
    let wake_failed: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
    let told = |context: &GitContext, git_dir: &Path| -> Result<(), String> {
        if let Some(already) = wake_failed.borrow().as_ref() {
            return Err(already.clone());
        }
        let answer: Result<(), String> = (place.told)(context, git_dir);
        if let Err(reason) = &answer {
            *wake_failed.borrow_mut() = Some(reason.clone());
        }
        return answer;
    };
    for one in &registered {
        if !candidates.contains(one.branch.as_str())
            || place.current_root == Some(one.worktree.as_path())
        {
            continue;
        }
        // A registration that was not there when the merge started belongs to work that began after it, so this merge is
        // not the event that ended it, and it is not examined at all.
        if !place.registered_before.holds(one) {
            continue;
        }
        match examine_and_remove(place, &told, one, registered.as_slice(), &LinuxOperations) {
            Examined::Removed => sweep.removed.push(RemovedWorktree {
                worktree: one.worktree.clone(),
                branch: one.branch.clone(),
            }),
            Examined::RemovedAround(left) => sweep.removed_around.push(RemovedAroundWorktree {
                worktree: one.worktree.clone(),
                branch: one.branch.clone(),
                left,
            }),
            Examined::EmptiedAround => sweep.removed_emptied.push(RemovedWorktree {
                worktree: one.worktree.clone(),
                branch: one.branch.clone(),
            }),
            Examined::Unfinished(report) => sweep.unfinished.push(UnfinishedWorktree {
                worktree: one.worktree.clone(),
                branch: one.branch.clone(),
                report,
            }),
            Examined::Refused(report) => sweep.refused.push(KeptWorktree {
                worktree: one.worktree.clone(),
                branch: one.branch.clone(),
                why: report,
            }),
            Examined::Handed(record) => sweep.handed_over.push(HandedOverWorktree {
                worktree: one.worktree.clone(),
                branch: one.branch.clone(),
                record,
            }),
            Examined::Kept(why) => sweep.kept.push(KeptWorktree {
                worktree: one.worktree.clone(),
                branch: one.branch.clone(),
                why,
            }),
            Examined::Unproven => sweep.unproven.push(UnprovenWorktree {
                worktree: one.worktree.clone(),
                branch: one.branch.clone(),
            }),
        }
    }
    return sweep;
}

/// The controls of the sweep stay out of the release executable.
#[cfg(test)]
#[path = "worktree_landed_removal_tests.rs"]
mod tests;
