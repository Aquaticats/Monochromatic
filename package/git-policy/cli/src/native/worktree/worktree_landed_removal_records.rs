//! What:
//!  The records one landed sweep is asked with and answers with:
//!  where it looks and which worktree it must never remove,
//!  a worktree it removed,
//!  one it removed and left something in,
//!  one it left with the reason,
//!  one whose files went and whose registration stayed,
//!  one it handed to the commit service,
//!  one nothing proves the wrapper made,
//!  and everything one sweep did.
//! Why:
//!   The notice a person reads is written from these records,
//!  and every one of them names a fact a removal told apart:
//!  a worktree that is gone is not one that stayed,
//!  and a registration that stayed with its files gone is neither.
//!  They are a file of their own because the sweep that fills them is over the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type LandedSweep = { landedBranch: string; removed: RemovedWorktree[]; kept: KeptWorktree[] };
//! ```

use crate::subvolume_remove_walk::Boundary;
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use crate::worktree_landed_registrations::RegisteredBefore;
use std::path::{
    Path,
    PathBuf,
};

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
    /// The Git directory whose commit service performs every removal this sweep hands over,
    /// and whose work queue the record is read from.
    ///
    /// It is the repository's common Git directory,
    /// which is where a person's own `git worktree remove` records its subvolume delete:
    /// the merge's own instance then stays free for the push that follows the merge,
    /// and a removal that takes minutes never queues a merge's work behind it.
    pub git_dir: &'a Path,
    /// How a handed-over removal tells that service a record waits,
    /// which a control replaces with its own answer so no test reaches a service.
    pub told: &'a dyn Fn(&GitContext, &Path) -> Result<(), String>,
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

/// A worktree this sweep removed and left something in:
/// what someone else put inside it stands where it was,
/// with the directories that lead to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedAroundWorktree {
    /// The worktree directory,
    /// which is gone from Git's records and holds nothing but what was left.
    pub worktree: PathBuf,
    /// The branch it had checked out,
    /// which is kept.
    pub branch: String,
    /// What stands where it was,
    /// each with what it is.
    pub left: Vec<Boundary>,
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

/// What:
///  A worktree this sweep removed the files of and could not unregister.
/// Why:
///   The files are gone and the registration is not,
/// which is neither a removed worktree nor one left in place,
/// and a
///  person who read either of those would look for a worktree that is not there or skip a registration that still
///  is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnfinishedWorktree = { worktree: string; branch: string; report: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnfinishedWorktree {
    /// The worktree directory,
    /// which is gone.
    pub worktree: PathBuf,
    /// The branch it had checked out,
    /// which is kept.
    pub branch: String,
    /// What the removal said,
    /// which names the registration that stayed and every command that finishes it.
    pub report: String,
}

/// A worktree whose branch landed and whose removal the commit service performs,
/// with the record that names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandedOverWorktree {
    /// The worktree directory,
    /// which is still there when the merge ends.
    pub worktree: PathBuf,
    /// The branch it has checked out,
    /// which is kept.
    pub branch: String,
    /// The record the service reads,
    /// whose id is the name this wrapper keeps what became of the removal under.
    pub record: WorkItem,
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
    /// What it removed and left something in,
    /// which the notice says in its own words.
    pub removed_around: Vec<RemovedAroundWorktree>,
    /// What it removed while the volume it went around had gone,
    /// so the emptied root went with one `rmdir`,
    /// which the notice says in its own words.
    pub removed_emptied: Vec<RemovedWorktree>,
    /// What it handed to the commit service to remove,
    /// because that removal is Git's own read of every tracked file and may not be run twice.
    pub handed_over: Vec<HandedOverWorktree>,
    /// What it left,
    /// with the reason.
    pub kept: Vec<KeptWorktree>,
    /// What landed and the wrapper cannot prove it made.
    pub unproven: Vec<UnprovenWorktree>,
    /// The worktrees whose files went and whose registration stayed,
    /// with what finishes each.
    pub unfinished: Vec<UnfinishedWorktree>,
    /// The worktrees whose removal refused,
    /// with the report that says what stands in the way and what clears it.
    pub refused: Vec<KeptWorktree>,
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
