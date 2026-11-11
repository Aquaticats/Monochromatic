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
//!  it is not the worktree the command runs in,
//!  it is not locked,
//!  no merge,
//!  rebase,
//!  cherry-pick,
//!  revert or bisect is in progress,
//!  no process works inside it,
//!  every ignored file of it is also in the worktree the merge ran in,
//!  and Git removes it without `--force`.
//!  The ignored files are the one hole Git leaves: `git worktree remove` deletes what Git ignores
//!  without a word, and the btrfs destroy takes the whole directory,
//!  so a file the person wrote there after the add would go with it.
//!  The source is the proof available (`worktree_landed_ignored_files.rs`):
//!  an ignored file this repository's worktree holds with the same kind, mode and bytes is state the repository already has,
//!  and one it does not hold that way may be the person's, so the worktree stays and the file is named.
//!  Ignored directories are the build state this sweep exists to give back and go with the worktree.
//!  What is left is Git's own decision in Git's own words:
//!  the removal runs real `git worktree remove` unless the btrfs destroy applies,
//!  so a worktree holding a modified or untracked file is refused by Git exactly as it is for a person,
//!  and what the person reads here is Git's sentence.
//!  Everything else is named with its reason and left alone.
//!  A commit a reference lock holds in a landed worktree lives in that worktree's own Git directory
//!  and on its per-worktree reference, and both go with the removal,
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
use crate::worktree_process_inside::process_working_inside;
use crate::worktree_landed_ignored_files::ignored_entries_alone_here;
use crate::worktree_landed_registrations::{RegisteredBefore, Registration, registrations_of};
use crate::worktree_removal_clears_the_pool::clear_the_pool_of_a_removed_worktree;
use crate::worktree_removal_held_commits::{KeptCommits, keep_held_commits};
use crate::worktree_remove::{RemoveOutcome, RemoveRequest, remove_subvolume_worktree};
use crate::worktree_remove_plan::RemovePlan;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLandedRemoval";

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
    /// Why no candidate was examined,
    /// when the repository could not be read.
    pub failed: Option<String>,
}


/// What:
///  The branches that are fully contained in `commit`.
/// Why:
///   A branch has landed when every commit it holds is in the branch that received the merge,
///  which one Git start answers for every branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const merged = branchesMergedInto(context, commit); // throws the reason
/// ```
fn branches_merged_into(place: &LandedPlace<'_>, commit: &str) -> Result<Vec<String>, String> {
    let arguments: Vec<OsString> = ["for-each-ref", "--merged", commit, "--format=%(refname)", "refs/heads"].map(OsString::from).to_vec();
    let output: GitOutput = match run_git(place.context, &GitRequest::new(place.cwd, arguments.as_slice())) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("Git cannot list the branches merged into {commit} ({error})")),
    };
    if output.code != Some(0) {
        return Err(format!("Git cannot list the branches merged into {commit}: {}", String::from_utf8_lossy(output.stderr.as_slice()).trim()));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).lines().map(str::to_owned).collect());
}

/// What:
///  The worktrees whose branch this merge landed,
///  removed when every condition holds and left with their reason otherwise.
/// Why:
///   The event that ends a worktree's use is its branch landing,
///  so the set is the branches contained in the new head and not those already contained in the head before the merge:
///  a branch that merely sat on an older commit of the landed branch is not work this merge brought in.
///  A branch with no commits of its own is in both sets and is therefore never a candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepLandedWorktrees(place, before, now, landedBranch)
/// ```
pub fn sweep_landed_worktrees(place: &LandedPlace<'_>, before: &str, now: &str, landed_branch: &str) -> LandedSweep {
    let mut sweep: LandedSweep = LandedSweep { landed_branch: String::from(landed_branch), ..LandedSweep::default() };
    let landed: Vec<String> = match branches_merged_into(place, now) {
        Ok(found) => found,
        Err(reason) => {
            sweep.failed = Some(reason);
            return sweep;
        }
    };
    let earlier: HashSet<String> = match branches_merged_into(place, before) {
        Ok(found) => found.into_iter().collect(),
        Err(reason) => {
            sweep.failed = Some(reason);
            return sweep;
        }
    };
    let mut candidates: HashSet<String> = HashSet::new();
    for branch in landed {
        if !earlier.contains(branch.as_str()) {
            candidates.insert(branch);
        }
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
        sweep.failed = Some(kept.refused.join("\n"));
        return sweep;
    }
    let registered: Vec<Registration> = match registrations_of(place.common_dir) {
        Ok(found) => found,
        Err(reason) => {
            sweep.failed = Some(reason);
            return sweep;
        }
    };
    for one in registered {
        if !candidates.contains(one.branch.as_str()) || place.current_root == Some(one.worktree.as_path()) {
            continue;
        }
        // A registration that was not there when the merge started belongs to work that began after it, so this merge is
        // not the event that ended it, and it is not examined at all.
        if !place.registered_before.holds(&one) {
            continue;
        }
        match examine_and_remove(place, &one) {
            Examined::Removed => sweep.removed.push(RemovedWorktree { worktree: one.worktree, branch: one.branch }),
            Examined::Kept(why) => sweep.kept.push(KeptWorktree { worktree: one.worktree, branch: one.branch, why }),
            Examined::Unproven => sweep.unproven.push(UnprovenWorktree { worktree: one.worktree, branch: one.branch }),
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// examineAndRemove(place, registration)
/// ```
fn examine_and_remove(place: &LandedPlace<'_>, one: &Registration) -> Examined {
    if !added_by_the_wrapper(one.admin.as_path(), one.worktree.as_path()) {
        return Examined::Unproven;
    }
    if one.admin.join("locked").exists() {
        return Examined::Kept(String::from("it is locked, and a lock is a promise to keep it"));
    }
    for marker in IN_PROGRESS_MARKERS {
        if one.admin.join(marker).exists() {
            return Examined::Kept(format!("a {marker} stands in it, so an operation there is unfinished"));
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
