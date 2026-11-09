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
use crate::fs_beneath::{ObjectId, handle_path, identity_at, identity_of, open_directory_at, open_file_at, open_root, remove_at};
use crate::fs_btrfs_ioctl::{snapshot_destroy_by_id, snapshot_destroy_in, subvolume_id};
use crate::fs_kind::is_subvolume_root;
use crate::fs_mount_options::read_mount_facts;
use crate::subvolume_remove_walk::{empty_directory, verify_below};
use crate::transaction_git::GitContext;
use crate::worktree_clean::check_clean;
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_removal_interrupted_destroy::{forget_destroy, record_destroy, record_destroyed};
use crate::worktree_remove_plan::RemovePlan;
use crate::worktree_remove_reports::{incomplete_report, leftover_stage_report, stage_state};
use crate::worktree_remove_registry::{Registration, registration_of};
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemove";

/// The mount option that lets a regular user destroy a subvolume with the ioctl.
const REMOVAL_OPTION: &str = "user_subvol_rm_allowed";

/// What the command says on standard error whenever it destroys a subvolume.
pub const DESTROY_NOTICE: &str = "monochromatic-git-policy-cli: destroyed the btrfs subvolume of the removed worktree in one step; this cannot be undone, and anything another process wrote or locked in the moments since the last check went with it (the risk of git worktree remove, here faster).";

/// What:
///  Whether the administrative directory holds a `locked` entry.
/// Why:
///   A lock is a promise to keep the worktree.
/// Only an entry that is certainly absent (`NotFound`) means unlocked;
/// any
///  other failure to look it up leaves the lock unknown,
/// and an unknown lock is not a reason to destroy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockState(admin: string): boolean; // throws when the entry cannot be looked up
/// ```
fn lock_state(admin: &Path) -> Result<bool, String> {
    let entry: PathBuf = admin.join("locked");
    return match entry.symlink_metadata() {
        Ok(_found) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("the lock entry {} cannot be looked up ({error}), so the worktree is not known to be unlocked", entry.display())),
    };
}

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
///  The directories one removal holds open from before the proof of cleanliness to the destroy,
/// and the identities they
///  had when they were looked at.
/// Why:
///   A descriptor names the object it was opened on,
/// whatever happens to the path afterwards;
/// the identities let the
///  last look before the destroy tell a swapped name from the held object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Held = { parent: FileHandle; name: string; target: FileHandle; identity: ObjectId; admin: FileHandle; ... };
/// ```
#[derive(Debug)]
pub struct Held {
    /// The directory that holds the worktree directory.
    pub parent: File,
    /// The canonical path of `parent`,
    /// for messages and for the check that the path still names it.
    pub parent_path: PathBuf,
    /// The name of the worktree directory in `parent`.
    pub name: OsString,
    /// The worktree directory itself (the root of its subvolume).
    pub target: File,
    /// The identity of `target` when it was opened.
    pub identity: ObjectId,
    /// The identity of `parent` when it was opened.
    pub parent_identity: ObjectId,
    /// The `worktrees` directory of the repository.
    pub worktrees: File,
    /// The administrative directory of the worktree.
    pub admin: File,
    /// The identity of `admin` when it was opened.
    pub admin_identity: ObjectId,
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
///  Open the worktree directory,
/// its parent,
/// and its registration,
/// and record what each one is.
/// Why:
///   Everything after this point is about these objects,
/// not about the paths that name them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function hold(target: string, registration: Registration): Promise<Held>; // throws the reason
/// ```
/// A step of holding the worktree's directories with its failure worded as a refusal to open `what` at `path` without
///  following a link.
fn refused<T>(outcome: std::io::Result<T>, what: &str, path: &Path) -> Result<T, String> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("{what} {} cannot be opened without following a link ({error})", path.display())),
    };
}

/// A read of a held object's identity with its failure worded as the identity of `target` being unreadable.
fn identity_unreadable<T>(outcome: std::io::Result<T>, target: &Path) -> Result<T, String> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("the identity of {} cannot be read ({error})", target.display())),
    };
}

fn hold(target: &Path, registration: &Registration) -> Result<Held, String> {
    let Some(parent_of_target) = target.parent() else {
        return Err(format!("{} has no parent directory", target.display()));
    };
    let parent_path: PathBuf = parent_of_target.to_path_buf();
    let Some(last_component) = target.file_name() else {
        return Err(format!("{} has no last component", target.display()));
    };
    let name: OsString = last_component.to_os_string();
    let parent: File = refused(open_root(parent_path.as_path()), "the directory", parent_path.as_path())?;
    let held_target: File = refused(open_directory_at(&parent, name.as_os_str()), "the worktree", target)?;
    let worktrees: File = refused(open_root(registration.worktrees.as_path()), "the directory", registration.worktrees.as_path())?;
    let admin: File = refused(open_directory_at(&worktrees, registration.id.as_os_str()), "the registration", registration.admin.as_path())?;
    return Ok(Held {
        identity: identity_unreadable(identity_of(&held_target), target)?,
        parent_identity: identity_unreadable(identity_of(&parent), target)?,
        admin_identity: identity_unreadable(identity_of(&admin), target)?,
        parent,
        parent_path,
        name,
        target: held_target,
        worktrees,
        admin,
    });
}

/// What:
///  The last look before the destroy:
/// the lock,
/// and that the names still lead to the objects that were held and proven.
/// Why:
///   The proof of cleanliness takes time (it starts Git);
/// a lock taken,
/// a directory moved away and another put in its
///  place,
/// an ancestor replaced by a link,
/// or a registration replaced in that time must stop the destroy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function confirm(held: Held, registration: Registration): Promise<void>; // throws the reason
/// ```
fn confirm(held: &Held, registration: &Registration) -> Result<(), String> {
    let shown: PathBuf = held.parent_path.join(held.name.as_os_str());
    if lock_state(handle_path(&held.admin).as_path())? {
        return Err(format!("{} was locked after it was proven clean", shown.display()));
    }
    let named_now: ObjectId = path_failure(identity_at(&held.parent, held.name.as_os_str()), shown.as_path(), "cannot be looked up again")?;
    if named_now != held.identity {
        return Err(format!("{} is not the directory that was proven clean any more", shown.display()));
    }
    let parent_now: File = path_failure(open_root(held.parent_path.as_path()), held.parent_path.as_path(), "cannot be opened again without following a link")?;
    if path_failure(identity_of(&parent_now), held.parent_path.as_path(), "cannot be looked up again")? != held.parent_identity {
        return Err(format!("{} is not the directory that held the worktree any more", held.parent_path.display()));
    }
    let admin_now: ObjectId = path_failure(identity_at(&held.worktrees, registration.id.as_os_str()), registration.admin.as_path(), "cannot be looked up again")?;
    if admin_now != held.admin_identity {
        return Err(format!("{} is not the registration that was proven any more", registration.admin.display()));
    }
    let recorded: Vec<u8> = path_failure(read_held_file(held, "gitdir"), registration.admin.join("gitdir").as_path(), "cannot be read again")?;
    if recorded != registration.gitdir_text {
        return Err(format!("{} changed after it was proven", registration.admin.join("gitdir").display()));
    }
    return Ok(());
}

/// The bytes of a file of the held administrative directory.
fn read_held_file(held: &Held, name: &str) -> std::io::Result<Vec<u8>> {
    let mut file: File = open_file_at(&held.admin, std::ffi::OsStr::new(name))?;
    let mut bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut bytes)?;
    return Ok(bytes);
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
    if let Err(reason) = operations.check_clean(request, registration.admin.as_path(), request.common_dir) {
        debug(TAG, format!("{} is not proven clean ({reason}); Git decides", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    }
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
    if let Err(error) = operations.destroy(&held) {
        debug(TAG, format!("the subvolume of {} was not destroyed ({error}); Git removes it", target.display()).as_str());
        forget_destroy(request.common_dir, registration.id.as_os_str());
        return RemoveOutcome::LeftToGit;
    }
    // The kernel call returned, so its record now proves the files are gone; only that proof lets a later command remove the
    // registration.
    record_destroyed(request.common_dir, registration.id.as_os_str());
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
