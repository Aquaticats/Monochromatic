//! What:
//!  The questions a repair asks of a checkout's files, and the one write that puts the branch's own
//!  entries back over the paths a move left holding the old tip's.
//! Why:
//!  What a checkout holds is three answers, not one: the branch it shows, the index against that
//!  branch, and every path of the move in the worktree.
//!  Each question here is one Git command, so the repair that decides what to write reads them in
//!  one place and the file that words the answer does not grow a second copy of them.
//!  These questions are kept apart from `auto_push_reconcile_branch_checkout.rs` because that file
//!  reached the repository's line budget, and because a path whose content is proven is a different
//!  subject from the words the report prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const disagreeing = await git(['diff','--name-only','-z','--no-renames','HEAD','--',...paths]);
//! const stale = paths.filter((path) => !(await git(['diff','--name-only','-z','--no-renames',local,'--',path])));
//! await git(['checkout','--quiet','HEAD','--',...stale]);
//! ```

use crate::auto_push_dirty_paths::diff_paths;
use crate::auto_push_reconcile_basics_and_limits::argv;
use crate::auto_push_single_flight::{PushGit, SingleFlightError, run_query_awaiting};
use std::ffi::OsString;

/// What:
///  The branch the worktree shows,
/// or nothing for a detached `HEAD`.
/// Why:
///   The report names it,
/// and the repair writes the tree of the branch it is,
/// which is not the branch the move was for once a switch took the checkout elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shows = await git(['symbolic-ref','--quiet','HEAD']); // undefined for a detached HEAD
/// ```
pub(crate) async fn shows_awaiting(git: &PushGit) -> Result<Option<String>, SingleFlightError> {
    let asked = run_query_awaiting(git, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if asked.code != Some(0) {
        return Ok(None);
    }
    let name: String = String::from_utf8_lossy(&asked.stdout).trim().to_owned();
    return Ok(if name.is_empty() { None } else { Some(name) });
}

/// What:
///  The Git arguments naming `paths`,
/// each as the operating system names it.
/// Why:
///   Every question below passes the paths of the move to Git as arguments,
/// so the bytes of a name are carried through one loop rather than a closure of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of paths) arguments.push(osFromPathBytes(path));
/// ```
fn naming(paths: &[Vec<u8>], mut arguments: Vec<OsString>) -> Vec<OsString> {
    for path in paths {
        arguments.push(crate::path_bytes::os_from_path_bytes(path.as_slice()));
    }
    return arguments;
}

/// What:
///  The paths of the move where the checkout does not agree with the branch it shows,
/// asked once so no place decides that on part of the answer.
/// Why:
///   `git diff <branch>` compares the index and the files to that branch in one answer,
/// so a reset that moves the index and leaves the files is a difference here,
///  while a place that asked only the index read the same state as agreement
///  (a review of commit `cae78e1d7e`, its finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// diffPaths((await git(['--no-optional-locks','diff','--name-only','-z','--no-renames','HEAD','--',...paths])).stdout)
/// ```
pub(crate) async fn the_paths_the_checkout_disagrees_with_its_branch_awaiting(
    git: &PushGit,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let arguments: Vec<OsString> =
        naming(paths, argv(&["--no-optional-locks", "diff", "--name-only", "-z", "--no-renames", "HEAD", "--"]));
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("diff --name-only -z --no-renames HEAD"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    return Ok(diff_paths(&listed.stdout));
}

/// What:
///  The paths of `paths` that still hold exactly what the old tip held,
/// which is this record's own leftover and not a change of the person's.
/// Why:
///   The record names the tip the branch moved from, and that is what tells a leftover of the move
/// from an edit: a path whose stored entry is the old tip's is one the move wrote or one it had not
///  written yet, and the branch's entry belongs there;
/// a path that holds neither the old tip's entry nor the merge's is a person's and stays theirs
///  (a review of commit `cae78e1d7e`, its findings 2 and 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// paths.filter((path) => !movedOn.includes(path)) // movedOn = git diff --name-only <local> -- <paths>
/// ```
pub(crate) async fn the_paths_still_holding_the_old_tip_awaiting(
    git: &PushGit,
    local: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let arguments: Vec<OsString> = naming(paths, argv(&["--no-optional-locks", "diff", "--name-only", "-z", "--no-renames", local, "--"]));
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("diff --name-only -z --no-renames {local}"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    let moved_on: Vec<Vec<u8>> = diff_paths(&listed.stdout);
    let mut still: Vec<Vec<u8>> = Vec::new();
    for path in paths {
        if !moved_on.contains(path) {
            still.push(path.clone());
        }
    }
    return Ok(still);
}

/// What:
///  Write the branch's own entry over the paths whose content is the old tip's,
/// path by path, by applying the patch between the two trees to the file alone.
/// Why:
///   Those bytes are the move's own, by the record's old tip, so writing the branch's entry there
/// overwrites nothing a person made;
///  and this write is an act that refuses: `git apply` compares each path's own content with the patch
/// it carries before it writes that path,
/// so a person's edit or a staged change made after the paths were looked at is refused rather than
///  replaced, which `git checkout <branch> -- <paths>` did
///  (a review of commit `4ca7aebcdb`, its finding 1).
///  It writes no index entry, because the index already holds the branch's own entries on these paths:
/// that is what made this the case of a checkout whose index agrees and whose files do not.
///  A refusal is Git's own words, and it names the path it refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of paths) await git(['apply','--whitespace=nowarn'], { input: await git(['diff','--binary',old,merge,'--',path]) });
/// ```
pub(crate) async fn the_branchs_own_entries_written_awaiting(
    git: &PushGit,
    old_oid: &str,
    merge_oid: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<String>, SingleFlightError> {
    let mut refusals: Vec<String> = Vec::new();
    for path in paths {
        let arguments: Vec<OsString> = naming(std::slice::from_ref(path), argv(&["--no-optional-locks", "diff", "--binary", old_oid, merge_oid, "--"]));
        let patch = run_query_awaiting(git, &arguments).await?;
        if patch.code != Some(0) {
            refusals.push(format!("the patch between the two tips could not be read: {}", crate::auto_push_reconcile_branch_advance::refusal_of(&patch.stderr)));
            continue;
        }
        if patch.stdout.is_empty() {
            // The path holds the same content in both tips, so there is nothing this move wrote there.
            continue;
        }
        let applied = crate::auto_push_single_flight::run_query_with_input_awaiting(git, &argv(&["apply", "--whitespace=nowarn"]), patch.stdout.clone()).await?;
        if applied.code != Some(0) {
            refusals.push(crate::auto_push_reconcile_branch_advance::refusal_of(&applied.stderr));
            continue;
        }
        crate::inner_git_words::record_streams(&["apply"], &applied.stdout, &applied.stderr);
    }
    return Ok(refusals);
}

/// What:
///  The paths of `paths` where the merge's content and the tree of the branch the checkout shows
/// differ, or all of them when `HEAD` names no commit.
/// Why:
///   These are the paths of the move that the branch the checkout shows does not hold,
/// so a worktree holding the merge's bytes there is the state a reset of the index can hide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// diffPaths((await git(['--no-optional-locks','diff','--name-only','-z','--no-renames',merge,'HEAD','--',...paths])).stdout)
/// ```
async fn the_paths_the_branch_lacks_awaiting(
    git: &PushGit,
    merge_oid: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let arguments: Vec<OsString> = naming(
        paths,
        argv(&["--no-optional-locks", "diff", "--name-only", "-z", "--no-renames", merge_oid, "HEAD", "--"]),
    );
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("diff --name-only -z --no-renames {merge_oid} HEAD"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    return Ok(diff_paths(&listed.stdout));
}

/// What:
///  The paths of `paths` whose worktree file holds the merge's own content.
/// Why:
///   A byte in a worktree is replaced only where the code shows what it is,
/// and this is the question that shows it:
///  `git diff <merge> -- <paths>` lists the paths whose worktree content differs from the merge's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// paths.filter((path) => !differingFromTheMerge.includes(path))
/// ```
async fn the_worktree_paths_holding_the_merge_awaiting(
    git: &PushGit,
    merge_oid: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let arguments: Vec<OsString> =
        naming(paths, argv(&["--no-optional-locks", "diff", "--name-only", "-z", "--no-renames", merge_oid, "--"]));
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("diff --name-only -z --no-renames {merge_oid}"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    let differing: Vec<Vec<u8>> = diff_paths(&listed.stdout);
    let mut holding: Vec<Vec<u8>> = Vec::new();
    for path in paths {
        if !differing.contains(path) {
            holding.push(path.clone());
        }
    }
    return Ok(holding);
}

/// What:
///  The paths of the move whose worktree holds the merge's own bytes while the branch the checkout shows does not,
///  which is the state a mixed reset leaves.
/// Why:
///   The index can be made to agree with anything by a reset,
/// so the answer "nothing of the merge is in this checkout" rests on the files and not on the index
///  (a review of commit `36cae084ae`, its finding 4).
/// Only the paths the branch lacks are asked about,
/// and on a checkout that shows the pushed branch there are none,
///  which is why a move whose checkout followed pays no query here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lacking = await thePathsTheBranchLacks({ git, merge, paths });
/// return lacking.length === 0 ? [] : (await theWorktreePathsHoldingTheMerge({ git, merge, paths: lacking })).filter((path) => lacking.includes(path));
/// ```
pub(crate) async fn the_merge_bytes_left_on_the_branch_awaiting(
    git: &PushGit,
    branch_ref: &str,
    merge_oid: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let shows: Option<String> = shows_awaiting(git).await?;
    if shows.as_deref() == Some(branch_ref) {
        // The branch the checkout shows is at the merge, so its own file is the merge's and nothing is hidden.
        return Ok(Vec::new());
    }
    let lacking: Vec<Vec<u8>> = the_paths_the_branch_lacks_awaiting(git, merge_oid, paths).await?;
    if lacking.is_empty() {
        return Ok(Vec::new());
    }
    return the_worktree_paths_holding_the_merge_awaiting(git, merge_oid, lacking.as_slice()).await;
}
