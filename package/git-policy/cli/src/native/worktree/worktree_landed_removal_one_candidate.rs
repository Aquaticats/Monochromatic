//! What:
//!  One candidate of a landed sweep:
//!  the conditions it must meet,
//!  the last look a removal asks again inside the window it destroys in,
//!  the fast destroy or the hand over to the commit service,
//!  and what the attempt came to.
//! Why:
//!   Each answer is a decision a person can read,
//!  and the removal runs only when all of them hold,
//!  so a worktree someone is still using is never the one that goes.
//!  The removal here is the fast destroy,
//!  or the record a service performs,
//!  because removing a worktree Git removes reads every tracked file and may not be run twice,
//!  and a merge that ran it kept the person's command open for as long as the deletion took.
//!  The last look is asked first here,
//!  so a candidate that would lose a record or a nested worktree costs no proof of cleanliness.
//!  It is a file of its own because the sweep that drives it is over the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const examined = examineAndRemove(place, told, registration, registrations, operations);
//! ```

use crate::subvolume_remove_walk::Boundary;
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use crate::worktree_added_by_the_wrapper::added_by_the_wrapper;
use crate::worktree_command_facts::RemovePlan;
use crate::worktree_landed_registrations::Registration;
use crate::worktree_landed_removal_records::LandedPlace;
use crate::worktree_removal_clears_the_pool::clear_the_pool_of_a_removed_worktree;
use crate::worktree_removal_second_look::{
    SecondLook,
    nested_worktree_reason,
    why_the_worktree_stays,
    worktrees_inside,
};
use crate::worktree_remove::{
    RemoveOperations,
    RemoveOutcome,
    RemoveRequest,
    remove_with,
};
use crate::worktree_remove_hand_over::hand_over_worktree_removal;
use std::ffi::OsString;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLandedRemoval";

/// What:
///  What one attempt to remove a candidate came to.
/// Why:
///   A removal that destroyed the files and could not empty the registration must be told apart from one that changed
///  nothing,
/// because the sweep's line about it says what happened,
/// not only why it stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Attempt = { kind: 'removed' } | { kind: 'unfinished'; report: string } | { kind: 'refused'; report: string } | { kind: 'handed'; record } | { kind: 'kept'; why: string };
/// ```
enum Attempt {
    /// The worktree and its registration are gone.
    Removed,
    /// The files are gone and the registration stayed,
    /// with what finishes it.
    Unfinished(String),
    /// The removal refused,
    /// so nothing changed and this says what clears the way.
    Refused(String),
    /// Nothing was removed,
    /// for this reason.
    Kept(String),
    /// The removal is recorded for the commit service,
    /// which performs it on its own with Git's own judgment of what it would keep.
    Handed(WorkItem),
    /// The worktree and its registration are gone,
    /// and what someone else put inside it stands where it was.
    RemovedAround(Vec<Boundary>),
    /// The worktree and its registration are gone,
    /// and the volume it went around had gone by the time its pass ended.
    EmptiedAround,
}

/// What:
///  Remove one worktree here when the fast destroy can take it,
///  and hand it to the commit service when Git must remove it.
/// Why:
///  On btrfs the destroy is what gives the snapshot's space back,
///  and it is one kernel call behind a rename the person does not wait for;
///  Git's own removal is the line every other case keeps,
///  and it is also the answer when the mount forbids the destroy:
///  it deletes what Git ignores and refuses what it would keep,
///  so nothing a person wrote that Git keeps is lost,
///  and its own words say why when it refuses.
///  That read of every tracked file is the step a merge may not run:
///  it can take minutes,
///  and the wrapper never ends a step that may not be run twice,
///  so it is recorded for the service and this command returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeHereOrHandOver(place, registration) // an Attempt: removed here, handed to the service, or kept with its reason
/// ```
fn remove_here_or_hand_over(
    place: &LandedPlace<'_>,
    told: &dyn Fn(&GitContext, &Path) -> Result<(), String>,
    operations: &dyn RemoveOperations,
    one: &Registration,
) -> Attempt {
    let plan: RemovePlan = RemovePlan {
        target: one.worktree.clone(),
        operand: OsString::from(one.worktree.as_os_str()),
        force: false,
    };
    // Asked again inside the removal, in the window it destroys in: its proof of cleanliness starts Git and walks the
    // tree, which is long enough for a command to add a worktree inside this one and for the service to record a refused
    // push (a cross-vendor review of the landed sweep, round 208).
    let look: NothingNewWouldGoWithIt<'_> = NothingNewWouldGoWithIt {
        common_dir: place.common_dir,
        one,
    };
    let request: RemoveRequest<'_> = RemoveRequest {
        context: place.context,
        plan: &plan,
        common_dir: place.common_dir,
        last_look: Some(&look),
    };
    match remove_with(operations, &request) {
        RemoveOutcome::Removed => {
            crate::diagnostic_log::debug(
                TAG,
                format!("destroyed the subvolume of {}", one.worktree.display()).as_str(),
            );
            return Attempt::Removed;
        }
        RemoveOutcome::Incomplete(report) => return Attempt::Unfinished(report),
        RemoveOutcome::Refused(report) => return Attempt::Refused(report),
        RemoveOutcome::DestroyedInPlace => return Attempt::Removed,
        RemoveOutcome::RemovedAround(left) => return Attempt::RemovedAround(left),
        // The volume went away during the pass, so the emptied root went with one `rmdir` and nothing of this removal's
        // deleted files is what freed the space its snapshot held: the sweep says so in its own line.
        RemoveOutcome::RemovedAroundNothingLeft => return Attempt::EmptiedAround,
        RemoveOutcome::LeftToGit => {}
    }
    let record: WorkItem = match hand_over_worktree_removal(
        place.context,
        place.git_dir,
        one.worktree.as_path(),
        one.admin.as_path(),
        place.current_root,
        told,
    ) {
        Ok(record) => record,
        Err(reason) => return Attempt::Kept(reason),
    };
    return Attempt::Handed(record);
}

/// The sweep's own last look and what a removal deletes beside the files Git keeps,
/// which is asked again inside the removal.
#[path = "worktree_landed_removal_last_look.rs"]
mod last_look;
use self::last_look::{
    NothingNewWouldGoWithIt,
    nothing_new_would_go_with_it,
};

/// What one candidate came to.
pub(crate) enum Examined {
    /// The worktree and its registration are gone.
    Removed,
    /// The removal refused,
    /// so nothing there changed and the report says what clears the way.
    Refused(String),
    /// The files are gone and the registration stayed,
    /// with what finishes it.
    Unfinished(String),
    /// The removal is recorded for the commit service,
    /// which performs it on its own.
    Handed(WorkItem),
    /// It stays,
    /// for this reason.
    Kept(String),
    /// It is gone and something someone else put inside it stayed.
    RemovedAround(Vec<Boundary>),
    /// It is gone and the volume it went around had gone by the time its pass ended.
    EmptiedAround,
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
///  what a removal deletes beside the files Git keeps is asked here first,
///  so a candidate that would lose one costs no proof of cleanliness,
///  and the removal asks it again in the window it destroys in
///  (`nothing_new_would_go_with_it`);
///  everything else such a removal deletes is read again here,
///  immediately before it runs,
///  because a worktree a command adds inside this one while the sweep runs,
///  and a push failure recorded in this one's own Git directory,
///  appear only after that first reading
///  (`worktree_removal_second_look.rs`).
///  The service that performs a handed-over removal makes that same reading again before it destroys anything,
///  because this one is spent as soon as the record is written
///  (`remove_the_worktree_of`).
///  The `operations` are the kernel and Git a control replaces,
///  exactly as `worktree_remove.rs` lets a control replace them for the person's own removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// examineAndRemove(place, registration, registered, operations)
/// ```
pub(crate) fn examine_and_remove(
    place: &LandedPlace<'_>,
    told: &dyn Fn(&GitContext, &Path) -> Result<(), String>,
    one: &Registration,
    registered: &[Registration],
    operations: &dyn RemoveOperations,
) -> Examined {
    if !added_by_the_wrapper(one.admin.as_path(), one.worktree.as_path()) {
        return Examined::Unproven;
    }
    let inside: Vec<&Registration> = worktrees_inside(one.worktree.as_path(), registered);
    if !inside.is_empty() {
        return Examined::Kept(nested_worktree_reason(inside.as_slice()));
    }
    // What the removal deletes beside the files Git keeps is asked here first, so a candidate that would lose a record or
    // a nested worktree costs no proof of cleanliness, and the same question is asked again inside the removal, in the
    // window it destroys in (`nothing_new_would_go_with_it`).
    if let Err(reason) = nothing_new_would_go_with_it(place.common_dir, one) {
        return Examined::Kept(reason);
    }
    let look: SecondLook<'_> = SecondLook {
        context: place.context,
        common_dir: place.common_dir,
        proc_root: place.proc_root,
        source: place.current_root,
    };
    if let Some(why) = why_the_worktree_stays(&look, one.worktree.as_path(), one.admin.as_path()) {
        return Examined::Kept(why);
    }
    let removal: Attempt = remove_here_or_hand_over(place, told, operations, one);
    if matches!(removal, Attempt::Removed) {
        // A pool names the worktree it serves in every member's config and sits in the common Git directory, which no
        // removal deletes, so the removal that just proved the worktree is gone takes it (`worktree_removal_clears_the_pool.rs`).
        clear_the_pool_of_a_removed_worktree(place.common_dir, one.admin.as_path());
    }
    return match removal {
        Attempt::Removed => Examined::Removed,
        Attempt::Unfinished(report) => Examined::Unfinished(report),
        Attempt::Refused(report) => Examined::Refused(report),
        Attempt::Handed(record) => Examined::Handed(record),
        Attempt::Kept(why) => Examined::Kept(why),
        Attempt::RemovedAround(left) => Examined::RemovedAround(left),
        Attempt::EmptiedAround => Examined::EmptiedAround,
    };
}
