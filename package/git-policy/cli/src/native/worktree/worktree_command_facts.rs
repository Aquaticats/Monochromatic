//! What:
//!  The plain facts of one worktree command that several folders pass around:
//!  what a forwarded run must copy,
//!  the plan of a `git worktree add` the cheap-worktree ladder serves,
//!  and the plan of a `git worktree remove` the wrapper may serve with a subvolume destroy.
//! Why:
//!  A lock,
//!  a push,
//!  a clone route and the top-level decision all carry these facts,
//!  and none of them is a worktree command's own machinery:
//!  keeping the three here lets those folders name one file
//!  instead of reaching into the worktree folders for one struct each.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CopyFacts = { identity: WorktreeIdentity; commonDir?: string; creates: boolean; addPlan?: AddPlan; removePlan?: RemovePlan };
//! type AddPlan = { destination: string; insertAt: number };
//! type RemovePlan = { target: string; operand: string; force: boolean };
//! ```

/// The repository shape an invocation selects,
/// which says whether the worktree is the main one,
/// a linked one,
/// a bare repository,
/// or none at all.
use crate::worktree_identity::WorktreeIdentity;
/// Owned operating-system text,
/// which holds any path Git accepts and no UTF-8 conversion requires.
use std::ffi::OsString;
/// An owned file-system path,
/// whose sibling `&Path` only lends one.
use std::path::PathBuf;

/// What:
///  Everything the copy needs to know about one forwarded invocation,
/// decided before Git starts.
/// Why:
///   The plan decides that the copy applies;
/// the run performs it with real Git and the process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyForward = { identity: GitWorktreeIdentity; command: ResolvedGitCommand; mainCheckoutPaths: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyFacts {
    /// The repository shape the invocation selects.
    pub identity: WorktreeIdentity,
    /// The Git common directory the command's repository has:
    /// the one the identity names,
    /// or the one Git named for a command that stands inside a Git directory of it and
    ///  so in no worktree (`git -C <repository>/.git worktree remove <slot>`);
    /// absent for a command that belongs to no repository.
    /// It is carried beside the identity because the identity has no value for a command that stands
    ///  outside every worktree and still belongs to a repository.
    pub common_dir: Option<PathBuf>,
    /// Whether the command,
    /// after alias resolution,
    /// creates a worktree.
    pub creates: bool,
    /// The destination the creating command's line names,
    /// in the form Git records it;
    /// absent for every other command and for
    ///  a form of `git worktree add` whose destination is not read.
    pub destination: Option<PathBuf>,
    /// The destination of a `git worktree add` in a form the cheap-worktree ladder serves;
    /// absent for every other
    ///  command and every form the ladder does not know.
    pub add_plan: Option<AddPlan>,
    /// The directory of a `git worktree remove` in the plain form the wrapper may serve with a subvolume destroy;
    ///  absent for every other command.
    pub remove_plan: Option<RemovePlan>,
    /// The directory of a `git worktree remove` in any form Git accepts,
    /// which is what the clone route's refusal reads;
    ///  absent for every other command, for a form that names no one directory, and for a directory that does not exist.
    pub removal_target: Option<PathBuf>,
    /// Whether the person asked a `git worktree add` for `--quiet`,
    /// read from the words whether or not the ladder serves the form,
    /// so a plainly run add prints no line a quiet one would suppress.
    pub quiet: bool,
}

/// What:
///  The facts of a `git worktree add` the ladder serves.
///  `usize` is an unsigned size or index;
/// `PathBuf` is an owned file-system path.
/// Why:
///   The destination decides the subvolume to create and the worktree to settle;
/// the insertion index decides where
///  `--no-checkout` goes in the arguments real Git receives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddPlan = { destination: string; insertAt: number; quiet: boolean; target?: string; checksOutABranch: boolean };
/// ```
#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddPlan {
    /// The destination as Git records it:
    /// the real path of its parent joined with its last component.
    pub destination: PathBuf,
    /// The index in the full argument list where options of `add` may be inserted:
    /// directly after the word `add`.
    pub insert_at: usize,
    /// Whether the person passed `--quiet`,
    /// which silences the line Git's checkout prints.
    pub quiet: bool,
    /// The commit-ish the person named,
    /// as written,
    /// or nothing when the add names none:
    /// Git then starts at the branch its destination is named after when that branch exists,
    /// and at the source's `HEAD` otherwise.
    pub target: Option<OsString>,
    /// Whether Git checks out the branch a name is instead of detaching at the commit it resolves to:
    /// true when the add has neither `-b` nor `--detach`.
    /// Why this decides the tree:
    ///  `builtin/worktree.c`'s `add_worktree` sets the new worktree's `HEAD` to `refs/heads/<name>` when that ref
    ///  exists and the add detaches nothing,
    ///  while `rev-parse` reaches a tag of the same name before that branch.
    pub checks_out_a_branch: bool,
}

/// What:
///  The worktree a `git worktree remove <path>` names,
/// resolved to a directory that exists.
/// Why:
///   The removal needs the worktree's real path;
/// Git also accepts a unique last component,
/// which is
///  found among the registered paths before the name is read as a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovePlan = { target: string; operand: string; force: boolean };
/// ```
#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovePlan {
    /// The real path of the worktree directory.
    pub target: PathBuf,
    /// The operand as the person typed it,
    /// which Git also matches against the last components of every registered path.
    pub operand: OsString,
    /// Whether the person passed `-f` or `--force`,
    /// which is what makes Git remove a worktree that holds changes.
    pub force: bool,
}
