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
use crate::auto_push_reconcile_branch_checkout_files::the_paths_the_checkout_disagrees_with_its_branch_awaiting;
/// The branch a checkout shows,
/// asked in one place for both the repair and the report.
pub(crate) use crate::auto_push_reconcile_branch_checkout_files::shows_awaiting;

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
pub(crate) async fn the_moves_paths_awaiting(
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
    /// No path of the move holds what the branch moved from, so a record of that move may end.
    NoOldTip {
        /// The paths that hold a change of the person's, which is theirs and is named.
        a_persons: Vec<Vec<u8>>,
    },
    /// These paths still hold what the branch moved from, and the write of them did not happen.
    OldTipLeft {
        /// The paths that still hold the tree the branch moved from.
        paths: Vec<Vec<u8>>,
        /// Why they were left,
        /// in Git's own words when its write refused.
        why: String,
    },
    /// The checkout could not be read or written, so nothing follows from it: a record stays.
    CouldNotTell {
        /// What could not be read or written.
        why: String,
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
pub(crate) async fn bring_the_checkout_to_its_branch_awaiting(git: &PushGit, old_oid: &str, tip_oid: &str) -> Brought {
    return match the_checkout_brought_awaiting(git, old_oid, tip_oid).await {
        Ok(brought) => brought,
        // A read or a write that failed is not an answer, so nothing follows from it and a record stays.
        Err(error) => Brought::CouldNotTell {
            why: format!("the checkout could not be read or written: {error}"),
        },
    };
}

/// What:
///  The repair's own work, whose every failure its caller turns into "could not tell".
/// Why:
///   The value that says whether a record may end is built only from an answer, never from a failure:
/// a failure here can be neither `NoOldTip` nor `OldTipLeft`, because neither was observed.
///
/// What the repair is about, in one paragraph: the index and the files of the worktree it runs in, on the
/// paths where the record's own move wrote something, and nothing else.
///  A path is the record's leftover when its index entry or its file holds an entry the move itself wrote
/// there, which is the tip the branch moved from or the merge it moved to;
/// such a path is written with the entry this worktree's HEAD names **now**, and HEAD is read again at each
/// attempt, because a `git switch` can land between any two of this command's questions and a cached reading
/// of HEAD is a reading of another branch's checkout
///  (a review of commit `7aef6368e7`, its findings 1 and 3).
///  A path holding anything else is a person's and is left, and the answer says so.
async fn the_checkout_brought_awaiting(git: &PushGit, old_oid: &str, tip_oid: &str) -> Result<Brought, SingleFlightError> {
    let mut from: [String; 2] = [old_oid.to_owned(), tip_oid.to_owned()];
    let mut differing: Vec<Vec<u8>> = Vec::new();
    for attempt in 0..2 {
        let target: String = crate::auto_push_reconcile_branch_checkout_files::the_head_commit_awaiting(git).await?;
        // What: for each end of the move, the paths where that end differs from the tree this worktree's HEAD names.
        // Why:  a path where the two agree holds nothing of the move that this checkout needs changed, and asking
        // the question of each end alone is what keeps the end the checkout is *supposed* to hold out of the answer
        //  (a review of commit `7aef6368e7`, its findings 1 and 3).
        let mut mine: Vec<(String, Vec<Vec<u8>>)> = Vec::new();
        for end in from.iter() {
            // What: the paths where this end differs from HEAD, kept to the paths the record's own move wrote.
            // Why:  a path outside them is not this record's to write, and a person's staged entry there can
            // happen to hold an entry the move also holds; writing it would take their change
            //  (a review of commit `7aef6368e7`, its findings 1 and 3).
            let paths: Vec<Vec<u8>> = the_move_paths_awaiting(git, end.as_str(), target.as_str(), old_oid, tip_oid).await?;
            if !paths.is_empty() {
                mine.push((end.clone(), paths));
            }
        }
        if mine.is_empty() {
            return Ok(Brought::NoOldTip { a_persons: Vec::new() });
        }
        if attempt == 0 {
            #[cfg(test)]
            play_in_the_window(Window::WhileTheCheckoutIsRepaired);
        }
        // What: each path that holds an end this move wrote, the entry this worktree's HEAD has now.
        // Why:  the path-by-path act refuses the path it no longer recognises, so a person's edit made since
        // the move is never replaced, and a path of the move is brought forward whatever stands on the others
        //  (a review of commit `777c491325`, its finding 3, and commit `7aef6368e7`, its finding 1).
        let mut refused: Vec<String> = Vec::new();
        let mut left: Vec<Vec<u8>> = Vec::new();
        let mut all: Vec<Vec<u8>> = Vec::new();
        for (end, paths) in mine.iter() {
            for path in paths {
                if !all.contains(path) {
                    all.push(path.clone());
                }
            }
            refused.extend(bring_the_paths_forward_awaiting(git, end.as_str(), target.as_str(), paths.as_slice()).await?);
            // What: what still holds this end, asked after the write.
            // Why:  that is the question the gate's and the record's proof is about, and it is asked at the moment
            // it is answered.
            // What: what still holds an entry this end really has, asked after the write.
            // Why:  an absent path holds nothing, so a person's deletion or `git rm --cached` is never read as the
            // move's leftover  (a review of this commit, its findings 2 and 3).
            let wrote: Vec<Vec<u8>> =
                crate::auto_push_reconcile_branch_checkout_files::the_paths_the_move_wrote_awaiting(git, end.as_str(), paths.as_slice()).await?;
            for path in wrote {
                if !left.contains(&path) {
                    left.push(path);
                }
            }
        }
        differing = the_paths_the_checkout_disagrees_with_its_branch_awaiting(git, all.as_slice()).await?;
        let why: String = if refused.is_empty() {
            String::from("the checkout still holds what the branch moved from, and nothing was written over it")
        } else {
            format!("the write refused those paths: {}", refused.join("; "))
        };
        // What: HEAD is read again after every write, and a branch that changed under it gets one more attempt.
        // Why:  the write that raced a switch wrote what the branch showed before, and the branch it shows now
        // needs its own entry: the checkout would be left holding another branch's files, which is what a commit
        // there would record
        //  (a review of commit `7aef6368e7`, its finding 1).
        let shows_again: String = crate::auto_push_reconcile_branch_checkout_files::the_head_commit_awaiting(git).await?;
        if attempt == 0 && shows_again != target {
            from = if target == tip_oid {
                [target, old_oid.to_owned()]
            } else {
                [target, tip_oid.to_owned()]
            };
            continue;
        }
        if left.is_empty() {
            return Ok(Brought::NoOldTip { a_persons: differing });
        }
        return Ok(Brought::OldTipLeft { paths: left, why });
    }
    return Ok(Brought::NoOldTip { a_persons: differing });
}

/// What:
///  The paths of `end` where `end` differs from `target`, kept to the paths where the record's own move wrote
/// something (`moves_from` against `moves_to`).
/// Why:
///   The repair writes only what the record's move can account for: a path outside that move is not this
/// command's, and a person's change there is never mistaken for a leftover of the move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wrote = await pathsOf(move_from, move_to); return (await pathsOf(end, target)).filter((path) => wrote.includes(path))
/// ```
async fn the_move_paths_awaiting(
    git: &PushGit,
    end: &str,
    target: &str,
    move_from: &str,
    move_to: &str,
) -> Result<Vec<Vec<u8>>, SingleFlightError> {
    let wrote: Vec<Vec<u8>> = the_moves_paths_awaiting(git, move_from, move_to).await?;
    let differ: Vec<Vec<u8>> = the_moves_paths_awaiting(git, end, target).await?;
    let mut mine: Vec<Vec<u8>> = Vec::new();
    for path in differ {
        if wrote.contains(&path) {
            mine.push(path);
        }
    }
    return Ok(mine);
}

/// What:
///  Write the branch's own entries over the paths that still hold what it moved from, singly,
/// saying what Git refused.
/// Why:
///  A path whose index still holds the moved-from entry is written with `git apply --index`, which requires
/// that very entry and takes the index lock, and one whose index is already the branch's is written in the
/// file alone by `git apply`:
///  both refuse a path whose bytes changed since, so a person's edit is never replaced.
async fn bring_the_paths_forward_awaiting(
    git: &PushGit,
    from: &str,
    to: &str,
    paths: &[Vec<u8>],
) -> Result<Vec<String>, SingleFlightError> {
    // What: only the paths where `from` really has an entry and this checkout holds it, in its index or in its file.
    // Why:  an absent path is not an entry, so a file the merge added and a person then deleted is not the move's
    // debris and is never brought back  (a review of this commit, its findings 2 and 3).
    let wrote: Vec<Vec<u8>> = crate::auto_push_reconcile_branch_checkout_files::the_paths_the_move_wrote_awaiting(git, from, paths).await?;
    let mut in_the_index: Vec<Vec<u8>> = Vec::new();
    let mut in_the_file_only: Vec<Vec<u8>> = Vec::new();
    let holds_in_the_index: Vec<Vec<u8>> =
        crate::auto_push_reconcile_branch_checkout_files::the_paths_whose_index_holds_awaiting(git, from, paths).await?;
    for path in wrote {
        if holds_in_the_index.contains(&path) {
            in_the_index.push(path);
        } else {
            in_the_file_only.push(path);
        }
    }
    return crate::auto_push_reconcile_branch_checkout_files::the_branchs_own_entries_written_awaiting(
        git,
        from,
        to,
        in_the_index.as_slice(),
        in_the_file_only.as_slice(),
    )
    .await;
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
    let brought: Brought = bring_the_checkout_to_its_branch_awaiting(git, local, merge_oid).await;
    // What: the branch the checkout shows is read after the repair, never reused from before it.
    // Why: a switch landing inside the repair leaves the report naming a branch the worktree no longer shows, which is a
    //      wrong report although every write stayed content-guarded
    //      (a review of commit `c15fc5a7b1`, its finding 5).
    let shows_now: Option<String> = shows_awaiting(git).await?;
    return Ok(match brought {
        Brought::NoOldTip { .. } => match shows_now.as_deref() {
            Some(named) if named == branch_ref => Checkout::Followed,
            _ => Checkout::WentTo { shows: shows_now },
        },
        Brought::OldTipLeft { paths, why } => Checkout::Holding {
            shows: shows_now,
            paths,
            reason: why,
        },
        Brought::CouldNotTell { why } => Checkout::Holding {
            shows: shows_now,
            paths: Vec::new(),
            reason: why,
        },
    });
}
