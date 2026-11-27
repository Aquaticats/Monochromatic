//! What:
//!  The service's own removal of one worktree a command handed over,
//!  and the choice of which destroy a record asks for.
//! Why:
//!  `git worktree remove` is Git's line:
//!  it deletes what Git ignores and refuses what it would keep,
//!  so a tracked edit or an untracked file stops it and its sentence is what the person reads.
//!  Everything the sweep proved is proved again here,
//!  because the proof was spent when the record was written and this destruction happens later.
//!  The other kind of record names a subvolume a removal already renamed out of the way,
//!  and that one is the kernel's delete,
//!  so one place decides which of the two a record asks for.
//!  A tree that holds a subvolume or a mount point this program did not make is never given to Git,
//!  because Git's own removal deletes what is mounted at a mount point and unlinks the files of a nested worktree,
//!  so this removal answers the removal made around such a volume as one of its own endings
//! (`worktree_remove_hand_over_around_a_foreign_volume.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! deleteSubvolumeOf(item): Verdict // Git's own removal, or the kernel's delete
//! removeTheWorktreeOf(item, procRoot): Verdict
//! ```

use crate::diagnostic_log::debug;
use crate::service_work::Verdict;
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::work_item::WorkItem;
use crate::worktree_added_by_the_wrapper::added_by_the_wrapper;
use crate::worktree_removal_held_commits::{
    KeptCommits,
    keep_held_commits,
};
use crate::worktree_removal_second_look::{
    SecondLook,
    why_the_worktree_stays,
};
// What:     `use crate::worktree_remove_hand_over_around_a_foreign_volume::remove_around_what_is_not_this_removals;`
//           imports the removal made around a volume this removal did not make.
// Why:      That answer needs the volume report and the record a person reads, which are
//           its own file's subject, and this file only decides to ask for it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { removeAroundWhatIsNotThisRemovals } from './worktree_remove_hand_over_around_a_foreign_volume';
// ```
use crate::worktree_remove_hand_over_around_a_foreign_volume::remove_around_what_is_not_this_removals;
use crate::worktree_remove_hand_over_begun_record::{
    begun_identity,
    forget_begun,
    record_begun,
};
use crate::worktree_remove_hand_over_kept_worktree_notice::keep_the_worktree;
use crate::worktree_remove_hand_over_queue_and_wake::WORKTREE_REMOVAL_REFERENCE;
use crate::worktree_remove_hand_over_records_beside_the_registration::{
    compared_with_of,
    identity_standing_there,
    why_this_is_another_worktree,
};
use crate::worktree_remove_hand_over_service_git_and_pool::{
    the_pool_of,
    the_service_git,
};
use crate::worktree_remove_hand_over_subvolume_delete::destroy_the_subvolume_of;
use crate::worktree_remove_registry::{
    Registration,
    registered_worktree,
};
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// Import the tag every log line of this subject carries.
use crate::worktree::worktree_remove_hand_over_begun_record::TAG;

/// The process table a removal reads to see whether anything works in the worktree.
const PROCESS_TABLE: &str = "/proc";

/// What:
///  The service's own call for one handed-over record,
///  as the release runs it:
///  the delete of `delete_subvolume_of_with` over this machine's own process table (`/proc`).
/// Why:
///  Nothing in the release picks a process table,
///  so the call that does the work takes one as an argument and this one supplies the machine's,
///  and every caller that must see the real processes keeps reading them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deleteSubvolumeOf = (item) => deleteSubvolumeOfWith(item, '/proc');
/// ```
pub fn delete_subvolume_of(item: &WorkItem) -> Verdict {
    return delete_subvolume_of_with(item, Path::new(PROCESS_TABLE));
}

/// What:
///  The service's own delete of one handed-over record:
///  Git's own removal of a whole worktree read against `proc_root`,
///  or the kernel's destroy of the renamed subvolume the record names.
/// Why:
///  One kind of record carries both jobs,
///  and the field that names which it is is the reference,
///  so the choice is made once,
///  here,
///  and each destroy says what became of the record.
///  A whole-worktree removal reads the machine's processes to see whether anything works inside it,
///  which a control replaces with a table of its own
///  (issue 6475:
/// a container gate refused a removal over another test's process).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deleteSubvolumeOfWith(item, procRoot): Verdict
/// ```
pub(crate) fn delete_subvolume_of_with(
    item: &WorkItem,
    proc_root: &Path,
) -> Verdict {
    // A record written for a whole worktree is Git's own removal, and one written for a renamed subvolume is the kernel's delete.
    if item.branch_ref == WORKTREE_REMOVAL_REFERENCE {
        return remove_the_worktree_of(item, proc_root);
    }
    return destroy_the_subvolume_of(item);
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
///  Everything the sweep proved is proved again here,
///  because the proof was spent when the record was written and this destruction happens later:
///  - the registration the record names is still the repository's registration of that path,
///    and its identity is the one the record was keyed by,
///    so a worktree a person added at the same path meanwhile is not this one and is left alone;
///  - the commit a reference lock holds in it reaches `refs/cli-git/kept/<batch>` of the common Git directory,
///    because the removal deletes the worktree's own reference and Git directory
///    (`worktree_removal_held_commits.rs`);
///  - the record that this program made the worktree is still beside the registration;
///  - and the rest of `worktree_removal_second_look.rs` holds:
///    no registered worktree inside it,
///    no lock,
///    no unfinished operation,
///    no process working in it,
///    read in the process table `proc_root` names,
///    no ignored file the worktree the proof was made from does not hold,
///    and every record of its own Git directory copied into a removal snapshot.
///
/// A worktree that stays is named in the repository's kept-worktree notices,
/// which the next command of any worktree shows,
/// so nothing is kept in silence.
///
/// `proc_root` is the process table the holder scan reads:
/// the release hands in this machine's `/proc`,
/// and a control hands in one of its fixture's own,
/// so a control's verdict is not decided by the programs the machine runs beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeTheWorktreeOf(item, procRoot): Verdict
/// ```
pub(crate) fn remove_the_worktree_of(
    item: &WorkItem,
    proc_root: &Path,
) -> Verdict {
    let target: PathBuf = PathBuf::from(item.parent.as_str());
    if !target.is_absolute() {
        return Verdict::Failed(format!(
            "the record names no absolute path to remove: {}",
            item.parent
        ));
    }
    if !target.exists() {
        debug(
            TAG,
            format!("{} is gone already; nothing to remove", target.display()).as_str(),
        );
        forget_begun(item.git_dir.as_path(), item.id().as_str());
        return Verdict::Done;
    }
    let registration: Registration =
        match registered_worktree(item.git_dir.as_path(), target.as_path()) {
            Ok(found) => found,
            // A directory the repository does not register under that path is no worktree this record may remove, and Git
            // would refuse to remove it too.
            Err(reason) => return Verdict::Failed(format!("nothing was removed: {reason}")),
        };
    // A path names whatever stands there when this service runs, so the worktree in front of this removal has to be the
    // one the proof was made for: the hand-over wrote that worktree's identity beside the registration, and a worktree a
    // person added at the same path since is another directory with another identity.
    if let Some(why) =
        why_this_is_another_worktree(item, target.as_path(), registration.admin.as_path())
    {
        debug(TAG, format!("{why}").as_str());
        return Verdict::Skipped(why);
    }
    // Read before Git deletes the worktree: the registration names the pool this worktree's shadow lives in, and a path
    //  that is gone names nothing.
    let pool: Option<(PathBuf, PathBuf)> = the_pool_of(target.as_path());
    let context: GitContext = match the_service_git(item) {
        Ok(found) => found,
        Err(reason) => return Verdict::Retry(reason),
    };
    // A commit a reference lock holds lives on the worktree's own reference and in its own Git directory, and the removal
    // deletes both. The sweep kept what was held when it ran; a batch that met a reference lock afterwards is held since.
    // A commit no reference can keep stops the removal, because nothing else would name it once the directory is gone.
    let kept: KeptCommits = keep_held_commits(item.git_dir.as_path(), &context);
    for line in &kept.said {
        crate::service_log!("{line}");
    }
    if !kept.refused.is_empty() {
        return keep_the_worktree(
            item,
            target.as_path(),
            registration.admin.as_path(),
            kept.refused.join("\n").as_str(),
            kept.said.as_slice(),
        );
    }
    if !added_by_the_wrapper(registration.admin.as_path(), target.as_path()) {
        return keep_the_worktree(
            item,
            target.as_path(),
            registration.admin.as_path(),
            "nothing records that this program made it, so the files Git ignores in it may be yours",
            kept.said.as_slice(),
        );
    }
    let source: Option<PathBuf> = compared_with_of(registration.admin.as_path());
    let look: SecondLook<'_> = SecondLook {
        context: &context,
        common_dir: item.git_dir.as_path(),
        proc_root,
        source: source.as_deref(),
    };
    if let Some(why) = why_the_worktree_stays(&look, target.as_path(), registration.admin.as_path())
    {
        return keep_the_worktree(
            item,
            target.as_path(),
            registration.admin.as_path(),
            why.as_str(),
            kept.said.as_slice(),
        );
    }
    // A subvolume or mount point that is not this removal's is removed around on this route too, and for the same reason as on
    // the person's: Git's own removal would delete what is mounted at a mount point and unlink the files of a nested worktree,
    // and the proofs above were made before such a volume may have appeared. The volume and the directories that lead to it
    // stay, what is left is recorded for the person, and the same check decides it on both routes.
    if !crate::worktree_removal_goes_around_a_foreign_volume::volumes_below(target.as_path())
        .is_empty()
    {
        return remove_around_what_is_not_this_removals(item, target.as_path(), &registration);
    }
    // What Git is about to delete, and that the proofs held for it, is recorded first: a service killed in the middle of
    // the deletion leaves a tree Git refuses, and the next attempt finishes this program's own half-done work from here.
    let identity: String = identity_standing_there(target.as_path(), registration.admin.as_path());
    let begun: Option<String> = begun_identity(item.git_dir.as_path(), item.id().as_str());
    record_begun(
        item.git_dir.as_path(),
        item.id().as_str(),
        identity.as_str(),
    );
    let mut force: bool = false;
    if let Some(before) = begun {
        if before != identity {
            return keep_the_worktree(
                item,
                target.as_path(),
                registration.admin.as_path(),
                "a removal of another worktree at this path began and was cut short, so nothing here is this record's to finish",
                kept.said.as_slice(),
            );
        }
        if !crate::worktree_removal_only_deletions::only_deletions(
            &context,
            target.as_path(),
            registration.admin.as_path(),
        ) {
            return keep_the_worktree(
                item,
                target.as_path(),
                registration.admin.as_path(),
                "a removal of this worktree began and a kill cut it short, and this worktree now holds a file Git would keep, so the removal is not finished over it",
                kept.said.as_slice(),
            );
        }
        force = true;
        debug(
            TAG,
            format!(
                "the removal of {} began and was cut short; it is finished now",
                target.display()
            )
            .as_str(),
        );
    }
    let mut arguments: Vec<OsString> = vec![OsString::from("worktree"), OsString::from("remove")];
    if force {
        arguments.push(OsString::from("--force"));
    }
    arguments.push(target.clone().into_os_string());
    let output: GitOutput = match run_git(
        &context,
        &GitRequest::new(item.repository.as_path(), arguments.as_slice()),
    ) {
        Ok(finished) => finished,
        Err(error) => {
            return Verdict::Retry(format!(
                "Git could not be started to remove {} ({error})",
                target.display()
            ));
        }
    };
    if output.code != Some(0) {
        let words: String = String::from_utf8_lossy(output.stderr.as_slice())
            .trim()
            .to_owned();
        // Git kept it, and the merge that asked for the removal is over: the person reads why on their next command, not
        // only in this service's journal.
        return keep_the_worktree(
            item,
            target.as_path(),
            registration.admin.as_path(),
            format!("Git refused to remove it: {words}").as_str(),
            kept.said.as_slice(),
        );
    }
    forget_begun(item.git_dir.as_path(), item.id().as_str());
    // A pool names the worktree it serves in every member's config and sits in the common Git directory, which Git's removal
    //  leaves alone, so the removal that just proved the worktree is gone takes it (`worktree_removal_clears_the_pool.rs`).
    if let Some((common, admin)) = pool {
        crate::worktree_removal_clears_the_pool::clear_the_pool_of_a_removed_worktree(
            common.as_path(),
            admin.as_path(),
        );
    }
    return Verdict::Done;
}
