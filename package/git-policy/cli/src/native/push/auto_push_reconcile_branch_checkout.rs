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
///  What became of the checkout a move wrote,
/// and what of the merge is still in it.
/// Why:
///   The move writes the checkout before it writes the reference,
/// and the repair after it must never replace bytes it cannot prove are the move's or a switch's:
///  a person's staged edit and their worktree edit are told apart from those by what the index holds,
/// not by how quickly the repair looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Brought = { kind: 'right' } | { kind: 'left'; paths: Uint8Array[]; reason: string };
/// ```
pub(crate) enum Brought {
    /// The checkout agrees with the branch it shows on the paths this move wrote.
    Right,
    /// These paths still hold something else, and were left exactly as they were.
    Left {
        /// The paths of the move that do not hold what the branch has.
        paths: Vec<Vec<u8>>,
        /// Why they were left,
        /// in Git's own words when its write refused.
        reason: String,
    },
}

/// What:
///  Bring the checkout to the branch it shows on the paths this move wrote,
/// writing only what the index proves is the move's or a switch's content,
/// or say which paths were left and why.
/// Why:
///   `git read-tree -u -m <from> <to>` writes a path only when the index holds `<from>`'s content
/// and the worktree still matches the index there,
/// and it refuses every other path:
///  a staged edit, a worktree edit and an untracked file all stop it,
/// which is the proof this step needs and what a look followed by an overwriting write cannot give.
/// The pair it is given is the tree of the branch the checkout shows:
/// from the pushed tip to the merge for the pushed branch,
///  and from the merge to that branch's own tree for any other,
/// so the write puts the branch's content in and the merge's carried content out,
///  and only over bytes whose stored identity is exactly one of those two.
///  Two attempts at most:
/// a switch that lands between the look and the write leaves the checkout on another branch whose tree
/// the second attempt writes from,
///  and each attempt refuses rather than overwrites.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function bringTheCheckoutToItsBranch({ git, branchRef, local, mergeOid }): Promise<Brought>;
/// ```
pub(crate) async fn bring_the_checkout_to_its_branch_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Brought, SingleFlightError> {
    let paths: Vec<Vec<u8>> = the_moves_paths_awaiting(git, local, merge_oid).await?;
    if paths.is_empty() {
        return Ok(Brought::Right);
    }
    if the_paths_held_unlike_the_branch_awaiting(git, paths.as_slice()).await?.is_empty() {
        return Ok(Brought::Right);
    }
    let mut refusal: String = String::new();
    for _attempt in 0..2 {
        let shows: Option<String> = shows_awaiting(git).await?;
        // What: the pushed tip and the merge, in the order that puts this checkout's paths right.
        // Why: these two commits differ on exactly the paths the move wrote, so the write cannot reach any other path
        //      (a person's staged change elsewhere stays theirs), and `read-tree` requires the index to hold the
        //      from-tree's content on each of them, which is the proof that the bytes written are the move's own.
        //      A checkout still on the pushed branch holds the old tip's content there and needs the merge written in;
        //      one a switch took to another branch holds the merge's carried content and needs the old tip's written
        //      out, which is that branch's own content on those paths, because a switch only carries the difference
        //      onto a branch whose tree matches the old tip there.
        let (from, to): (&str, &str) = match shows.as_deref() {
            Some(named) if named == branch_ref => (local, merge_oid),
            _ => (merge_oid, local),
        };
        let written = run_query_awaiting(git, &argv(&["read-tree", "-u", "-m", from, to])).await?;
        if written.code != Some(0) {
            refusal = crate::auto_push_reconcile_branch_advance::refusal_of(&written.stderr);
            break;
        }
        // The fast-forward carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
        crate::inner_git_words::record_streams(&["read-tree", "-u", "-m", from, to], &written.stdout, &written.stderr);
        if the_paths_held_unlike_the_branch_awaiting(git, paths.as_slice()).await?.is_empty() {
            return Ok(Brought::Right);
        }
    }
    let left: Vec<Vec<u8>> = the_paths_held_unlike_the_branch_awaiting(git, paths.as_slice()).await?;
    if left.is_empty() {
        return Ok(Brought::Right);
    }
    if refusal.is_empty() {
        refusal = String::from("a switch of the checkout landed while the branch moved, and the branch's own tree is not what those paths hold");
    }
    return Ok(Brought::Left { paths: left, reason: refusal });
}

/// The two instants of a move a control can step into.
/// Why:
///   Each is a window this file closes,
/// and a control that proves the closure must stop there by its own hand:
///  a `git switch` in the first carries the merge's content onto another branch,
/// and a person's staged or unstaged edit in the second is what the repair must leave alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Window = 'before-the-branch-moves' | 'after-the-branch-moved';
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Window {
    /// Between the last look at the checkout and the write of the reference.
    BeforeTheBranchMoves,
    /// Between the write of the reference and the look that puts the checkout right.
    AfterTheBranchMoved,
}

// The step a control plays in one of those instants, as another command finishing there; nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The steps of the controls running on this thread, one per window.
    static IN_THE_WINDOW: std::cell::RefCell<[Option<Box<dyn Fn()>>; 2]> = const { std::cell::RefCell::new([None, None]) };
}

/// The slot of one window in the thread's list.
#[cfg(test)]
fn slot_of(window: Window) -> usize {
    return match window {
        Window::BeforeTheBranchMoves => 0,
        Window::AfterTheBranchMoved => 1,
    };
}

/// What:
///  Plays `step` in `window` until the guard drops.
/// Why:
///   That is the instant the fix has to survive,
/// so a control that proves it must be able to stop there by its own hand.
#[cfg(test)]
pub(crate) fn in_the_window_in_a_control(window: Window, step: impl Fn() + 'static) -> WindowGuard {
    let slot: usize = slot_of(window);
    IN_THE_WINDOW.with(|steps| {
        steps.borrow_mut()[slot] = Some(Box::new(step));
    });
    return WindowGuard { window };
}

/// Ends the control's step when dropped.
#[cfg(test)]
pub(crate) struct WindowGuard {
    /// The window whose step this guard set.
    window: Window,
}

#[cfg(test)]
impl Drop for WindowGuard {
    fn drop(&mut self) {
        let slot: usize = slot_of(self.window);
        IN_THE_WINDOW.with(|steps| {
            steps.borrow_mut()[slot] = None;
        });
    }
}

/// Plays the control's step for `window`,
/// if one is set.
#[cfg(test)]
pub(crate) fn play_in_the_window(window: Window) {
    let slot: usize = slot_of(window);
    // The step borrows nothing of the caller, so it runs here rather than through a reference that would hold the borrow.
    let played: Option<Box<dyn Fn()>> = IN_THE_WINDOW.with(|steps| {
        return steps.borrow_mut()[slot].take();
    });
    if let Some(step) = played {
        step();
        IN_THE_WINDOW.with(|steps| {
            steps.borrow_mut()[slot] = Some(step);
        });
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
    #[cfg(test)]
    play_in_the_window(Window::AfterTheBranchMoved);
    let brought: Brought = match bring_the_checkout_to_its_branch_awaiting(git, branch_ref, local, merge_oid).await {
        Ok(brought) => brought,
        Err(error) => Brought::Left { paths: Vec::new(), reason: format!("the checkout could not be read: {error}") },
    };
    let shows_now: Option<String> = match shows {
        Some(found) => Some(found),
        None => shows_awaiting(git).await?,
    };
    return Ok(match brought {
        Brought::Right => match shows_now.as_deref() {
            Some(named) if named == branch_ref => Checkout::Followed,
            _ => Checkout::WentTo { shows: shows_now },
        },
        Brought::Left { paths, reason } => Checkout::Holding { shows: shows_now, paths, reason },
    });
}
