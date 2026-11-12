//! What:
//!  The removal of a clean linked worktree that is a btrfs subvolume,
//! by one subvolume destroy call and the
//!  deletion of its administrative directory,
//! in place of Git's file-by-file removal.
//! Why:
//!  Git unlinks every file of the worktree (0.7 s without ignored state,
//! 3.6 s with a large ignored tree in the
//!  measurements of `doc/planning/cli-git-btrfs-worktree.md`),
//! and the subvolume call takes milliseconds.
//! The call
//!  cannot be undone,
//! so the wrapper takes it only when the same conditions hold that make Git remove without
//!  `--force`:
//! the worktree is registered in the repository the command runs in,
//! under the name Git would pick,
//! is not
//!  locked,
//! is clean (`worktree_clean.rs`),
//! and holds no submodule.
//! The mount must also allow the call to an
//!  unprivileged user (`user_subvol_rm_allowed`).
//! Every other case,
//! and every call that the kernel refuses,
//! is left to
//!  Git,
//! which then words the refusal itself.
//!
//!  The check and the destroy are not one step,
//!  as Git's check and its deletion are not.
//!  Three things keep the
//!  window small and its worst case no worse than Git's:
//!  the directories are held open from before the proof of
//!  cleanliness to the destroy and the destroy names the subvolume by id (or by name below the held parent),
//!  the lock,
//!  the identity of the worktree directory and of the registration are looked at again immediately before the destroy,
//!  and the registration is emptied through its held descriptor,
//!  never by a path that could name another directory by
//!  then.
//!  What remains is a write or a lock made by another process in the milliseconds between the last look and the
//!  destroy;
//!  Git has the same window for its whole deletion,
//!  which takes seconds (`SPEC.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await removeSubvolumeWorktree(request); // 'left-to-git': run git worktree remove
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::{handle_path, identity_at, remove_at};
use crate::fs_btrfs_ioctl::{snapshot_destroy_by_id, snapshot_destroy_in, subvolume_id};
use crate::fs_kind::is_subvolume_root;
use crate::fs_mount_options::read_mount_facts;
use crate::subvolume_remove_walk::{empty_directory, find_boundary_below_except, verify_below};
use crate::transaction_git::GitContext;
use crate::worktree_clean::check_clean;
use crate::worktree_removal_interrupted_destroy::{forget_destroy, record_destroy, record_destroyed};
use crate::worktree_remove_plan::RemovePlan;
use crate::worktree_removal_clears_the_ahead_pool::{clear_the_pools_ahead_of_a_worktree, data_folders};
use crate::worktree_removal_refuses_a_nested_volume::refusal_for_nested_volume;
use crate::worktree_removal_takes_its_pending_deletions::{VolumesBelow, take_the_pending_deletions_below};
use crate::worktree_remove_reports::{incomplete_report, leftover_stage_report, pending_deletion_report, registration_volume_report, snapshots_ahead_report, stage_state, unwalkable_registration_report};
pub use crate::worktree_removal_holds_the_worktree::Held;
use crate::worktree_removal_holds_the_worktree::{confirm, hold, lock_state};
use crate::worktree_remove_registry::{Registration, registration_of};
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemove";

/// The mount option that lets a regular user destroy a subvolume with the ioctl.
const REMOVAL_OPTION: &str = "user_subvol_rm_allowed";

/// What:
///  The line the command prints on standard error whenever a removed worktree was a btrfs subvolume.
/// Why:
///  The subvolume leaves the person's path in one step that cannot be undone,
/// and its files are deleted by the commit
///  service after the command returned,
/// so a person who looks for the files has to be told which of the two happened and
///  why nothing of theirs was lost:
/// what went is what another process wrote or locked in the moments since the last check,
/// which is the same risk
///  `git worktree remove` takes.
pub const DESTROY_NOTICE: &str = "monochromatic-git-policy-cli: removed the btrfs subvolume of the worktree in one step, and the commit service deletes the files after this command returns, in its own round (a later command of the repository wakes it); this cannot be undone, and anything another process wrote or locked in the moments since the last check went with it (the risk of git worktree remove, here faster).";

/// What:
///  The line the command prints on standard error when a removed worktree was a btrfs subvolume and the person forced the removal.
/// Why:
///   `--force` is the one form that removes a worktree holding changes,
/// and the destroy cannot be undone,
/// so the
///  line has to say that nothing was checked for changes before it,
///  and the files leave later than the path does,
/// because the same commit service the unforced line names deletes them after this command returns.
pub const DESTROY_FORCED_NOTICE: &str = "monochromatic-git-policy-cli: removed the btrfs subvolume of the forced removal in one step, without checking it for changes first, as `--force` asks, and the commit service deletes the files after this command returns, in its own round (a later command of the repository wakes it); this cannot be undone, and everything the worktree held went with it, which is the set `git worktree remove --force` deletes (the risk of git worktree remove, here faster).";

/// What:
///  How one attempt to remove a worktree with a subvolume destroy ended.
/// Why:
///   Four endings need four different next steps:
/// Git removes it (nothing changed),
/// the work is done,
/// the wrapper refuses
///  because something beside the worktree would be left behind by a removal,
/// or the files are gone and the registration
///  is not,
/// which only the person can finish and which must never reach Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoveOutcome = { kind: 'left-to-git' } | { kind: 'removed' } | { kind: 'refused'; report: string } | { kind: 'incomplete'; report: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoveOutcome {
    /// Nothing was changed;
    /// real Git removes the worktree or states its refusal.
    LeftToGit,
    /// The subvolume and the administrative directory are gone.
    Removed,
    /// The removal is refused and nothing was changed;
    /// the report is for standard error and says what stands in the way and how to clear it.
    Refused(String),
    /// The subvolume was destroyed but the registration could not be removed;
    /// the report is for standard error.
    Incomplete(String),
}

/// What:
///  What one removal is asked to do and in which repository.
/// Why:
///   The repository that was invoked decides which registrations count;
/// the worktree's own pointers never do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoveRequest = { context: GitContext; plan: RemovePlan; commonDir: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RemoveRequest<'a> {
    /// How to start real Git for the cleanliness proof.
    pub context: &'a GitContext,
    /// The directory the person named.
    pub plan: &'a RemovePlan,
    /// The canonical common Git directory of the repository the command was run in.
    pub common_dir: &'a Path,
}

/// What:
///  The operations of one removal that touch the kernel or start Git,
/// behind one trait.
///  `&self` borrows the implementation read-only;
/// `dyn RemoveOperations` is how a caller holds "any implementation"
///  (a TS interface).
/// Why:
///   The ordering of the checks is the same on any filesystem;
/// a stand-in lets the controls run it on a plain
///  directory tree,
/// and swap objects between two steps to prove the identity checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemoveOperations { isSubvolumeRoot(path): boolean; removalAllowed(path): boolean; checkClean(...): void; destroy(held): void }
/// ```
pub trait RemoveOperations {
    /// Whether `path` is the root of a subvolume.
    fn is_subvolume_root(&self, path: &Path) -> bool;
    /// Whether an unprivileged destroy is allowed on the mount holding `path`.
    fn removal_allowed(&self, path: &Path) -> bool;
    /// The proof that the worktree is clean,
    /// or the reason it is not proven.
    fn check_clean(&self, request: &RemoveRequest<'_>, admin: &Path, common_dir: &Path) -> Result<(), String>;
    /// Destroy the subvolume that `held` holds,
    /// and nothing else.
    fn destroy(&self, held: &Held) -> Result<(), String>;
    /// What:
    ///  Record the subvolume for the service and rename it out of the person's way,
    /// so that no command waits for its delete.
    /// Why:
    ///  A stand-in that hands nothing over answers the reason,
    /// and the removal then destroys the subvolume in place as it always did,
    /// so every
    ///  control of the deletion keeps working on the destroy below.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handOver(context, commonDir, held, target, registration): void; // throws the reason it did not
    /// ```
    fn hand_over(&self, _context: &GitContext, _common_dir: &Path, _held: &Held, _target: &Path, _registration: &OsStr) -> Result<(), String> {
        return Err(String::from("this stand-in hands nothing to the service, so the subvolume is destroyed here"));
    }
}

/// The kernel and Git,
/// as they are.
#[derive(Clone, Copy, Debug, Default)]
pub struct LinuxOperations;

/// Whether an errno from the destroy by id means this kernel lacks the call,
/// so that the destroy by name below the held
///  parent is the way left.
fn by_id_unavailable(errno: i32) -> bool {
    return errno == libc::ENOTTY || errno == libc::EOPNOTSUPP || errno == libc::EINVAL;
}

/// Every operation is the primitive of the same name.
impl RemoveOperations for LinuxOperations {
    fn is_subvolume_root(&self, path: &Path) -> bool {
        return is_subvolume_root(path).unwrap_or(false);
    }

    fn removal_allowed(&self, path: &Path) -> bool {
        return match read_mount_facts(path) {
            Ok(Some(facts)) => facts.has_option(REMOVAL_OPTION),
            _ => false,
        };
    }

    fn check_clean(&self, request: &RemoveRequest<'_>, admin: &Path, common_dir: &Path) -> Result<(), String> {
        return match check_clean(request.context, request.plan.target.as_path(), admin, common_dir) {
            Ok(_proof) => Ok(()),
            Err(refusal) => Err(refusal.reason),
        };
    }

    fn hand_over(&self, context: &GitContext, common_dir: &Path, held: &Held, target: &Path, registration: &OsStr) -> Result<(), String> {
        // The id names the subvolume that was held, whatever path it has by the time the service acts.
        let id: u64 = match subvolume_id(&held.target, target) {
            Ok(found) => found,
            Err(error) => return Err(format!("the subvolume of {} cannot be identified ({error})", target.display())),
        };
        return crate::worktree_remove_hand_over::hand_over_removal(context, common_dir, &held.parent, held.parent_path.as_path(), held.name.as_os_str(), id, target, registration);
    }

    fn destroy(&self, held: &Held) -> Result<(), String> {
        let shown: PathBuf = held.parent_path.join(held.name.as_os_str());
        // The id names the subvolume that was held, whatever path it has by now.
        match subvolume_id(&held.target, shown.as_path()) {
            Ok(id) => match snapshot_destroy_by_id(&held.parent, id, shown.as_path()) {
                Ok(()) => return Ok(()),
                Err(error) if by_id_unavailable(error.errno) => {
                    debug(TAG, format!("the destroy of {} by id is not available ({error}); destroying it by name below the held parent", shown.display()).as_str());
                }
                Err(error) => return Err(error.to_string()),
            },
            Err(error) if by_id_unavailable(error.errno) => {
                debug(TAG, format!("the id of the subvolume {} is not available ({error}); destroying it by name below the held parent", shown.display()).as_str());
            }
            Err(error) => return Err(error.to_string()),
        }
        return match snapshot_destroy_in(&held.parent, held.name.as_os_str(), held.parent_path.as_path()) {
            Ok(()) => Ok(()),
            Err(error) => Err(error.to_string()),
        };
    }
}

/// What:
///  After the destroy,
/// remove the registration through the descriptors that were held,
/// and report how that ended.
/// Why:
///   The registration is emptied through its held descriptor and removed from the held `worktrees` directory by name,
/// which
///  succeeds only for an empty directory;
/// a registration that another process replaced is therefore never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finish(held: Held, registration: Registration, request: RemoveRequest): Promise<RemoveOutcome>;
/// ```
/// The outcome of a removal whose subvolume is gone but whose registration could not be cleared:
/// the report is also logged.
fn incomplete(registration: &Registration, request: &RemoveRequest<'_>, reason: String) -> RemoveOutcome {
    let report: String = incomplete_report(request.plan.target.as_path(), registration, request.common_dir, reason.as_str());
    warn(TAG, report.as_str());
    return RemoveOutcome::Incomplete(report);
}

fn finish(held: &Held, registration: &Registration, request: &RemoveRequest<'_>) -> RemoveOutcome {
    let target: &Path = request.plan.target.as_path();
    match identity_at(&held.worktrees, registration.id.as_os_str()) {
        Ok(found) if found == held.admin_identity => {}
        Ok(_other) => return incomplete(registration, request, String::from("another registration took its place while the worktree was being removed, and it was left untouched")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            debug(TAG, format!("the registration {} is already gone", registration.admin.display()).as_str());
            return RemoveOutcome::Removed;
        }
        Err(error) => return incomplete(registration, request, format!("it cannot be looked up ({error})")),
    }
    let admin: File = match held.admin.try_clone() {
        Ok(admin) => admin,
        Err(error) => return incomplete(registration, request, format!("its directory cannot be used ({error})")),
    };
    // The snapshots ahead of an add are this program's own and sit in the registration, so they go before the walk that
    // refuses every nested subvolume. A pool a fill made in the moments since the removal cleared it is the one case this
    // second clear answers, and a member that cannot go is the reason the registration stays.
    if let Err(reason) = clear_the_pools_ahead_of_a_worktree(request.common_dir, handle_path(&held.admin).as_path()) {
        return incomplete(registration, request, format!("its content cannot be removed ({reason})"));
    }
    let emptied = match verify_below(&admin, registration.admin.as_path()) {
        Ok(()) => empty_directory(&admin, registration.admin.as_path()),
        Err(error) => Err(error),
    };
    if let Err(error) = emptied {
        return incomplete(registration, request, format!("its content cannot be removed ({error})"));
    }
    if let Err(error) = remove_at(&held.worktrees, registration.id.as_os_str(), true) {
        return incomplete(registration, request, format!("the directory cannot be removed from {} ({error})", registration.worktrees.display()));
    }
    // Git removes the `worktrees` directory when the last registration goes; a directory that still holds others stays.
    let _kept = std::fs::remove_dir(registration.worktrees.as_path());
    debug(TAG, format!("destroyed the subvolume {} and its administrative directory", target.display()).as_str());
    return RemoveOutcome::Removed;
}

/// What:
///  The refusal of a removal whose worktree holds a subvolume or a mount point below its root,
///  or nothing when every volume below it is this program's own and now out of the way,
///  or when the walk proves nothing and Git may have the tree.
/// Why:
///   Git's own removal unlinks the files of a nested worktree,
/// so a tree that holds one is never given to it;
/// a tree
///  the walk cannot read proves nothing either way,
/// and the removal is then Git's,
/// which is what it was before
///  this proof existed.
///  The subvolumes this program left there are its own to take,
/// never a reason to refuse or to hand a
///  person a command to delete them.
///  The proof is made twice:
/// once before anything is renamed or destroyed,
/// and once after a destroy the
///  kernel refused,
/// because the same worktree can be added in the moments between them (a review finding,
///  F1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = proveTheTreeBelowIsClear(commonDir, held, request); // RemoveOutcome.Refused | undefined
/// ```
fn prove_the_tree_below_is_clear(common_dir: &Path, held: &Held, request: &RemoveRequest<'_>) -> Option<RemoveOutcome> {
    let target: &Path = request.plan.target.as_path();
    return match take_the_pending_deletions_below(common_dir, held, target) {
        Ok(VolumesBelow::Clear) => None,
        Ok(VolumesBelow::Foreign(found)) => {
            let report: String = refusal_for_nested_volume(target, request.common_dir, &found);
            warn(TAG, report.as_str());
            Some(RemoveOutcome::Refused(report))
        }
        Ok(VolumesBelow::Unreadable(reason)) => {
            debug(TAG, format!("{reason}; Git decides").as_str());
            None
        }
        Err(reason) => {
            let report: String = pending_deletion_report(target, reason.as_str());
            warn(TAG, report.as_str());
            Some(RemoveOutcome::Refused(report))
        }
    };
}

/// What:
///  Remove the worktree if every condition holds,
/// with the kernel and Git as they are.
/// Why:
///   See `remove_with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeSubvolumeWorktree(request: RemoveRequest): Promise<RemoveOutcome>;
/// ```
pub fn remove_subvolume_worktree(request: &RemoveRequest<'_>) -> RemoveOutcome {
    return remove_with(&LinuxOperations, request);
}

/// What:
///  Remove the worktree if every condition holds;
/// report how it ended.
/// Why:
///   `LeftToGit` means Git has to do the removal,
/// and nothing was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeWith(operations: RemoveOperations, request: RemoveRequest): Promise<RemoveOutcome>;
/// ```
pub fn remove_with(operations: &dyn RemoveOperations, request: &RemoveRequest<'_>) -> RemoveOutcome {
    let target: &Path = request.plan.target.as_path();
    let registration: Registration = match registration_of(request.common_dir, target, request.plan.operand.as_os_str()) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("{} is not served ({reason}); Git decides", target.display()).as_str());
            return RemoveOutcome::LeftToGit;
        }
    };
    match lock_state(registration.admin.as_path()) {
        Ok(false) => {}
        Ok(true) => {
            debug(TAG, format!("{} is locked; Git decides", target.display()).as_str());
            return RemoveOutcome::LeftToGit;
        }
        Err(reason) => {
            debug(TAG, format!("{reason}; Git decides").as_str());
            return RemoveOutcome::LeftToGit;
        }
    }
    if !operations.is_subvolume_root(target) {
        debug(TAG, format!("{} is not a subvolume; Git removes it file by file", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    }
    // A conversion's copy beside the worktree is named by the journal in the registration this removal deletes, so a removal
    // now would leave it behind with nothing to find it by. The person finishes or discards it first. What proves the copy is
    // a conversion's is the record the conversion writes beside the worktree: a directory at that name without one is the
    // person's own, and refusing on the name alone would block the removal with nothing able to clear it.
    let converting: PathBuf = crate::worktree_convert::converting_path(target);
    if crate::worktree_convert_leftover_stage_record::recorded_stage_of(target).is_some() {
        let report: String = leftover_stage_report(target, converting.as_path(), stage_state(registration.admin.as_path(), target, converting.as_path()));
        debug(TAG, report.as_str());
        return RemoveOutcome::Refused(report);
    }
    if !operations.removal_allowed(target) {
        debug(TAG, format!("the mount of {} lacks {REMOVAL_OPTION}; Git removes the worktree file by file", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    }
    let held: Held = match hold(target, &registration) {
        Ok(held) => held,
        Err(reason) => {
            debug(TAG, format!("{reason}; Git decides").as_str());
            return RemoveOutcome::LeftToGit;
        }
    };
    // Git's own removal makes this proof only when the person did not force it (`remove_worktree` in
    // `builtin/worktree.c`: `if (!force) check_clean_worktree(wt, av[0])`), and a forced removal deletes the worktree
    // it was given whatever it holds, so the destroy answers for the same set of files.
    if !request.plan.force
        && let Err(reason) = operations.check_clean(request, registration.admin.as_path(), request.common_dir)
    {
        debug(TAG, format!("{} is not proven clean ({reason}); Git decides", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    }
    if let Err(reason) = confirm(&held, &registration) {
        debug(TAG, format!("{reason}; Git decides").as_str());
        return RemoveOutcome::LeftToGit;
    }
    // The kernel refuses a destroy for a subvolume that contains another one, so a worktree holding a subvolume of its own
    // could never go this way: it has to be refused while nothing has happened, and never handed to Git, whose removal
    // unlinks the inner worktree's files. A tree that cannot be walked proves nothing either way, and the removal is then
    // Git's, which is what it was before this check existed.
    if let Some(refused) = prove_the_tree_below_is_clear(request.common_dir, &held, request) {
        return refused;
    }
    // The snapshots ahead of an add are this program's own and stand in the registration the removal empties after the
    // destroy, so they go first: a pool that cannot be taken stops the removal while nothing has happened.
    if let Err(reason) = clear_the_pools_ahead_of_a_worktree(request.common_dir, handle_path(&held.admin).as_path()) {
        let report: String = snapshots_ahead_report(target, registration.admin.as_path(), reason.as_str());
        warn(TAG, report.as_str());
        return RemoveOutcome::Refused(report);
    }
    // The registration is emptied after the destroy, and that emptying refuses a nested subvolume, so a volume this
    // program does not own inside it would leave the files gone and the registration stuck, with a notice on every later
    // command. It is proved now, while the worktree and every file of it are still there, so the person reads what stands in
    // the way instead of a removal that cannot finish.
    // This program's own folders there are left out of the proof: the clear above emptied their pools, a fill that made a
    // pool of its own in the moments since is this program's as well, and it is taken before the destroy rather than
    // refused.
    let own: [PathBuf; 2] = data_folders(registration.admin.as_path());
    let except: [&Path; 2] = [own[0].as_path(), own[1].as_path()];
    match find_boundary_below_except(&held.admin, registration.admin.as_path(), &except) {
        Ok(None) => {}
        Ok(Some(found)) => {
            let report: String = registration_volume_report(target, registration.admin.as_path(), &found);
            warn(TAG, report.as_str());
            return RemoveOutcome::Refused(report);
        }
        Err(error) => {
            let report: String = unwalkable_registration_report(target, registration.admin.as_path(), error.to_string().as_str());
            warn(TAG, report.as_str());
            return RemoveOutcome::Refused(report);
        }
    }
    // The walk and the clear above took time, and the destroy names the held subvolume, so the names and the lock are looked
    // at once more: what the first look proved is what this destroy answers for, and a lock or a swap since it is Git's.
    if let Err(reason) = confirm(&held, &registration) {
        debug(TAG, format!("{reason}; Git decides").as_str());
        return RemoveOutcome::LeftToGit;
    }
    // What is about to be destroyed is recorded before the kernel call, so a command killed between the destroy and the
    // removal of the registration leaves a state the next command can finish instead of one only `git worktree prune`
    // could. A record that cannot be written costs that finish and nothing else, so the destruction goes on.
    if let Err(reason) = record_destroy(request.common_dir, registration.id.as_os_str(), &held.admin_identity, target) {
        warn(TAG, format!("the destroy of {} is not recorded, so a command that dies before its registration is removed leaves it to `git worktree prune`: {reason}", target.display()).as_str());
    }
    // The subvolume is handed to the service when it can be: it is renamed inside the parent this removal holds, a durable record
    // names it, and the service's own call deletes it later, so the person waits for one rename and no kernel delete.
    let mut marked: bool = false;
    match operations.hand_over(request.context, request.common_dir, &held, target, registration.id.as_os_str()) {
        Ok(()) => debug(TAG, format!("the subvolume of {} is recorded for the service's delete", target.display()).as_str()),
        Err(why) => {
            debug(TAG, format!("{why}; the subvolume of {} is destroyed here", target.display()).as_str());
            // The kernel call is next and nothing marks it afterwards, so the record says now that the files leave this path:
            // a command killed inside the call leaves a marked record whose path is gone, which the next command finishes,
            // while a path that is still there is dropped with nothing changed (a review finding, F4).
            record_destroyed(request.common_dir, registration.id.as_os_str());
            marked = true;
            if let Err(error) = operations.destroy(&held) {
                debug(TAG, format!("the subvolume of {} was not destroyed ({error}); Git removes it", target.display()).as_str());
                forget_destroy(request.common_dir, registration.id.as_os_str());
                // The destroy ran after the walk, and the two are not one step: a worktree added below this one since the
                // walk is what makes the kernel refuse the call (`ENOTEMPTY`), and Git's own removal would unlink that
                // worktree's files. The tree is proved again before Git is given it, and a refusal leaves the worktree and
                // every file of it where they are.
                if let Some(refused) = prove_the_tree_below_is_clear(request.common_dir, &held, request) {
                    return refused;
                }
                return RemoveOutcome::LeftToGit;
            }
        }
    }
    // Either the kernel call returned or the directory was renamed away, so the record now proves the files are gone from the
    // path the person named; only that proof lets a later command remove the registration.
    if !marked {
        record_destroyed(request.common_dir, registration.id.as_os_str());
    }
    let outcome: RemoveOutcome = finish(&held, &registration, request);
    // A removal that finished has nothing left for a later command; one that could not remove the registration keeps its
    // record, which is what the next command finishes.
    if outcome == RemoveOutcome::Removed {
        forget_destroy(request.common_dir, registration.id.as_os_str());
    }
    return outcome;
}

/// Subvolume removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_tests.rs"]
mod tests;

/// Controls that swap objects between the steps of a removal run on a stand-in for the kernel.
#[cfg(test)]
#[path = "worktree_remove_guard_tests.rs"]
mod guard_tests;
