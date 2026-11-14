//! What:
//!  What became of a worktree's checkout when a branch was moved to the published merge,
//!  and the step that makes the checkout agree with the branch it shows.
//! Why:
//!  The move writes the checkout's index and files and then moves the reference,
//! and a command of that worktree may check another branch out at any instant between the two.
//! A `git switch` to a branch whose tree matches the old tip on the paths the merge changes does not refuse:
//! it carries that difference over as a staged change,
//!  so the merge's content is left on a branch nobody pushed,
//! and a commit there would publish it
//!  (a review of commit `b0c133f42b`, its finding).
//! So the move looks at the checkout once more after the reference moved,
//! and writes the tree of the branch the checkout shows, on the paths the merge changes and nowhere else:
//! the merge's content for the pushed branch,
//! and the branch's own content for any other.
//!  A path a person has changed since is left as it is and named,
//! because their bytes are not this command's to overwrite,
//!  and a switch of theirs is never undone:
//! the worktree stays on the branch they chose, without this merge's content.
//!  No lock is taken (`AGENTS.md`,
//! rule `NLK`):
//! Git's index lock serialises each write by itself,
//!  and the reference's compare-and-swap already decides who moves the branch.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await bringTheCheckoutToItsBranch({ git, local, mergeOid }); // undefined | { paths, reason }
//! ```

use crate::auto_push_single_flight::{
    PushGit, SingleFlightError, run_query_awaiting,
};
use crate::auto_push_dirty_paths::diff_paths;
use crate::auto_push_reconcile_basics_and_limits::argv;
use std::ffi::OsString;

/// What:
///  What became of the checkout a move wrote.
/// Why:
///   The branch is at the merge either way,
/// so the report must say which of these it found:
/// the checkout that followed,
///  one that a switch took elsewhere and that holds nothing of the merge,
/// and one that still holds some of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Checkout = { kind: 'followed' } | { kind: 'went-to'; shows?: string } | { kind: 'holding'; shows?: string; paths: Uint8Array[]; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Checkout {
    /// The checkout shows the pushed branch with the merge.
    Followed,
    /// The checkout shows another branch, or a detached commit, and holds nothing of the merge.
    WentTo {
        /// The branch the checkout shows,
        /// or nothing for a detached commit.
        shows: Option<String>,
    },
    /// The checkout still holds something of the merge.
    Holding {
        /// The branch the checkout shows,
        /// or nothing for a detached commit.
        shows: Option<String>,
        /// The paths of the merge that are still there,
        /// empty when the checkout could not be read at all.
        paths: Vec<Vec<u8>>,
        /// Why they are still there.
        reason: String,
    },
}

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
async fn shows_awaiting(git: &PushGit) -> Result<Option<String>, SingleFlightError> {
    let asked = run_query_awaiting(git, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if asked.code != Some(0) {
        return Ok(None);
    }
    let name: String = String::from_utf8_lossy(&asked.stdout).trim().to_owned();
    return Ok(if name.is_empty() { None } else { Some(name) });
}

/// What:
///  The paths the move wrote:
/// those the merge changes against the pushed tip.
/// Why:
///   Only these are this command's to write;
/// a staged entry or an edit of the person's on any other path is theirs,
///  and a repair that wrote the whole checkout would replace it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paths = diffPaths((await git(['diff','--name-only','-z','--no-renames',local,merge])).stdout);
/// ```
async fn the_moves_paths_awaiting(
    git: &PushGit,
    local: &str,
    merge_oid: &str,
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let listed = run_query_awaiting(
        git,
        &argv(&["diff", "--name-only", "-z", "--no-renames", local, merge_oid]),
    )
    .await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: format!("diff --name-only -z --no-renames {local} {merge_oid}"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    return Ok(diff_paths(&listed.stdout));
}

/// What:
///  Run `git diff-index --cached --name-only -z --no-renames HEAD -- <paths>`
/// and answer the paths the index holds unlike the tree of the branch the checkout shows.
/// Why:
///   These are exactly the paths where the checkout disagrees with its own branch,
/// restricted to the ones this move wrote:
///  the carried difference of a switch,
/// or the merge's content on a checkout that is still on the pushed branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = diffPaths((await git(['diff-index','--cached','--name-only','-z','--no-renames','HEAD','--',...paths])).stdout);
/// ```
async fn the_paths_held_unlike_the_branch_awaiting(
    git: &PushGit,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let mut arguments: Vec<OsString> = argv(&[
        "diff-index",
        "--cached",
        "--name-only",
        "-z",
        "--no-renames",
        "HEAD",
        "--",
    ]);
    arguments.extend(paths.iter().map(|path| return crate::path_bytes::os_from_path_bytes(path.as_slice())));
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("diff-index --cached --name-only -z --no-renames HEAD"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    return Ok(diff_paths(&listed.stdout));
}

/// What:
///  The paths among `paths` whose worktree file differs from the index.
/// Why:
///   A file that differs is a person's own change made since the move wrote it,
/// and writing the branch's tree over it would throw their bytes away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const edited = diffPaths((await git(['diff','--name-only','-z','--no-renames','--',...paths])).stdout);
/// ```
async fn the_paths_changed_since_awaiting(
    git: &PushGit,
    paths: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let mut arguments: Vec<OsString> = argv(&["diff", "--name-only", "-z", "--no-renames", "--"]);
    arguments.extend(paths.iter().map(|path| return crate::path_bytes::os_from_path_bytes(path.as_slice())));
    let listed = run_query_awaiting(git, &arguments).await?;
    if listed.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: String::from("diff --name-only -z --no-renames --"),
            code: listed.code,
            stderr: String::from_utf8_lossy(&listed.stderr).into_owned(),
        });
    }
    return Ok(diff_paths(&listed.stdout));
}

/// What:
///  Write the branch's own tree over the paths this move wrote and the checkout holds otherwise,
/// leaving a path a person changed since as it is.
///  `Ok(None)` is a checkout that agrees with its branch;
/// `Ok(Some(paths))` the paths that still hold the merge;
/// `Err(reason)` a checkout that could not be put right.
/// Why:
///   The branch the checkout shows is at the merge or at its own tip,
/// and either way its tree is what the checkout must hold on those paths;
///  `git restore --source=HEAD --staged --worktree` writes exactly those paths from it,
/// and a path the branch does not have is removed rather than left behind.
/// The restriction to the move's paths is what keeps a person's staged entry elsewhere,
///  and the earlier question about the worktree keeps their edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function bringTheCheckoutToItsBranch({ git, local, mergeOid }): Promise<{ paths, reason } | undefined>;
/// ```
pub(crate) async fn bring_the_checkout_to_its_branch_awaiting(
    git: &PushGit,
    local: &str,
    merge_oid: &str,
) -> Result<Result<Option<Vec<Vec<u8>>>, String>, SingleFlightError> {
    let paths: Vec<Vec<u8>> = the_moves_paths_awaiting(git, local, merge_oid).await?;
    if paths.is_empty() {
        return Ok(Ok(None));
    }
    let held: Vec<Vec<u8>> = the_paths_held_unlike_the_branch_awaiting(git, paths.as_slice()).await?;
    if held.is_empty() {
        return Ok(Ok(None));
    }
    let changed: Vec<Vec<u8>> = the_paths_changed_since_awaiting(git, held.as_slice()).await?;
    let writeable: Vec<Vec<u8>> = held.iter().filter(|path| return !changed.contains(path)).cloned().collect();
    if !writeable.is_empty() {
        let mut arguments: Vec<OsString> = argv(&["restore", "--source=HEAD", "--staged", "--worktree", "--"]);
        arguments.extend(writeable.iter().map(|path| return crate::path_bytes::os_from_path_bytes(path.as_slice())));
        let written = run_query_awaiting(git, &arguments).await?;
        if written.code != Some(0) {
            return Ok(Err(format!(
                "the checkout could not be brought to the branch it shows: {}",
                crate::auto_push_reconcile_branch_advance::refusal_of(&written.stderr)
            )));
        }
        crate::inner_git_words::record_streams(&["restore", "--source=HEAD", "--staged", "--worktree", "--"], &written.stdout, &written.stderr);
    }
    if changed.is_empty() {
        return Ok(Ok(None));
    }
    return Ok(Ok(Some(changed)));
}

// The step a control plays in the instant between the last look at the checkout and the write of the reference, as
// another command's `git switch` finishing there; nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The step of the control running on this thread.
    static BEFORE_THE_BRANCH_MOVES: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Plays `step` in the window this file closes:
/// between the last look at the checkout and the write of the reference,
///  until the guard drops.
/// Why:
///   That is the instant a `git switch` carries the merge's content onto another branch,
/// so a control that proves the fix must be able to stop there by its own hand.
#[cfg(test)]
pub(crate) fn before_the_branch_moves_in_a_control(step: impl Fn() + 'static) -> BeforeTheBranchMovesGuard {
    BEFORE_THE_BRANCH_MOVES.with(|slot| { *slot.borrow_mut() = Some(Box::new(step)); });
    return BeforeTheBranchMovesGuard;
}

/// Ends the control's step when dropped.
#[cfg(test)]
pub(crate) struct BeforeTheBranchMovesGuard;

#[cfg(test)]
impl Drop for BeforeTheBranchMovesGuard {
    fn drop(&mut self) {
        BEFORE_THE_BRANCH_MOVES.with(|slot| { *slot.borrow_mut() = None; });
    }
}

/// Plays the control's step,
/// if one is set.
#[cfg(test)]
pub(crate) fn play_before_the_branch_moves() {
    let played: Option<Box<dyn Fn()>> = BEFORE_THE_BRANCH_MOVES.with(|slot| { return slot.borrow_mut().take(); });
    if let Some(step) = played {
        step();
        BEFORE_THE_BRANCH_MOVES.with(|slot| { *slot.borrow_mut() = Some(step); });
    }
}

/// What:
///  Look at the checkout after the reference moved and answer what became of it.
/// Why:
///   The reference's move is the last step that can race a switch,
/// so this is the look that decides what the report says and what is put right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checkout = await theCheckoutAfterTheMove({ git, branchRef, local, mergeOid });
/// ```
pub(crate) async fn the_checkout_after_the_move_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Checkout, SingleFlightError> {
    let shows: Option<String> = shows_awaiting(git).await?;
    let (paths, reason): (Vec<Vec<u8>>, String) = match bring_the_checkout_to_its_branch_awaiting(git, local, merge_oid).await {
        Ok(Ok(None)) => (Vec::new(), String::new()),
        Ok(Ok(Some(paths))) => (
            paths,
            String::from("a change in those files is not this command's to overwrite"),
        ),
        Ok(Err(reason)) => (Vec::new(), reason),
        Err(error) => (Vec::new(), format!("the checkout could not be read: {error}")),
    };
    if !reason.is_empty() {
        return Ok(Checkout::Holding { shows, paths, reason });
    }
    return Ok(match shows.as_deref() {
        Some(named) if named == branch_ref => Checkout::Followed,
        _ => Checkout::WentTo { shows },
    });
}
