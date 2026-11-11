//! What:
//!  The one place a merge that moved the branch hands the landed worktrees to the sweep.
//! Why:
//!   The sweep needs the branch as it was before the merge and as it is after,
//!  and only the command that ran the merge knows both.
//!  It also needs the directory to start Git in and the root of the worktree it must not remove,
//!  which come from the repository shape the command ran with.
//!  A merge that ended with a nonzero status,
//!  or one that moved nothing,
//!  sweeps nothing:
//!  no branch landed in it.
//!  Nothing here can fail the merge,
//!  and nothing here writes to standard output,
//!  which carries the merge's own report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = headOf(context, identity);            // one Git start, before the merge
//! afterAMerge(context, identity, before, code);        // after it ended; prints one line per worktree
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_landed_removal::{LandedPlace, LandedSweep, sweep_landed_worktrees};
use crate::worktree_landed_sweep_notice::landed_sweep_lines;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The process table the sweep reads.
const PROCESS_TABLE: &str = "/proc";

/// What:
///  A directory of the repository a query can run in,
///  and the branch the worktree there has checked out.
/// Why:
///   Every query names absolute paths,
///  so any directory of the repository works,
///  and a bare repository's common directory is itself a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cwd = identity.worktreeRoot ?? identity.commonDir;
/// ```
fn working_directory(identity: &WorktreeIdentity) -> Option<PathBuf> {
    // `match` on the repository shape: a worktree root, or the bare repository's directory.
    match identity {
        WorktreeIdentity::MainWorktree { worktree_root, .. } | WorktreeIdentity::LinkedWorktree { worktree_root, .. } => return Some(worktree_root.clone()),
        WorktreeIdentity::BareRepository { common_dir, .. } => return Some(common_dir.clone()),
        WorktreeIdentity::OutsideWorktree => return None,
    }
}

/// The root of the worktree the command runs in,
/// which the sweep never removes.
fn current_root(identity: &WorktreeIdentity) -> Option<PathBuf> {
    // `match` on the repository shape: only a worktree has a root.
    match identity {
        WorktreeIdentity::MainWorktree { worktree_root, .. } | WorktreeIdentity::LinkedWorktree { worktree_root, .. } => return Some(worktree_root.clone()),
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => return None,
    }
}

/// The repository's common Git directory,
/// which owns every registration.
fn common_directory(identity: &WorktreeIdentity) -> Option<PathBuf> {
    // `match` on the repository shape: a bare repository has one too.
    match identity {
        WorktreeIdentity::MainWorktree { common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, .. }
        | WorktreeIdentity::BareRepository { common_dir, .. } => return Some(common_dir.clone()),
        WorktreeIdentity::OutsideWorktree => return None,
    }
}

/// What:
///  The commit the branch in this worktree points at,
///  one Git start,
///  or nothing when the branch has no commit yet.
/// Why:
///   The merge's own report does not say where the branch was before it ran,
///  and a branch that has no commit has nothing that could land.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// headOf(context, identity) // string | undefined
/// ```
pub fn head_of(context: &GitContext, identity: &WorktreeIdentity) -> Option<String> {
    let directory: PathBuf = working_directory(identity)?;
    let arguments: Vec<OsString> = [OsString::from("rev-parse"), OsString::from("HEAD")].to_vec();
    let output: GitOutput = run_git(context, &GitRequest::new(directory.as_path(), arguments.as_slice())).ok()?;
    if output.code != Some(0) {
        return None;
    }
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return (!text.is_empty()).then_some(text);
}

/// The branch this worktree has checked out,
/// in Git's own short spelling,
/// or nothing for a detached `HEAD`.
fn current_branch(context: &GitContext, directory: &Path) -> Option<String> {
    let arguments: Vec<OsString> = ["symbolic-ref", "--quiet", "--short", "HEAD"].map(OsString::from).to_vec();
    let output: GitOutput = run_git(context, &GitRequest::new(directory, arguments.as_slice())).ok()?;
    if output.code != Some(0) {
        return None;
    }
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    return (!text.is_empty()).then_some(text);
}

/// What:
///  Sweep the worktrees whose branches this merge landed,
///  and print what the sweep removed,
///  left,
///  or could not prove it made.
/// Why:
///   Called once per merge that ended well,
///  from the one place that knows where the branch was before it:
///  a merge that moved nothing,
///  or one that failed,
///  sweeps nothing and costs one comparison.
///  The lines go to standard error because standard output is the merge's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// afterAMerge(context, identity, before, code) // LandedSweep | undefined
/// ```
pub fn after_a_merge(context: &GitContext, identity: &WorktreeIdentity, before: &str, code: i32) -> Option<LandedSweep> {
    if code != 0 {
        return None;
    }
    let now: String = head_of(context, identity)?;
    if now == before {
        return None;
    }
    let directory: PathBuf = working_directory(identity)?;
    let common_dir: PathBuf = common_directory(identity)?;
    let root: Option<PathBuf> = current_root(identity);
    let branch: String = current_branch(context, directory.as_path()).unwrap_or_else(|| String::from("a head with no branch"));
    let place: LandedPlace<'_> = LandedPlace {
        context,
        cwd: directory.as_path(),
        common_dir: common_dir.as_path(),
        current_root: root.as_deref(),
        proc_root: Path::new(PROCESS_TABLE),
    };
    let sweep: LandedSweep = sweep_landed_worktrees(&place, before, now.as_str(), branch.as_str());
    let mut stderr: std::io::Stderr = std::io::stderr();
    for line in landed_sweep_lines(&sweep) {
        let _written = writeln!(stderr, "{line}");
    }
    return Some(sweep);
}
