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
/// The repair that writes only what its own write proves is the move's own content.
use crate::auto_push_reconcile_branch_checkout::{bring_the_checkout_to_its_branch_awaiting, shown_branch_awaiting};
/// The record of the move, written before the reference moves, and the command that finishes a stop.
use crate::auto_push_branch_move_record::AMoveInHand;
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
///  The move's record is written before the reference moves and removed once the checkout is finished,
/// so a stop at any instruction leaves a record the next command reads
/// (`auto_push_branch_move_record.rs`).
/// Why:
///   The reference has a compare-and-swap and the checkout does not,
/// so the two looks are what keeps a worktree that takes the branch from being left with the older files.
///  The tip is read before the write for the same reason the elsewhere path reads it:
/// a branch that moved since the push is not the branch the merge was made for.
///  A listing Git cannot answer is never read as "nobody holds it":
/// the branch is not moved then, because a worktree that has it checked out would otherwise be left behind
/// (a review of commit `36cae084ae`, its finding 2).
///  Without the record nothing is moved:
/// a stop after the write would leave a checkout nothing finishes
/// (its finding 3).
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
    let taken: Option<PathBuf> = match live_holder_awaiting(git, branch_ref).await {
        Ok(found) => found,
        Err(error) => {
            // What: the holders of the branch could not be read, so the branch is not moved.
            // Why: a worktree that has it checked out would be left with the older files, which the report cannot know.
            debug(TAG, &format!("the worktrees of {branch_ref} could not be read ({error}), so the branch was not moved"));
            return Ok(NowhereMove::Answered(Advanced::Not {
                reason: format!("the worktrees that may have {branch_ref} checked out could not be read: {error}"),
                advice: advice_to_move(merge_oid, where_to),
            }));
        }
    };
    if let Some(holder) = taken {
        debug(
            TAG,
            &format!("a worktree at {} took {branch_ref} before its reference moved, so the branch is moved there", holder.display()),
        );
        return Ok(NowhereMove::Held(holder));
    }
    // What: the branch's tip and the common Git directory in one call.
    // Why: the record's directory is asked for only when the branch is about to move, and one Git spawn is the whole of
    //      what the record adds to a person's push, so the question the move already asked carries it
    //      (`keep/timing-the-record-against-36cae084ae.log` in the review's scratch folder).
    let asked = run_query_awaiting(
        git,
        &argv(&[
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
            "--verify",
            format!("{branch_ref}^{{commit}}").as_str(),
        ]),
    )
    .await?;
    let spoke: String = String::from_utf8_lossy(&asked.stdout).into_owned();
    let mut said = spoke.lines();
    let common_dir_said: Option<String> = said.next().map(str::to_owned);
    let tip_said: Option<String> = said.next().map(str::to_owned);
    if asked.code != Some(0) || tip_said.as_deref() != Some(local) {
        return Ok(NowhereMove::Answered(Advanced::Not {
            reason: String::from(BRANCH_MOVED),
            advice: advice_to_move(merge_oid, where_to),
        }));
    }
    // An answer with no directory in it leaves an empty name, whose record write fails and stops the move with the
    //  sentence below, which is the one place a repository whose Git directory is gone is reported.
    let common_dir: PathBuf = PathBuf::from(common_dir_said.unwrap_or_default());
    let Some(mut in_hand) = AMoveInHand::take(common_dir.as_path(), branch_ref, local, merge_oid) else {
        // What: the record could not be written, so the reference is not moved.
        // Why: a stop after the write would leave a checkout with the older files and nothing that finishes it
        //      (a review of commit `36cae084ae`, its finding 3).
        debug(TAG, &format!("the record of the move of {branch_ref} could not be written in {}", common_dir.display()));
        return Ok(NowhereMove::Answered(Advanced::Not {
            reason: format!("the record of this move could not be written in {}", common_dir.display()),
            advice: advice_to_move(merge_oid, where_to),
        }));
    };
    #[cfg(test)]
    play_in_the_window(Window::BeforeTheWrite);
    let message: String = format!("merge: fast-forward {branch_ref} to the published reconcile merge");
    let moved = run_query_awaiting(
        git,
        &argv(&["update-ref", "-m", message.as_str(), branch_ref, merge_oid, local]),
    )
    .await?;
    if moved.code != Some(0) {
        // What: the compare-and-swap refused, which is the proof that the branch never moved.
        // Why: nothing can need repair then, and no other outcome of this move may end the record
        //      (a review of commit `777c491325`, its findings 4 and 5).
        in_hand.finish(crate::auto_push_branch_move_record::AFinishedRecord::NeverMoved);
        return Ok(NowhereMove::Answered(Advanced::Not {
            reason: refusal_of(&moved.stderr),
            advice: advice_to_move(merge_oid, where_to),
        }));
    }
    record_streams(&["update-ref", branch_ref, merge_oid, local], &moved.stdout, &moved.stderr);
    #[cfg(test)]
    play_in_the_window(Window::AfterTheWrite);
    let appeared: Option<PathBuf> = match live_holder_awaiting(git, branch_ref).await {
        Ok(found) => found,
        Err(error) => {
            // What: the look after the write failed, so the record stays and the next command asks again.
            // Why: a checkout that took the branch may hold the older files, and nothing here can tell which it is.
            debug(TAG, &format!("the worktrees of {branch_ref} could not be read after its reference moved ({error}); the record stays"));
            return Ok(NowhereMove::Answered(Advanced::HoldersUnreadable {
                reason: format!("the worktrees that may have {branch_ref} checked out could not be read after the branch moved: {error}"),
            }));
        }
    };
    let Some(took_it) = appeared else {
        #[cfg(test)]
        play_in_the_window(Window::AfterTheHoldersWereLooked);
        // What: no worktree holds the branch, and the record stays anyway.
        // Why: a checkout that read the old tip may finish publishing its `HEAD` after this listing, and the
        //      record is the only thing that can find it then; it is kept until a command in the worktree that
        //      shows the branch agrees with it, or until the branch moves again
        //      (a review of commit `4ca7aebcdb`, its finding 4).
        in_hand.hand_on();
        return Ok(NowhereMove::Answered(Advanced::Nowhere));
    };
    let repaired: Repaired =
        finish_the_checkout_that_raced_the_move(git, took_it.as_path(), branch_ref, local, merge_oid).await?;
    match repaired.proof {
        // What: the repair answered that no path of the move holds what the branch moved from.
        // Why: that is the one proof of a finished checkout, and the record ends only on it
        //      (a review of commit `777c491325`, its findings 4 and 5).
        Some(proof) => in_hand.finish(crate::auto_push_branch_move_record::AFinishedRecord::TheCheckoutHoldsTheTip(proof)),
        None => {
            // What: a path still holds the moved-from tree, or the checkout could not be read, so the record stays.
            // Why: a later command may finish the write, and a commit here would record the older tree.
            debug(TAG, &format!("the checkout at {} is not proved to hold the merge, so the record stays for a later command", took_it.display()));
            in_hand.hand_on();
        }
    }
    return Ok(NowhereMove::Answered(repaired.advanced));
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
/// What:
///  The answer about the raced checkout,
/// and whether a write of the branch's own content was refused.
/// Why:
///  A refusal that came from the write itself is one a later command may finish,
/// so the move's record stays for it;
///  a path the worktree's own bytes kept from being written is told in the answer and needs no later command.
struct Repaired {
    /// The report's answer.
    advanced: Advanced,
    /// The proof that the checkout holds the branch's own entries, when the repair answered that it does.
    proof: Option<crate::auto_push_branch_move_record::TheCheckoutHoldsTheTip>,
}

async fn finish_the_checkout_that_raced_the_move(
    git: &PushGit,
    place: &Path,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Repaired, SingleFlightError> {
    let named: String = format!("the worktree at {}", place.display());
    let there: PushGit = git_at(git, place);
    #[cfg(test)]
    play_in_the_window(Window::BeforeTheRepairReads);
    let shows: Option<String> = shown_branch_awaiting(&there).await?;
    if shows.as_deref() != Some(branch_ref) {
        // What: the branch is at the merge, and this checkout went elsewhere inside the window.
        // Why: the reference has already moved, so this is not a branch that stayed behind, and the answer must not say
        //      so nor advise `git merge --ff-only`, which is a no-op there
        //      (a review of commit `c15fc5a7b1`, its finding 4).
        return Ok(Repaired {
            advanced: Advanced::WorktreeHolding {
                worktree: place.to_path_buf(),
                shows,
                paths: Vec::new(),
                reason: format!("{named} checked another branch out while {branch_ref} moved, so it holds nothing of the merge"),
            },
            proof: None,
        });
    }
    let head = run_query_awaiting(&there, &argv(&["rev-parse", "--verify", "HEAD^{commit}"])).await?;
    if head.code != Some(0) || String::from_utf8_lossy(&head.stdout).trim() != merge_oid {
        return Ok(Repaired {
            advanced: Advanced::WorktreeHolding {
                worktree: place.to_path_buf(),
                shows: Some(branch_ref.to_owned()),
                paths: Vec::new(),
                reason: format!("{named} moved off the merge while {branch_ref} was moved, so it holds nothing of it"),
            },
            proof: None,
        });
    }
    // What: the record of the move in the common Git directory stays until the repair finished.
    // Why: the repair writes the merge's content path by path, so a kill inside it leaves some paths written and the
    //      rest holding the pushed tip; the next command in that worktree reads the record and finishes them
    //      (`a_stop_inside_the_repair_leaves_a_record_the_next_command_finishes`).
    let brought: Brought = bring_the_checkout_to_its_branch_awaiting(&there, local, merge_oid).await;
    // What: the branch the checkout shows is read again, after the repair, and never reused from before it.
    // Why: a switch landing inside the repair leaves the report naming a branch the worktree no longer shows
    //      (a review of commit `c15fc5a7b1`, its finding 5).
    let shows_now: Option<String> = shown_branch_awaiting(&there).await?;
    let proof: Option<crate::auto_push_branch_move_record::TheCheckoutHoldsTheTip> = match brought {
        // What: the repair answered that no path of the move holds what the branch moved from.
        // Why: the proof of a finished checkout is taken from that answer and from nothing else
        //      (a review of commit `777c491325`, its findings 4 and 5).
        Brought::NoOldTip { .. } => Some(crate::auto_push_branch_move_record::TheCheckoutHoldsTheTip::from_the_checkout(branch_ref, merge_oid)),
        Brought::OldTipLeft { paths, why } => {
            return Ok(Repaired {
                advanced: Advanced::WorktreeHolding {
                    worktree: place.to_path_buf(),
                    shows: shows_now,
                    paths,
                    reason: format!("{named} took {branch_ref} while its reference moved, and its files there are older than the merge: {why}"),
                },
                proof: None,
            });
        }
        Brought::CouldNotTell { why } => {
            return Ok(Repaired {
                advanced: Advanced::WorktreeHolding {
                    worktree: place.to_path_buf(),
                    shows: shows_now,
                    paths: Vec::new(),
                    reason: format!("{named} could not be read after it took {branch_ref}: {why}"),
                },
                proof: None,
            });
        }
    };
    let status_arguments = argv(&["--no-optional-locks", "status", "--porcelain=v1", "-z", "--untracked-files=no"]);
    let status = run_query_awaiting(&there, &status_arguments).await?;
    if status.code != Some(0) {
        return Ok(Repaired {
            advanced: Advanced::WorktreeHolding {
                worktree: place.to_path_buf(),
                shows: shows_now,
                paths: Vec::new(),
                reason: format!("the status of {named} could not be read after its files were brought to the merge"),
            },
            proof: None,
        });
    }
    let kept: bool = !status_paths(&status.stdout).is_empty();
    // What: a checkout that shows another branch is said to show it, and never that it is at the merge.
    // Why: the repair put that branch's own tree back, so the worktree followed its own branch and not this
    // merge; the report that said it was at the merge named the wrong thing
    //  (a review of commit `777c491325`, its finding 3, and commit `c15fc5a7b1`, its finding 5).
    if shows_now.as_deref() != Some(branch_ref) {
        return Ok(Repaired {
            advanced: Advanced::WorktreeWentTo {
                worktree: place.to_path_buf(),
                shows: shows_now,
            },
            proof,
        });
    }
    return Ok(Repaired {
        advanced: Advanced::Elsewhere {
            worktree: place.to_path_buf(),
            kept,
        },
        proof,
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
    /// The listing has named the worktree that took the branch, before the repair reads it.
    BeforeTheRepairReads,
    /// The listing after the write named nobody, before the record is decided.
    AfterTheHoldersWereLooked,
}

// The step a control plays inside a window, as a `git worktree add` of the branch finishing there; nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The steps of the controls running on this thread, one per window.
    static IN_THE_WINDOW: std::cell::RefCell<[Option<Box<dyn Fn()>>; 4]> = const { std::cell::RefCell::new([None, None, None, None]) };
}

/// Plays `step` in `window` until the guard drops.
#[cfg(test)]
pub(crate) fn in_the_window_in_a_control(window: Window, step: impl Fn() + 'static) -> WindowGuard {
    let slot: usize = match window {
        Window::BeforeTheWrite => 0,
        Window::AfterTheWrite => 1,
        Window::BeforeTheRepairReads => 2,
        Window::AfterTheHoldersWereLooked => 3,
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
            Window::BeforeTheRepairReads => 2,
            Window::AfterTheHoldersWereLooked => 3,
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
        Window::BeforeTheRepairReads => 2,
        Window::AfterTheHoldersWereLooked => 3,
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
