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
/// The questions about a checkout's files: what the branch lacks, what holds the merge's bytes, where
/// the checkout disagrees with its branch, and which paths still hold the old tip's.
use crate::auto_push_reconcile_branch_checkout_files::{
    the_branchs_own_entries_written_awaiting, the_merge_bytes_left_on_the_branch_awaiting,
    the_paths_still_holding_the_old_tip_awaiting, the_paths_the_checkout_disagrees_with_its_branch_awaiting,
};
/// The branch a checkout shows,
/// asked in one place for both the repair and the report.
pub(crate) use crate::auto_push_reconcile_branch_checkout_files::shows_awaiting;
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
///  Bring the files of the move to the branch the checkout shows,
/// writing the branch's own entry over a path that holds what the old tip held,
/// and saying which paths were left because they are a person's.
/// Why:
///   The index is already the branch's here, so what remains is the files:
/// a path of the move whose stored entry is the old tip's is this move's own leftover (a reset of the
///  index can leave it, and a checkout that took the branch after the write keeps it), and the branch's
///  entry belongs there;
/// a path whose entry is the merge's is the finding of the review before this one, and is named;
///  a path that holds neither the old tip's entry nor the merge's is a person's, and stays theirs
///  (a review of commit `cae78e1d7e`, its finding 3, and commit `36cae084ae`, its finding 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function theFilesOfTheMoveBrought({ git, branchRef, local, merge, paths }): Promise<Brought>;
/// ```
async fn the_files_of_the_move_brought_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
    paths: &[Vec<u8>],
) -> Result<Brought, SingleFlightError> {
    let differing: Vec<Vec<u8>> = the_paths_the_checkout_disagrees_with_its_branch_awaiting(git, paths).await?;
    if differing.is_empty() {
        return Ok(Brought::Right);
    }
    let holding_the_old_tip: Vec<Vec<u8>> = the_paths_still_holding_the_old_tip_awaiting(git, local, differing.as_slice()).await?;
    if !holding_the_old_tip.is_empty() {
        #[cfg(test)]
        play_in_the_window(Window::WhileTheCheckoutIsRepaired);
        // What: the patch between the two tips, applied path by path, which refuses a path whose bytes are
        // no longer the old tip's.
        // Why: a person's edit or staged change made since the paths were read must keep its bytes, and the
        // write that only compares before it writes replaced them
        // (a review of commit `4ca7aebcdb`, its finding 1).
        let refusals: Vec<String> = the_branchs_own_entries_written_awaiting(git, local, merge_oid, holding_the_old_tip.as_slice()).await?;
        let still: Vec<Vec<u8>> = the_paths_the_checkout_disagrees_with_its_branch_awaiting(git, paths).await?;
        if still.is_empty() {
            return Ok(Brought::Right);
        }
        let reason: String = if refusals.is_empty() {
            String::from("the worktree holds neither the old tip's content nor the merge's on those paths, so they are a change of your own")
        } else {
            format!("the write refused the path, because it no longer holds what the branch moved from: {}", refusals.join("; "))
        };
        return Ok(Brought::Left {
            paths: still,
            reason,
            write_refused: false,
        });
    }
    let hidden: Vec<Vec<u8>> = the_merge_bytes_left_on_the_branch_awaiting(git, branch_ref, merge_oid, differing.as_slice()).await?;
    return Ok(Brought::Left {
        paths: if hidden.is_empty() { differing } else { hidden },
        reason: String::from("the worktree holds neither the old tip's content nor the merge's on those paths, so they are a change of your own"),
        write_refused: false,
    });
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
#[derive(Debug, Eq, PartialEq)]
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
        /// Whether a write of the branch's own content was attempted and refused,
        /// so a later command may finish it;
        /// a path the worktree's own bytes kept from being written is not one of these.
        write_refused: bool,
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
        // What: the index agrees with the branch, and the files of the move are asked next.
        // Why: no single comparison finishes a record: a reset moves the index without the files, and a path
        //      whose stored entry is the old tip's is this move's own leftover, which the branch's entry replaces
        //      (a review of commit `cae78e1d7e`, its finding 3, and commit `36cae084ae`, its finding 4).
        return the_files_of_the_move_brought_awaiting(git, branch_ref, local, merge_oid, paths.as_slice()).await;
    }
    #[cfg(test)]
    play_in_the_window(Window::WhileTheCheckoutIsRepaired);
    let mut refusal: String = String::new();
    // Whether the write itself was refused, which a later command may finish, rather than a path the proof kept it from.
    let mut write_refused: bool = false;
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
            write_refused = true;
            break;
        }
        // The fast-forward carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
        crate::inner_git_words::record_streams(&["read-tree", "-u", "-m", from, to], &written.stdout, &written.stderr);
        if the_paths_the_checkout_disagrees_with_its_branch_awaiting(git, paths.as_slice()).await?.is_empty() {
            return Ok(Brought::Right);
        }
    }
    let left: Vec<Vec<u8>> = the_paths_held_unlike_the_branch_awaiting(git, paths.as_slice()).await?;
    if left.is_empty() {
        // What: the write left the index in the branch's own state, so what still differs is the files', and the same
        // answer that decides a checkout agrees decides this one.
        // Why: asking the index alone read a reset of the index plus stale files as a finished checkout
        //      (a review of commit `cae78e1d7e`, its finding 3).
        return the_files_of_the_move_brought_awaiting(git, branch_ref, local, merge_oid, paths.as_slice()).await;
    }
    if refusal.is_empty() {
        refusal = String::from("a switch of the checkout landed while the branch moved, and the branch's own tree is not what those paths hold");
    }
    // A refusal that came from the write itself is one a later command may finish; the fallback reason is a path the
    //  proof kept the write from, which no later command can change without the person.
    return Ok(Brought::Left { paths: left, reason: refusal, write_refused });
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
    /// Inside the repair, after the paths it would write are read and before it writes any of them.
    WhileTheCheckoutIsRepaired,
}

// The step a control plays in one of those instants, as another command finishing there; nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The steps of the controls running on this thread, one per window.
    static IN_THE_WINDOW: std::cell::RefCell<[Option<Box<dyn Fn()>>; 3]> = const { std::cell::RefCell::new([None, None, None]) };
}

/// The slot of one window in the thread's list.
#[cfg(test)]
fn slot_of(window: Window) -> usize {
    return match window {
        Window::BeforeTheBranchMoves => 0,
        Window::AfterTheBranchMoved => 1,
        Window::WhileTheCheckoutIsRepaired => 2,
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
///  The branch a worktree shows, or nothing for a detached commit,
/// as the move's answers read it.
/// Why:
///   Two paths write a checkout and both must say which branch it shows at the end,
/// so the question is asked in one place.
pub(crate) async fn shown_branch_awaiting(git: &PushGit) -> Result<Option<String>, SingleFlightError> {
    return shows_awaiting(git).await;
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
    #[cfg(test)]
    play_in_the_window(Window::AfterTheBranchMoved);
    let brought: Brought = match bring_the_checkout_to_its_branch_awaiting(git, branch_ref, local, merge_oid).await {
        Ok(brought) => brought,
        Err(error) => Brought::Left { paths: Vec::new(), reason: format!("the checkout could not be read: {error}"), write_refused: false },
    };
    // What: the branch the checkout shows is read after the repair, never reused from before it.
    // Why: a switch landing inside the repair leaves the report naming a branch the worktree no longer shows, which is a
    //      wrong report although every write stayed content-guarded
    //      (a review of commit `c15fc5a7b1`, its finding 5).
    let shows_now: Option<String> = shows_awaiting(git).await?;
    return Ok(match brought {
        Brought::Right => match shows_now.as_deref() {
            Some(named) if named == branch_ref => Checkout::Followed,
            _ => Checkout::WentTo { shows: shows_now },
        },
        Brought::Left { paths, reason, .. } => Checkout::Holding { shows: shows_now, paths, reason },
    });
}
