//! What:
//!  The record one removal queues for the commit service,
//!  and the wake that makes the service of the repository read it.
//! Why:
//!  A record is what outlives the command that wrote it:
//!  the person's command writes it,
//!  wakes the service once,
//!  and returns,
//!  and the record is run later by whichever command arrives next.
//!  A wake that cannot be made is never the removal's failure,
//!  because the record is already durable and the next `git commit` of the repository wakes the same service.
//!  Two records are queued this way:
//!  the delete of a subvolume a removal renamed out of the person's way,
//!  and the removal of a whole worktree Git has to judge itself,
//!  and both carry the shape every reader of a record already parses.
//!  The same file queues the records a service of an earlier release refused for good,
//!  because that is the queue read a second time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const item = worktreeRemovalItem(gitDir, target, registration);
//! await queueTheRemoval(commonDir, item); // one record, however often it runs
//! await wakeTheService(context, gitDir); // the service that reads that queue
//! ```

use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::iso_time::now_iso;
use crate::private_storage::write_private_file_unsynced;
use crate::service_client::{
    Activation,
    ClientConfig,
    manager_watch_for,
    wake,
};
use crate::service_messages::render;
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_provision::provision;
use crate::service_route::{
    Route,
    route_from_environment,
};
use crate::service_work::work_directory;
use crate::transaction_git::GitContext;
use crate::wait_ceilings::{
    ANSWER_CEILING,
    HAND_OVER_GREETING_WAIT,
    SHORT_EXTRA_WAIT,
    WITHDRAW_CEILING,
    waiting_notice_after,
};
use crate::work_item::{
    FAILED,
    PENDING,
    WorkItem,
    WorkKind,
    enqueue,
    state_directory,
};
use crate::worktree_removal_leaves_a_subvolume_for_the_service::{
    deleting_name,
    removal_item,
};
use crate::worktree_remove_hand_over_records_beside_the_registration::{
    identity_recorded,
    record_what_the_service_compares,
    removal_key,
};
use crate::worktree_remove_registry::registered_worktree;
use std::path::{
    Path,
    PathBuf,
};

/// The tag every log line of this subject carries.
///
/// What:
///  `const TAG: &str = "worktreeRemoveHandOver"` is the name this program's own log lines are tagged with.
///  The subject is one,
/// so every file it was split into carries the same word,
///  and a person's search of the log keeps finding all of it.
/// Why:
///  A diagnostic is read beside the others of its subject,
///  and a tag that changed with the file would hide half of them.
const TAG: &str = "worktreeRemoveHandOver";

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
pub fn worktree_removal_item(
    git_dir: &Path,
    target: &Path,
    registration: &Path,
) -> WorkItem {
    let named: String = crate::hexadecimal_of_bytes::hexadecimal_of_bytes(
        crate::subvolume_sha::sha1(removal_key(target, registration).as_slice()).as_slice(),
    );
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
        // A removal speaks through this service's own transport: no command's settings travel with it, so it carries none.
        tries: 0,
        transport: false,
    };
}

/// What:
///  The item that will delete the renamed subvolume,
///  built and refused before anything of the removal moves,
///  and the queueing of it that a later round reads.
/// Why:
///   The record is written before the rename,
/// so the two orders a kill can leave are both safe:
/// a record whose path
///  does not exist deletes nothing,
/// and a rename whose record is missing is named by the leaving record written beside it
///  (`worktree_removal_leaving_that_outlives_its_record.rs`).
/// Writing
///  it twice is one record:
/// the queue's id is the subvolume's and the directory that holds it,
/// so a second write finds the first.
/// A directory whose path is not valid UTF-8 is refused here,
/// because a work item names its paths in text:
///  a record of the replacement character names another directory,
/// which no delete could ever find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const item = theRemovalOf(commonDir, parentPath, id); // throws when the parent path is not text
/// await queueTheRemoval(commonDir, item);
/// ```
pub(crate) fn the_removal_of(
    common_dir: &Path,
    parent_path: &Path,
    id: u64,
) -> Result<WorkItem, String> {
    if parent_path.to_str().is_none() {
        return Err(format!(
            "the directory that holds {} is not named by text, so no work record can name the directory the removal would rename it to ({} is not valid UTF-8)",
            parent_path.join(deleting_name(id)).display(),
            parent_path.display()
        ));
    }
    return Ok(removal_item(common_dir, parent_path, id));
}

/// What:
///  Put one removal's delete in the queue a round of the commit service reads.
/// Why:
///   A record whose id the queue already holds is the same work and is left as it stands,
/// so a hand-over that ran twice writes one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(join(workDir(commonDir), 'pending'), item)
/// ```
pub(crate) fn queue_the_removal(
    common_dir: &Path,
    item: &WorkItem,
) -> Result<(), String> {
    return match enqueue(work_directory(common_dir).as_path(), item) {
        Ok(_added) => Ok(()),
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
///   A wake that cannot be made leaves the record for the next `git commit` of the worktree,
/// which is the command that finishes it,
/// so the delete happening later is not a failure of the removal.
///  No other ordinary command wakes this service:
/// of thirteen run against a record waiting in the queue,
/// only a commit did
/// (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
/// "Which commands wake the service"),
/// and the warning a failed wake prints names both that commit and `git worktree remove`.
///  A control replaces this with its own answer,
/// so no test reaches a service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wakeService({ context, gitDir });
/// ```
pub fn wake_the_service(
    context: &GitContext,
    git_dir: &Path,
) -> Result<(), String> {
    let Some(Route {
        address,
        provision: host,
    }) = route_from_environment(context.real_git.as_path(), git_dir)
    else {
        return Err(String::from(
            "no commit service can be started from this session",
        ));
    };
    if let Some(setup) = host.as_ref()
        && let Err(failure) = provision(setup)
    {
        return Err(render(&failure));
    }
    // What: The record is written and the directory renamed before this wake, so the wake waits the hand-over's own greeting
    //       bound and never the commit's start wait.
    // Why:  A person removing a worktree waits for none of what the service does with the record, and the next commit of the
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
    };
    let mut silent: std::io::Sink = std::io::sink();
    return match wake(&config, git_dir, None, &[], &mut silent) {
        Ok(()) => Ok(()),
        Err(failure) => Err(render(&failure)),
    };
}

/// What:
///  Who is told that a deletion record waits for the service.
/// Why:
///   A removal's wake is one function bound to the context the command holds,
/// and a closure carrying that binding is a shape this crate's closure rule refuses
///  (`package/git-policy/cli/doc/closure-lint-suppressions.md`:
/// only the standard library's own callables are suppressed),
/// so the state travels in a value and the wake is a method of it.
pub trait ToldOfHandOver {
    /// What:
    ///  Tell the service of `common_dir` that a record waits.
    /// Why:
    ///   A hand-over is never what makes a removal fail,
    /// so the answer is a reason the removal warns about rather than an error it returns.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// told(commonDir): string | undefined
    /// ```
    fn told(
        &self,
        common_dir: &Path,
    ) -> Result<(), String>;
}

/// The service of the person's session,
/// woken through the context the removal holds.
pub struct ThroughTheService<'a> {
    /// The context whose repository is the session's.
    pub context: &'a GitContext,
}

impl ToldOfHandOver for ThroughTheService<'_> {
    fn told(
        &self,
        common_dir: &Path,
    ) -> Result<(), String> {
        return wake_the_service(self.context, common_dir);
    }
}

/// What:
///  Whether a record for `item` is still queued for a consumer to run.
/// Why:
///   A record a service already settled names no directory,
/// and neither does a claimed one:
/// the claim is taken before the consumer looks,
/// so a consumer that looked while
///  the directory still stood at its old name found nothing to delete
/// and settles the record without a delete.
/// A record still
///  queued is work a consumer has yet to begin,
/// so only that one leaves the directory to the service.
/// A queued record is asked for rather than listed,
///  which costs one lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exists(pending/<id>)
/// ```
pub(crate) fn a_record_is_queued(
    common_dir: &Path,
    item: &WorkItem,
) -> bool {
    return pending_record_path(common_dir, item).is_file();
}

/// What:
///  The warning a removal prints when the service that reads its record was not told,
/// naming the two commands that finish the removal and the path the record names.
/// Why:
///   A record whose wake failed waits in the queue,
/// and making the removal happen is a command the person chooses:
/// the next `git commit` here wakes the service that runs the record,
/// and `git worktree remove` of the same path removes the worktree now.
///  The path is printed twice because
/// the two commands name the same directory:
/// an abbreviation or the wrong one of the removal's paths would leave a person running a command over another worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the removal of ${target} is recorded and not woken (${why}). Your next \`git commit\` here finishes it; \`git worktree remove ${target}\` removes it now.`
/// ```
pub(crate) fn not_woken_warning(
    target: &Path,
    why: &str,
) -> String {
    let path: String = target.display().to_string();
    return format!(
        "the removal of {path} is recorded and not woken ({why}). Your next `git commit` here finishes it; `git worktree remove {path}` removes it now."
    );
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
///  The record names the worktree by its path and by the identity of the registration that stands beside it,
///  so the service can tell the worktree the proof was made for from one a person added at the same path meanwhile.
///  The comparison the sweep made of the worktree's ignored files is not repeated at the service by itself:
///  no command of the merge is there any more,
///  so the worktree those files were compared with is written beside the registration before the record is queued,
///  and the service reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverWorktreeRemoval({ context, gitDir, target, registration, comparedWith, told }); // record, then wake; a wake that fails leaves the record
/// ```
/// # Errors
///
/// The words for a record that cannot be written,
/// and for a worktree whose path is not absolute or not nameable in text.
pub fn hand_over_worktree_removal(
    context: &GitContext,
    git_dir: &Path,
    target: &Path,
    registration: &Path,
    compared_with: Option<&Path>,
    told: &dyn Fn(&GitContext, &Path) -> Result<(), String>,
) -> Result<WorkItem, String> {
    if !target.is_absolute() {
        return Err(format!(
            "the removal of {} names no absolute path",
            target.display()
        ));
    }
    // A work item names its paths in text, so a path that is not text would be a record naming another directory.
    if target.to_str().is_none() {
        return Err(format!(
            "the path of {} is not valid UTF-8, so no work record can name the worktree to remove",
            target.display()
        ));
    }
    record_what_the_service_compares(registration, target, compared_with)?;
    let item: WorkItem = worktree_removal_item(git_dir, target, registration);
    enqueue(work_directory(git_dir).as_path(), &item)?;
    debug(
        TAG,
        format!(
            "the removal of {} waits for the commit service's own Git at {}",
            target.display(),
            pending_record_path(git_dir, &item).display()
        )
        .as_str(),
    );
    if let Err(why) = told(context, git_dir) {
        warn(TAG, not_woken_warning(target, why.as_str()).as_str());
    }
    return Ok(item);
}

/// What:
///  The removals a commit service of an earlier release refused for good,
///  queued once more for this release's service,
///  and the lines that say so.
/// Why:
///  A record of a whole-worktree removal carries the kind `subvolume-delete`,
///  the reference `refs/worktree/cli-git-service/worktree-remove`,
///  and a digest in `oid`,
///  because a record has one shape for every reader
///  (`remove_the_worktree_of`).
///  A service of the release before this one does not look at the reference:
///  it reads `oid` as a subvolume number,
///  answers a failure when the digest does not fit,
///  and settles the record in `failed`,
///  which nothing ever reads again.
///  The landed worktree then stays for ever with nothing said about it,
///  and a record an older service cannot understand must not be lost to it.
///  So a worktree-removal record found refused with no attempt of its own behind it
///  (`tries` is nought,
///  which only a record that never ran a retry has)
///  is marked as tried,
///  queued again,
///  and said out loud on this command,
///  once:
///  the second attempt is marked in the record itself,
///  so a removal Git refuses again is not queued a third time by every later command.
///  A record that is not a worktree removal is left exactly as it is,
///  and so is one whose worktree is already gone,
///  and one whose hand-over recorded the worktree's identity:
///  the service of this release reads every record it writes,
///  so a failure of one of those is a decision this release made and already said on the person's next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = requeueRemovalsAnEarlierReleaseRefused(commonDir); // one line per record queued again
/// ```
pub fn requeue_removals_an_earlier_release_refused(common_dir: &Path) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    let work: PathBuf = work_directory(common_dir);
    for (id, _queued) in crate::work_item::entries_oldest_first(work.as_path(), FAILED) {
        let failed: PathBuf = state_directory(work.as_path(), FAILED).join(id.as_str());
        // A record that is not a plain file is one this command cannot read: a named pipe there answers at once instead of
        // waiting for a writer that may never come (`crate::plain_file::read_plain_text`).
        let Ok(text) = crate::plain_file::read_plain_text(failed.as_path()) else {
            continue;
        };
        let Ok(item) = WorkItem::decode(text.as_str()) else {
            continue;
        };
        if item.kind != WorkKind::SubvolumeDelete
            || item.branch_ref != WORKTREE_REMOVAL_REFERENCE
            || item.tries != 0
        {
            continue;
        }
        let target: PathBuf = PathBuf::from(item.parent.as_str());
        if !target.is_absolute() || !target.exists() {
            continue;
        }
        // Only a record an earlier release wrote is a candidate: the hand-over of this release writes the identity of the
        // worktree beside the registration, and its service reads every record it wrote, so a failure of one of its own is
        // a decision and not a misunderstanding.
        let Ok(registration) = registered_worktree(common_dir, target.as_path()) else {
            continue;
        };
        if identity_recorded(registration.admin.as_path()).is_some() {
            continue;
        }
        // The record is queued with one attempt behind it, so a removal Git refuses again is not queued by every later
        // command: the attempt count travels with the record into whatever state it settles in.
        let again: WorkItem = WorkItem {
            tries: 1,
            transport: false,
            ..item.clone()
        };
        let pending: PathBuf = state_directory(work.as_path(), PENDING);
        if crate::private_storage::create_private_directory_all(pending.as_path()).is_err() {
            continue;
        }
        if write_private_file_unsynced(
            pending.join(id.as_str()).as_path(),
            again.encode().as_bytes(),
        )
        .is_err()
        {
            continue;
        }
        let _removed = std::fs::remove_file(failed.as_path());
        said.push(format!(
            "monochromatic-git-policy-cli: the removal of the worktree {}, which a merge handed over, was recorded as failed and the worktree is still there, so it is queued once more; a service that keeps it again names it and says why.",
            target.display()
        ));
    }
    return said;
}

/// The path of a pending record,
/// for the words that name it.
pub fn pending_record_path(
    common_dir: &Path,
    inventory: &WorkItem,
) -> PathBuf {
    return state_directory(work_directory(common_dir).as_path(), PENDING).join(inventory.id());
}
