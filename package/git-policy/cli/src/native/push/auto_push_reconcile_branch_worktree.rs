//! What:
//!  Which worktree has the branch a push sent,
//!  and a Git context that runs in that worktree.
//! Why:
//!   A branch is checked out in at most one worktree,
//! and only the worktree that holds it has files to write when it moves;
//! a branch no worktree holds needs no file written at all.
//! The listing Git prints is the only place that answers which worktree it is,
//!  and a worktree whose folder is gone still appears in it:
//! such a worktree holds no files,
//!  so it is not one to move and the branch moves as a ref
//!  (a review of commit `98de70959d`,
//!  finding 1: a start in the gone folder turned a published merge into a reported failure).
//!  Every start of a move made for another worktree has to run there,
//! and the caller's own repository variables would otherwise keep it in the repository
//!  this command was started in,
//! so this file also builds that Git context.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const place = liveHolder(await git(['worktree','list','--porcelain','-z']), branchRef); // undefined when none
//! const there = gitAt(git, place);
//! ```

use crate::auto_push_single_flight::{
    PushGit, SingleFlightError, run_query_awaiting,
};
use crate::auto_push_reconcile_basics_and_limits::argv;
use crate::diagnostic_log::debug;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// What:
///  The worktree that has `branch_ref` checked out and still has its folder,
/// or nothing when no worktree has it.
/// Why:
///   A worktree whose folder is gone is listed by Git as prunable and holds no files:
/// starting a Git in it fails,
///  and a failure after the merge was published told a person the push had failed
///  (a review of commit `98de70959d`,
///  finding 1).
/// A branch such a worktree still names is a branch no live worktree has,
///  so it moves as a ref and nothing is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = liveHolder((await git(['worktree','list','--porcelain','-z'])).split('\0'), branchRef);
/// ```
pub(crate) async fn live_holder_awaiting(git: &PushGit, branch_ref: &str) -> Result<Option<PathBuf>, SingleFlightError> {
    let listed = run_query_awaiting(git, &argv(&["worktree", "list", "--porcelain", "-z"])).await?;
    if listed.code != Some(0) {
        return Ok(None);
    }
    let Some(place) = worktree_holding(&listed.stdout, branch_ref) else {
        return Ok(None);
    };
    if !place.is_dir() {
        debug(
            TAG,
            &format!(
                "the worktree at {} still lists {branch_ref} but has no folder, so it holds no files and the branch moves as a ref",
                place.display()
            ),
        );
        return Ok(None);
    }
    return Ok(Some(place));
}

/// What:
///  The worktree a `worktree list --porcelain -z` listing says has `branch_ref` checked out.
/// Why:
///   The listing is the only place Git answers that question for every worktree,
/// and `-z` keeps a path that holds bytes which are not text whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeHolding(fields, branchRef) { let path; for (const field of fields) { if (field.startsWith('worktree ')) path = field.slice(9); else if (field === `branch ${branchRef}`) return path; } return undefined; }
/// ```
pub(crate) fn worktree_holding(listing: &[u8], branch_ref: &str) -> Option<PathBuf> {
    let mut current: Option<PathBuf> = None;
    for field in listing.split(|byte| return *byte == 0) {
        if field.is_empty() {
            continue;
        }
        if let Some(path) = field.strip_prefix(b"worktree ".as_slice()) {
            current = Some(PathBuf::from(crate::path_bytes::os_from_path_bytes(path)));
            continue;
        }
        let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(field);
        if let Some(named) = text.strip_prefix("branch ")
            && named == branch_ref
        {
            return current;
        }
    }
    return None;
}

/// Whether this worktree is the one at `place`,
/// as far as the file system says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameDirectory = (left, right) => fs.realpathSync(left) === fs.realpathSync(right);
/// ```
fn same_directory(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    return match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(here), Ok(there)) => here == there,
        _ => false,
    };
}

/// What:
///  Whether this command runs in the worktree at `place`.
/// Why:
///   The two moves are not the same:
/// the one here asks this checkout and writes its files,
///  and the one elsewhere runs every question and every write in the other worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isHere = sameDirectory(await git(['rev-parse','--show-toplevel']), place);
/// ```
pub(crate) async fn is_this_worktree(git: &PushGit, place: &Path) -> Result<bool, SingleFlightError> {
    let top = run_query_awaiting(git, &argv(&["rev-parse", "--show-toplevel"])).await?;
    if top.code != Some(0) {
        return Ok(false);
    }
    let printed: &[u8] = top.stdout.strip_suffix(b"\n".as_slice()).unwrap_or(top.stdout.as_slice());
    let here: PathBuf = PathBuf::from(crate::path_bytes::os_from_path_bytes(printed));
    return Ok(same_directory(here.as_path(), place));
}

/// What:
///  Whether an environment variable would keep a Git started for another worktree from reading that worktree's
///  repository, its index or its files.
/// Why:
///   The caller's environment may name a Git directory,
/// a worktree or an index,
///  and every start of a move in another worktree must read the one it was asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesTheRepository = (name) => ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE','GIT_COMMON_DIR'].includes(name);
/// ```
fn names_the_repository(name: &std::ffi::OsStr) -> bool {
    return matches!(
        name.to_str(),
        Some("GIT_DIR" | "GIT_WORK_TREE" | "GIT_INDEX_FILE" | "GIT_COMMON_DIR")
    );
}

/// What:
///  The same Git context, run in another worktree.
/// Why:
///   The move of a branch checked out elsewhere writes that worktree's index and files,
/// so every start of it has to run there,
///  and the caller's own repository variables would otherwise keep it in the repository this command was started in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const there = (git, directory) => ({ ...git, cwd: directory, environment: git.environment.filter(([name]) => !namesTheRepository(name)) });
/// ```
pub(crate) fn git_at(git: &PushGit, directory: &Path) -> PushGit {
    let environment: Vec<(OsString, OsString)> = git
        .environment
        .iter()
        .filter(|(name, _)| return !names_the_repository(name.as_os_str()))
        .cloned()
        .collect();
    return PushGit {
        real_git: git.real_git.clone(),
        cwd: directory.to_path_buf(),
        environment,
        lfs: git.lfs,
    };
}
