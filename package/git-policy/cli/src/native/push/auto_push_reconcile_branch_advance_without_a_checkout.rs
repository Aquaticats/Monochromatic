//! What:
//!  The move of the branch a push sent when no live worktree has it checked out:
//!  the compare-and-swap of the reference,
//! which looks at the worktree listing on either side of it,
//!  and the repair of a checkout that took the branch inside that window.
//! Why:
//!   A branch that is checked out nowhere has only its ref to move,
//! and the compare-and-swap of `update-ref` refuses a branch that moved since the push
//!  without touching it.
//!  A `git worktree add` or `git switch` of that branch at the same moment is not part of that swap:
//! the new checkout writes its index and files from the tip it read,
//!  and a reference moved afterwards leaves it showing the older files,
//! which a commit there would turn into a reversal of the merge
//!  (a review of commit `98de70959d`,
//!  finding 3).
//! So the listing is read again just before the write,
//! where a worktree that has taken the branch is handed the move that writes its files,
//!  and again just after it,
//! where a worktree that appeared inside the window has its checkout brought to the merge.
//!  No lock can make that window empty:
//!  the checkout and the reference write are two commands of different programs,
//! so what is closed here is the window the listing leaves,
//! and the write's own compare-and-swap closes the reference's part of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const moved = await moveTheRefOnly({ git, branchRef, local, mergeOid }); // { held: path } | { answered }
//! ```

use crate::auto_push_single_flight::{
    PushGit, SingleFlightError, run_query_awaiting,
};
use crate::auto_push_dirty_paths::status_paths;
/// The answers a move gives, the words of a refusal, and the steps both moves share.
use crate::auto_push_reconcile_branch_advance::{Advanced, BRANCH_MOVED, Brought, advice_to_move, refusal_of};
/// The repair that writes only what the index proves is the move's own content.
use crate::auto_push_reconcile_branch_checkout::bring_the_checkout_to_its_branch_awaiting;
use crate::auto_push_reconcile_branch_worktree::{git_at, live_holder_awaiting};
use crate::auto_push_reconcile_basics_and_limits::{TAG, argv};
use crate::diagnostic_log::debug;
use crate::inner_git_words::record_streams;
use std::path::{Path, PathBuf};

/// What:
///  What the move of a reference alone came to.
/// Why:
///   Two answers ask the caller for something the ref's move cannot do itself:
/// a worktree that took the branch before the write has a checkout to write,
///  and that is the elsewhere move rather than this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NowhereMove = { kind: 'answered'; advanced: Advanced } | { kind: 'held'; worktree: string };
/// ```
pub(crate) enum NowhereMove {
    /// The branch moved, was refused, or was repaired with the worktree that raced it.
    Answered(Advanced),
    /// A worktree took the branch before the write, so the branch is moved there.
    Held(PathBuf),
}

/// What:
///  Move a branch no live worktree has checked out,
/// which writes no file,
/// or say why it was not moved,
/// and look at the listing on either side of the write.
/// Why:
///   The reference has a compare-and-swap and the checkout does not,
/// so the two looks are what keeps a worktree that takes the branch from being left with the older files.
///  The tip is read before the write for the same reason the elsewhere path reads it:
/// a branch that moved since the push is not the branch the merge was made for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function moveTheRefOnly(git, branchRef, local, mergeOid): Promise<NowhereMove>;
/// ```
pub(crate) async fn move_the_ref_only_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<NowhereMove, SingleFlightError> {
    let where_to: &str = "in a worktree that has it checked out";
    // A worktree that took the branch since the first look has files of its own to write, and only its own move may write them.
    if let Some(taken) = live_holder_awaiting(git, branch_ref).await? {
        debug(
            TAG,
            &format!("a worktree at {} took {branch_ref} before its reference moved, so the branch is moved there", taken.display()),
        );
        return Ok(NowhereMove::Held(taken));
    }
    let tip = run_query_awaiting(git, &argv(&["rev-parse", "--verify", format!("{branch_ref}^{{commit}}").as_str()])).await?;
    if tip.code != Some(0) || String::from_utf8_lossy(&tip.stdout).trim() != local {
        return Ok(NowhereMove::Answered(Advanced::Not {
            reason: String::from(BRANCH_MOVED),
            advice: advice_to_move(merge_oid, where_to),
        }));
    }
    #[cfg(test)]
    play_in_the_window(Window::BeforeTheWrite);
    let message: String = format!("merge: fast-forward {branch_ref} to the published reconcile merge");
    let moved = run_query_awaiting(
        git,
        &argv(&["update-ref", "-m", message.as_str(), branch_ref, merge_oid, local]),
    )
    .await?;
    if moved.code != Some(0) {
        return Ok(NowhereMove::Answered(Advanced::Not {
            reason: refusal_of(&moved.stderr),
            advice: advice_to_move(merge_oid, where_to),
        }));
    }
    record_streams(&["update-ref", branch_ref, merge_oid, local], &moved.stdout, &moved.stderr);
    #[cfg(test)]
    play_in_the_window(Window::AfterTheWrite);
    return match live_holder_awaiting(git, branch_ref).await? {
        None => Ok(NowhereMove::Answered(Advanced::Nowhere)),
        Some(appeared) => Ok(NowhereMove::Answered(
            finish_the_checkout_that_raced_the_move(git, appeared.as_path(), branch_ref, local, merge_oid).await?,
        )),
    };
}

/// What:
///  Bring a worktree's index and files from the pushed tip to the merge,
/// for a checkout that took the branch while its reference moved,
///  and say what is true of that checkout when it cannot be brought there.
/// Why:
///   The checkout read the branch before the reference moved and wrote the pushed tip's files,
/// while the branch now names the merge:
///  the index holds a reversal of the merge that a commit there would publish.
/// The repair writes only what it can prove is the pushed tip's own content
/// (`bring_the_checkout_to_its_branch_awaiting`),
///  and a person's edit to one of those files keeps its bytes.
///  The reference has already moved, so a checkout that cannot be brought along is NOT a branch that stayed behind:
/// the answer says the branch is at the merge and that worktree is not,
///  names the paths, and gives the step that finishes it
///  (a review of commit `6c39c3719a`, its defect 2:
/// the answer here said the branch was not at the merge, which was false,
///  and advised `git merge --ff-only`, which prints "Already up to date." there).
/// The two questions before it are what proves this is the worktree meant:
///  it still has the branch checked out, and its `HEAD` is this merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finishTheCheckoutThatRacedTheMove(git, place, branchRef, local, mergeOid): Promise<Advanced>;
/// ```
async fn finish_the_checkout_that_raced_the_move(
    git: &PushGit,
    place: &Path,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Advanced, SingleFlightError> {
    let named: String = format!("the worktree at {}", place.display());
    let where_to: String = format!("in {named}");
    let there: PushGit = git_at(git, place);
    let checked_out = run_query_awaiting(&there, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if checked_out.code != Some(0) || String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(Advanced::Not {
            reason: format!("{named} checked out another branch while the merge was published"),
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    let head = run_query_awaiting(&there, &argv(&["rev-parse", "--verify", "HEAD^{commit}"])).await?;
    if head.code != Some(0) || String::from_utf8_lossy(&head.stdout).trim() != merge_oid {
        return Ok(Advanced::Not {
            reason: String::from(BRANCH_MOVED),
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    let brought: Brought = match bring_the_checkout_to_its_branch_awaiting(&there, branch_ref, local, merge_oid).await {
        Ok(brought) => brought,
        Err(error) => Brought::Left { paths: Vec::new(), reason: format!("{named} could not be read: {error}") },
    };
    if let Brought::Left { paths, reason } = brought {
        return Ok(Advanced::WorktreeHolding {
            worktree: place.to_path_buf(),
            shows: Some(branch_ref.to_owned()),
            paths,
            reason: format!("{named} took {branch_ref} while its reference moved, and its files there are older than the merge: {reason}"),
        });
    }
    let status_arguments = argv(&["--no-optional-locks", "status", "--porcelain=v1", "-z", "--untracked-files=no"]);
    let status = run_query_awaiting(&there, &status_arguments).await?;
    if status.code != Some(0) {
        return Ok(Advanced::WorktreeHolding {
            worktree: place.to_path_buf(),
            shows: Some(branch_ref.to_owned()),
            paths: Vec::new(),
            reason: format!("the status of {named} could not be read after its files were brought to the merge"),
        });
    }
    let kept: bool = !status_paths(&status.stdout).is_empty();
    return Ok(Advanced::Elsewhere {
        worktree: place.to_path_buf(),
        kept,
    });
}

/// The two windows of the reference's move a control can play a step in,
/// as the here-path's own window has one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Window = 'before-the-write' | 'after-the-write';
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Window {
    /// After the worktree listing, before the reference's compare-and-swap.
    BeforeTheWrite,
    /// After the reference's compare-and-swap, before the worktree listing is read again.
    AfterTheWrite,
}

// The step a control plays inside a window, as a `git worktree add` of the branch finishing there; nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The steps of the controls running on this thread, one per window.
    static IN_THE_WINDOW: std::cell::RefCell<[Option<Box<dyn Fn()>>; 2]> = const { std::cell::RefCell::new([None, None]) };
}

/// Plays `step` in `window` until the guard drops.
#[cfg(test)]
pub(crate) fn in_the_window_in_a_control(window: Window, step: impl Fn() + 'static) -> WindowGuard {
    let slot: usize = match window {
        Window::BeforeTheWrite => 0,
        Window::AfterTheWrite => 1,
    };
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
        let slot: usize = match self.window {
            Window::BeforeTheWrite => 0,
            Window::AfterTheWrite => 1,
        };
        IN_THE_WINDOW.with(|steps| {
            steps.borrow_mut()[slot] = None;
        });
    }
}

/// Plays the control's step for `window`,
/// if one is set.
#[cfg(test)]
fn play_in_the_window(window: Window) {
    let slot: usize = match window {
        Window::BeforeTheWrite => 0,
        Window::AfterTheWrite => 1,
    };
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
