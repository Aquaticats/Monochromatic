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
//!  and the branch stays a candidate as it did before.
//!  A branch that has moved holds no commit of its own either when another branch this merge did not land on had already
//!  moved onto the same commit:
//!  its own reflog records that move from another commit,
//!  while a branch created at a commit records no commit as the value before it and proves nothing.
//!  A person who runs `git reset --hard main` in a task worktree moves its branch onto main's commit and makes nothing,
//!  and the tip differs from the commit the branch was made at,
//!  so movement alone read it as the branch's own work and the merge that brought main in took the worktree
//!  (a cross-vendor review of the landed sweep,
//!  round 208).
//!  A branch moved onto a commit this one made is the mirror of that,
//!  and keeps the worktree:
//!  Git records no owner of a commit,
//!  and a branch moved onto a commit another branch holds below its own tip is no state this rule can see at all,
//!  so a worktree it cannot prove it may remove stays.
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
//!  What the removal deletes besides the files Git keeps is read again immediately before it runs:
//!  a worktree a command added inside the candidate while the sweep ran is in the registrations only after that command
//!  finishes,
//!  so the list the sweep read at its start proves nothing about it,
//!  and the removal took the nested worktree's changed files and its own record with the parent
//!  (a cross-vendor review of the landed sweep,
//!  round 208);
//!  and a push failure recorded in the candidate's own Git directory after the one removal snapshot the sweep takes went
//!  with that directory,
//!  so the snapshot is retaken for the worktree being removed just as the removal runs,
//!  and a worktree whose records could not all be copied stays and says where they are.
//!  A commit a reference lock holds in a landed worktree lives in that worktree's own Git directory
//!  and on its per-worktree reference,
//! and both go with the removal,
//!  so every held commit reaches `refs/cli-git/kept/<batch>` of the common Git directory before a candidate is examined
//!  (`worktree_removal_held_commits.rs`),
//!  and the person reads where the commit is before reading that the worktree went.
//!  Nothing here can fail the merge that woke it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sweep = sweepLandedWorktrees({ context, commonDir, currentRoot, procRoot }, before, now, branch);
//! ```

use crate::merge_isolation_run_types::IN_PROGRESS_MARKERS;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_added_by_the_wrapper::added_by_the_wrapper;
use crate::worktree_branch_reflog::{branch_made_at, branch_moved_onto};
use crate::worktree_process_inside::process_working_inside;
use crate::worktree_landed_ignored_files::ignored_entries_alone_here;
use crate::worktree_landed_registrations::{RegisteredBefore, Registration, registrations_of};
use crate::worktree_removal_clears_the_pool::clear_the_pool_of_a_removed_worktree;
use crate::worktree_removal_held_commits::{KeptCommits, keep_held_commits};
use crate::worktree_removal_rescue::{snapshot_linked_worktrees, snapshot_one};
use crate::worktree_remove::{RemoveOutcome, RemoveRequest, remove_subvolume_worktree};
use crate::worktree_remove_plan::RemovePlan;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLandedRemoval";

/// The marks of an unfinished operation the sweep keeps a worktree for,
/// on top of the ones Git's own merge isolation already checks.
/// Why:
///  `IN_PROGRESS_MARKERS` answers the same question for a merge the wrapper runs itself,
///  so its list is reused rather than written again,
///  and this one names what that question has to include for a removal.
///  An unfinished bisect leaves `BISECT_LOG` in a worktree and never touches `HEAD`:
///  the checkout stays on its branch and every file stays clean,
///  and `git worktree remove` deletes the worktree and that log without a word
///  (measured with Git 2.56 on a fixture:
/// exit 0,
/// the log gone).
///  The sweep's own module doc promises a bisect keeps the worktree,
///  and it did not,
/// which a cross-vendor audit of the landed sweep reproduced.
const FURTHER_UNFINISHED_MARKERS: [&str; 1] = ["BISECT_LOG"];

/// What:
///  Where one sweep looks,
/// and which worktree it must never remove.
/// Why:
///   The repository decides which registrations count,
///  the worktree the command runs in is the person's own,
///  and the process table answers whether anything is working in a candidate.
pub struct LandedPlace<'a> {
    /// How to start real Git.
    pub context: &'a GitContext,
    /// The directory every Git start of the sweep runs in.
    pub cwd: &'a Path,
    /// The canonical common Git directory of the repository the merge ran in.
    pub common_dir: &'a Path,
    /// The root of the worktree the merge ran in,
    /// which is never removed.
    pub current_root: Option<&'a Path>,
    /// The process table to read,
    /// which a control replaces with a fixture listing.
    pub proc_root: &'a Path,
    /// The registrations the repository had before the merge ran.
    ///
    /// A worktree a command added while the merge ran,
    /// or after it landed,
    /// is not in it,
    /// and it did not exist when the merge did.
    pub(crate) registered_before: &'a RegisteredBefore,
}

/// A worktree this sweep removed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedWorktree {
    /// The worktree directory.
    pub worktree: PathBuf,
    /// The branch it had checked out,
    /// which is kept.
    pub branch: String,
}

/// A worktree this sweep left,
/// and why.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeptWorktree {
    /// The worktree directory.
    pub worktree: PathBuf,
    /// The branch it has checked out.
    pub branch: String,
    /// What kept it,
    /// in words a person reads without knowing this program.
    pub why: String,
}

/// A worktree whose branch landed and which nothing proves the wrapper made.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnprovenWorktree {
    /// The worktree directory.
    pub worktree: PathBuf,
    /// The branch it has checked out.
    pub branch: String,
}

/// What one sweep did,
/// and the landed branch it did it for.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LandedSweep {
    /// The branch the merge ran in,
    /// which received the landed commits.
    pub landed_branch: String,
    /// What it removed.
    pub removed: Vec<RemovedWorktree>,
    /// What it left,
    /// with the reason.
    pub kept: Vec<KeptWorktree>,
    /// What landed and the wrapper cannot prove it made.
    pub unproven: Vec<UnprovenWorktree>,
    /// The lines that say which commit a removal would have deleted,
    /// and the reference of this repository that now names it.
    pub kept_commits: Vec<String>,
    /// What could not be saved before a removal the sweep was about to make,
    /// which the person reads as it stands.
    /// A removal deletes the worktree's own Git directory,
    /// and with it the push failures nobody saw and the pushes the service had not run yet,
    /// so they are copied into the removal snapshot first and shown by the next command
    /// (`worktree_removal_rescue.rs`).
    pub rescue_problems: Vec<String>,
    /// Whether a held commit no reference of this repository could hold stopped this sweep,
    /// which no later merge finishes.
    pub refused_commits: bool,
    /// Why no candidate was examined,
    /// when the repository could not be read.
    pub failed: Option<String>,
}


/// One branch a Git query named,
/// with the commit it points at.
/// Why:
///   Which worktrees a merge landed is decided from the branches its new head contains and the commit each one holds:
///  a branch's own commits are what says it landed,
///  and a name alone cannot answer that.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Landing {
    /// The full reference name.
    reference: String,
    /// The commit it points at now.
    tip: String,
}

/// What:
///  The branches that are fully contained in `commit`,
///  each with the commit it points at.
/// Why:
///   A branch has landed when every commit it holds is in the branch that received the merge,
///  which one Git start answers for every branch,
///  and the commit each one points at is what says whether it holds a commit of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const merged = branchesMergedInto(context, commit); // throws the reason
/// ```
fn branches_merged_into(place: &LandedPlace<'_>, commit: &str) -> Result<Vec<Landing>, String> {
    let arguments: Vec<OsString> = ["for-each-ref", "--merged", commit, "--format=%(refname)%09%(objectname)", "refs/heads"].map(OsString::from).to_vec();
    let output: GitOutput = match run_git(place.context, &GitRequest::new(place.cwd, arguments.as_slice())) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("Git cannot list the branches merged into {commit} ({error})")),
    };
    if output.code != Some(0) {
        return Err(format!("Git cannot list the branches merged into {commit}: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim()));
    }
    let mut found: Vec<Landing> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        let (reference, tip) = line.split_once('\t').unwrap_or((line, ""));
        found.push(Landing { reference: reference.to_owned(), tip: tip.to_owned() });
    }
    return Ok(found);
}

/// The reference a landing names,
/// for the set of branches that were already contained in a head.
fn reference_of(one: Landing) -> String {
    return one.reference;
}

/// Whether the branch holds a commit of its own,
/// which is false only when its reflog says it has not moved since it was made.
/// Why:
///   A branch made at a commit another branch holds and left there did no work of its own,
///  so nothing of it can land,
///  and a merge that brings that other branch in must not take its worktree
///  (a cross-vendor audit of `574063773d`,
///  finding F4).
///  A repository that keeps no reflog for the branch proves nothing about it,
///  so it stays a candidate as it was before this rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdsItsOwnCommits(commonDir, 'refs/heads/task-z', 'dfc6c4e...') // false when its reflog begins at that commit
/// ```
fn holds_its_own_commits(common_dir: &Path, reference: &str, tip: &str) -> bool {
    return branch_made_at(common_dir, reference) != Some(tip.to_owned());
}

/// Whether a branch this merge did not land on moved onto `tip`,
/// which is a commit the branch named by `reference` did not make.
/// Why:
///  A person who runs `git reset --hard main` in a task worktree moves its branch onto main's commit and makes nothing.
///  The tip then differs from the commit that worktree's own reflog says the branch was made at,
///  so the rule above read the move as work of the branch's own and the merge that brought main in removed the worktree
///  (a cross-vendor review of the landed sweep,
///  round 208).
///  A branch at the head this merge landed on is not counted:
///  a fast-forward merge leaves the landed branch on the tip the landed worktree's branch has,
///  and that is where the work of that branch is.
///  A branch moved onto a commit another branch holds below its own tip is the state this rule cannot see,
///  because Git records no owner of a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// anotherBranchTookTheTip(landed, commonDir, 'refs/heads/task-z', 'dfc6c4e...', 'a1b2c3d...') // true when a branch moved onto another branch's commit
/// ```
fn another_branch_took_the_tip(landed: &[Landing], common_dir: &Path, reference: &str, tip: &str, landed_head: &str) -> bool {
    for other in landed {
        if other.reference == reference || other.tip != tip || other.tip == landed_head {
            continue;
        }
        if branch_moved_onto(common_dir, other.reference.as_str(), tip) {
            return true;
        }
    }
    return false;
}

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
pub fn sweep_landed_worktrees(place: &LandedPlace<'_>, before: &str, now: &str, landed_branch: &str) -> LandedSweep {
    let mut sweep: LandedSweep = LandedSweep { landed_branch: String::from(landed_branch), ..LandedSweep::default() };
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
        if another_branch_took_the_tip(landed.as_slice(), place.common_dir, one.reference.as_str(), one.tip.as_str(), now) {
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
    for one in &registered {
        if !candidates.contains(one.branch.as_str()) || place.current_root == Some(one.worktree.as_path()) {
            continue;
        }
        // A registration that was not there when the merge started belongs to work that began after it, so this merge is
        // not the event that ended it, and it is not examined at all.
        if !place.registered_before.holds(one) {
            continue;
        }
        match examine_and_remove(place, one, registered.as_slice()) {
            Examined::Removed => sweep.removed.push(RemovedWorktree { worktree: one.worktree.clone(), branch: one.branch.clone() }),
            Examined::Kept(why) => sweep.kept.push(KeptWorktree { worktree: one.worktree.clone(), branch: one.branch.clone(), why }),
            Examined::Unproven => sweep.unproven.push(UnprovenWorktree { worktree: one.worktree.clone(), branch: one.branch.clone() }),
        }
    }
    return sweep;
}

/// What:
///  Remove one worktree,
/// by one subvolume destroy when the filesystem allows it and by Git otherwise.
/// Why:
///  On btrfs the destroy is what gives the snapshot's space back;
///  Git's own removal is the line every other case keeps,
///  and it is also the answer when the mount forbids the destroy:
///  it deletes what Git ignores and refuses what it would keep,
///  so nothing a person wrote that Git keeps is lost,
///  and its own words say why when it refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remove(place, registration) // throws Git's refusal, in Git's words
/// ```
fn remove(place: &LandedPlace<'_>, one: &Registration) -> Result<(), String> {
    let plan: RemovePlan = RemovePlan { target: one.worktree.clone(), operand: OsString::from(one.worktree.as_os_str()), force: false };
    let request: RemoveRequest<'_> = RemoveRequest { context: place.context, plan: &plan, common_dir: place.common_dir };
    match remove_subvolume_worktree(&request) {
        RemoveOutcome::Removed => {
            crate::diagnostic_log::debug(TAG, format!("destroyed the subvolume of {}", one.worktree.display()).as_str());
            return Ok(());
        }
        RemoveOutcome::Refused(report) | RemoveOutcome::Incomplete(report) => return Err(report),
        RemoveOutcome::LeftToGit => {}
    }
    let arguments: Vec<OsString> = vec![OsString::from("worktree"), OsString::from("remove"), one.worktree.clone().into_os_string()];
    let output: GitOutput = match run_git(place.context, &GitRequest::new(place.cwd, arguments.as_slice())) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("Git could not be started to remove it ({error})")),
    };
    if output.code == Some(0) {
        return Ok(());
    }
    let words: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned();
    return Err(format!("Git refused to remove it: {words}"));
}

/// What:
///  The other worktrees this repository registers whose checkout lies inside `one`,
///  which a removal of `one` would take with it.
/// Why:
///  A worktree can be added from inside another one,
/// and this repository's own agents do it:
///  the nested checkout is a directory of the outer worktree,
///  usually under an ignored name.
///  Removing the outer one took the nested worktree's files,
///  a modified tracked file and an untracked file among them,
///  and left its registration naming a path that is gone,
///  so `git worktree prune` could then delete the nested worktree's own record,
///  its reflog and the commits held in it
///  (a cross-vendor audit of `574063773d`,
///  finding F2).
///  The paths compared are the canonical ones Git's own registration names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreesInside(one, registered) // the registrations whose worktree starts with one.worktree
/// ```
fn worktrees_inside<'a>(one: &Registration, registered: &'a [Registration]) -> Vec<&'a Registration> {
    let mut inside: Vec<&'a Registration> = Vec::new();
    for other in registered {
        if other.worktree != one.worktree && other.worktree.starts_with(one.worktree.as_path()) {
            inside.push(other);
        }
    }
    return inside;
}

/// What:
///  The paths of the worktrees found inside,
/// as the person reads them,
///  a few named and the rest counted.
/// Why:
///  A person decides about their own worktrees,
///  so the ones that keep this one are named,
///  and a list longer than a few would bury the sentence it sits in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesOf(inside) // '`a` and 2 more'
/// ```
fn names_of(inside: &[&Registration]) -> String {
    const SHOWN: usize = 2;
    let mut parts: Vec<String> = Vec::new();
    for other in inside.iter().take(SHOWN) {
        parts.push(format!("`{}`", other.worktree.display()));
    }
    let rest: usize = inside.len().saturating_sub(parts.len());
    if rest > 0 {
        parts.push(format!("and {rest} more"));
    }
    return parts.join(", ");
}

/// What:
///  Why a worktree that holds another registered worktree stays.
/// Why:
///  The two checks that keep it,
///  the one over the registrations the sweep read at its start and the one over the registrations read as the removal
///  runs,
///  answer a person in the same words,
///  so the sentence is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nestedWorktreeReason(inside) // the sentence a kept parent prints
/// ```
fn nested_worktree_reason(inside: &[&Registration]) -> String {
    return format!(
        "another worktree this repository registers is inside it ({}), and a removal of this one would take that worktree's files and its own record with it",
        names_of(inside)
    );
}

/// What:
///  What an unfinished operation's mark means,
///  in a person's words.
/// Why:
///   The mark is the file Git leaves behind,
///  and `BISECT_LOG` says nothing to a person who does not know Git,
///  while `MERGE_HEAD` is Git's own name for an operation they were in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unfinishedOperationReason('BISECT_LOG') // 'a bisect is unfinished there ...'
/// ```
fn unfinished_operation_reason(marker: &str) -> String {
    return match marker {
        "BISECT_LOG" => format!("a bisect is unfinished there, and its {marker} is still in the worktree"),
        _ => format!("a {marker} stands in it, so an operation there is unfinished"),
    };
}

/// What one candidate came to.
enum Examined {
    /// The worktree and its registration are gone.
    Removed,
    /// It stays,
    /// for this reason.
    Kept(String),
    /// Nothing proves the wrapper made it.
    Unproven,
}

/// What:
///  The conditions one worktree must meet,
///  and its removal.
/// Why:
///   Each answer is a decision a person can read,
///  and the removal runs only when all of them hold,
///  so a worktree someone is still using is never the one that goes.
///  The list of registrations the sweep read at its start answers early,
///  and it is not what the removal rests on:
///  what a removal deletes besides the files Git keeps is read again here,
///  immediately before it runs,
///  because a worktree a command adds inside this one while the sweep runs,
///  and a push failure recorded in this one's own Git directory,
///  appear only after that first reading
///  (`nested_worktree_reason`,
///  `snapshot_one`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// examineAndRemove(place, registration, registered)
/// ```
fn examine_and_remove(place: &LandedPlace<'_>, one: &Registration, registered: &[Registration]) -> Examined {
    if !added_by_the_wrapper(one.admin.as_path(), one.worktree.as_path()) {
        return Examined::Unproven;
    }
    let inside: Vec<&Registration> = worktrees_inside(one, registered);
    if !inside.is_empty() {
        return Examined::Kept(nested_worktree_reason(inside.as_slice()));
    }
    if one.admin.join("locked").exists() {
        return Examined::Kept(String::from("it is locked, and a lock is a promise to keep it"));
    }
    for marker in IN_PROGRESS_MARKERS.iter().copied().chain(FURTHER_UNFINISHED_MARKERS.iter().copied()) {
        if one.admin.join(marker).exists() {
            return Examined::Kept(unfinished_operation_reason(marker));
        }
    }
    // How long the worktree has existed, which is what tells a process that hides its working folder whether it could be
    //  inside at all: a process older than the directory cannot have changed into it.
    if let Some(working) = process_working_inside(one.worktree.as_path(), place.proc_root, age_of(one.worktree.as_path())) {
        return Examined::Kept(working.why());
    }
    match place.current_root {
        Some(source) => match ignored_entries_alone_here(place.context, one.worktree.as_path(), one.admin.as_path(), source) {
            Ok(alone) if alone.is_empty() => {}
            Ok(alone) => {
                return Examined::Kept(format!(
                    "it holds ignored files that the worktree this merge ran in does not hold with the same bytes ({}), and nothing proves they are not yours",
                    alone.list()
                ));
            }
            Err(reason) => {
                return Examined::Kept(format!("nothing proves the ignored files there are not yours, so nothing is removed: {reason}"));
            }
        },
        None => {
            return Examined::Kept(String::from(
                "this command runs in a repository without a worktree, so nothing proves the ignored files there are not yours",
            ));
        }
    }
    // The unshown push failures and the unfinished pushes of this worktree live in its own Git directory, which the
    // removal deletes. The sweep copied them before it examined a candidate, and the push service writes a refused push at
    // any moment, so the copy is made again here, as the removal runs: a record written after the sweep's own copy went with
    // the directory (a cross-vendor review of the landed sweep, round 208). A record that cannot be copied keeps the
    // worktree, because the copy is the only thing left of it after the removal.
    let unsaved: Vec<String> = snapshot_one(place.common_dir, one.admin.as_path());
    if !unsaved.is_empty() {
        return Examined::Kept(format!(
            "the unshown push failures and the unfinished pushes its own Git directory holds could not all be copied into a removal snapshot, so nothing is removed and they stay in {}: {}",
            one.admin.display(),
            unsaved.join(" ")
        ));
    }
    // A worktree a command added inside this one while the sweep ran is in the registrations only once that command has
    // finished, so the list read at the start of the sweep proves nothing about it: the removal took the nested worktree's
    // changed files and its own record with the parent (a cross-vendor review of the landed sweep, round 208).
    let registered_now: Vec<Registration> = match registrations_of(place.common_dir) {
        Ok(found) => found,
        Err(reason) => return Examined::Kept(format!("nothing proves no other worktree lies inside this one, so nothing is removed: {reason}")),
    };
    let inside_now: Vec<&Registration> = worktrees_inside(one, registered_now.as_slice());
    if !inside_now.is_empty() {
        return Examined::Kept(nested_worktree_reason(inside_now.as_slice()));
    }
    let removal: Result<(), String> = remove(place, one);
    if removal.is_ok() {
        // A pool names the worktree it serves in every member's config and sits in the common Git directory, which no
        // removal deletes, so the removal that just proved the worktree is gone takes it (`worktree_removal_clears_the_pool.rs`).
        clear_the_pool_of_a_removed_worktree(place.common_dir, one.admin.as_path());
    }
    return match removal {
        Ok(()) => Examined::Removed,
        Err(reason) => Examined::Kept(reason),
    };
}

/// What:
///  How long a directory has existed,
///  as the filesystem records when it was made.
/// Why:
///   A process that hides its working folder can only be working inside a directory that existed when it started,
///  so the worktree's age is what separates the services that start at login from a process this worktree's own work began.
///  A filesystem that records no birth time says nothing,
///  and the hidden process then keeps the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ageOf(directory) // Duration | undefined
/// ```
#[cfg(target_os = "linux")]
fn age_of(directory: &Path) -> Option<Duration> {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(directory.as_os_str().as_bytes()).ok()?;
    // SAFETY: an all-zero `statx` is a valid value for the call to fill.
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: `name` outlives the call, `found` is a live `statx`, and `AT_FDCWD` with an absolute path names the directory.
    let called: libc::c_int = unsafe { libc::statx(libc::AT_FDCWD, name.as_ptr(), 0, libc::STATX_BTIME, &mut found) };
    if called != 0 || found.stx_mask & libc::STATX_BTIME == 0 {
        return None;
    }
    let seconds: u64 = u64::try_from(found.stx_btime.tv_sec).ok()?;
    let made: SystemTime = SystemTime::UNIX_EPOCH + Duration::new(seconds, found.stx_btime.tv_nsec);
    return SystemTime::now().duration_since(made).ok();
}

// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
/// No birth time to read on a platform without `statx`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ageOf(directory) // undefined
/// ```
#[cfg(not(target_os = "linux"))]
fn age_of(_directory: &Path) -> Option<Duration> {
    return None;
}

/// The controls of the sweep stay out of the release executable.
#[cfg(test)]
#[path = "worktree_landed_removal_tests.rs"]
mod tests;
