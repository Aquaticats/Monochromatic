//! What:
//!  The removal of a worktree's subvolume handed to the commit service,
//! and the service's own delete of it.
//! Why:
//!  A subvolume destroy is one kernel call,
//! but it takes `down_write(&fs_info->subvol_sem)` and joins the running
//!  transaction (`btrfs_delete_subvolume`,
//! `fs/btrfs/inode.c:4801` and `:4848` at `v7.2.15`),
//! so it waits for whatever snapshot another add is committing and for the transaction that is running,
//!  and the cleaner is busy behind it
//! (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
//! A person removing a worktree waits for none of that:
//! the worktree's
//!  directory is renamed out of the way inside its own parent in one `rename(2)`,
//! a durable work item names the renamed
//!  subvolume,
//! and the service deletes it later.
//! The rename is what makes the person's view right at once:
//! the path `git worktree remove` was given no longer exists,
//! and every file of it is still under a name this wrapper chose.
//! Nothing is lost:
//! the worktree was proven clean and unlocked before either step,
//! and the rename is inside the directory the
//!  removal already holds open.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await handOverRemoval({ context, commonDir, parent, parentPath, name, id, target }); // rename, record, wake
//! const verdict = deleteSubvolumeOf(item); // the service's own delete
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::{identity_at, open_root, rename_at};
use crate::fs_btrfs_ioctl::{snapshot_destroy_by_id, snapshot_destroy_in, subvolume_id};
use crate::iso_time::now_iso;
use crate::service_client::{Activation, ClientConfig, manager_watch_for, wake};
use crate::service_messages::render;
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_provision::provision;
use crate::real_git::{ResolutionInputs, ResolvedGit, process_resolution_inputs, resolve_real_git};
use crate::service_provision_host_and_manager::units_record_dir_of;
use crate::service_route::{Route, route_from_environment};
use crate::service_work::work_directory;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::wait_ceilings::{ANSWER_CEILING, HAND_OVER_GREETING_WAIT, SHORT_EXTRA_WAIT, WITHDRAW_CEILING, waiting_notice_after};
use crate::work_item::{PENDING, WorkItem, WorkKind, enqueue, state_directory};
use crate::service_work::Verdict;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemoveHandOver";

/// What a renamed subvolume's name starts with while the service deletes it:
/// a name no person gives a worktree,
/// so it is never mistaken for one,
/// and one the next commands report when it stays.
pub const DELETING_PREFIX: &str = "cli-git-deleting-";

/// The reference an item of this kind names,
/// which is no branch:
/// an item carries a reference-shaped field so that every reader of a record keeps one shape,
/// and no push is ever made for this kind.
pub const DELETE_REFERENCE: &str = "refs/worktree/cli-git-service/subvolume-delete";

/// What:
///  The reference an item of this kind names when what waits for the service is a whole worktree Git removes file by file,
///  and not a subvolume a removal already renamed out of the way.
/// Why:
///   One kind of record carries both because both are the same job:
///  finishing a removal the person's command did not wait for,
///  and the field that names which job it is is the one every reader already keeps.
///  A rename cannot stand in for a worktree Git must judge:
///  Git's own refuse-or-remove of the tracked files it finds is the whole proof of the removal,
///  and a path renamed away first would leave a registration naming nothing.
pub const WORKTREE_REMOVAL_REFERENCE: &str = "refs/worktree/cli-git-service/worktree-remove";

/// What:
///  The name a removed worktree's subvolume takes while it waits for the service's delete.
/// Why:
///   The subvolume id names one subvolume for good,
/// while a name names whatever is there when the call runs,
/// so a name built
///  from the id can never stand for another person's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deletingName = (id: bigint) => `cli-git-deleting-${id.toString(16).padStart(16, '0')}`;
/// ```
pub fn deleting_name(id: u64) -> OsString {
    return OsString::from(format!("{DELETING_PREFIX}{id:016x}"));
}

/// What:
///  The item that names one renamed subvolume for the service.
/// Why:
///   The record is the only thing that outlives the command,
/// so it names the exact path the rename made and the subvolume id it
///  holds:
/// a delete acts on that path alone,
/// and a path that is gone is nothing to delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removalItem = ({ commonDir, parentPath, id, deleting }) => ({ kind: 'subvolume-delete', repository: parentPath, gitDir: commonDir, ref: DELETE_REFERENCE, oid: hex40(id), parent: join(parentPath, deleting), created: now, tries: 0 });
/// ```
pub fn removal_item(common_dir: &Path, parent_path: &Path, id: u64) -> WorkItem {
    let deleting: PathBuf = parent_path.join(deleting_name(id));
    return WorkItem {
        kind: WorkKind::SubvolumeDelete,
        repository: parent_path.to_path_buf(),
        git_dir: common_dir.to_path_buf(),
        branch_ref: String::from(DELETE_REFERENCE),
        oid: format!("{id:040x}"),
        parent: deleting.display().to_string(),
        created: now_iso(),
        tries: 0,
    };
}

/// What:
///  The item that names one landed worktree for the commit service to remove with Git,
///  because the subvolume destroy could not stand for its removal here.
/// Why:
///  Git's own removal reads the worktree's tracked files and refuses what it would keep,
///  so it is the step that may not be run twice and may run for minutes on a large tree:
///  the person's command must not wait for it,
///  and the record is what outlives that command.
///  The id is a digest of the path and of the registration's own identity,
///  so two sweeps of one landing write one record,
///  a worktree added again at the same path is a new one,
///  and the `oid` field keeps the shape every reader of a record requires.
///  The repository is the repository's own Git directory,
///  which is the one directory the record names that outlives the worktree the merge ran in:
///  a record whose `repository` is no directory is refused by the service,
///  and a merge's worktree can be removed by a later merge.
///  Git is started there,
///  and Git reads a Git directory as the repository it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktreeRemovalItem = ({ gitDir, target }) => ({ kind: 'subvolume-delete', repository: dirname(target), gitDir, ref: WORKTREE_REMOVAL_REFERENCE, oid: sha1(pathBytes(target)), parent: target, created: now, tries: 0 });
/// ```
pub fn worktree_removal_item(git_dir: &Path, target: &Path, registration: &Path) -> WorkItem {
    let named: String = crate::hexadecimal_of_bytes::hexadecimal_of_bytes(crate::subvolume_sha::sha1(removal_key(target, registration).as_slice()).as_slice());
    return WorkItem {
        kind: WorkKind::SubvolumeDelete,
        // The repository's own Git directory, which is the one directory the record names that outlives the worktree the
        //  merge ran in: `is_acceptable` refuses a record whose `repository` is no directory, and Git runs from there.
        repository: git_dir.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(WORKTREE_REMOVAL_REFERENCE),
        oid: named,
        parent: target.display().to_string(),
        created: now_iso(),
        tries: 0,
    };
}

/// What:
///  Write the item that will delete the renamed subvolume,
/// so that a kill after this point leaves it named.
/// Why:
///   The record is written before the rename,
/// so the two orders a kill can leave are both safe:
/// a record whose path
///  does not exist deletes nothing,
/// and a rename whose record is missing would leave a directory nothing names.
/// Writing
///  it twice is one record:
/// the queue's id is the subvolume's,
/// so a second write finds the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(join(workDir(commonDir), 'pending'), removalItem({ commonDir, parentPath, id }))
/// ```
fn record_the_removal(common_dir: &Path, parent_path: &Path, id: u64) -> Result<WorkItem, String> {
    let item: WorkItem = removal_item(common_dir, parent_path, id);
    return match enqueue(work_directory(common_dir).as_path(), &item) {
        Ok(_added) => Ok(item),
        Err(reason) => Err(reason),
    };
}

/// What:
///  Wake the commit service of the worktree whose Git directory a record names,
/// saying nothing of its own,
/// as an add wakes it for its trash.
/// Why:
///   A record is read from the queue of the Git directory it names,
///  and one instance reads each queue,
///  so this is the instance that must be told.
///   A wake that cannot be made leaves the record for the next command of the worktree,
/// which is every commit
///  and every add,
/// so the delete happening later is not a failure of the removal.
///  A control replaces this with its own answer,
/// so no test reaches a service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wakeService({ context, gitDir });
/// ```
pub fn wake_the_service(context: &GitContext, git_dir: &Path) -> Result<(), String> {
    let Some(Route { address, provision: host }) = route_from_environment(context.real_git.as_path(), git_dir) else {
        return Err(String::from("no commit service can be started from this session"));
    };
    if let Some(setup) = host.as_ref()
        && let Err(failure) = provision(setup)
    {
        return Err(render(&failure));
    }
    // What: The record is written and the directory renamed before this wake, so the wake waits the hand-over's own greeting
    //       bound and never the commit's start wait.
    // Why:  A person removing a worktree waits for none of what the service does with the record, and every later command of the
    //       repository wakes the same service (`wait_ceilings.rs`, `HAND_OVER_GREETING_WAIT`).
    let config: ClientConfig = ClientConfig {
        address,
        activation: Activation::Managed,
        start_wait: HAND_OVER_GREETING_WAIT,
        heartbeat_silence: HEARTBEAT_SILENCE,
        start_watch: host.as_ref().map(manager_watch_for),
        keep_fresh: None,
        answer_ceiling: ANSWER_CEILING,
        withdraw_ceiling: WITHDRAW_CEILING,
        waiting_notice_after: waiting_notice_after(),
        extra_wait: SHORT_EXTRA_WAIT,
        start_ceiling: HAND_OVER_GREETING_WAIT,
        units_record_dir: host.as_ref().map(units_record_dir_of),
    };
    let mut silent: std::io::Sink = std::io::sink();
    return match wake(&config, git_dir, None, &mut silent) {
        Ok(()) => Ok(()),
        Err(failure) => Err(render(&failure)),
    };
}

/// What:
///  Hand the subvolume of a proven,
/// unlocked worktree to the service:
/// record the item,
/// rename the subvolume out of the
///  person's way,
/// and wake the service.
/// Why:
///  The rename is the whole of what the person's command does to the files:
/// one call inside the parent directory the removal
///  already holds open,
/// so nothing is deleted on their path and nothing is waited for.
/// Every step before it is the
///  removal's own proof that the worktree is clean and unlocked.
/// A step that fails puts the directory back and answers the
///  reason,
/// and the caller then destroys it in place as before,
/// so a hand-over is never what makes a removal fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverRemoval({ context, commonDir, parent, parentPath, name, id });
/// ```
pub fn hand_over_removal(context: &GitContext, common_dir: &Path, parent: &File, parent_path: &Path, name: &OsStr, id: u64, target: &Path) -> Result<(), String> {
    return hand_over_with(context, common_dir, parent, parent_path, name, id, target, &wake_the_service);
}

/// What:
///  `hand_over_removal` with the wake injected,
/// so a control reaches no service and no session bus.
/// Why:
///   The record is written and the directory renamed before the service is told,
/// so a control can assert both without a
///  service,
/// and a wake that fails is never the removal's failure:
/// the record stays for the next command of the
///  repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ context, commonDir, parent, parentPath, name, id, target, told });
/// ```
pub fn hand_over_with(context: &GitContext, common_dir: &Path, parent: &File, parent_path: &Path, name: &OsStr, id: u64, target: &Path, told: &dyn Fn(&GitContext, &Path) -> Result<(), String>) -> Result<(), String> {
    let item: WorkItem = record_the_removal(common_dir, parent_path, id)?;
    let deleting: OsString = deleting_name(id);
    if let Err(error) = rename_at(parent, name, parent, deleting.as_os_str(), false) {
        // Nothing was renamed, so the record names a path that does not exist and the service will find nothing to delete.
        return Err(format!("the subvolume of {} cannot be renamed out of the way ({error})", target.display()));
    }
    debug(TAG, format!("the subvolume of {} waits for the service's delete at {}", target.display(), Path::new(&item.parent).display()).as_str());
    if let Err(why) = told(context, common_dir) {
        // The record stays, and the next command of this repository wakes the service that reads it.
        warn(TAG, format!("the delete of {} is recorded and not woken ({why})", target.display()).as_str());
    }
    return Ok(());
}

/// What:
///  Whether metadata the filesystem answered describes a directory.
/// Why:
///  The answer is asked of what was read once,
/// so a path replaced between the two calls cannot make the proof about one object and the delete about another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isADirectory = (found) => found.isDirectory();
/// ```
fn is_a_directory(found: std::fs::Metadata) -> bool {
    return found.is_dir();
}

/// What:
///  Whether a path is a directory this process may remove as a subvolume.
/// Why:
///  A record names a path,
/// and a path can be replaced by a link or by nothing;
/// only a directory that stands exactly there is deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lstat(path).isDirectory()
/// ```
fn is_directory(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok_and(is_a_directory);
}

/// What:
///  Hand one landed worktree whose removal the fast path could not take to the commit service:
///  record it,
///  and wake the service of the worktree whose command found it landed.
/// Why:
///   The removal runs real `git worktree remove`,
///  which reads every tracked file of the worktree and can take minutes on a large tree,
///  and it is a step that may not be run twice:
///  a merge that waited for it kept the person's command open for as long as the deletion took
///  (the coordinator's integration passes of 2026-11-11 waited more than twenty seconds on one).
///  Nothing here touches the worktree:
///  Git's own judgment of what it would keep is the removal's whole proof,
///  so a path renamed or emptied here would take away what it judges.
///  What became of it is the record's state,
///  which the service settles into `done` or `failed` with Git's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverWorktreeRemoval({ context, gitDir, target, registration, told }); // record, then wake; a wake that fails leaves the record
/// ```
/// # Errors
///
/// The words for a record that cannot be written,
/// and for a worktree whose path is not absolute.
pub fn hand_over_worktree_removal(context: &GitContext, git_dir: &Path, target: &Path, registration: &Path, told: &dyn Fn(&GitContext, &Path) -> Result<(), String>) -> Result<WorkItem, String> {
    if !target.is_absolute() {
        return Err(format!("the removal of {} names no absolute path", target.display()));
    }
    let item: WorkItem = worktree_removal_item(git_dir, target, registration);
    enqueue(work_directory(git_dir).as_path(), &item)?;
    debug(TAG, format!("the removal of {} waits for the commit service's own Git at {}", target.display(), pending_record_path(git_dir, &item).display()).as_str());
    if let Err(why) = told(context, git_dir) {
        warn(TAG, format!("the removal of {} is recorded and not woken ({why})", target.display()).as_str());
    }
    return Ok(item);
}

/// What:
///  The bytes one worktree removal is keyed by:
///  its path,
///  and the identity the filesystem gives the registration that is being removed.
/// Why:
///  A record's id is its idempotency key,
///  so two sweeps of one landing must write one record,
///  and two worktrees added one after another at the same path must not share one:
///  a `done` record is never removed,
///  and a second removal keyed by the path alone would be taken for the first
///  and never run.
///  A registration that cannot be read leaves the path alone,
///  which is the record this release wrote before the identity was added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removalKey = (target, registration) => concat(pathBytes(target), identityBytes(stat(registration)));
/// ```
fn removal_key(target: &Path, registration: &Path) -> Vec<u8> {
    let mut key: Vec<u8> = path_bytes(target);
    use std::os::unix::fs::MetadataExt as _;
    if let Ok(found) = std::fs::metadata(registration) {
        key.extend_from_slice(found.dev().to_le_bytes().as_slice());
        key.extend_from_slice(found.ino().to_le_bytes().as_slice());
        key.extend_from_slice(found.ctime().to_le_bytes().as_slice());
        key.extend_from_slice(found.ctime_nsec().to_le_bytes().as_slice());
    }
    return key;
}

/// What:
///  The service's own removal of one landed worktree a command handed over,
///  with Git's own words as the verdict.
/// Why:
///  `git worktree remove` is Git's line:
///  it deletes what Git ignores and refuses what it would keep,
///  so a tracked edit or an untracked file stops it and its sentence is what the person reads.
///  A path that is gone is nothing to remove:
///  either another attempt removed the worktree,
///  or the person did.
///  A Git that cannot be found or started leaves the record to be tried again,
///  because nothing about the worktree was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeTheWorktreeOf(item): Verdict
/// ```
fn remove_the_worktree_of(item: &WorkItem) -> Verdict {
    let target: PathBuf = PathBuf::from(item.parent.as_str());
    if !target.is_absolute() {
        return Verdict::Failed(format!("the record names no absolute path to remove: {}", item.parent));
    }
    if !target.exists() {
        debug(TAG, format!("{} is gone already; nothing to remove", target.display()).as_str());
        return Verdict::Done;
    }
    // Read before Git deletes the worktree: the registration names the pool this worktree's shadow lives in, and a path
    //  that is gone names nothing.
    let pool: Option<(PathBuf, PathBuf)> = the_pool_of(target.as_path());
    let context: GitContext = match the_service_git(item) {
        Ok(found) => found,
        Err(reason) => return Verdict::Retry(reason),
    };
    let arguments: Vec<OsString> = vec![OsString::from("worktree"), OsString::from("remove"), target.clone().into_os_string()];
    let output: GitOutput = match run_git(&context, &GitRequest::new(item.repository.as_path(), arguments.as_slice())) {
        Ok(finished) => finished,
        Err(error) => return Verdict::Retry(format!("Git could not be started to remove {} ({error})", target.display())),
    };
    if output.code != Some(0) {
        let words: String = String::from_utf8_lossy(output.stderr.as_slice()).trim().to_owned();
        return Verdict::Failed(format!("Git refused to remove {}: {words}", target.display()));
    }
    // A pool names the worktree it serves in every member's config and sits in the common Git directory, which Git's removal
    //  leaves alone, so the removal that just proved the worktree is gone takes it (`worktree_removal_clears_the_pool.rs`).
    if let Some((common, admin)) = pool {
        crate::worktree_removal_clears_the_pool::clear_the_pool_of_a_removed_worktree(common.as_path(), admin.as_path());
    }
    return Verdict::Done;
}

/// What:
///  The common Git directory of this repository and the registration directory of `target`,
///  as a removal reads them before Git deletes the worktree.
/// Why:
///   The pool a worktree's commits fill is named from its registration directory and lives in the common Git directory,
///  and both are named by the worktree's own `.git` pointer,
///  which the removal deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const poolOf = (target) => { const admin = gitDirOf(target); return [commonDirectoryOf(admin), admin]; };
/// ```
fn the_pool_of(target: &Path) -> Option<(PathBuf, PathBuf)> {
    let pointer: Vec<u8> = std::fs::read(target.join(".git")).ok()?;
    let text: String = String::from_utf8_lossy(pointer.as_slice()).into_owned();
    let named: PathBuf = PathBuf::from(text.trim_end().strip_prefix("gitdir: ")?.trim());
    let admin: PathBuf = if named.is_absolute() { named } else { target.join(named) };
    let common: PathBuf = crate::common_directory_of_a_git_directory::common_directory_of(admin.as_path());
    return Some((common, admin));
}

/// What:
///  How the commit service starts real Git for a removal it was handed a record for.
/// Why:
///  The service's handler of a record is handed the record alone,
///  so the Git it starts is the one this process resolves,
///  as the wrapper's own start does:
///  the same search,
///  the same refusal of a wrapper,
///  and the same record of a Git found from the unit.
///  The overlay of a command does not apply here:
///  the hooks are off for this whole process (`hooks_off::activate`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const serviceGit = (item) => ({ realGit: resolveRealGit(processResolutionInputs(process.env)), overlay: [], commonDir: item.gitDir });
/// ```
/// What:
///  The words for process facts a Git search needs and this machine would not give.
/// Why:
///  The reason names what could not be read,
///  so a person reads why no Git was started rather than a sentence about a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noProcessFacts = (error) => `the process facts a Git search needs cannot be read (${error})`;
/// ```
fn no_process_facts(error: std::io::Error) -> String {
    return format!("the process facts a Git search needs cannot be read ({error})");
}

/// What:
///  The words for a removal that found no Git to run.
/// Why:
///  Nothing about the worktree was changed,
///  so the record is kept for another try,
///  and the reason says which record and why nothing ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noGitForARemoval = (failure) => `no Git can be started to remove the worktree a record names: ${failure}`;
/// ```
fn no_git_for_a_removal(failure: crate::real_git::RealGitNotFound) -> String {
    return format!("no Git can be started to remove the worktree a record names: {failure}");
}

fn the_service_git(item: &WorkItem) -> Result<GitContext, String> {
    let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let inputs: ResolutionInputs = process_resolution_inputs(environment.as_slice()).map_err(no_process_facts)?;
    let resolved: ResolvedGit = resolve_real_git(&inputs).map_err(no_git_for_a_removal)?;
    return Ok(GitContext { real_git: resolved.path, overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(item.git_dir.clone()) });
}

/// What:
///  The service's own delete of one handed-over subvolume,
/// with the words for what became of it.
/// Why:
///  A path that is gone is nothing to delete:
/// either the rename never happened and the removal destroyed the
///  subvolume in place,
/// or a service already deleted it,
/// so the item is done.
/// A path that is not a directory
///  is not this item's to remove.
/// The destroy names the subvolume by id when the kernel allows it,
/// so a replaced
///  ancestor cannot redirect it,
/// and by name below the opened parent otherwise.
/// A mount that does not allow an unprivileged
///  destroy is a failure the person must hear,
/// because the files stay;
/// anything else may pass and is tried again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deleteSubvolumeOf(item): Verdict
/// ```
pub fn delete_subvolume_of(item: &WorkItem) -> Verdict {
    // A record written for a whole worktree is Git's own removal, and one written for a renamed subvolume is the kernel's delete.
    if item.branch_ref == WORKTREE_REMOVAL_REFERENCE {
        return remove_the_worktree_of(item);
    }
    let path: PathBuf = PathBuf::from(item.parent.as_str());
    if !path.is_absolute() {
        return Verdict::Failed(format!("the record names no absolute path to delete: {}", item.parent));
    }
    let Some(parent_path) = path.parent() else {
        return Verdict::Failed(format!("the record names no directory to delete below: {}", item.parent));
    };
    let Some(name) = path.file_name() else {
        return Verdict::Failed(format!("the record names no directory to delete: {}", item.parent));
    };
    if !is_directory(path.as_path()) {
        debug(TAG, format!("{} is gone already; nothing to delete", path.display()).as_str());
        return Verdict::Done;
    }
    let parent: File = match open_root(parent_path) {
        Ok(opened) => opened,
        Err(error) => return Verdict::Retry(format!("the directory {} cannot be opened ({error})", parent_path.display())),
    };
    // The name still has to lead to the object that was looked at, or the delete would act on whatever took its place.
    match identity_at(&parent, name) {
        Ok(_held) => {}
        Err(error) => return Verdict::Retry(format!("{} cannot be looked up ({error})", path.display())),
    }
    match subvolume_id(&parent, path.as_path()) {
        Ok(id) => match snapshot_destroy_by_id(&parent, id, path.as_path()) {
            Ok(()) => return Verdict::Done,
            Err(error) if error.errno == libc::EPERM => {
                return Verdict::Failed(format!(
                    "{} is still there: this mount does not allow an unprivileged subvolume destroy ({error}), so remove the directory by hand, and remount it with user_subvol_rm_allowed to let the wrapper do it",
                    path.display()
                ));
            }
            Err(error) if by_id_unavailable(error.errno) => {
                debug(TAG, format!("the destroy of {} by id is not available ({error}); destroying it by name", path.display()).as_str());
            }
            Err(error) => return Verdict::Retry(format!("the subvolume of {} cannot be destroyed ({error})", path.display())),
        },
        Err(error) if by_id_unavailable(error.errno) => {
            debug(TAG, format!("the id of the subvolume {} is not available ({error}); destroying it by name", path.display()).as_str());
        }
        Err(error) if error.errno == libc::ENOENT => return Verdict::Done,
        Err(error) => return Verdict::Retry(format!("the subvolume of {} cannot be identified ({error})", path.display())),
    }
    return match snapshot_destroy_in(&parent, name, parent_path) {
        Ok(()) => Verdict::Done,
        Err(error) if error.errno == libc::ENOENT => Verdict::Done,
        Err(error) if error.errno == libc::EPERM => Verdict::Failed(format!(
            "{} is still there: this mount does not allow an unprivileged subvolume destroy ({error}), so remove the directory by hand",
            path.display()
        )),
        Err(error) => Verdict::Retry(format!("the subvolume of {} cannot be destroyed ({error})", path.display())),
    };
}

/// Whether an errno from the destroy by id means this kernel lacks the call,
/// so that the destroy by name below the held parent is the way left.
fn by_id_unavailable(errno: i32) -> bool {
    return errno == libc::ENOTTY || errno == libc::EOPNOTSUPP || errno == libc::EINVAL;
}

/// The path of a pending record,
/// for the words that name it.
pub fn pending_record_path(common_dir: &Path, inventory: &WorkItem) -> PathBuf {
    return state_directory(work_directory(common_dir).as_path(), PENDING).join(inventory.id());
}

/// The bytes of a path,
/// for a control that checks the record names the same place twice.
pub fn path_bytes(path: &Path) -> Vec<u8> {
    return path.as_os_str().as_bytes().to_vec();
}

/// Controls of the hand-over and of the service's delete stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_hand_over_tests.rs"]
mod tests;
